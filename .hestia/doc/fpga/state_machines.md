# fpga-conductor Build State Machine

**Target Conductor**: fpga-conductor
**Source**: Design Specification §5.3 (around lines 1542-1575)

## Build State Machine

```
Idle
 │  build_start(target, steps)
 ▼
Resolving            ← VersionSelector resolves toolchain
 │                      References [toolchain] section in fpga.toml
 │                      CompatibilityMatrix selects best version
 ▼
ContainerStarting    ← Podman container startup
 │                      --userns=keep-id --network=none
 │                      Selects vendor tool image
 ▼
Synthesizing         ← adapter.synthesize(ctx)
 │  success           Auto-generates TCL/QSF/Python scripts
 │                      Real-time log parsing
 ▼
Implementing         ← adapter.implement(ctx)
 │  success           Applies place-and-route and timing constraints
 ▼
Bitstreamming        ← adapter.generate_bitstream(ctx)
 │  success           Generates bitstream/JED/BIN
 ▼
Success              → Saves timing and resource reports to reports/
                        Updates fpga.lock
```

## State Definitions

| State | Description | Main Processing |
|-------|-------------|-----------------|
| Idle | Initial state | — |
| Resolving | Resolving toolchain version | VersionSelector, CompatibilityMatrix |
| ContainerStarting | Starting Podman container | PodmanRuntime (only when using containers) |
| Synthesizing | Running RTL synthesis | adapter.synthesize, auto-generates TCL/QSF scripts, real-time log parsing |
| Implementing | Running place-and-route | adapter.implement, applies timing constraints |
| Bitstreamming | Generating bitstream | adapter.generate_bitstream, generates bitstream/JED/BIN |
| Success | Build succeeded | Saves reports, updates fpga.lock |

## Failure Handling

```
On failure at any step → SelfHealingPipeline.on_build_failure()
                          ↓
                    Diagnose via CompatibilityMatrix
                          ↓
                    Known patch available → Auto-apply/notify
                    Unknown error         → Launch PatcherAgent
```

SelfHealingPipeline references the CompatibilityMatrix on build failure to diagnose the issue. If a known patch exists, it is auto-applied or the user is notified. For unknown errors, PatcherAgent (TypeScript + Anthropic SDK) is launched to generate a patch using Tool Use.

## Multi-Target Parallel Builds

When building multiple targets (e.g., artix7 / cyclone10 / trion) simultaneously, each target runs its own independent state machine instance. The Synthesizing / Implementing steps execute in parallel per target.

## Reproducibility via fpga.lock

On successful build, the tool versions, container image hashes, and build parameters used are recorded in fpga.lock, guaranteeing reproducibility in the same environment.

## Related Documentation

- [fpga/binary_spec.md](binary_spec.md) — hestia-fpga-cli binary specification
- [fpga/error_types.md](error_types.md) — fpga-conductor error codes
- [fpga/vendor_adapter.md](vendor_adapter.md) — VendorAdapter trait
- [fpga/config_schema.md](config_schema.md) — fpga.toml schema