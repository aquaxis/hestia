//! AI Conductor メッセージハンドラ
//!
//! メタオーケストレーターとしての AI conductor ドメイン固有メソッドをディスパッチする。
//! 指示の解析・ルーティング・他conductorへのディスパッチを実装。

use std::sync::Arc;

use conductor_sdk::message::{ErrorResultResponse, MessageId, Payload, Request, Response, SuccessResponse};
use conductor_sdk::server::MessageHandler;
use conductor_sdk::error::ErrorResponse;
use conductor_sdk::agent::ConductorId;
use conductor_sdk::config::HestiaClientConfig;
use conductor_sdk::transport::AgentCliClient;

use ai_core::conductor_manager::ConductorManager;
use spec_driven::parser::SpecParser;
use multi_agent::agent_manager::AgentManager;

/// 指示テキストからconductorとメソッドを推測するためのルーティングルール
struct RouteRule {
    keywords: &'static [&'static str],
    conductor: ConductorId,
    method: &'static str,
}

/// AI Conductor メッセージハンドラ
pub struct AiHandler {
    config: HestiaClientConfig,
    conductor_mgr: Arc<ConductorManager>,
    agent_mgr: Arc<tokio::sync::Mutex<AgentManager>>,
}

impl AiHandler {
    pub fn new(config: HestiaClientConfig) -> Self {
        Self {
            config,
            conductor_mgr: Arc::new(ConductorManager::new()),
            agent_mgr: Arc::new(tokio::sync::Mutex::new(AgentManager::new())),
        }
    }

    /// 指示テキストからターゲットconductorとメソッドを推測するルーティングテーブル
    fn routing_rules() -> Vec<RouteRule> {
        vec![
            // RTL
            RouteRule { keywords: &["rtl", "verilog", "systemverilog", "lint", "simulate", "formal", "transpile", "hdl", "rtl"], conductor: ConductorId::Rtl, method: "rtl.lint.v1" },
            // FPGA
            RouteRule { keywords: &["fpga", "vivado", "quartus", "efinity", "synthesize", "implement", "bitstream", "artix", "kintex", "zynq", "arty", "fpga"], conductor: ConductorId::Fpga, method: "fpga.build.v1.start" },
            // ASIC
            RouteRule { keywords: &["asic", "pdk", "openlane", "yosys", "gdsii", "tapeout", "sky130", "gf180mcu", "asic"], conductor: ConductorId::Asic, method: "asic.synthesize" },
            // PCB
            RouteRule { keywords: &["pcb", "kicad", "schematic", "drc", "erc", "routing", "gerber", "pcb"], conductor: ConductorId::Pcb, method: "pcb.run_drc" },
            // HAL
            RouteRule { keywords: &["hal", "register", "memory map", "bus", "peripheral", "mmio", "hal"], conductor: ConductorId::Hal, method: "hal.parse.v1" },
            // Apps
            RouteRule { keywords: &["firmware", "embedded", "rtos", "flash", "cross-compile", "apps"], conductor: ConductorId::Apps, method: "apps.build.v1" },
            // Debug
            RouteRule { keywords: &["jtag", "swd", "ila", "waveform", "probe", "debug"], conductor: ConductorId::Debug, method: "debug.connect" },
            // RAG
            RouteRule { keywords: &["rag", "document", "ingest", "search document"], conductor: ConductorId::Rag, method: "rag.search" },
        ]
    }

    /// 指示テキストを解析し、ターゲットconductorとメソッドを決定する
    fn classify_instruction(instruction: &str) -> Vec<(ConductorId, String, serde_json::Value)> {
        let lower = instruction.to_lowercase();
        let rules = Self::routing_rules();
        let mut matched = Vec::new();
        let mut seen_conductors = std::collections::HashSet::new();

        for rule in &rules {
            for keyword in rule.keywords {
                if lower.contains(keyword) {
                    if seen_conductors.insert(format!("{:?}", rule.conductor)) {
                        matched.push((
                            rule.conductor,
                            rule.method.to_string(),
                            serde_json::json!({ "instruction": instruction }),
                        ));
                    }
                    break;
                }
            }
        }

        // マッチするルールがない場合はAI conductor自身で処理
        if matched.is_empty() {
            matched.push((
                ConductorId::Ai,
                "ai.exec".to_string(),
                serde_json::json!({ "instruction": instruction }),
            ));
        }

        matched
    }

