# Security Design

**Scope**: Security
**Source**: Design specification §1.2 Principle 4, §11 (Podman container strategy), §12.5 (Artifact signing and SBOM), §12.8 (Operational rules), §20.4

---

## 1. Security Design Policy

### Principle 4: Security

Tool execution can be selected as either container execution or local execution, allowing users to choose based on their use case and operational requirements. When container execution is selected, Podman rootless provides unprivileged execution, `--network=none` provides network isolation, and `--security-opt=no-new-privileges` prevents privilege escalation. When local execution is selected, the host OS's user permissions and SELinux/AppArmor settings apply. In both execution modes, intellectual property (HDL sources, bitstreams) protection is strictly enforced.

---

## 2. Unprivileged Execution with Podman rootless

Podman operates daemonless and does not require root privileges (unlike Docker's dependency on dockerd). Containers execute in user space, eliminating the risk of affecting the host's root privileges.

- **Daemonless**: No system service required. Does not require a resident daemon like Docker's dockerd
- **rootless execution**: Uses user namespaces to run container processes with host user privileges
- **Linux host prerequisite**: user namespace / cgroup / SELinux depend on Linux kernel features

---

## 3. Network Isolation (--network=none)

Build containers are launched with `--network=none`, completely blocking external communication. This prevents the following risks:

- Unintended external communication during the build process
- Intellectual property exfiltration from containers
- Supply chain attacks sending data to unauthorized destinations

`podman-runtime::run_build()` always specifies `--network=none` when launching containers for batch builds.

---

## 4. Privilege Escalation Prevention (--security-opt=no-new-privileges)

`--security-opt=no-new-privileges` prevents container processes from escalating to host root privileges. This prevents:

- Privilege escalation via setuid / setgid binaries
- Host privilege acquisition through container breakout

---

## 5. SELinux Support (label=type:container_runtime_t)

`--security-opt=label=type:container_runtime_t` assigns an appropriate security context to container processes on hosts with SELinux enabled. Used for GUI launches (`run_gui()`), enabling X11/Wayland forwarding and SELinux to coexist.

---

## 6. UID Matching (--userns=keep-id)

`--userns=keep-id` matches the container's UID with the host user's UID. This avoids the following issues:

- Files generated inside the container being owned by root on the host
- Read/write permission issues for host project directories
- Build artifacts being inaccessible from the host

---

## 7. License Protection (Read-Only Mount)

Vendor license files are mounted as read-only (`:ro`). Specific protection measures include:

- Installer and license files are not burned into images (mounted via `--secret` only)
- Runtime licenses (FlexLM etc.) are injected via `podman run -e LM_LICENSE_FILE=...` or mounts
- `.hestia/secure/` directory has mode 0700 and is excluded from Git via `.gitignore`

---

## 8. Intellectual Property Protection (Preventing HDL Source and Bitstream Exfiltration)

In both execution modes (container / local), intellectual property (HDL sources, bitstreams) protection is strictly enforced:

- **Container execution**: `--network=none` for network isolation, mount scope limited to project directories
- **Local execution**: Subject to host OS user permissions and SELinux/AppArmor settings
- **Self-learning accumulation**: HDL sources / bitstream bodies are not stored in RAG; only metadata and summaries are accumulated

---

## 9. Artifact Signing and SBOM

### 9.1 Post-Build Processing Pipeline

After building container images, signing and SBOM generation are performed through the following pipeline:

```
podman build → image generation → SBOM generation with syft → vulnerability scanning with grype
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

### 9.2 SBOM Generation (syft)

Generate SPDX format and CycloneDX format SBOMs using `syft`:

```bash
# SPDX format SBOM
syft ghcr.io/hestia/fpga/oss:latest -o spdx-json > .hestia/containers/fpga/oss/sbom.spdx.json

# CycloneDX format
syft ghcr.io/hestia/fpga/oss:latest -o cyclonedx-json > .hestia/containers/fpga/oss/sbom.cdx.json
```

### 9.3 Vulnerability Scanning (grype)

Scan container image vulnerabilities using `grype`:

```bash
grype ghcr.io/hestia/fpga/oss:latest \
  --fail-on high \
  --only-fixed \
  -o json > .hestia/containers/fpga/oss/vuln.json
```

**Evaluation Gate (grype thresholds):**

| Severity | Threshold | Behavior |
|-------|-----|------|
| Critical | All 0 | **Build failure** (push blocked) |
| High (fixable) | 0 | **Build failure** |
| High (unfixable) | Log | Warning + exception approval (via Review Agent) |
| Medium or below | Log | Push normally allowed |

### 9.4 Image Signing (cosign keyless OIDC)

Perform keyless signing using `cosign`:

```bash
# Keyless signing on GitHub Actions
COSIGN_EXPERIMENTAL=1 cosign sign ghcr.io/hestia/fpga/oss:latest \
  --attachment sbom.spdx.json

# SBOM attestation
cosign attach sbom --sbom .hestia/containers/fpga/oss/sbom.spdx.json \
  ghcr.io/hestia/fpga/oss:latest
```

**Unsigned image rejection policy:**

- Run `cosign verify` before `podman-runtime::run_build()`
- Reject container startup on signature verification failure (`SignatureVerificationError`)
- Development environments can bypass with `HESTIA_ALLOW_UNSIGNED=1` (production always verifies)

---

## 10. Operational Rules (§12.8)

1. **Weekly build**: `cron: '0 2 * * 1'` (every Monday 02:00 UTC) rebuild with base image + dependency updates
2. **Patch version (e.g., 2.4.1 → 2.4.2)**: Automatic build + automatic push (differential detection by `updater` module)
3. **Minor version (e.g., 2.4.x → 2.5.0)**: Automatic build, then 1 approval via Review Agent required
4. **Major version**: Manual trigger, gradual deployment via UpgradeManager's Canary strategy
5. **On failure**: Log `container.build.failed` to `action-log`, notify PatcherAgent
6. **Image size limit**: OSS 5GB, commercial 20GB. Review Agent considers splitting on overflow
7. **BuildKit secret management**: `.hestia/secure/` is read-only, `0700` permissions, excluded from Git
8. **Upstream vulnerability (CVE) notification**: Detect changes in `grype` weekly scan results, immediate escalation to security team on Critical findings

---

## 11. API Key Protection (§20.4)

### 11.1 No Plaintext API Keys

Never write API keys directly in `config.toml` as `anthropic_api_key = "sk-..."`. This is consistent with the existing `security-validation::secrets::audit_text` (31 tests) detection mechanism, and this feature serves as its "input guard version."

### 11.2 Environment Variable Only

Always specify the environment variable name via `anthropic_api_key_env`, and resolve it from a secret backend on the host such as 1Password CLI / `direnv` / systemd EnvironmentFile / GPG.

### 11.3 Explicit Error on Unset

If the environment variable is unset or empty, `AgentCliSection::build_env()` returns `AgentCliEnvError::MissingApiKeyEnv`, and `hestia-runner` / `ai-conductor` fails before startup with `-32602 Invalid params` (fail-fast).

### 11.4 Log Output Masking

When launching child processes, logs display only the API key length in the format `ANTHROPIC_API_KEY=<set, len=N>`, not the actual API key value.

### 11.5 agent-cli IPC Registry

The `registry_dir` (default `$XDG_RUNTIME_DIR/agent-cli`) is created with permissions 0700 to prevent peer discovery and impersonation by other users.

---

## Related Documentation

- [Architecture Overview](architecture_overview.md) — Overall design principles (Principle 4 overview)
- [Container Execution](container_execution.md) — Podman container strategy and container-manager details
- [Shared Services](shared_services.md) — Observability-based metrics monitoring
- [Hestia Flow](hestia_flow.md) — Security-related items in AI utilization concepts
- `.hestia/doc/container/security_hardening.md` — Container security hardening details