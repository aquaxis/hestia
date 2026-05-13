# ai-conductor Detailed Design — Meta-Orchestrator

**Scope**: ai-conductor (meta-orchestrator)
**Source**: Design specification §3 (lines 745-1240)

---

## Overview

ai-conductor is the top-level conductor in HESTIA, serving as the sole entry point from the frontend (VSCode / Tauri / CLI). It orchestrates all subordinate conductors (rtl / fpga / asic / pcb / hal / apps / debug / rag) and provides four core functions: task decomposition and routing, health checks, skill management, and container management.

---

## 1. Four Core Functions

| Function | Role | Related Section |
|------|------|--------|
| **Task decomposition and routing** | Understands natural language or structured instructions from the frontend, decomposes tasks, and routes them to the appropriate subordinate conductor | §3.3 task-router / §3.3.1 / §3.5 WorkflowEngine / §3.6 SpecDriven |
| **Health check** | Periodically polls all conductors, aggregating Online / Offline / Degraded / Upgrading states. Automatically restarts or escalates to the frontend on failure | §3.1 ai-core/health_check.rs / §3.2 ConductorStatus / §3.3.2 |
| **Skill management** | Registers specialized skills (HDL generation, constraint generation, testbench generation, etc.) as plugins in SkillRegistry and provides them to subordinate conductor agent-cli personas | §3.1 skill-system/ / §3.7 |
| **Container management** | Automatic Containerfile generation, build, differential update, provisioning, and registry management based on `container.toml` declarations (only when container execution is selected) | §3.1 container-manager/ / §3.8 / §12 |

Additionally, auxiliary functions include sustainable upgrade (§3.4 UpgradeManager), DAG-based workflow (§3.5 WorkflowEngine), spec-driven development (§3.6 SpecDriven), and LLM backend switching (§20 agent-cli endpoint configuration).

---

## 2. Crate Structure

```
ai-conductor/
├── Cargo.toml
├── crates/
│   ├── ai-core/                    # ConductorManager, health check
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── conductor_manager.rs # All conductor lifecycle management
│   │       └── health_check.rs      # Periodic health check
│   ├── conductor-client/           # agent-cli IPC client
│   │   └── src/
│   │       ├── lib.rs              # ConductorClient
│   │       └── transport.rs        # Unix Socket transport
│   ├── upgrade-manager/            # Sustainable upgrade management
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── version_policy.rs   # Semantic versioning
│   │       ├── rollout.rs          # Gradual rollout
│   │       └── rollback.rs        # Automatic rollback
│   ├── workflow-engine/            # DAG-based workflow engine
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── dag.rs              # DAG definition and execution
│   │       └── pipeline.rs        # Cross-conductor pipeline
│   ├── spec-driven/                # Spec-driven development engine
│   │   └── src/
│   │       ├── lib.rs
│   │       └── parser.rs           # SpecParser → DesignSpec
│   ├── skill-system/               # Skill plugin system
│   │   └── src/
│   │       ├── lib.rs              # SkillRegistry
│   │       └── skill.rs            # Skill trait
│   ├── multi-agent/                # Hierarchical agent management
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── agent_manager.rs    # Agent launch, stop, and monitoring
│   │       ├── message_broker.rs   # Message routing
│   │       └── session.rs          # Session management
│   ├── agent-communication/        # Message broker
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── protocol.rs         # Message protocol definition
│   │       └── message.rs          # AgentMessage format
│   ├── agent-monitoring/           # Real-time monitoring
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── live_view.rs        # Real-time display
│   │       └── health_check.rs     # Agent health check
│   └── container-manager/          # Container management
│       └── src/
│           ├── lib.rs
│           ├── builder.rs          # Containerfile auto-generation and build
│           ├── registry.rs         # Container image registry
│           ├── updater.rs          # Image differential update
│           ├── provisioner.rs      # Tool provisioning
│           └── tool_updater.rs      # Tool update management
├── ai-cli/                         # Rust CLI (hestia-ai-cli)
└── conductor-sdk/                  # Common SDK (transport/message/agent/config/error)
```

**Note**: The former `rag-engine` crate (TypeScript + LangChain) and `rag-ingest` module (Rust) have been **separated into rag-conductor** (independent Conductor). ai-conductor calls it by sending `rag.*` structured messages to the `rag` peer via agent-cli IPC.

---

## 3. ConductorManager (MetaOrchestrator)

