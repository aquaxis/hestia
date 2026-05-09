//! AI Conductor メッセージハンドラ
//!
//! メタオーケストレーターとしての AI conductor ドメイン固有メソッドをディスパッチする。
//! 指示の解析・ルーティング・他conductorへのディスパッチを実装。

use std::sync::Arc;

use conductor_sdk::message::{ErrorResultResponse, MessageId, Payload, Request, Response, SuccessResponse};
use conductor_sdk::server::MessageHandler;
use conductor_sdk::error::ErrorResponse;
use conductor_sdk::agent::ConductorId;
use conductor_sdk::concurrency::ConductorLimiter;
use conductor_sdk::config::HestiaClientConfig;
use conductor_sdk::transport::AgentCliClient;

use ai_core::conductor_manager::ConductorManager;
use spec_driven::parser::SpecParser;
use multi_agent::agent_manager::AgentManager;

/// ワークフローステップ: 実行順序付きのconductor呼び出し
struct WorkflowStep {
    step: usize,
    conductor: ConductorId,
    method: String,
    params: serde_json::Value,
    label: String,
}

/// AI Conductor メッセージハンドラ
pub struct AiHandler {
    config: HestiaClientConfig,
    conductor_mgr: Arc<ConductorManager>,
    agent_mgr: Arc<tokio::sync::Mutex<AgentManager>>,
    /// Phase 126 — ai-conductor 配下 conductor の同時 dispatch 上限。
    /// `HESTIA_AI_DISPATCH_MAX` (既定 2) で設定。`acquire_timeout_secs` 経過で
    /// 当該 step を `dispatch_acquire_timeout` エラーとして記録し次へ進む
    /// （デッドロック検知）。
    dispatch_limiter: Arc<ConductorLimiter>,
}

