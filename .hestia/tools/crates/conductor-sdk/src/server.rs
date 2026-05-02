//! Conductor サーバー側トランスポート
//!
//! agent-cli レジストリへの登録・Unix ソケットリスナー・メッセージ受信ループ

use crate::agent::ConductorId;
use crate::error::HestiaError;
use crate::message::{Request, Response, SuccessResponse};
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;
use tokio::signal;

/// メッセージハンドラトレイト
///
/// 各 Conductor はこのトレイトを実装して、CLI からの要求を処理する。
#[async_trait::async_trait]
pub trait MessageHandler: Send + Sync {
    /// 構造化リクエストを処理し、応答を返す
    async fn handle_request(&self, request: Request) -> Response;
}

/// agent-cli レジストリエントリ
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RegistryEntry {
    pub id: String,
    pub name: String,
    pub pid: u32,
    pub started_at: String,
    pub provider: String,
    pub model: String,
    pub socket: String,
    pub persona: PersonaInfo,
}

/// ペルソナ情報
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PersonaInfo {
    pub role: String,
    pub skills: Vec<String>,
    pub description: String,
    pub source_path: Option<String>,
}

/// Conductor サーバー
///
/// agent-cli レジストリに自身を登録し、Unix ソケットでメッセージを待ち受ける。
pub struct ConductorServer {
    conductor_id: ConductorId,
    registry_dir: PathBuf,
    agent_id: String,
    handler: Box<dyn MessageHandler>,
}

impl ConductorServer {
    /// 新しい ConductorServer を作成する
    pub fn new(conductor_id: ConductorId, handler: Box<dyn MessageHandler>) -> Result<Self, HestiaError> {
        let registry_dir = std::env::var("XDG_RUNTIME_DIR")
            .map(|d| PathBuf::from(d).join("agent-cli"))
            .unwrap_or_else(|_| PathBuf::from("/tmp/agent-cli"));

        let agent_id = format!("agent-{}", uuid::Uuid::new_v4().simple());

        Ok(Self {
            conductor_id,
            registry_dir,
            agent_id,
            handler,
        })
    }

    /// レジストリディレクトリをカスタマイズ
    pub fn with_registry_dir(mut self, dir: PathBuf) -> Self {
        self.registry_dir = dir;
        self
    }

    /// サーバーを起動し、メッセージ待受ループに入る
    pub async fn run(self) -> Result<(), HestiaError> {
        // レジストリディレクトリを作成
        std::fs::create_dir_all(&self.registry_dir).map_err(|e| {
            HestiaError::Transport(format!("failed to create registry dir: {e}"))
        })?;

        let socket_path = self.registry_dir.join(format!("{}.sock", self.agent_id));
        let json_path = self.registry_dir.join(format!("{}.json", self.agent_id));

        // 既存のソケットがあれば削除
        if socket_path.exists() {
            std::fs::remove_file(&socket_path)?;
        }

        // Unix ソケットリスナーを作成
        let listener = UnixListener::bind(&socket_path).map_err(|e| {
            HestiaError::Transport(format!("failed to bind socket {}: {e}", socket_path.display()))
        })?;

        // ソケットのパーミッションを 0600 に設定
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o600))?;
        }

        // レジストリエントリを書き込み
        let entry = RegistryEntry {
            id: self.agent_id.clone(),
            name: self.conductor_id.peer_name().to_string(),
            pid: std::process::id(),
            started_at: chrono::Utc::now().to_rfc3339(),
            provider: "hestia".to_string(),
            model: "native".to_string(),
            socket: socket_path.to_string_lossy().to_string(),
            persona: PersonaInfo {
                role: format!("{} conductor", self.conductor_id.peer_name()),
                skills: vec!["tool execution".to_string()],
                description: format!(
                    "Hestia {} conductor daemon",
                    self.conductor_id.peer_name()
                ),
                source_path: None,
            },
        };

        let json_content = serde_json::to_string_pretty(&entry)
            .map_err(|e| HestiaError::Transport(format!("failed to serialize registry entry: {e}")))?;
        std::fs::write(&json_path, &json_content)?;

        tracing::info!(
            "conductor {} registered at {} (socket: {})",
            self.conductor_id.peer_name(),
            json_path.display(),
            socket_path.display()
        );

        // グレースフルシャットダウンのためのクリーンアップを設定
        let reg_dir = self.registry_dir.clone();
        let aid = self.agent_id.clone();
        let cleanup = move || {
            let sp = reg_dir.join(format!("{aid}.sock"));
            let jp = reg_dir.join(format!("{aid}.json"));
            let _ = std::fs::remove_file(&sp);
            let _ = std::fs::remove_file(&jp);
        };

        // メッセージ受信ループ
        tracing::info!("{} conductor ready, listening for messages", self.conductor_id.peer_name());

        let conductor_name = self.conductor_id.peer_name().to_string();

        loop {
            tokio::select! {
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((stream, _addr)) => {
                            let handler = &self.handler;
                            Self::handle_connection(stream, handler).await;
                        }
                        Err(e) => {
                            tracing::error!("failed to accept connection: {e}");
                        }
                    }
                }
                _ = signal::ctrl_c() => {
                    tracing::info!("{} conductor shutting down", conductor_name);
                    cleanup();
                    return Ok(());
                }
            }
        }
    }

    /// 個別の接続を処理する
    async fn handle_connection(
        mut stream: tokio::net::UnixStream,
        handler: &Box<dyn MessageHandler>,
    ) {
        let mut buf = vec![0u8; 16 * 1024 * 1024]; // 16 MiB max
        match stream.read(&mut buf).await {
            Ok(n) if n > 0 => {
                let raw = String::from_utf8_lossy(&buf[..n]);
                let raw_str = raw.trim();

                // ペイロードをパース
                if raw_str.starts_with('{') {
                    match serde_json::from_str::<Request>(raw_str) {
                        Ok(request) => {
                            let method = request.method.clone();
                            let id = request.id.clone();
                            tracing::info!("received request: {method}");
                            let response = handler.handle_request(request).await;
                            let response_json = serde_json::to_string(&response).unwrap_or_else(|e| {
                                serde_json::to_string(&Response::Success(SuccessResponse {
                                    result: serde_json::json!({"error": format!("serialization failed: {e}")}),
                                    id,
                                    trace_id: None,
                                })).unwrap_or_default()
                            });
                            let _ = stream.write_all(response_json.as_bytes()).await;
                            let _ = stream.write_all(b"\n").await;
                        }
                        Err(e) => {
                            tracing::warn!("failed to parse request: {e}");
                            let error_response = serde_json::json!({
                                "error": {"code": -32700, "message": format!("Parse error: {e}")},
                                "id": null
                            });
                            let _ = stream.write_all(error_response.to_string().as_bytes()).await;
                            let _ = stream.write_all(b"\n").await;
                        }
                    }
                } else {
                    // 自然言語メッセージ — 汎用応答を返す
                    let response = serde_json::json!({
                        "result": {"status": "acknowledged", "message": format!("{} conductor received natural language input", "conductor")},
                        "id": format!("msg_{}", uuid::Uuid::new_v4())
                    });
                    let _ = stream.write_all(response.to_string().as_bytes()).await;
                    let _ = stream.write_all(b"\n").await;
                }
            }
            Ok(_) => { /* empty read, connection closed */ }
            Err(e) => {
                tracing::warn!("failed to read from connection: {e}");
            }
        }
    }
}