```rust
pub struct ConductorManager {
    conductors: Arc<RwLock<HashMap<ConductorId, ConductorInfo>>>,
    pub config: OrchestratorConfig,
}

pub enum ConductorId {
    Ai,          // agent-cli peer "ai"          (self / loopback for health check)
    Rtl,         // agent-cli peer "rtl"         (RTL upstream)
    Fpga,        // agent-cli peer "fpga"
    Asic,        // agent-cli peer "asic"
    Pcb,         // agent-cli peer "pcb"
    Hal,         // agent-cli peer "hal"          (HAL generation)
    Apps,        // agent-cli peer "apps"         (application FW)
    Debug,       // agent-cli peer "debug"
    Rag,         // agent-cli peer "rag"          (separated from former ai-conductor::rag-engine)
}

pub enum ConductorStatus {
    Online,     // Running normally
    Offline,    // Stopped
    Degraded,   // Degraded state (some functionality restricted)
    Upgrading,  // Upgrading
}
```

---

## 4. Meta-Orchestration Functions

ai-conductor itself is launched as an agent-cli process (peer name `ai`) and provides the following functions. Communication with downstream conductors uses agent-cli native IPC exclusively.

```
ai-conductor (= agent-cli process / peer name "ai")
    │
    ├── task-router ───── Understanding, decomposition, and routing of frontend instructions
    ├── health-checker ── Periodic health checks for all conductors
    ├── conductor-router ─── agent-cli IPC routing to downstream conductors
    │   ├── rtl-conductor         (peer "rtl")
    │   ├── fpga-conductor        (peer "fpga")
    │   ├── asic-conductor        (peer "asic")
    │   ├── pcb-conductor         (peer "pcb")
    │   ├── hal-conductor         (peer "hal")
    │   ├── apps-conductor        (peer "apps")
    │   ├── debug-conductor       (peer "debug")
    │   └── rag-conductor         (peer "rag")
    ├── conductor-startup ─── Startup sequence orchestration
    │   ├── Group 0: ai-conductor (highest priority, serial)
    │   └── Group 1: rtl / fpga / asic / pcb / hal / apps / debug / rag (8 in parallel, after ai readiness)
    ├── upgrade-manager ─── Sustainable upgrade
    ├── workflow-engine ─── DAG-based workflow
    ├── spec-driven ─── Spec-driven development
    ├── skill-system ─── Skill plugin
    ├── backend-switching ─── LLM backend switching
    └── container-manager ─── Container lifecycle management
```

---

## 5. Task Routing Flow

Instructions from the frontend to ai-conductor are received as natural language or structured JSON payloads. `task-router` leverages agent-cli's LLM inference to decompose tasks, analyze dependencies, and dispatch them to subordinate conductors.

```
[Frontend (VSCode/Tauri/CLI)]
       │
       │ agent-cli send ai <payload>
       ▼
[ai-conductor: task-router]
       │
       │ Step 1. Intent understanding (intent classification)
       │    - Natural language → design task type classification
       │    - Structured JSON → direct classification via method namespace
       │
       │ Step 2. Task decomposition
       │    - Single conductor completion → dispatch directly
       │    - Cross-conductor → delegate to workflow-engine and DAG-ify
       │    - Specification-based → generate DesignSpec via spec-driven → DAG-ify
       │
       │ Step 3. Routing (via conductor-router)
       │    - agent-cli send <peer> <payload> to appropriate peer
       │
       ▼
[Subordinate conductor]
       │ Result response (agent-cli send ai <result> with same trace_id)
       ▼
[ai-conductor: Result aggregation → notification to frontend]
```

**Task Decomposition and Routing Examples:**

| Input example | Decomposed tasks | Routing target |
|--------|-----------|----------|
| "Build with Vivado for artix7" | `fpga.build.v1.start { target: "artix7" }` | fpga-conductor |
| "Lint RTL and check if synthesizable" | `rtl.lint.v1` → `rtl.handoff.v1 { target: "fpga" }` | rtl-conductor → fpga-conductor |
| "Convert FPGA prototype to ASIC and generate GDSII" | DAG: `rtl.handoff` → `asic.synth` → ... → `asic.gdsii` | via workflow-engine / multiple conductors |
| `{"method":"meta.dualBuild.v1", "params":{...}}` | DAG: `fpga.build` ‖ `asic.synth` → `meta.collect` | workflow-engine |

---

## 6. Health Check Function

`health-checker` periodically confirms the liveness and health of all conductors and updates `ConductorStatus` in ConductorManager. On failure detection, it attempts automatic restart, and if unrecoverable, escalates to the upgrade-manager or humans (frontend notification).

