# ai-conductor Configuration Schema

**Target Conductor**: ai-conductor
**Source**: Design Specification §3.8 (around lines 1109-1164), §3.9 (around lines 1189-...)

## container.toml

A file that declaratively defines the container environment used by each conductor. Only used when container execution is selected (not needed for local execution).

### Section List

| Section | Required | Description |
|-----------|------|------|
| `[container]` | Required | Container base configuration (name, base image, target conductor) |
| `[tools.*]` | Optional | Tool definitions to install |
| `[env]` | Optional | Environment variables |
| `[[volumes]]` | Optional | Volume mount definitions |
| `[health]` | Optional | Health check settings |
| `[update]` | Optional | Update policy |

### `[container]` Section

| Field | Type | Description |
|-----------|---|------|
| `name` | string | Container name |
| `base_image` | string | Base image (e.g., `ubuntu:24.04`) |
| `conductor` | string | Target conductor name |

### `[tools.*]` Section

| Field | Type | Description |
|-----------|---|------|
| `name` | string | Tool display name |
| `version` | string | Version constraint (semver, e.g., `>=2025.1`) |
| `install_script` | string | Install script |
| `version_cmd` | string | Version verification command |

### `[health]` Section

| Field | Type | Default | Description |
|-----------|---|-------|------|
| `cmd` | string | — | Health check command |
| `interval_secs` | integer | 60 | Polling interval (seconds) |
| `timeout_secs` | integer | 3 | Response timeout (seconds) |
| `max_retries` | integer | 3 | Consecutive retry count |
| `escalate_on_fail` | boolean | true | Notify frontend on consecutive failures |
| `restart_on_fail` | boolean | true | Attempt automatic restart |

### `[update]` Section

| Field | Type | Description |
|-----------|---|------|
| `auto` | boolean | Enable automatic updates |
| `schedule` | string | Cron schedule |
| `rollback_on_failure` | boolean | Automatic rollback on failure |

### Configuration Example

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

## upgrade.toml

Configuration file for sustainable upgrades (§3.4 UpgradeManager).

### Section List

| Section | Description |
|-----------|------|
| `[upgrade]` | Upgrade base configuration |
| `[strategy.major]` | Major version strategy |
| `[strategy.minor]` | Minor version strategy |
| `[strategy.patch]` | Patch version strategy |
| `[rollback]` | Rollback configuration |

### `[upgrade]` Section

| Field | Type | Description |
|-----------|---|------|
| `check_interval_hours` | integer | New version check interval (hours) |
| `auto_upgrade` | boolean | Enable automatic upgrades |
| `notification_email` | string | Notification email address |

### `[strategy.*]` Section

| Strategy Type | Description | Use Case |
|-----------|------|---------|
| `canary` | Deploy to a small number of environments first | Major version changes |
| `staging` | Deploy to production after verification in staging environment | Minor version updates |
| `production` | Deploy directly to production | Patch releases |

### `[rollback]` Section

| Field | Type | Default | Description |
|-----------|---|-------|------|
| `auto` | boolean | true | Enable automatic rollback |
| `timeout_secs` | integer | 300 | Timeout (seconds) |
| `max_retries` | integer | 3 | Maximum retry count |

### Configuration Example

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

## Related Documentation

- [ai/binary_spec.md](binary_spec.md) — hestia-ai-cli binary specification
- [ai/error_types.md](error_types.md) — ai-conductor error codes
- [ai/state_machines.md](state_machines.md) — Task state transitions
- [../common/container_manager.md](../container/container_manager.md) — Container management details