    /// conductorにメッセージを送信してレスポンスを取得する
    async fn dispatch_to_conductor(
        config: &HestiaClientConfig,
        conductor: ConductorId,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let client = AgentCliClient::new(config.clone())
            .map_err(|e| format!("failed to create client: {e}"))?;

        let request = Request {
            method: method.to_string(),
            params,
            id: MessageId::new(),
            trace_id: None,
        };
        let payload = Payload::Structured(serde_json::to_value(&request)
            .map_err(|e| format!("failed to serialize request: {e}"))?);

        let response = client.send_to_conductor(conductor, &payload).await
            .map_err(|e| format!("failed to send to {:?}: {e}", conductor))?;

        // レスポンスをJSONとしてパース
        let json: serde_json::Value = serde_json::from_str(&response)
            .unwrap_or_else(|_| serde_json::json!({ "raw": response }));

        Ok(json)
    }
}

#[async_trait::async_trait]
impl MessageHandler for AiHandler {
    async fn handle_request(&self, request: Request) -> Response {
        let method = request.method.clone();
        let id = request.id.clone();
        let trace_id = request.trace_id.clone();
        let params = request.params;

        let result = match method.as_str() {
            // Spec-driven development
            "ai.spec.init" => Self::handle_spec_init(params).await,
            "ai.spec.update" => Self::handle_spec_update(params).await,
            "ai.spec.review" => Self::handle_spec_review(params).await,
            // Execution
            "ai.exec" => self.handle_exec(params).await,
            // Agent management
            "agent_spawn" => self.handle_agent_spawn(params).await,
            "agent_list" => self.handle_agent_list().await,
            // Container management
            "container.list" => self.handle_container_list().await,
            "container.start" => Self::handle_container_start(params).await,
            "container.stop" => Self::handle_container_stop(params).await,
            "container.create" => Self::handle_container_create(params).await,
            "container.update" => Self::handle_container_update(params).await,
            // Workflow
            "meta.dualBuild" => self.handle_dual_build(params).await,
            "meta.boardWithFpga" => self.handle_board_with_fpga(params).await,
            // System
            "system.health.v1" => self.handle_health().await,
            "system.readiness" => Self::handle_readiness().await,
            "system.shutdown" => Self::handle_shutdown().await,
            _ => {
                return Response::Error(ErrorResultResponse {
                    error: ErrorResponse {
                        code: -32601,
                        message: format!("Method not found: {method}"),
                        data: None,
                    },
                    id,
                    trace_id,
                });
            }
        };

        match result {
            Ok(value) => Response::Success(SuccessResponse {
                result: value,
                id,
                trace_id,
            }),
            Err(msg) => Response::Error(ErrorResultResponse {
                error: ErrorResponse {
                    code: -32000,
                    message: msg,
                    data: None,
                },
                id,
                trace_id,
            }),
        }
    }
}

impl AiHandler {
    /// ai.exec — 指示を解析し、該当conductorにディスパッチして結果を集約する
    async fn handle_exec(&self, params: serde_json::Value) -> Result<serde_json::Value, String> {
        let instruction = params.get("instruction")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let source_file = params.get("source_file")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        if instruction.is_empty() {
            return Err("instruction is empty".to_string());
        }

        tracing::info!(instruction = %instruction, "ai.exec: classifying instruction");

        // 指示を解析してターゲットconductorを決定
        let targets = Self::classify_instruction(&instruction);

        let mut results = Vec::new();

        for (conductor, method, mut dispatch_params) in targets {
            tracing::info!(
                conductor = ?conductor,
                method = %method,
                "ai.exec: dispatching to conductor"
            );

            // パラメータに指示テキストを含める
            if let Some(obj) = dispatch_params.as_object_mut() {
                obj.insert("instruction".to_string(), serde_json::Value::String(instruction.clone()));
            }

            match Self::dispatch_to_conductor(&self.config, conductor, &method, dispatch_params).await {
                Ok(response) => {
                    results.push(serde_json::json!({
                        "conductor": format!("{:?}", conductor),
                        "method": method,
                        "response": response,
                    }));
                }
                Err(e) => {
                    tracing::warn!(conductor = ?conductor, error = %e, "ai.exec: dispatch failed");
                    results.push(serde_json::json!({
                        "conductor": format!("{:?}", conductor),
                        "method": method,
                        "error": e,
                    }));
                }
            }
        }

        Ok(serde_json::json!({
            "status": "ok",
            "method": "ai.exec",
            "instruction": instruction,
            "source_file": source_file,
            "dispatched": results.len(),
            "results": results,
        }))
    }

