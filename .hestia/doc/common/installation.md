# Build Procedure

**Domain**: common — Installation
**Source**: Design Specification §15.4

## Overview

Defines the build and test procedures for the HESTIA Rust workspace. All binaries can be built in bulk with `cargo build --release`.

## Prerequisites

| Requirement | Version |
|------|----------|
| Rust toolchain | stable (latest recommended)|
| Cargo | Included with Rust |
| Linux host OS | Kernel 5.x or later recommended |
| Podman | Only when using containers |

## Build Procedure

### Build All Binaries

```bash
cd .hestia/tools
cargo build --release
```

This generates 9 conductors + 10 CLIs = 19 binaries.

### Build Specific Conductor Only

```bash
cargo build --release -p hestia-ai-conductor
cargo build --release -p hestia-rtl-conductor
cargo build --release -p hestia-fpga-conductor
cargo build --release -p hestia-asic-conductor
cargo build --release -p hestia-pcb-conductor
cargo build --release -p hestia-hal-conductor
cargo build --release -p hestia-apps-conductor
cargo build --release -p hestia-debug-conductor
cargo build --release -p hestia-rag-conductor
```

### Build Specific Crate Only

```bash
cargo build --release -p container-manager
cargo build --release -p conductor-sdk
```

## Running Tests

### All Tests

```bash
cargo test
```

### Specific Conductor Tests

```bash
cargo test -p hestia-fpga-conductor
```

### Specific Crate Tests

```bash
cargo test -p container-manager
```

## Build Artifact Location

Build artifacts are placed in `.hestia/tools/target/release/`:

```
.hestia/tools/target/release/
├── hestia-ai-conductor
├── hestia-rtl-conductor
├── hestia-fpga-conductor
├── ...
├── hestia                # Unified runner CLI
├── hestia-ai-cli
├── hestia-fpga-cli
└── ...
```

## Debug Build

```bash
cargo build                # debug build
cargo test -- --nocapture  # Show stdout
RUST_LOG=debug cargo run -p hestia-fpga-conductor  # Specify log level
```

## Related Documents

- [cargo_workspace.md](cargo_workspace.md) — Workspace configuration
- [conductor_startup.md](conductor_startup.md) — Daemon startup order
- [error_handling_strategy.md](error_handling_strategy.md) — Error handling