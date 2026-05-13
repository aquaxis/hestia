# Container Execution Strategy

**Domain**: Container Execution and Management
**Source**: Design Specification §11 (around lines 2551-2641), §12 (around lines 2644-3185)

---

## 1. Podman Container Strategy (§11)

> **Containers are not mandatory.** Locally installed tools (Vivado / Quartus / KiCad, etc.) can also be used, and this chapter describes the strategy when container execution is selected. Local execution and container execution can be mixed per tool and per build, and whichever is chosen, reproducibility via fpga.lock / asic.lock is guaranteed.

This chapter assumes a **Linux host OS** (TC-10). The user namespace / cgroup / SELinux that Podman rootless depends on are Linux kernel features, and all designs in this chapter assume operation on a Linux environment. (If only local execution is selected, the Podman-related settings in this chapter do not apply.)

### 1.1 Design Principles

- **Daemonless**: No system service required (does not depend on Docker's dockerd)
- **UID matching**: Avoid file permission issues with `--userns=keep-id`
- **SELinux support**: `--security-opt=label=type:container_runtime_t`
- **Privilege escalation prevention**: `--security-opt=no-new-privileges`
- **License protection**: Read-only mount of vendor license files
- **Network isolation**: Build containers block external communication with `--network=none`
- **Intellectual property protection**: Prevent HDL source and bitstream leakage outside containers

---

## 2. podman-runtime Crate (§11.2)

```rust
pub struct PodmanRuntime {
    socket: PathBuf,  // /run/user/1000/podman/podman.sock
}

impl PodmanRuntime {
    /// Launch container for batch build
    pub async fn run_build(
        &self, image: &str, project_dir: &Path, cmd: &[&str],
    ) -> Result<ExitStatus, RuntimeError> {
        let mut args = vec![
            "run", "--rm",
            "--userns=keep-id",
            "--security-opt=no-new-privileges",
            "--network=none",
            "-v", &format!("{}:/workspace:Z", project_dir.display()),
            "-v", &format!("{}:/reports:Z", project_dir.join("reports").display()),
        ];
        if self.needs_jtag(image) {
            args.extend(["--device", "/dev/bus/usb"]);
        }
        args.extend([image]);
        args.extend(cmd);
        Command::new("podman").args(args).status().await
    }

    /// GUI launch (X11/Wayland forwarding)
    pub async fn run_gui(&self, image: &str) -> Result<Child, RuntimeError> {
        let display = std::env::var("DISPLAY").unwrap_or(":0".into());
        let xauth = std::env::var("XAUTHORITY")
            .unwrap_or_else(|_| format!("{}/.Xauthority", std::env::var("HOME").unwrap()));

        Command::new("podman").args([
            "run", "--rm", "--userns=keep-id",
            "-e", &format!("DISPLAY={display}"),
            "-v", "/tmp/.X11-unix:/tmp/.X11-unix:ro",
            "-v", &format!("{xauth}:{xauth}:ro"),
            "-e", &format!("XAUTHORITY={xauth}"),
            "--security-opt=label=type:container_runtime_t",
            image,
        ]).spawn()
    }
}
```

---

## 3. Container Operation Patterns (§11.3)

**Table HD-008: Container Operation Patterns**

| Pattern | Use Case | Configuration |
|---------|------|------|
| Batch build | Synthesis, place-and-route | `--rm`, `--network=none`, project directory mount |
| GUI launch | Vivado GUI, etc. | X11/Wayland forwarding, DISPLAY environment variable |
| systemd service | Always-on containers | Podman Quad definition (.container file) |
| Device access | JTAG programming | `--device /dev/bus/usb` |

---

## 4. Supported Container Images (§11.4)

The list of 8 supported container images is shown below. For details such as base OS configuration, primary tools, license handling, and Containerfile samples, **refer to Table HD-021 in §12.2 "Containerfile Auto-Generation"**.

**Table HD-009: Supported Container Images (summary; for details see §12.2 Table HD-021)**

| Image | Primary Use |
|---------|--------|
| fpga/vivado:2025.2 | AMD/Xilinx FPGA synthesis and place-and-route (Vivado 2025.2) |
| fpga/quartus:25.1 | Intel/Altera FPGA synthesis and place-and-route (Quartus Prime Pro 25.1) |
| fpga/efinity:2025.2 | Efinix Trion/Titanium FPGA (Efinity 2025.2) |
| fpga/radiant:2024.2 | Lattice LIFCL/LFD2NX/iCE40 FPGA (Radiant 2024.2) |
| fpga/oss:latest | OSS FPGA flow (Yosys + nextpnr + icestorm + Verilator) |
| asic/openlane:latest | RTL-to-GDSII automation (OpenLane 2 + OpenROAD + Magic + Netgen) |
| pcb/kicad:latest | PCB design (KiCad + SKiDL + Freerouting) |
| debug/tools:latest | Debug and logic analysis (OpenOCD + sigrok + PulseView + pyOCD) |

---

## 5. container-manager (§12)

> **container-manager is used only when container execution is selected.** When using locally installed tools, the features in this chapter do not apply; local execution paths are called directly via adapters. Container execution and local execution can be mixed per tool (e.g., Vivado in a container, KiCad locally).

### 5.1 Module Structure (§12.1)

**Table HD-010: container-manager Module Structure**

| Module | Role |
|-----------|------|
| builder | Containerfile auto-generation and build (based on tool definition files) |
| registry | Container image registry management (local/remote) |
| updater | Image differential update, version management, and automatic rebuild |
| provisioner | Provisioning based on tool definition files (package installation) |
| tool_updater | Tool version detection, compatibility checking, and staged updates |

### 5.2 Containerfile Auto-Generation (§12.2, FR-CTN-11)

Takes `container.toml` (§3.8 reference) as input and auto-generates a Containerfile (Podman build specification compatible with Dockerfile) using the minijinja template engine.

**Generation flow**:

```
container.toml ──▶ builder::parse ──▶ Container Spec (Rust struct)
                                        │
                                        ▼
                                   Containerfile template (minijinja)
                                        │
                                        ▼
                                   Containerfile (text)
                                        │
                                        ▼
                                   podman build (§12.3)
```

**Table HD-021: 8 Images × Primary Build Parameters**

| Image | Base | Primary Tools | License Handling |
|---------|-------|----------|--------------|
| `fpga/vivado:2025.2` | `registry.access.redhat.com/ubi9/ubi:9.5` | AMD Vivado 2025.2 | FlexLM license server reference (`LM_LICENSE_FILE`) |
| `fpga/quartus:25.1` | `registry.access.redhat.com/ubi9/ubi:9.5` | Intel Quartus Prime Pro 25.1 | QPF/QSF, FlexLM |
| `fpga/efinity:2025.2` | `docker.io/library/ubuntu:24.04` | Efinix Efinity 2025.2 | Efinity bundled Python |
| `fpga/radiant:2024.2` | `registry.access.redhat.com/ubi9/ubi:9.5` | Lattice Radiant 2024.2 | FlexLM |
| `fpga/oss:latest` | `docker.io/library/ubuntu:24.04` | Yosys + nextpnr + icestorm + Verilator | Not required (OSS) |
| `asic/openlane:latest` | `docker.io/library/ubuntu:24.04` | OpenLane 2 + Yosys + OpenROAD + Magic + Netgen | Not required (OSS) |
| `pcb/kicad:latest` | `docker.io/library/ubuntu:24.04` | KiCad + SKiDL + Freerouting | Not required (OSS) |
| `debug/tools:latest` | `docker.io/library/ubuntu:24.04` | OpenOCD + sigrok + PulseView + pyOCD | Not required (OSS) |

### 5.3 Provisioning (§12.4, FR-CTN-13)

Automatically translates `[tools.*]` declarations from `container.toml` into package manager commands.

**Translation Matrix**:

| Tool Definition | apt (Debian / Ubuntu) | dnf / yum (RHEL / UBI) | Other |
|----------|---------------------|---------------------|------|
| `install_method = "apt"` | `apt-get install -y ${package}` | Error at conversion time | — |
| `install_method = "dnf"` | Error at conversion time | `dnf install -y ${package}` | — |
| `install_method = "tarball"` | `wget -O - ${url} \| tar -xz -C ${prefix}` | Same as left | — |
| `install_method = "install_script"` | `bash -c "${install_script}"` | Same as left | Vendor-specific installer |
| `install_method = "pip"` | `pip install --no-cache-dir ${package}` | Same as left | Python packages |
| `install_method = "cargo"` | `cargo install ${package}` | Same as left | Rust packages |

**Vendor License Protection**:

- Installer and license files are not baked into the image (mounted only via `--secret`)
- Runtime licenses (FlexLM, etc.) are injected via `podman run -e LM_LICENSE_FILE=...` or mount
- `.hestia/secure/` directory has mode 0700, excluded from Git via `.gitignore`

**Provisioning Verification**:

1. After installation completes, execute each tool's `version_cmd`
2. Periodic verification via `HEALTHCHECK` instruction (`interval=60s`)
3. On failure, log `prov.failed` to `action-log`, retry (up to 3 times, exponential backoff)

### 5.4 Artifact Signing & SBOM (§12.5, FR-CTN-14)

**Post-build processing pipeline**:

```
podman build → Image generation → SBOM generation via syft → Vulnerability scan via grype
                                  │                   │
                                  ▼                   ▼
                                sbom.spdx.json       vuln.json
                                  │                   │
                                  └──▶ Evaluation gate ◀───┘
                                        │
                                        ▼
                                   cosign sign (keyless)
                                        │
                                        ▼
                                   podman push (§12.6)
```

**SBOM generation via syft**: Supports both SPDX and CycloneDX formats.

**Vulnerability scanning via grype**: Fails the build when fixable Critical/High vulnerabilities are present, using `--fail-on high --only-fixed`.

**Image signing via cosign (keyless OIDC)**: Executes keyless signing on GitHub Actions and attaches the SBOM attachment.

**Unsigned image rejection policy**: `cosign verify` is executed before `podman-runtime::run_build()`. On verification failure, container launch is refused with `SignatureVerificationError`. Development environments can bypass this with `HESTIA_ALLOW_UNSIGNED=1`.

### 5.5 Registry Management (§12.6, FR-CTN-15)

**Table HD-022: Supported Registries and Use Cases**

| Registry | Use Case | Authentication |
|-----------|-----|------|
| `localhost:5000` | Local development | None |
| `ghcr.io/hestia/*` | Official publishing (OSS images only) | GitHub OIDC |
| `quay.io/hestia/*` | Mirror (OSS images only) | Robot account |
| Private Harbor | Images containing commercial tools | mTLS + Robot account |

**Tag Convention**:

```
<registry>/<org>/<conductor>/<tool>:<version>
<registry>/<org>/<conductor>/<tool>:<YYYYMMDD>   # Date tag
<registry>/<org>/<conductor>/<tool>:cache        # Build cache only
```

**Retention Policy**:

- `latest` / `<version>` tags: Permanent
- Date tags (`YYYYMMDD`): Keep only the most recent 30 days; delete older ones via `skopeo delete`
- `cache` tags: Keep only the latest
- Automatic deletion: `skopeo` + cron / GitHub Actions Schedule

**Rate Limit Mitigation**: Use a private mirror (Harbor) as a pull-through cache to avoid Docker Hub pull limits (100 pulls/6h anonymous). On GitHub Actions, use GHCR as the pull source to avoid limits.

### 5.6 CI/CD Integration & Monitoring (§12.7, extension of FR-CTN-12 / FR-CTN-14)

GitHub Actions workflow example (`.github/workflows/container-build.yaml`):

- **Trigger**: Push to `.hestia/containers/**` / `.hestia/container-manager/**`, and scheduled every Monday at 02:00 UTC
- **Matrix**: fpga/oss / asic/openlane / pcb/kicad / debug/tools — 4 images
- **Permissions**: `contents: read`, `packages: write`, `id-token: write` (cosign keyless)
- **Steps**: Install → Build → SBOM → Vulnerability scan → Login to GHCR → Push → Sign

**ObservabilityLayer Integration** (§19.8 / §13.4):

| Metric | Type | Meaning |
|----------|----|-----|
| `hestia_container_build_total{image,status}` | Counter | Build count (success / failure) |
| `hestia_container_build_duration_seconds{image,stage}` | Histogram | Duration per stage |
| `hestia_container_image_size_bytes{image,tag}` | Gauge | Image size |
| `hestia_container_vuln_total{image,severity}` | Gauge | Vulnerability count |
| `hestia_container_signature_verified{image}` | Gauge | Signature verification success (1/0) |

### 5.7 Operational Rules (§12.8)

1. **Weekly build**: `cron: '0 2 * * 1'` (every Monday 02:00 UTC) to incorporate base image + dependency updates and rebuild
2. **Patch version**: Automatic build + automatic push
3. **Minor version**: After automatic build, 1 approval required via Review Agent
4. **Major version**: Manual trigger, staged deployment via UpgradeManager Canary strategy
5. **On failure**: Log `container.build.failed` to `action-log`, notify PatcherAgent
6. **Image size limits**: OSS 5GB, commercial 20GB
7. **BuildKit secret management**: `.hestia/secure/` is read-only, `0700` permissions, excluded from Git
8. **CVE notification**: Detect changes in `grype` weekly scan results; escalate to security team immediately on Critical occurrence

---

## 6. Implementation Crate Structure (§12.9 Summary)

container-manager is located at `ai-conductor/crates/container-manager/`:

```
container-manager/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── builder/
    │   ├── mod.rs            # Containerfile generation + podman build invocation
    │   ├── templates/*.j2    # minijinja templates
    │   └── parser.rs         # container.toml → ContainerSpec
    ├── registry/
    │   └── mod.rs            # push/pull/retention (skopeo wrapper)
    ├── updater/
    │   └── mod.rs            # Differential update / automatic rebuild
    ├── provisioner/
    │   └── mod.rs            # [tools.*] → install command translation
    ├── tool_updater/
    │   └── mod.rs            # Version detection / semver matching
    ├── signer/
    │   └── mod.rs            # cosign wrapper
    ├── sbom/
    │   └── mod.rs            # syft / grype wrapper
    └── observability.rs      # Metrics dispatch
```

---

## 7. Mixed Local and Container Execution

Container execution and local execution can be mixed per tool (e.g., Vivado in a container, KiCad locally). Locally installed tools are called directly via adapters through local execution paths. Whichever is chosen, reproducibility via fpga.lock / asic.lock is guaranteed.

---

## Related Documentation

- [Security](security.md) — Container security (rootless / network isolation / privilege escalation prevention / artifact signing / API key protection)
- [Architecture Overview](architecture_overview.md) — Position of the container layer in the overall architecture
- [Shared Services](shared_services.md) — Observability-based metric monitoring
- `.hestia/doc/container/container_manager.md` — container-manager detailed specification
- `.hestia/doc/container/security_hardening.md` — Container security hardening details
- `.hestia/doc/container/vivado_container.md` — Vivado container details
- `.hestia/doc/container/quartus_container.md` — Quartus container details
- `.hestia/doc/container/efinity_container.md` — Efinity container details
- `.hestia/doc/container/radiant_container.md` — Radiant container details
- `.hestia/doc/container/oss_container.md` — OSS container details