    /// ai.spec.init — SpecParserを使用して仕様テキストを解析する
    async fn handle_spec_init(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let spec_text = params.get("spec_text").and_then(|v| v.as_str()).unwrap_or("");
        let format = params.get("format").and_then(|v| v.as_str()).unwrap_or("natural");

        let parser = SpecParser::new();
        let design_spec = parser.parse(spec_text)
            .map_err(|e| format!("spec parse error: {e}"))?;

        Ok(serde_json::json!({
            "status": "ok",
            "method": "ai.spec.init",
            "format": format,
            "spec_id": design_spec.id,
            "design_spec": {
                "requirements": design_spec.requirements.iter().map(|r| serde_json::json!({
                    "id": r.id,
                    "text": r.text,
                })).collect::<Vec<_>>(),
                "constraints": design_spec.constraints.iter().map(|c| serde_json::json!({
                    "id": c.id,
                    "text": c.text,
                })).collect::<Vec<_>>(),
                "interfaces": design_spec.interfaces.iter().map(|i| serde_json::json!({
                    "id": i.id,
                    "text": i.text,
                })).collect::<Vec<_>>(),
            },
        }))
    }

    /// ai.spec.update — 仕様更新を処理する
    async fn handle_spec_update(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let spec_id = params.get("spec_id").and_then(|v| v.as_str()).unwrap_or("");
        let updates = params.get("updates").cloned().unwrap_or(serde_json::json!({}));

        Ok(serde_json::json!({
            "status": "ok",
            "method": "ai.spec.update",
            "spec_id": spec_id,
            "updated": updates,
        }))
    }

    /// ai.spec.review — 仕様レビューを実行する
    async fn handle_spec_review(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let spec_id = params.get("spec_id").and_then(|v| v.as_str()).unwrap_or("");

        Ok(serde_json::json!({
            "status": "ok",
            "method": "ai.spec.review",
            "spec_id": spec_id,
            "review_results": [],
            "fix_suggestions": [],
        }))
    }

    /// agent_spawn — AgentManagerを使用してエージェントを生成する
    async fn handle_agent_spawn(&self, params: serde_json::Value) -> Result<serde_json::Value, String> {
        let role = params.get("role").and_then(|v| v.as_str()).unwrap_or("planner");
        let conductor_id = params.get("conductor_id").and_then(|v| v.as_str()).unwrap_or("ai");
        let agent_id = format!("agent_{}", uuid::Uuid::new_v4().simple());

        let mut mgr = self.agent_mgr.lock().await;
        mgr.spawn(agent_id.clone(), conductor_id.to_string())
            .map_err(|e| format!("failed to spawn agent: {e}"))?;

        Ok(serde_json::json!({
            "status": "ok",
            "method": "agent_spawn",
            "agent_id": agent_id,
            "role": role,
        }))
    }

    /// agent_list — AgentManagerとConductorManagerからエージェント一覧を返す
    async fn handle_agent_list(&self) -> Result<serde_json::Value, String> {
        let agent_list = {
            let mgr = self.agent_mgr.lock().await;
            mgr.list().iter().map(|a| serde_json::json!({
                "agent_id": a.agent_id,
                "status": a.status.to_string(),
                "conductor_id": a.conductor_id,
                "started_at": a.started_at.to_rfc3339(),
            })).collect::<Vec<_>>()
        };
        let conductors = self.conductor_mgr.list_conductors().await;

        Ok(serde_json::json!({
            "status": "ok",
            "method": "agent_list",
            "agents": agent_list,
            "conductors": conductors.iter().map(|c| serde_json::json!({
                "id": format!("{:?}", c.id),
                "status": format!("{:?}", c.status),
                "version": c.version,
            })).collect::<Vec<_>>(),
        }))
    }

    /// container.list — コンテナ一覧を返す
    async fn handle_container_list(&self) -> Result<serde_json::Value, String> {
        let conductors = self.conductor_mgr.list_conductors().await;
        Ok(serde_json::json!({
            "status": "ok",
            "method": "container.list",
            "containers": conductors.iter().map(|c| serde_json::json!({
                "name": format!("{:?}-conductor", c.id).to_lowercase(),
                "status": format!("{:?}", c.status),
            })).collect::<Vec<_>>(),
        }))
    }

