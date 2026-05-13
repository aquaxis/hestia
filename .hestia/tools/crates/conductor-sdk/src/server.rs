//! Conductor server-side transport
//!
//! Registration to agent-cli registry, Unix socket listener, and message receive loop

use crate::agent::ConductorId;
use crate::error::HestiaError;
use crate::message::{Request, Response, SuccessResponse};
use std::path::PathBuf;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixListener;
use tokio::signal;

/// Message handler trait
///
/// Each Conductor implements this trait to handle requests from the CLI.
#[async_trait::async_trait]
pub trait MessageHandler: Send + Sync {
    /// Process a structured request and return a response
    async fn handle_request(&self, request: Request) -> Response;
}

/// agent-cli registry entry
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

/// Persona information
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PersonaInfo {
    pub role: String,
    pub skills: Vec<String>,
    pub description: String,
    pub source_path: Option<String>,
}

/// Persona file YAML frontmatter
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct PersonaFrontmatter {
    name: Option<String>,
    role: String,
    #[serde(default)]
    skills: Vec<String>,
    #[serde(default)]
    description: Option<String>,
}

/// Load persona file
fn load_persona(domain: &str) -> Option<(PersonaFrontmatter, PathBuf)> {
    let persona_path = PathBuf::from(format!(".hestia/personas/{domain}.md"));
    if !persona_path.exists() {
        return None;
    }
    let content = std::fs::read_to_string(&persona_path).ok()?;
    let content = content.trim_start();
    if !content.starts_with("---") {
        return None;
    }
    let end = content[3..].find("---")?;
    let frontmatter_str = &content[3..3 + end];
    let frontmatter: PersonaFrontmatter = serde_yaml::from_str(frontmatter_str).ok()?;
    Some((frontmatter, persona_path))
}

/// Conductor server
///
/// Registers itself with the agent-cli registry and listens for messages on a Unix socket.
pub struct ConductorServer {
    conductor_id: ConductorId,
    registry_dir: PathBuf,
    agent_id: String,
    handler: Box<dyn MessageHandler>,
    provider: String,
    model: String,
}

