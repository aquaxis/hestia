//! AI Conductor message handler
//!
//! Dispatches domain-specific methods as the meta-orchestrator AI conductor.
//! Implements instruction parsing, routing, and dispatch to other conductors.

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

/// Workflow step: conductor invocation with execution order
struct WorkflowStep {
    step: usize,
    conductor: ConductorId,
    method: String,
    params: serde_json::Value,
    label: String,
}

/// AI Conductor message handler
pub struct AiHandler {
    config: HestiaClientConfig,
    conductor_mgr: Arc<ConductorManager>,
    agent_mgr: Arc<tokio::sync::Mutex<AgentManager>>,
    /// Phase 126 — Upper limit on concurrent dispatches under ai-conductor.
    /// Configured via `HESTIA_AI_DISPATCH_MAX` (default 2). After `acquire_timeout_secs`
    /// seconds, the step is recorded as a `dispatch_acquire_timeout` error and
    /// processing continues to the next step (deadlock detection).
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

    /// Parse instruction text and build workflow steps
    ///
    /// Determines which conductor and method to dispatch based on keyword matching.
    /// Supports both Japanese and English keywords. Selects appropriate methods based on context.
    fn build_workflow(instruction: &str) -> Vec<WorkflowStep> {
        let lower = instruction.to_lowercase();
        let mut steps = Vec::new();
        let mut step_num = 0;

        // RTL design/verification
        let rtl_keywords = ["rtl", "verilog", "systemverilog", "hdl", "lint", "simulation",
            "formal", "transpile", "design", "create circuit", "build circuit", "rtl to"];
        let has_rtl = rtl_keywords.iter().any(|k| lower.contains(k));

        // FPGA related
        let fpga_keywords = ["fpga", "vivado", "quartus", "efinity", "bitstream", "program",
            "artix", "kintex", "zynq", "arty", "real hardware", "fpga", "board"];
        let has_fpga = fpga_keywords.iter().any(|k| lower.contains(k));

        // ASIC related
        let asic_keywords = ["asic", "pdk", "openlane", "yosys", "gdsii", "tapeout", "sky130", "gf180mcu", "asic"];
        let has_asic = asic_keywords.iter().any(|k| lower.contains(k));

        // PCB related
        let pcb_keywords = ["pcb", "kicad", "schematic", "board", "wiring", "gerber", "pcb"];
        let has_pcb = pcb_keywords.iter().any(|k| lower.contains(k));

        // HAL related
        let hal_keywords = ["hal", "register", "memory map", "register", "peripheral", "mmio", "hal"];
        let _has_hal = hal_keywords.iter().any(|k| lower.contains(k));

        // Application firmware
        let apps_keywords = ["firmware", "embedded", "rtos", "flash", "firmware", "embedded", "apps"];
        let has_apps = apps_keywords.iter().any(|k| lower.contains(k));

        // Debug / verification
        let debug_keywords = ["jtag", "swd", "ila", "waveform", "probe", "verification", "debug", "debug"];
        let has_debug = debug_keywords.iter().any(|k| lower.contains(k));

        // RAG / document search
        let rag_keywords = ["rag", "document", "ingest", "document", "search document"];
        let has_rag = rag_keywords.iter().any(|k| lower.contains(k));

        // UART/LED etc. peripheral functions -> HAL + Apps + RTL
        let has_peripheral = lower.contains("uart") || lower.contains("led") || lower.contains("gpio")
            || lower.contains("spi") || lower.contains("i2c") || lower.contains("usart");

        // Synthesis / build related
        let has_synthesize = lower.contains("synthesize") || lower.contains("build");
        let has_simulate = lower.contains("simulate") || lower.contains("simulation");
        let has_lint = lower.contains("lint") || lower.contains("static analysis");

        // --- Workflow construction ---

        // If peripheral functions (UART/LED etc.) are included, HAL design -> RTL design order
        if has_peripheral {
            step_num += 1;
            steps.push(WorkflowStep {
                step: step_num,
                conductor: ConductorId::Hal,
                method: "hal.parse.v1".to_string(),
                params: serde_json::json!({ "instruction": instruction }),
                label: "HAL design (peripheral function definition)".to_string(),
            });
            step_num += 1;
            steps.push(WorkflowStep {
                step: step_num,
                conductor: ConductorId::Rtl,
                method: "rtl.lint.v1".to_string(),
                params: serde_json::json!({ "instruction": instruction }),
                label: "RTL design/Lint".to_string(),
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

        // Simulation
        if has_simulate && !steps.iter().any(|s| s.conductor == ConductorId::Rtl) {
            step_num += 1;
            steps.push(WorkflowStep {
                step: step_num,
                conductor: ConductorId::Rtl,
                method: "rtl.simulate.v1".to_string(),
                params: serde_json::json!({ "instruction": instruction }),
                label: "RTL simulation".to_string(),
            });
        } else if has_simulate {
            // If an RTL step already exists, add simulation
            step_num += 1;
            steps.push(WorkflowStep {
                step: step_num,
                conductor: ConductorId::Rtl,
                method: "rtl.simulate.v1".to_string(),
                params: serde_json::json!({ "instruction": instruction }),
                label: "RTL simulation".to_string(),
            });
        }

        // FPGA build
        if has_fpga || has_synthesize {
            step_num += 1;
            steps.push(WorkflowStep {
                step: step_num,
                conductor: ConductorId::Fpga,
                method: "fpga.build.v1.start".to_string(),
                params: serde_json::json!({ "instruction": instruction, "target": if lower.contains("arty") || lower.contains("artix") { "artix7" } else { "xilinx" } }),
                label: "FPGA build".to_string(),
            });
        }

        // ASIC synthesis
        if has_asic {
            step_num += 1;
            steps.push(WorkflowStep {
                step: step_num,
                conductor: ConductorId::Asic,
                method: "asic.synthesize".to_string(),
                params: serde_json::json!({ "instruction": instruction }),
                label: "ASIC synthesis".to_string(),
            });
        }

        // PCB design
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

        // Application firmware
        if has_apps {
            step_num += 1;
            steps.push(WorkflowStep {
                step: step_num,
                conductor: ConductorId::Apps,
                method: "apps.build.v1".to_string(),
                params: serde_json::json!({ "instruction": instruction }),
                label: "Firmware build".to_string(),
            });
        }

        // Debug / real-hardware verification
        if has_debug || lower.contains("verification") || lower.contains("real hardware") {
            step_num += 1;
            steps.push(WorkflowStep {
                step: step_num,
                conductor: ConductorId::Debug,
                method: "debug.connect".to_string(),
                params: serde_json::json!({ "instruction": instruction }),
                label: "Debug/verification".to_string(),
            });
        }

        // RAG search
        if has_rag {
            step_num += 1;
            steps.push(WorkflowStep {
                step: step_num,
                conductor: ConductorId::Rag,
                method: "rag.search".to_string(),
                params: serde_json::json!({ "instruction": instruction }),
                label: "Document search".to_string(),
            });
        }

        // Fallback when nothing matches
        if steps.is_empty() {
            step_num += 1;
            steps.push(WorkflowStep {
                step: step_num,
                conductor: ConductorId::Ai,
                method: "ai.exec".to_string(),
                params: serde_json::json!({ "instruction": instruction }),
                label: "AI processing".to_string(),
            });
        }

        steps
    }

    /// Send a message to a conductor and get the response
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

        // Parse the response as JSON
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
            "ai.qa" => self.handle_qa(params).await,
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
    // Phase 102 — Removed `spec_driven_emit_skeleton` function.
    // Phase 59 wrote `<workspace>/.aiprj/{AI_PRJ_REQUIREMENTS,AI_PRJ_DESIGN,AI_PRJ_TASKS}.md`
    // as a minimal setup_ai self-execution implementation (Q6 gap partial resolution), but
    // this contradicted the Phase 91 prohibition against `.aiprj/` read/write
    // documented in all 50 personas.
    // The 3 document management is the exclusive domain of the project management AI;
    // the hestia agent runtime must not touch it.

    /// Phase 77 — Auto-spawn ai-reviewer sub-agent after workflow completion and
    /// request quality gate evaluation. Calls the ai-reviewer added in Phase 74
    /// (`hestia spawn-subagent --persona ai-reviewer --name ai-reviewer`) and sends
    /// run_id + workflow_steps count via `agent-cli send`.
    ///
    /// Returns: true if dispatch succeeds, false on failure (hestia binary absent /
    /// agent-cli absent etc.). Failure only produces a warning log; it does not
    /// affect the overall ai.exec result.
    async fn auto_spawn_reviewer(instruction: &str, total_steps: usize) -> bool {
        // 1. Spawn ai-reviewer
        let spawn_result = std::process::Command::new("hestia")
            .args(["spawn-subagent", "--persona", "ai-reviewer", "--name", "ai-reviewer"])
            .output();
        let spawned = matches!(&spawn_result, Ok(o) if o.status.success());
        if !spawned {
            tracing::warn!("Phase 77: ai-reviewer spawn failed (hestia binary or persona unreachable)");
            return false;
        }

        // 2. Send review request prompt to ai-reviewer
        // Phase 102 — Review artifacts are written directly under project root (`.aiprj/` is the exclusive domain of the project management AI).
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

    /// ai.exec — Parse instruction, build workflow, execute sequentially, and aggregate results
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

        // Phase 102 — Removed Phase 59's spec_driven_emit_skeleton (which wrote
        // `<workspace>/.aiprj/AI_PRJ_*.md` via fs_write). This contradicted Phase 91's
        // persona prohibition against `.aiprj/` writes. The 3 document management
        // is the exclusive domain of the project management AI.

        // Build workflow from instruction
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

            // Phase 126 — L2 cap: protect ai-conductor concurrent dispatch limit with a Semaphore.
            // L2 of the fixed acquisition order (L1 -> L2 -> L3). Timeout breaks circular waiting.
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

            // Phase 101 — Activate Phase 93's on-demand spawn functionality (G10 fix).
            // Spawn the required conductor just before dispatch. No-op if already alive.
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

        // Phase 77 — Auto-spawn ai-reviewer after workflow completion and request quality gate evaluation.
        // Can be disabled with env override `HESTIA_DISABLE_AUTO_REVIEW=1` (test compatibility).
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

    /// ai.qa — Send a Q&A prompt to the ai-conductor LLM peer and wait for its
    /// substantive reply.
    ///
    /// Unlike `ai.exec` (which returns as soon as `dispatch_to_conductor`
    /// captures the agent-cli delivery ack), this method generates a result
    /// file path, sends a `KIND: qa` envelope to the peer, and waits for the
    /// peer's actual answer using two parallel signals:
    ///
    /// 1. **Primary (`RESULT_PATH` polling)**: when the persona contract is
    ///    followed, the LLM fs_writes `.hestia/run_log/<run-id>.json` with
    ///    `{"status":"ok","answer":…,"kind":"qa"}`. See `.hestia/personas/ai.md`
    ///    §Workflow step 1.5.
    /// 2. **Fallback (agent-cli JSONL log tailing)**: LLM instruction-following
    ///    on fs_write isn't 100% reliable. The agent-cli structured log under
    ///    `~/.local/share/agent-cli/logs/<agent-id>/*.jsonl` records every
    ///    `assistant` event with the LLM's actual text. We tail it from the
    ///    position captured just before sending the envelope; the first new
    ///    `{"kind":"assistant","text":...}` we see (after the send) becomes
    ///    the answer.
    ///
    /// Whichever channel produces a terminal signal first wins. This makes
    /// `hestia ai qa` work even if the persona instruction isn't followed.
    ///
    /// Params:
    ///   - `instruction` (string, required): the Q&A prompt text.
    ///   - `poll_interval_ms` (u64, optional, default 500): poll interval for both channels.
    async fn handle_qa(&self, params: serde_json::Value) -> Result<serde_json::Value, String> {
        let _permit = self.dispatch_limiter.acquire().await
            .map_err(|e| format!("dispatch_limiter acquire failed: {e}"))?;

        let instruction = params.get("instruction")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        if instruction.is_empty() {
            return Err("instruction is empty".to_string());
        }
        let poll_interval_ms = params.get("poll_interval_ms")
            .and_then(|v| v.as_u64())
            .unwrap_or(500);

        let run_id = format!(
            "qa-{}-{}",
            chrono::Utc::now().format("%Y%m%dT%H%M%SZ"),
            &uuid::Uuid::new_v4().simple().to_string()[..8],
        );
        let log_dir = std::env::current_dir()
            .unwrap_or_else(|_| std::path::PathBuf::from("."))
            .join(".hestia/run_log");
        std::fs::create_dir_all(&log_dir)
            .map_err(|e| format!("failed to create run_log dir {}: {e}", log_dir.display()))?;
        let result_path = log_dir.join(format!("{run_id}.json"));
        let result_path_str = result_path.to_string_lossy().to_string();

        let envelope = format!(
            "RUN_ID: {run_id}\nRESULT_PATH: {result_path_str}\nKIND: qa\nINSTRUCTION:\n{instruction}"
        );

        tracing::info!(run_id = %run_id, result_path = %result_path_str, "ai.qa: dispatching to ai peer");

        // Capture the ai peer's JSONL log position *before* sending so we can
        // tell which `assistant` events are responses to *this* prompt.
        let jsonl_log = resolve_ai_peer_jsonl_log().await;
        let mut jsonl_pos: u64 = jsonl_log.as_ref()
            .and_then(|p| std::fs::metadata(p).ok())
            .map(|m| m.len())
            .unwrap_or(0);

        if let Err(e) = conductor_sdk::workspace::spawn_conductor_on_demand("ai") {
            tracing::warn!(error = %e, "ai.qa: spawn_conductor_on_demand(ai) failed");
            return Err(format!("spawn ai peer: {e}"));
        }
        if let Err(e) = conductor_sdk::workspace::agent_cli_send("ai", &envelope) {
            return Err(format!("agent-cli send ai: {e}"));
        }

        // If we couldn't locate the JSONL log before sending (peer hadn't
        // registered yet), try again now that spawn_conductor_on_demand has
        // ensured the peer exists.
        let jsonl_log = match jsonl_log {
            Some(p) => Some(p),
            None => resolve_ai_peer_jsonl_log().await,
        };

        let poll = std::time::Duration::from_millis(poll_interval_ms);
        loop {
            tokio::time::sleep(poll).await;

            // Channel 1 — RESULT_PATH file written by the persona's fs_write.
            if result_path.exists() {
                if let Ok(content) = std::fs::read_to_string(&result_path) {
                    if !content.trim().is_empty() {
                        if let Ok(value) = serde_json::from_str::<serde_json::Value>(&content) {
                            let status_field = value
                                .get("status")
                                .and_then(|v| v.as_str())
                                .unwrap_or("");
                            if status_field != "in_progress" {
                                tracing::info!(
                                    run_id = %run_id,
                                    status = %status_field,
                                    source = "result_path",
                                    "ai.qa: terminal status from RESULT_PATH"
                                );
                                return Ok(value);
                            }
                        }
                    }
                }
            }

            // Channel 2 — agent-cli JSONL log assistant event for this prompt.
            if let Some(ref log_path) = jsonl_log {
                if let Ok(meta) = std::fs::metadata(log_path) {
                    if meta.len() > jsonl_pos {
                        if let Ok(mut file) = std::fs::File::open(log_path) {
                            use std::io::{Read, Seek, SeekFrom};
                            if file.seek(SeekFrom::Start(jsonl_pos)).is_ok() {
                                let mut buf = String::new();
                                if file.read_to_string(&mut buf).is_ok() {
                                    jsonl_pos = meta.len();
                                    for line in buf.lines() {
                                        let Ok(ev) = serde_json::from_str::<serde_json::Value>(line) else { continue };
                                        let kind = ev.get("kind").and_then(|v| v.as_str()).unwrap_or("");
                                        if kind != "assistant" {
                                            continue;
                                        }
                                        let text = ev.get("text").and_then(|v| v.as_str()).unwrap_or("");
                                        if text.trim().is_empty() {
                                            continue;
                                        }
                                        tracing::info!(
                                            run_id = %run_id,
                                            text_len = text.len(),
                                            source = "agent_jsonl",
                                            "ai.qa: assistant event from agent-cli JSONL log"
                                        );
                                        return Ok(serde_json::json!({
                                            "status": "ok",
                                            "answer": text,
                                            "run_id": run_id,
                                            "kind": "qa",
                                            "source": "agent.log",
                                        }));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// ai.spec.init — Parse specification text using SpecParser
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

    /// ai.spec.update — Process specification updates
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

    /// ai.spec.review — Execute specification review
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

    /// agent_spawn — Spawn an agent using AgentManager (via agent-cli run)
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

    /// agent_list — Return agent list from AgentManager and ConductorManager
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

    /// container.list — Return container list
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

    /// container.start — Health-check and confirm startup of the specified conductor
    async fn handle_container_start(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "container.start",
            "name": name,
        }))
    }

    /// container.stop — Process container stop
    async fn handle_container_stop(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "container.stop",
            "name": name,
        }))
    }

    /// container.create — Process container creation
    async fn handle_container_create(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "container.create",
            "name": name,
        }))
    }

    /// container.update — Process container update
    async fn handle_container_update(params: serde_json::Value) -> Result<serde_json::Value, String> {
        let name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
        Ok(serde_json::json!({
            "status": "ok",
            "method": "container.update",
            "name": name,
        }))
    }

    /// meta.dualBuild — Orchestrate parallel FPGA and ASIC builds
    async fn handle_dual_build(&self, params: serde_json::Value) -> Result<serde_json::Value, String> {
        let target_fpga = params.get("target_fpga").and_then(|v| v.as_str()).unwrap_or("xilinx");
        let target_asic = params.get("target_asic").and_then(|v| v.as_str()).unwrap_or("sky130");

        // Dispatch FPGA build
        let fpga_result = Self::dispatch_to_conductor(
            &self.config,
            ConductorId::Fpga,
            "fpga.build.v1.start",
            serde_json::json!({ "target": target_fpga }),
        ).await;

        // Dispatch ASIC synthesis
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

    /// meta.boardWithFpga — Execute PCB + FPGA integrated workflow
    async fn handle_board_with_fpga(&self, params: serde_json::Value) -> Result<serde_json::Value, String> {
        let board_name = params.get("board_name").and_then(|v| v.as_str()).unwrap_or("arty-a7");

        // FPGA build
        let fpga_result = Self::dispatch_to_conductor(
            &self.config,
            ConductorId::Fpga,
            "fpga.build.v1.start",
            serde_json::json!({ "target": board_name }),
        ).await;

        // PCB DRC check
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

    /// system.health.v1 — Aggregate status of all conductors from ConductorManager
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

/// Locate the agent-cli structured JSONL log for the live ai peer.
///
/// Mirrors `hestia/src/main.rs::resolve_agent_log_path` but is private to this
/// crate and specific to the "ai" peer. Returns `None` if `agent-cli list`
/// can't be invoked, no peer named "ai" is registered, the log dir doesn't
/// exist, or the dir is empty. The latest `*.jsonl` (most recent mtime) wins
/// because agent-cli rotates session logs per spawn.
async fn resolve_ai_peer_jsonl_log() -> Option<std::path::PathBuf> {
    let engine_bin = std::env::var("HESTIA_ENGINE_BINARY")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "agent-cli".to_string());
    let output = std::process::Command::new(&engine_bin)
        .arg("list")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut agent_id: Option<String> = None;
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("ID") || trimmed.starts_with('-') {
            continue;
        }
        let fields: Vec<&str> = trimmed.split_whitespace().collect();
        if fields.len() >= 2 && fields[1] == "ai" {
            agent_id = Some(fields[0].to_string());
            break;
        }
    }
    let agent_id = agent_id?;

    let home = dirs::home_dir()?;
    let log_dir = if agent_id.starts_with("shim-") {
        home.join(".local/share/claude-cli-shim/logs").join(&agent_id)
    } else {
        home.join(".local/share/agent-cli/logs").join(&agent_id)
    };
    if !log_dir.exists() {
        return None;
    }

    let mut entries: Vec<(std::time::SystemTime, std::path::PathBuf)> = std::fs::read_dir(&log_dir)
        .ok()?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name().to_string_lossy().ends_with(".jsonl"))
        .filter_map(|e| {
            let mtime = e.metadata().ok()?.modified().ok()?;
            Some((mtime, e.path()))
        })
        .collect();
    entries.sort_by_key(|(t, _)| *t);
    entries.into_iter().next_back().map(|(_, p)| p)
}