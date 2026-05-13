# IP Manager

**Domain**: common — IP Core Management
**Source**: Design Specification §13.4

## Overview

A shared service that provides IP core registration, search, version resolution, license management, and dependency resolution. It uses `petgraph`'s DAG-based resolution algorithm (topological sort) to resolve multi-level dependencies. Provided as agent-cli peer `ip-manager`.

## Key Types

### IpCore

```rust
pub struct IpCore {
    pub id: String,                // "com.vendor.name"
    pub version: String,           // semver
    pub vendor: String,
    pub library: String,
    pub device_families: Vec<String>,
    pub supported_languages: Vec<String>,
    pub dependencies: Vec<IpDependency>,
    pub files: Vec<IpFile>,
    pub parameters: Vec<IpParameter>,
}
```

### IpDependency

```rust
pub struct IpDependency {
    pub ip_id: String,
    pub version_req: VersionReq,    // semver VersionReq
    pub optional: bool,
}
```

### IpFile

```rust
pub struct IpFile {
    pub path: String,
    pub file_type: IpFileType,     // rtl | testbench | doc | constraint
    pub language: IpLanguage,       // verilog | vhdl | other
}
```

## Dependency Resolution Algorithm

A DAG of dependencies between IP cores is constructed using `petgraph`, and the resolution order is determined via topological sort.

```
IpCore A (depends on B, C)
  +-- IpCore B (depends on D)
  +-- IpCore C (depends on D)
      +-- IpCore D (no dependencies)

Topological sort result: [D, B, C, A]
```

If a circular dependency is detected, an error is raised (not a DAG).

## License Classification

| Classification | Target Licenses | Treatment |
|------|-------------|------|
| `Oss` | MIT / Apache-2.0 / BSD / GPL / ISC / CC0 | Freely usable and publishable |
| `VendorProprietary` | FlexLM / seat-limited | `terms_accepted=true` required, internal use only |
| `Unknown` | Unknown | **Rejected** (cannot be imported)|

## Version Resolution

Version constraint resolution based on semver:

| Constraint | Meaning |
|------|------|
| `>=0.40` | 0.40 or higher |
| `^1.0.0` | 1.x.x (maintaining compatibility)|
| `~1.2.0` | 1.2.x (patch updates only)|
| `=2025.2` | Exact match |

## Crate Structure

```
ip-manager/
├── Cargo.toml
└── src/
    ├── lib.rs              # IpCore, IpRegistry
    ├── resolver.rs         # DAG resolution (petgraph)
    ├── license.rs          # License classification and verification
    └── version.rs          # semver version resolution
```

## Related Documents

- [constraint_bridge.md](constraint_bridge.md) — Constraint file conversion
- [database_schema.md](database_schema.md) — ip_registry schema
- [observability.md](observability.md) — Monitoring