impl ConductorServer {
    /// Create a new ConductorServer
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
            provider: "ollama".to_string(),
            model: "glm-5.1:cloud".to_string(),
        })
    }

    /// Customize registry directory
    pub fn with_registry_dir(mut self, dir: PathBuf) -> Self {
        self.registry_dir = dir;
        self
    }

    /// Set provider and model
    pub fn with_provider(mut self, provider: &str, model: &str) -> Self {
        self.provider = provider.to_string();
        self.model = model.to_string();
        self
    }

    /// Start the server and enter the message receive loop
    pub async fn run(self) -> Result<(), HestiaError> {
        // Create registry directory
        std::fs::create_dir_all(&self.registry_dir).map_err(|e| {
            HestiaError::Transport(format!("failed to create registry dir: {e}"))
        })?;

        let socket_path = self.registry_dir.join(format!("{}.sock", self.agent_id));
        let json_path = self.registry_dir.join(format!("{}.json", self.agent_id));

        // Remove existing socket if present
        if socket_path.exists() {
            std::fs::remove_file(&socket_path)?;
        }

        // Create Unix socket listener
        let listener = UnixListener::bind(&socket_path).map_err(|e| {
            HestiaError::Transport(format!("failed to bind socket {}: {e}", socket_path.display()))
        })?;

        // Set socket permissions to 0600
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o600))?;
        }

        // Read information from persona file
        let domain = self.conductor_id.peer_name();
        let (persona_info, _persona_source_path) = match load_persona(domain) {
            Some((fm, path)) => {
                tracing::info!("loaded persona from {}", path.display());
                (
                    PersonaInfo {
                        role: fm.role,
                        skills: fm.skills,
                        description: fm.description.unwrap_or_default(),
                        source_path: Some(path.to_string_lossy().to_string()),
                    },
                    Some(path),
                )
            }
            None => (
                PersonaInfo {
                    role: format!("{domain} conductor"),
                    skills: vec!["tool execution".to_string()],
                    description: format!("Hestia {domain} conductor daemon"),
                    source_path: None,
                },
                None,
            ),
        };

        // Write registry entry
        let entry = RegistryEntry {
            id: self.agent_id.clone(),
            name: domain.to_string(),
            pid: std::process::id(),
            started_at: chrono::Utc::now().to_rfc3339(),
            provider: self.provider.clone(),
            model: self.model.clone(),
            socket: socket_path.to_string_lossy().to_string(),
            persona: persona_info,
        };

        let json_content = serde_json::to_string_pretty(&entry)
            .map_err(|e| HestiaError::Transport(format!("failed to serialize registry entry: {e}")))?;
        std::fs::write(&json_path, &json_content)?;

        tracing::info!(
            "conductor {} registered at {} (socket: {})",
            domain,
            json_path.display(),
            socket_path.display()
        );

        // Set up cleanup for graceful shutdown
        let reg_dir = self.registry_dir.clone();
        let aid = self.agent_id.clone();
        let cleanup = move || {
            let sp = reg_dir.join(format!("{aid}.sock"));
            let jp = reg_dir.join(format!("{aid}.json"));
            let _ = std::fs::remove_file(&sp);
            let _ = std::fs::remove_file(&jp);
        };

        // Message receive loop
        tracing::info!("{} conductor ready, listening for messages", domain);

        let conductor_name = domain.to_string();

        loop {
            tokio::select! {
                accept_result = listener.accept() => {
                    match accept_result {
                        Ok((stream, _addr)) => {
                            let handler = &self.handler;
                            let name = conductor_name.clone();
                            Self::handle_connection(stream, handler, &name).await;
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

    /// Handle an individual connection
    ///
    /// Expects incoming messages in agent-cli's `IpcMessage::Prompt` compatible wrapper
    /// `{"kind":"prompt","from":"...","text":"..."}`. If the `text` content is a JSON object,
    /// decode it as hestia's `Request` and pass it to the handler; otherwise, route it
    /// as natural language to the LLM path.
    async fn handle_connection(
        mut stream: tokio::net::UnixStream,
        handler: &Box<dyn MessageHandler>,
        conductor_name: &str,
    ) {
        let mut buf = vec![0u8; 16 * 1024 * 1024]; // 16 MiB max
        match stream.read(&mut buf).await {
            Ok(n) if n > 0 => {
                let raw = String::from_utf8_lossy(&buf[..n]);
                let raw_str = raw.trim();

                // Extract `text` from agent-cli wire format. On failure, treat raw as text.
                let inner = Self::extract_text(raw_str);
                let inner_trim = inner.trim();

                if inner_trim.starts_with('{') {
                    match serde_json::from_str::<Request>(inner_trim) {
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
                    // Natural language message — routed to LLM via agent-cli send
                    let llm_response = Self::forward_to_llm(conductor_name, inner_trim).await;
                    let response = match llm_response {
                        Some(resp) => serde_json::json!({
                            "result": {"status": "processed", "response": resp},
                            "id": format!("msg_{}", uuid::Uuid::new_v4())
                        }),
                        None => serde_json::json!({
                            "result": {"status": "acknowledged", "message": format!("{conductor_name} conductor received natural language input")},
                            "id": format!("msg_{}", uuid::Uuid::new_v4())
                        }),
                    };
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

    /// Extract the `text` field from agent-cli wire format.
    ///
    /// If the input is in `{"kind":"prompt","from":"...","text":"..."}` format, return `text`;
    /// otherwise (legacy format, pure natural language, direct JSON send), return the input as-is.
    fn extract_text(raw: &str) -> String {
        if !raw.trim_start().starts_with('{') {
            return raw.to_string();
        }
        match serde_json::from_str::<serde_json::Value>(raw) {
            Ok(v) => {
                let kind = v.get("kind").and_then(|k| k.as_str()).unwrap_or("");
                if kind == "prompt" {
                    if let Some(t) = v.get("text").and_then(|t| t.as_str()) {
                        return t.to_string();
                    }
                }
                raw.to_string()
            }
            Err(_) => raw.to_string(),
        }
    }

    /// Forward natural language messages to LLM via engine binary (Phase 113).
    async fn forward_to_llm(conductor_name: &str, text: &str) -> Option<String> {
        let bin = crate::workspace::engine_binary();
        let output = tokio::process::Command::new(&bin)
            .arg("send")
            .arg(conductor_name)
            .arg(text)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::null())
            .output()
            .await
            .ok()?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let trimmed = stdout.trim();
            if !trimmed.is_empty() {
                return Some(trimmed.to_string());
            }
        }
        None
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

/// Default message handler
///
/// Generic handler that dispatches based on method name.
/// Each conductor can register methods with `add_method` or provide its own
/// `MessageHandler` implementation.
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

    /// Register a method handler
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
            // When method is not found, return a common response within the domain
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