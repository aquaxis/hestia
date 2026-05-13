# fpga-conductor VendorAdapter Trait

**Target Conductor**: fpga-conductor
**Source**: Design Specification §5.2 (around lines 1489-1540), §5.5-5.7 (around lines 1643-1729)

## VendorAdapter Trait Definition

A unified interface that all adapters must implement.

```rust
#[async_trait::async_trait]
pub trait VendorAdapter: Send + Sync + 'static {
    // --- Required: Self-description ---
    fn manifest(&self) -> &AdapterManifest;
    fn capabilities(&self) -> CapabilitySet;

    // --- Required: Core flow ---
    async fn synthesize(&self, ctx: &BuildContext) -> Result<StepResult, AdapterError>;
    async fn implement(&self, ctx: &BuildContext) -> Result<StepResult, AdapterError>;
    async fn generate_bitstream(&self, ctx: &BuildContext) -> Result<StepResult, AdapterError>;

    // --- Optional (default: returns CapabilityUnsupported) ---
    async fn timing_analysis(&self, ctx: &BuildContext) -> Result<TimingReport, AdapterError>;
    async fn start_debug_session(&self, ctx: &BuildContext) -> Result<DebugSession, AdapterError>;
    async fn hls_compile(&self, ctx: &BuildContext) -> Result<StepResult, AdapterError>;
    async fn program_device(&self, ctx: &ProgramContext) -> Result<(), AdapterError>;
    async fn simulate(&self, ctx: &SimContext) -> Result<SimResult, AdapterError>;

    // --- Log diagnostics (default: None) ---
    fn parse_log_line(&self, line: &str) -> Option<Diagnostic> { None }
}
```

## AdapterManifest

```rust
pub struct AdapterManifest {
    pub id:                String,        // "com.xilinx.vivado"
    pub name:              String,        // "AMD Vivado"
    pub version:           String,        // "0.5.1" (adapter's own version)
    pub vendor:            String,        // "AMD/Xilinx"
    pub api_version:       u32,           // ABI compatibility check
    pub supported_devices: Vec<String>,   // glob: ["xc7*", "xcvu*", "xck*"]
    pub capabilities:      CapabilitySet,
    pub release_notes_url: Option<String>, // Used by WatcherAgent
}
```

## CapabilitySet

| Field | Type | Description |
|-------|------|-------------|
| `synthesis` | bool | Synthesis capability |
| `implementation` | bool | Place-and-route capability |
| `bitstream` | bool | Bitstream generation capability |
| `timing_analysis` | bool | Timing analysis capability |
| `on_chip_debug` | bool | On-chip debug capability |
| `device_program` | bool | Device programming capability |
| `hls` | bool | HLS capability |
| `simulation` | bool | Simulation capability |
| `ip_catalog` | bool | IP catalog capability |

## Vivado Adapter Implementation (§5.5)

Adapter for AMD Vivado. Auto-generates TCL scripts (minijinja templates) + batch mode execution + real-time log parsing.

- Synthesis: `vivado -mode batch -source synth.tcl`
- Log parsing: Regex `^(ERROR|WARNING|INFO):\s+\[(\w+)\s+([\d-]+)\]\s+(.+)$`
- Templates: `vivado_synth.tcl.j2` / `vivado_impl.tcl.j2` / `vivado_bit.tcl.j2`

## Quartus Adapter Implementation (§5.6)

Adapter for Intel Quartus Prime. Auto-generates QPF/QSF files + executes via `quartus_sh --flow compile`.

- Synthesis: Generates .qpf (project file) + .qsf (settings file)
- Execution: `quartus_sh --flow compile <project>.qpf`

## Efinity Adapter Implementation (§5.7)

Adapter for Efinix Efinity. Generates interface scripts (XML) + build scripts (Python) + runs using Efinity-bundled Python.

- Interface: `interface.peri.xml` (generated via Rust serde)
- Build: `build.py` (generated via Rust template engine)
- Execution: Efinity-bundled `python3/bin/python3` (no external Python dependency)

## ScriptAdapter (extension via adapter.toml)

When adding a new vendor tool, you can add an adapter simply by writing an `adapter.toml` without modifying any Rust code (Principle 2: zero-modification extension). The adapter.toml defines commands, log parsing rules, and report extraction rules using regular expressions.

## Adapter Types

| Type | Description |
|------|-------------|
| ScriptAdapter | adapter.toml-based (no code changes required) |
| DynamicAdapter | Dynamic loading via dlopen |
| RemoteAdapter | Remote adapter via gRPC |

## Related Documentation

- [fpga/config_schema.md](config_schema.md) — fpga.toml schema
- [fpga/state_machines.md](state_machines.md) — Build state machine
- [fpga/error_types.md](error_types.md) — fpga-conductor error codes
- [../rtl/rtl_tool_adapter.md](../rtl/rtl_tool_adapter.md) — RtlToolAdapter trait
- [../asic/tool_adapter.md](../asic/tool_adapter.md) — AsicToolAdapter trait