- **Polling interval**: Default 30 seconds (configurable via `[health] interval_secs`)
- **Method**: `agent-cli send <peer> '{"method":"system.health.v1","id":"hc_<ts>"}'`
- **Response patterns**:
  - "online" response within 3 seconds → Online
  - Timeout (3 seconds) → Offline
  - "degraded" response → Degraded
  - "upgrading" response → Upgrading
- **Actions on state change**:
  - Online → Offline / Degraded → Automatic restart attempt (max 3)
  - 3 consecutive failures → Frontend notification
  - Upgrading → Online → Notify upgrade-manager of success
  - Any → Persist state history to sled

**Health Check Configuration Example (`container.toml` `[health]` section):**

```toml
[health]
cmd = "vivado -version || true"
interval_secs = 30
timeout_secs = 3
max_retries = 3
escalate_on_fail = true
restart_on_fail = true
```

---

## 7. UpgradeManager Details

Provides compatibility assessment based on semantic versioning, gradual rollout, and automatic rollback.

### 7.1 Compatibility Assessment

| Version change | Compatibility | Required strategy |
|--------------|--------|-------------|
| `1.0.0` → `1.1.0` | Compatible | Production OK |
| `1.0.0` → `1.0.1` | Compatible | Production OK |
| `1.0.0` → `2.0.0` | Incompatible | Canary or Staging required |

### 7.2 Gradual Rollout Strategy

| Strategy | Description | Use case |
|------|------|---------|
| `Canary` | Deploy to a small number of environments first | Major version changes |
| `Staging` | Deploy to production after staging environment verification | Minor version updates |
| `Production` | Deploy directly to production | Patch releases |

### 7.3 Agent Chain

```
WatcherAgent → ProbeAgent → PatcherAgent → ValidatorAgent
```

- **WatcherAgent**: Monitors and detects vendor tool release notes
- **ProbeAgent**: Analyzes change content and assesses impact from release notes
- **PatcherAgent**: Automatically generates patches using Anthropic SDK's Tool Use functionality within the agent loop
- **ValidatorAgent**: Verifies generated patches

### 7.4 RollbackConfig

```rust
pub struct RollbackConfig {
    pub auto_rollback: bool,     // Enable automatic rollback
    pub timeout_secs: u64,       // Timeout (default: 300 seconds)
    pub max_retries: u32,        // Maximum retry count (default: 3)
}
```

---

## 8. WorkflowEngine Details

A DAG-based cross-conductor pipeline engine. It determines execution order via topological sort using Kahn's algorithm and persists state in sled.

```rust
pub struct WorkflowStep {
    pub id: String,              // Step ID
    pub name: String,            // Step name
    pub conductor: String,       // Target conductor
    pub method: String,          // agent-cli message method to execute
    pub params: Option<Value>,   // Parameters
    pub depends_on: Vec<String>, // Dependency step IDs (DAG structure)
    pub status: StepStatus,      // Current status
}
```

**Diamond Dependency Example:**

```
        [A: FPGA Synthesis]
       /              \
[B: ASIC Synthesis]    [C: PCB Design]
       \              /
        [D: Integration Verification]
```

---

## 9. SpecDriven (Spec-Driven Development) Details

Automatically generates design data from natural language specifications. Automatically analyzes requirements, constraints, and interfaces using `REQ:` / `CON:` / `IF:` prefixes.

```rust
pub struct SpecParser;

impl SpecParser {
    pub fn parse(spec_text: &str) -> Result<DesignSpec, SpecError> {
        // Lines starting with REQ: → requirements
        // Lines starting with CON: → constraints
        // Lines starting with IF:  → interface definitions
        // Error if no mandatory requirements exist
    }
}
```

**Flow**: `Specification (natural language) → SpecParser → DesignSpec → AI generation engine → HDL / constraints / testbench`

Public methods: `ai.spec.init` / `ai.spec.update` / `ai.spec.review`

---

## 10. SkillSystem (Skill Plugin) Details

Registers specialized skills in SkillRegistry for AI agents (agent-cli processes) to invoke. Skills are combined with agent-cli persona files (YAML+Markdown) to define the capabilities of each conductor's main agent and sub-agents.

**Default Skills:**

| Skill | Description |
|--------|------|
| HDL generation | Automatic SystemVerilog / Verilog / VHDL code generation |
| Constraint generation | Automatic XDC / SDC / PCF constraint file generation |
| Testbench generation | Automatic testbench skeleton + assertion generation |

Custom skills can be registered in SkillRegistry by implementing the `Skill` trait.

---

## 11. container.toml Reference

A file that declaratively defines the container environment used by each conductor.

