# Method Namespace and Versioning

**Domain**: common — API Versioning
**Source**: Design Specification §14.2

## Overview

Method names for structured messages on agent-cli IPC follow a three-level naming scheme: domain x feature x version. To ensure backward compatibility and gradual deprecation, a semantic versioning-based version prefix is adopted.

## Naming Convention

```
{domain}.{method_group}.{version_prefix}.{action}
```

- Example: `fpga.build.v1.synthesize`
- Shorthand `{domain}.{action}` is equivalent (defaults to v1)

## Key Types

### ApiVersion

```rust
pub struct ApiVersion {
    pub major: u32,
    pub minor: u32,
}
```

### MethodGroup

```rust
pub struct MethodGroup {
    pub domain: String,
    pub group: String,
    pub current_version: ApiVersion,
    pub supported_versions: Vec<ApiVersion>,
    pub deprecated_versions: Vec<ApiVersion>,
    pub methods: Vec<String>,
}
```

### DeprecationNotice

```rust
pub struct DeprecationNotice {
    pub deprecated_since: ApiVersion,
    pub removal_scheduled: ApiVersion,
    pub replacement: String,
}
```

## Compatibility Rules

| Change Type | Version Impact | Backward Compatible |
|---------|-------------|---------|
| Required parameter added | Major bump | No |
| Existing parameter type changed | Major bump | No |
| Method removed | Major bump | No |
| Optional parameter added | Minor bump | Yes |
| Response field added | Minor bump | Yes |

## Domain Listing

| Domain | Examples |
|---------|----|
| `ai.*` | `ai.spec.init` / `ai.spec.update` / `ai.spec.review` / `ai.exec` / `agent_spawn` / `agent_list` |
| `fpga.*` | `fpga.synthesize` / `fpga.implement` / `fpga.bitstream` / `fpga.simulate` / `fpga.program` |
| `asic.*` | `asic.synthesize` / `asic.floorplan` / `asic.place` / `asic.cts` / `asic.route` / `asic.gdsii` / `asic.drc` / `asic.lvs` |
| `pcb.*` | `pcb.generate_schematic` / `pcb.run_drc` / `pcb.run_erc` / `pcb.generate_bom` / `pcb.place_components` / `pcb.route_traces` / `pcb.generate_output` / `pcb.ai_synthesize` / `pcb.status` |
| `debug.*` | `debug.connect` / `debug.disconnect` / `debug.program` / `debug.start_capture` / `debug.stop_capture` / `debug.read_signals` / `debug.set_trigger` / `debug.reset` / `debug.status` |
| `rag.*` | `rag.ingest` / `rag.search` / `rag.cleanup` / `rag.status` |
| `meta.*` | `meta.dualBuild` / `meta.boardWithFpga` and other cross-Conductor workflows |
| `system.*` | `system.readiness` / `system.health` / `system.shutdown` |

## Deprecation Notice Flow

1. Add the new version of the method
2. Attach a `DeprecationNotice` to the old method
3. During the migration period, the old method still works (with warning log output)
4. Remove the old method when the `removal_scheduled` version is reached

## Related Documents

- [agent_message.md](agent_message.md) — Message payload format
- [error_registry.md](error_registry.md) — Error code conventions
- [agent_cli_messaging.md](agent_cli_messaging.md) — Complete messaging specification