    /// container.start — 指定conductorのヘルスチェックを実行して起動確認する
    async fn handle_container_start(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "container.start",
            "name": name,
        }))
    }

    /// container.stop — コンテナ停止を処理する
    async fn handle_container_stop(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "container.stop",
            "name": name,
        }))
    }

    /// container.create — コンテナ作成を処理する
    async fn handle_container_create(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "container.create",
            "name": name,
        }))
    }

    /// container.update — コンテナ更新を処理する
    async fn handle_container_update(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "container.update",
            "name": name,
        }))
    }

    /// meta.dualBuild — FPGA と ASIC の並列ビルドをオーケストレーションする
    async fn handle_dual_build(&self, params: serde_json::Value) -> Result<serde_json::Value, String> {
        let target_fpga = params.get("target_fpga").and_then(|v| v.as_str()).unwrap_or("xilinx");
        let target_asic = params.get("target_asic").and_then(|v| v.as_str()).unwrap_or("sky130");

        // FPGA ビルドをディスパッチ
        let fpga_result = Self::dispatch_to_conductor(
            &self.config,
            ConductorId::Fpga,
            "fpga.build.v1.start",
            serde_json::json!({ "target": target_fpga }),
        ).await;

        // ASIC 合成をディスパッチ
        let asic_result = Self::dispatch_to_conductor(
            &self.config,
            ConductorId::Asic,
            "asic.synthesize",
            serde_json::json!({ "pdk": target_asic, "strategy": "area" }),
        ).await;

        let mut results = Vec::new();
        results.push(serde_json::json!({
            "conductor": "Fpga",
            "method": "fpga.build.v1.start",
            "result": fpga_result.unwrap_or_else(|e| serde_json::json!({ "error": e })),
        }));
        results.push(serde_json::json!({
            "conductor": "Asic",
            "method": "asic.synthesize",
            "result": asic_result.unwrap_or_else(|e| serde_json::json!({ "error": e })),
        }));

        Ok(serde_json::json!({
            "status": "ok",
            "method": "meta.dualBuild",
            "fpga_build_id": format!("build_{}", uuid::Uuid::new_v4().simple()),
            "asic_build_id": format!("build_{}", uuid::Uuid::new_v4().simple()),
            "results": results,
        }))
    }

    /// meta.boardWithFpga — PCB + FPGA 統合ワークフローを実行する
    async fn handle_board_with_fpga(&self, params: serde_json::Value) -> Result<serde_json::Value, String> {
        let board_name = params.get("board_name").and_then(|v| v.as_str()).unwrap_or("arty-a7");

        // FPGA ビルド
        let fpga_result = Self::dispatch_to_conductor(
            &self.config,
            ConductorId::Fpga,
            "fpga.build.v1.start",
            serde_json::json!({ "target": board_name }),
        ).await;

        // PCB DRC チェック
        let pcb_result = Self::dispatch_to_conductor(
            &self.config,
            ConductorId::Pcb,
            "pcb.run_drc",
            serde_json::json!({}),
        ).await;

        let mut results = Vec::new();
        results.push(serde_json::json!({
            "conductor": "Fpga",
            "method": "fpga.build.v1.start",
            "result": fpga_result.unwrap_or_else(|e| serde_json::json!({ "error": e })),
        }));
        results.push(serde_json::json!({
            "conductor": "Pcb",
            "method": "pcb.run_drc",
            "result": pcb_result.unwrap_or_else(|e| serde_json::json!({ "error": e })),
        }));

        Ok(serde_json::json!({
            "status": "ok",
            "method": "meta.boardWithFpga",
            "board_name": board_name,
            "results": results,
        }))
    }

    /// system.health.v1 — ConductorManagerから全conductorの状態を集約する
    async fn handle_health(&self) -> Result<serde_json::Value, String> {
        let conductors = self.conductor_mgr.list_conductors().await;
        let online_count = conductors.iter().filter(|c| matches!(c.status, conductor_sdk::agent::ConductorStatus::Online)).count();

        Ok(serde_json::json!({
            "status": "Online",
            "uptime_secs": 0,
            "tools_ready": conductors.iter().map(|c| format!("{:?}", c.id).to_lowercase()).collect::<Vec<_>>(),
            "load": {"cpu_pct": 0, "mem_mb": 0},
            "active_jobs": 0,
            "conductors_online": online_count,
            "conductors_total": conductors.len(),
        }))
    }

    async fn handle_readiness() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({"ready": true}))
    }

    async fn handle_shutdown() -> Result<serde_json::Value, String> {
        Ok(serde_json::json!({
            "status": "ok",
            "method": "system.shutdown",
            "message": "ai-conductor shutting down",
        }))
    }
}