| Section | Required | Description |
|-----------|------|------|
| `[container]` | Required | Container basic settings (name, base image, target conductor) |
| `[tools.*]` | Optional | Tool definitions to install |
| `[env]` | Optional | Environment variables |
| `[[volumes]]` | Optional | Volume mount definitions |
| `[health]` | Optional | Health check settings |
| `[update]` | Optional | Update policy |

**container.toml sample:**

```toml
[container]
name = "vivado-build"
base_image = "ubuntu:24.04"
conductor = "fpga"

[tools.vivado]
name = "AMD Vivado"
version = ">=2025.1"
install_script = "apt-get update && apt-get install -y wget && ..."
version_cmd = "vivado -version"

[tools.yosys]
name = "Yosys"
version = ">=0.40"
install_script = "apt-get install -y yosys"
version_cmd = "yosys --version"

[env]
XILINX_ROOT = "/opt/Xilinx"
PATH = "/opt/Xilinx/Vivado/2025.2/bin:$PATH"

[[volumes]]
host = "/workspace"
container = "/workspace"
options = "Z"

[[volumes]]
host = "/opt/Xilinx/license"
container = "/opt/Xilinx/license"
options = "ro"

[health]
cmd = "vivado -version || true"
interval_secs = 60

[update]
auto = true
schedule = "0 3 * * 0"
rollback_on_failure = true
```

---

## 12. upgrade.toml Reference

```toml
[upgrade]
check_interval_hours = 6
auto_upgrade = true
notification_email = "team@example.com"

[strategy.major]
type = "canary"
canary_percentage = 10

[strategy.minor]
type = "staging"

[strategy.patch]
type = "production"

[rollback]
auto = true
timeout_secs = 300
max_retries = 3
```

---

## 13. Sub-agent Configuration

ai-conductor has **two types of sub-agents** to support task routing and SpecDriven. Each sub-agent is launched as an independent agent-cli process and coordinates with the ai-conductor main body (peer name `ai`) via `agent-cli send <peer>` IPC.

| Sub-agent | Peer name | Role | Multiplicity | Persona file |
|----------------|---------|------|-------|-----------------|
| **planner** | `ai-planner` | Creates task decomposition and execution planning (DAG-ification, dependency analysis, dispatch strategy to subordinate conductors) for frontend instructions | 1 (N in parallel under high load) | `.hestia/personas/ai-planner.md` |
| **designer** | `ai-designer` | Creates overall specifications (DesignSpec, HW/SW integration high-level design, inter-conductor coordination contracts) based on frontend instructions | 1 | `.hestia/personas/ai-designer.md` |

**Startup and Coordination Flow:**

```
[Frontend (VSCode/Tauri/CLI)]
       │ agent-cli send ai <payload>
       ▼
[ai-conductor (peer "ai")]
       │
       ├── agent-cli send ai-planner '{"method":"plan.v1.create",...}'
       │       ↓ Plan response (DAG / Step list / subordinate conductor assignment proposal)
       │
       ├── agent-cli send ai-designer '{"method":"design.v1.create",...}'
       │       ↓ DesignSpec response (high-level specification / inter-conductor coordination contract)
       │
       ▼
[ai-conductor: Integrate planner + designer output → dispatch via conductor-router]
```

**Scaling and Lifetime:**

- Both planner and designer are resident, started and stopped in sync with ai-conductor's lifetime
- Under high load, multiple planner instances can be launched (peer names `ai-planner-1`, `ai-planner-2`, ...)
- Discoverable via `agent-cli list`, included in health-checker targets

**Startup Command Example:**

```bash
agent-cli run --persona-file ./.hestia/personas/ai-planner.md  --name ai-planner  &
agent-cli run --persona-file ./.hestia/personas/ai-designer.md --name ai-designer &
```

---

## Related Documentation

- [ai_conductor.md](ai_conductor.md) — ai-conductor overview (summary version)
- [rtl_conductor.md](rtl_conductor.md) — RTL design flow orchestrator
- [fpga_conductor.md](fpga_conductor.md) — FPGA design flow orchestrator
- [asic_conductor.md](asic_conductor.md) — ASIC design flow orchestrator
- [pcb_conductor.md](pcb_conductor.md) — PCB design flow orchestrator
- [hal_conductor.md](hal_conductor.md) — HAL generation orchestrator
- [apps_conductor.md](apps_conductor.md) — Application software development orchestrator
- [debug_conductor.md](debug_conductor.md) — Debug environment orchestrator
- [rag_conductor.md](rag_conductor.md) — Knowledge base orchestrator
- [architecture_overview.md](architecture_overview.md) — Overall architecture overview