impl Drop for ConductorServer {
    fn drop(&mut self) {
        let socket_path = self.registry_dir.join(format!("{}.sock", self.agent_id));
        let json_path = self.registry_dir.join(format!("{}.json", self.agent_id));
        let _ = std::fs::remove_file(&socket_path);
        let _ = std::fs::remove_file(&json_path);
    }
}

/// デフォルトのメッセージハンドラ
///
/// メソッド名に基づいてディスパッチする汎用ハンドラ。
/// 各 conductor は `add_method` でメソッドを登録するか、
/// 独自の `MessageHandler` を実装して渡すことができる。
pub struct DefaultHandler {
    conductor_id: ConductorId,
    methods: std::collections::HashMap<String, Box<dyn Fn(serde_json::Value) -> serde_json::Value + Send + Sync>>,
}

impl DefaultHandler {
    pub fn new(conductor_id: ConductorId) -> Self {
        Self {
            conductor_id,
            methods: std::collections::HashMap::new(),
        }
    }

    /// メソッドハンドラを登録する
    pub fn add_method<F>(&mut self, method: &str, handler: F)
    where
        F: Fn(serde_json::Value) -> serde_json::Value + Send + Sync + 'static,
    {
        self.methods.insert(method.to_string(), Box::new(handler));
    }
}

#[async_trait::async_trait]
impl MessageHandler for DefaultHandler {
    async fn handle_request(&self, request: Request) -> Response {
        let domain = self.conductor_id.peer_name();

        if let Some(handler) = self.methods.get(&request.method) {
            let result = handler(request.params);
            Response::Success(SuccessResponse {
                result,
                id: request.id,
                trace_id: request.trace_id,
            })
        } else {
            // メソッドが見つからない場合、ドメイン内の共通応答を返す
            let method = &request.method;
            if method.starts_with(domain) || method.starts_with(&format!("{domain}.")) {
                Response::Success(SuccessResponse {
                    result: serde_json::json!({
                        "status": "ok",
                        "method": method,
                        "domain": domain,
                    }),
                    id: request.id,
                    trace_id: request.trace_id,
                })
            } else {
                Response::Error(crate::message::ErrorResultResponse {
                    error: crate::error::ErrorResponse {
                        code: -32601,
                        message: format!("Method not found: {method}"),
                        data: None,
                    },
                    id: request.id,
                    trace_id: request.trace_id,
                })
            }
        }
    }
}