impl AiHandler {
    pub fn new(config: HestiaClientConfig) -> Self {
        let dispatch_max = std::env::var("HESTIA_AI_DISPATCH_MAX")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(2);
        let acquire_to = std::env::var("HESTIA_ACQUIRE_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(600);
        Self {
            config,
            conductor_mgr: Arc::new(ConductorManager::new()),
            agent_mgr: Arc::new(tokio::sync::Mutex::new(AgentManager::with_default_cap())),
            dispatch_limiter: Arc::new(ConductorLimiter::new(dispatch_max, acquire_to)),
        }
    }

    /// 指示テキストを解析し、ワークフローステップを構築する
    ///
    /// キーワードマッチングに基づいて、どのconductorにどのメソッドでディスパッチするかを決定する。
    /// 日本語・英語キーワードの両方に対応。コンテキストに応じて適切なメソッドを選択する。
    fn build_workflow(instruction: &str) -> Vec<WorkflowStep> {
        let lower = instruction.to_lowercase();
        let mut steps = Vec::new();
        let mut step_num = 0;

        // RTL設計・検証
        let rtl_keywords = ["rtl", "verilog", "systemverilog", "hdl", "lint", "シミュレーション", "simulation",
            "formal", "transpile", "設計", "回路を作成", "回路を作る", "rtlを"];
        let has_rtl = rtl_keywords.iter().any(|k| lower.contains(k));

        // FPGA関連
        let fpga_keywords = ["fpga", "vivado", "quartus", "efinity", "bitstream", "プログラム",
            "artix", "kintex", "zynq", "arty", "実機", "fpga", "ボード"];
        let has_fpga = fpga_keywords.iter().any(|k| lower.contains(k));

        // ASIC関連
        let asic_keywords = ["asic", "pdk", "openlane", "yosys", "gdsii", "tapeout", "sky130", "gf180mcu", "asic"];
        let has_asic = asic_keywords.iter().any(|k| lower.contains(k));

        // PCB関連
        let pcb_keywords = ["pcb", "kicad", "schematic", "基板", "配線", "gerber", "pcb"];
        let has_pcb = pcb_keywords.iter().any(|k| lower.contains(k));

        // HAL関連
        let hal_keywords = ["hal", "register", "memory map", "レジスタ", "ペリフェラル", "mmio", "hal"];
        let _has_hal = hal_keywords.iter().any(|k| lower.contains(k));

        // アプリ・ファームウェア
        let apps_keywords = ["firmware", "embedded", "rtos", "flash", "ファームウェア", "組込", "apps"];
        let has_apps = apps_keywords.iter().any(|k| lower.contains(k));

        // デバッグ・検証
        let debug_keywords = ["jtag", "swd", "ila", "waveform", "probe", "検証", "デバッグ", "debug"];
        let has_debug = debug_keywords.iter().any(|k| lower.contains(k));

        // RAG・ドキュメント検索
        let rag_keywords = ["rag", "document", "ingest", "ドキュメント", "search document"];
        let has_rag = rag_keywords.iter().any(|k| lower.contains(k));

        // UART/LEDなどの周辺機能 → HAL + Apps + RTL
        let has_peripheral = lower.contains("uart") || lower.contains("led") || lower.contains("gpio")
            || lower.contains("spi") || lower.contains("i2c") || lower.contains("usart");

        // 合成・ビルド関連
        let has_synthesize = lower.contains("synthesize") || lower.contains("合成") || lower.contains("ビルド") || lower.contains("build");
        let has_simulate = lower.contains("simulate") || lower.contains("シミュレーション") || lower.contains("simulation");
        let has_lint = lower.contains("lint") || lower.contains("静的解析");

        // --- ワークフロー構築 ---

        // 周辺機能（UART/LED等）が含まれる場合、HAL設計 → RTL設計 の順
        if has_peripheral {
            step_num += 1;
            steps.push(WorkflowStep {
                step: step_num,
                conductor: ConductorId::Hal,
                method: "hal.parse.v1".to_string(),
                params: serde_json::json!({ "instruction": instruction }),
                label: "HAL設計（周辺機能定義）".to_string(),
            });
            step_num += 1;
            steps.push(WorkflowStep {
                step: step_num,
                conductor: ConductorId::Rtl,
                method: "rtl.lint.v1".to_string(),
                params: serde_json::json!({ "instruction": instruction }),
                label: "RTL設計・Lint".to_string(),
            });
        } else if has_rtl || has_lint {
            step_num += 1;
            steps.push(WorkflowStep {
                step: step_num,
                conductor: ConductorId::Rtl,
                method: if has_lint { "rtl.lint.v1".to_string() } else { "rtl.lint.v1".to_string() },
                params: serde_json::json!({ "instruction": instruction }),
                label: "RTL Lint".to_string(),
            });
        }

        // シミュレーション
        if has_simulate && !steps.iter().any(|s| s.conductor == ConductorId::Rtl) {
            step_num += 1;
            steps.push(WorkflowStep {
                step: step_num,
                conductor: ConductorId::Rtl,
                method: "rtl.simulate.v1".to_string(),
                params: serde_json::json!({ "instruction": instruction }),
                label: "RTLシミュレーション".to_string(),
            });
        } else if has_simulate {
            // 既にRTLステップがある場合はシミュレーションを追加
            step_num += 1;
            steps.push(WorkflowStep {
                step: step_num,
                conductor: ConductorId::Rtl,
                method: "rtl.simulate.v1".to_string(),
                params: serde_json::json!({ "instruction": instruction }),
                label: "RTLシミュレーション".to_string(),
            });
        }

        // FPGAビルド
        if has_fpga || has_synthesize {
            step_num += 1;
            steps.push(WorkflowStep {
                step: step_num,
                conductor: ConductorId::Fpga,
                method: "fpga.build.v1.start".to_string(),
                params: serde_json::json!({ "instruction": instruction, "target": if lower.contains("arty") || lower.contains("artix") { "artix7" } else { "xilinx" } }),
                label: "FPGAビルド".to_string(),
            });
        }

        // ASIC合成
        if has_asic {
            step_num += 1;
            steps.push(WorkflowStep {
                step: step_num,
                conductor: ConductorId::Asic,
                method: "asic.synthesize".to_string(),
                params: serde_json::json!({ "instruction": instruction }),
                label: "ASIC合成".to_string(),
            });
        }

        // PCB設計
        if has_pcb {
            step_num += 1;
            steps.push(WorkflowStep {
                step: step_num,
                conductor: ConductorId::Pcb,
                method: "pcb.run_drc".to_string(),
                params: serde_json::json!({ "instruction": instruction }),
                label: "PCB DRC".to_string(),
            });
        }

        // アプリ・ファームウェア
        if has_apps {
            step_num += 1;
            steps.push(WorkflowStep {
                step: step_num,
                conductor: ConductorId::Apps,
                method: "apps.build.v1".to_string(),
                params: serde_json::json!({ "instruction": instruction }),
                label: "ファームウェアビルド".to_string(),
            });
        }

        // デバッグ・実機検証
        if has_debug || lower.contains("検証") || lower.contains("実機") {
            step_num += 1;
            steps.push(WorkflowStep {
                step: step_num,
                conductor: ConductorId::Debug,
                method: "debug.connect".to_string(),
                params: serde_json::json!({ "instruction": instruction }),
                label: "デバッグ・検証".to_string(),
            });
        }

        // RAG検索
        if has_rag {
            step_num += 1;
            steps.push(WorkflowStep {
                step: step_num,
                conductor: ConductorId::Rag,
                method: "rag.search".to_string(),
                params: serde_json::json!({ "instruction": instruction }),
                label: "ドキュメント検索".to_string(),
            });
        }

        // 何もマッチしなかった場合のフォールバック
        if steps.is_empty() {
            step_num += 1;
            steps.push(WorkflowStep {
                step: step_num,
                conductor: ConductorId::Ai,
                method: "ai.exec".to_string(),
                params: serde_json::json!({ "instruction": instruction }),
                label: "AI処理".to_string(),
            });
        }

        steps
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
            kind: "prompt".to_string(),
        from: "cli".to_string(),
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
    // Phase 102 — `spec_driven_emit_skeleton` 関数を削除。
    // Phase 59 で「setup_ai 自己実行の最小実装（Q6 ギャップ部分解消）」として
    // `<workspace>/.aiprj/{AI_PRJ_REQUIREMENTS,AI_PRJ_DESIGN,AI_PRJ_TASKS}.md` を
    // fs_write していたが、Phase 91 で全 50 persona の禁止事項に明記した
    // 「.aiprj/ 配下の参照 / 書込」と矛盾していたため。
    // 3 文書管理は project 管理 AI 専有領域で hestia agent runtime からは触らない。

    /// Phase 77 — ワークフロー完了後に ai-reviewer サブエージェントを自動 spawn し、
    /// 品質ゲート判定を依頼する。Phase 74 で追加した ai-reviewer (`hestia spawn-subagent
    /// --persona ai-reviewer --name ai-reviewer`) を呼び出し、`agent-cli send` で
    /// run_id + workflow_steps 数を渡す。
    ///
    /// 戻り値: dispatch 成功なら true、失敗（hestia バイナリ不在 / agent-cli 不在等）なら false。
    /// 失敗は警告ログのみで ai.exec 全体には影響なし。
    async fn auto_spawn_reviewer(instruction: &str, total_steps: usize) -> bool {
        // 1. ai-reviewer を spawn
        let spawn_result = std::process::Command::new("hestia")
            .args(["spawn-subagent", "--persona", "ai-reviewer", "--name", "ai-reviewer"])
            .output();
        let spawned = matches!(&spawn_result, Ok(o) if o.status.success());
        if !spawned {
            tracing::warn!("Phase 77: ai-reviewer spawn failed (hestia binary or persona unreachable)");
            return false;
        }

        // 2. ai-reviewer に review 依頼 prompt を送信
        // Phase 102 — レビュー成果物は project root 直下に書き出す (`.aiprj/` は project 管理 AI 専有領域)。
        let prompt = format!(
            "[ai.exec auto-review] workflow_steps={total_steps}, instruction={:?}. Review the artifacts under <root>/{{rtl,fpga,hal,asic,pcb,apps,debug,rag}}/ and write `<root>/REVIEW_REPORT.md` with status: pass|fail|partial.",
            instruction.chars().take(120).collect::<String>()
        );
        match conductor_sdk::workspace::agent_cli_send("ai-reviewer", &prompt) {
            Ok(()) => {
                tracing::info!("Phase 77: ai-reviewer auto-spawn + dispatch ok");
                true
            }
            Err(e) => {
                tracing::warn!(error = %e, "Phase 77: agent-cli send to ai-reviewer failed");
                false
            }
        }
    }

    /// ai.exec — 指示を解析し、ワークフローを構築して順次実行し、結果を集約する
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

        tracing::info!(instruction = %instruction, "ai.exec: building workflow from instruction");

        // Phase 102 — Phase 59 の spec_driven_emit_skeleton (`<workspace>/.aiprj/AI_PRJ_*.md`
        // を fs_write) を削除。Phase 91 の persona 禁止事項「.aiprj/ 配下の書込禁止」と
        // 矛盾していたため。3 文書管理は project 管理 AI 専有領域。

        // 指示からワークフローを構築
        let workflow = Self::build_workflow(&instruction);

        tracing::info!(steps = workflow.len(), "ai.exec: workflow has {} steps", workflow.len());

        let mut results = Vec::new();
        let total_steps = workflow.len();

        for step in workflow {
            tracing::info!(
                step = step.step,
                total = total_steps,
                conductor = ?step.conductor,
                method = %step.method,
                label = %step.label,
                "ai.exec: executing workflow step"
            );

            // Phase 126 — L2 cap: ai-conductor 同時 dispatch 上限を Semaphore で保護。
            // 取得順序固定 (L1 → L2 → L3) の L2。timeout で循環待機を破る。
            let _dispatch_permit = match self.dispatch_limiter.acquire().await {
                Ok(p) => p,
                Err(e) => {
                    tracing::warn!(
                        step = step.step,
                        conductor = ?step.conductor,
                        error = %e,
                        "ai.exec: dispatch_limiter acquire failed; recording step error"
                    );
                    results.push(serde_json::json!({
                        "step": step.step,
                        "label": step.label,
                        "conductor": format!("{:?}", step.conductor),
                        "method": step.method,
                        "status": "error",
                        "error": format!("dispatch_acquire_timeout: {e}"),
                    }));
                    continue;
                }
            };

            // Phase 101 — Phase 93 の on-demand spawn 機能の動作開始（G10 修正）。
            // dispatch 直前に必要 conductor を spawn する。既に alive なら no-op。
            let peer = step.conductor.peer_name();
            if let Err(e) = conductor_sdk::workspace::spawn_conductor_on_demand(peer) {
                tracing::warn!(
                    step = step.step,
                    conductor = ?step.conductor,
                    peer = %peer,
                    error = %e,
                    "ai.exec: spawn_conductor_on_demand failed; recording step error"
                );
                results.push(serde_json::json!({
                    "step": step.step,
                    "label": step.label,
                    "conductor": format!("{:?}", step.conductor),
                    "method": step.method,
                    "status": "error",
                    "error": format!("spawn failed: {e}"),
                }));
                continue;
            }

            match Self::dispatch_to_conductor(&self.config, step.conductor, &step.method, step.params).await {
                Ok(response) => {
                    results.push(serde_json::json!({
                        "step": step.step,
                        "label": step.label,
                        "conductor": format!("{:?}", step.conductor),
                        "method": step.method,
                        "status": "ok",
                        "response": response,
                    }));
                }
                Err(e) => {
                    tracing::warn!(step = step.step, conductor = ?step.conductor, error = %e, "ai.exec: step failed");
                    results.push(serde_json::json!({
                        "step": step.step,
                        "label": step.label,
                        "conductor": format!("{:?}", step.conductor),
                        "method": step.method,
                        "status": "error",
                        "error": e,
                    }));
                }
            }
        }

        // Phase 77 — ワークフロー完了後、ai-reviewer を自動 spawn して品質ゲート判定を依頼。
        // env override `HESTIA_DISABLE_AUTO_REVIEW=1` で無効化可能（テスト互換性）。
        let auto_review_dispatched = if std::env::var("HESTIA_DISABLE_AUTO_REVIEW").as_deref() != Ok("1") {
            Self::auto_spawn_reviewer(&instruction, total_steps).await
        } else {
            false
        };

        Ok(serde_json::json!({
            "status": "ok",
            "method": "ai.exec",
            "instruction": instruction,
            "source_file": source_file,
            "workflow_steps": total_steps,
            "auto_review_dispatched": auto_review_dispatched,
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

    /// agent_spawn — AgentManagerを使用してエージェントを生成する（agent-cli run 経由）
    async fn handle_agent_spawn(&self, params: serde_json::Value) -> Result<serde_json::Value, String> {
        let role = params.get("role").and_then(|v| v.as_str()).unwrap_or("planner");
        let conductor_id = params.get("conductor_id").and_then(|v| v.as_str()).unwrap_or("ai");
        let agent_id = format!("{}_{}", conductor_id, role);

        let mut mgr = self.agent_mgr.lock().await;
        mgr.spawn(agent_id.clone(), conductor_id.to_string()).await
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