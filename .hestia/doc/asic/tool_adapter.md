# asic-conductor Tool Adapter

**Target Conductor**: asic-conductor
**Source**: Design specification §6.4 (around lines 1842-1865), §6.6 (around lines 1886-1893), §6.7 (around lines 1894-1919)

## AsicToolAdapter Trait

ASIC-specific tool adapter interface. Unlike the FPGA VendorAdapter, it covers physical design steps (floorplan, CTS, parasitic extraction, etc.).

```rust
#[async_trait]
pub trait AsicToolAdapter: Send + Sync + 'static {
    fn manifest(&self) -> &AdapterManifest;
    fn capabilities(&self) -> &AsicCapabilitySet;

    // Core flow (7 steps)
    async fn synthesize(&self, ctx: &AsicBuildContext) -> Result<StepResult, AdapterError>;
    async fn floorplan(&self, ctx: &AsicBuildContext) -> Result<StepResult, AdapterError>;
    async fn place(&self, ctx: &AsicBuildContext) -> Result<StepResult, AdapterError>;
    async fn cts(&self, ctx: &AsicBuildContext) -> Result<StepResult, AdapterError>;
    async fn route(&self, ctx: &AsicBuildContext) -> Result<StepResult, AdapterError>;
    async fn extract(&self, ctx: &AsicBuildContext) -> Result<StepResult, AdapterError>;
    async fn generate_gdsii(&self, ctx: &AsicBuildContext) -> Result<StepResult, AdapterError>;

    // Signoff
    async fn timing_signoff(&self, ctx: &AsicBuildContext) -> Result<TimingReport, AdapterError>;
    async fn drc(&self, ctx: &AsicBuildContext) -> Result<SignoffResult, AdapterError>;
    async fn lvs(&self, ctx: &AsicBuildContext) -> Result<SignoffResult, AdapterError>;
}
```

## AsicCapabilitySet

The set of capabilities supported by AsicToolAdapter. Indicates support availability for each step.

## AsicCapabilityRouter (Routing Strategy)

Routing strategy for adapter selection.

| Strategy | Description |
|---------|------------|
| `PreferOpenLane` | Delegate steps that OpenLane2 can handle to OpenLane2 (default) |
| `StepOptimal` | Select the optimal adapter individually for each step |
| `Explicit` | Use the adapter explicitly specified in asic.toml |

## SignoffChecker

Responsible for final verification before tape-out.

### SignoffResult

```rust
pub struct SignoffResult {
    pub tool: SignoffTool,
    pub check_type: CheckType,     // DRC or LVS
    pub passed: bool,
    pub violations: Vec<Violation>,
    pub summary: SignoffSummary,
}

pub struct Violation {
    pub rule: String,              // Violation rule name
    pub description: String,       // Violation description
    pub location: Option<GdsCoord>,// GDSII coordinates
    pub severity: ViolationSeverity,
}
```

### Signoff Tools

| Tool | Verification Type | Description |
|-------|------------------|------------|
| Magic | DRC | Layout DRC engine |
| Netgen | LVS | SPICE-level circuit comparison |
| KLayout | DRC + LVS | Scriptable layout verification |

## Main Crate Structure

```
asic-conductor/
├── crates/
│   ├── conductor-core/             # agent-cli persona, main.rs
│   ├── project-model/              # asic.toml parser
│   ├── plugin-registry/            # Tool registration and resolution (AsicToolAdapter trait)
│   ├── adapter-openlane/           # OpenLane 2 integration
│   ├── adapter-yosys/              # Yosys logic synthesis
│   ├── adapter-openroad/           # OpenROAD placement and routing
│   ├── pdk-manager/                # PDK management
│   ├── podman-runtime/             # Container management
│   └── conductor-sdk/              # Shared SDK
```

## Related Documentation

- [asic/config_schema.md](config_schema.md) — asic.toml schema
- [asic/state_machines.md](state_machines.md) — ASIC build state machine
- [asic/error_types.md](error_types.md) — asic-conductor error codes
- [../fpga/vendor_adapter.md](../fpga/vendor_adapter.md) — FPGA VendorAdapter trait