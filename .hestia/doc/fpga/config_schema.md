# fpga-conductor Configuration Schema

**Target Conductor**: fpga-conductor
**Source**: Design Specification §5.4 (around lines 1542-1683)

## fpga.toml — Unified Project Format

A file that declaratively defines FPGA project settings, target definitions, toolchain constraints, IP management, and build configuration.

### Sections

| Section | Required | Description |
|---------|----------|-------------|
| `[project]` | Required | Project basic settings |
| `[targets.*]` | Required | Target device definitions (multiple allowed) |
| `[toolchain]` | Optional | Toolchain version constraints (semver) |
| `[toolchain.lock]` | Optional | Toolchain lock (reproducibility guarantee) |
| `[ip.*]` | Optional | IP core management |
| `[build]` | Optional | Build settings |
| `[sim]` | Optional | Simulation settings |

### `[project]` Section

| Field | Type | Description |
|-------|------|-------------|
| `name` | string | Project name |
| `version` | string | Version |
| `hdl_files` | string[] | HDL source files |
| `include_dirs` | string[] | Include directories |
| `testbenches` | string[] | Testbench files |

### `[targets.*]` Section

Define a separate section for each target.

| Field | Type | Description |
|-------|------|-------------|
| `vendor` | string | Vendor name (`xilinx` / `intel` / `efinix` / `yosyshq`) |
| `device` | string | Device name (e.g., `xc7a35tcsg324-1`) |
| `top` | string | Top module name |
| `constraints` | string[] | Constraint files (XDC / SDC / PCF / peri.xml) |
| `interface_script` | string | Efinity interface script (optional) |

### `[toolchain]` Section

| Field | Type | Description |
|-------|------|-------------|
| `vivado` | string | Vivado version constraint (semver, e.g., `>=2023.1, <2026`) |
| `quartus` | string | Quartus version constraint |
| `efinity` | string | Efinity version constraint |

### `[toolchain.lock]` Section

| Field | Type | Description |
|-------|------|-------------|
| `vivado` | string | Pinned version (e.g., `2025.2.0`) |
| `quartus` | string | Pinned version |
| `efinity` | string | Pinned version |

### `[ip.*]` Section

| Field | Type | Description |
|-------|------|-------------|
| `vendor` | string | IP vendor |
| `name` | string | IP core name |
| `version` | string | IP version |
| `config` | string | Configuration file path (.xci, etc.) |

### `[build]` Section

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `parallel_jobs` | integer | — | Number of parallel jobs |
| `incremental_compile` | boolean | — | Enable incremental compilation |
| `cache_dir` | string | `.fpga-cache` | Cache directory |

### `[sim]` Section

| Field | Type | Description |
|-------|------|-------------|
| `tool` | string | Simulation tool (`iverilog` / `modelsim` / `questa` / `xsim`) |
| `top_tb` | string | Top testbench name |
| `plusargs` | string[] | Plus arguments |

### Configuration Example

```toml
[project]
name    = "my_dsp_core"
version = "0.2.0"
hdl_files   = ["hdl/top.sv", "hdl/fir_filter.sv", "hdl/bram_ctrl.sv"]
include_dirs = ["hdl/include"]
testbenches = ["sim/tb_top.sv", "sim/tb_fir.sv"]

[targets.artix7_dev]
vendor      = "xilinx"
device      = "xc7a35tcsg324-1"
top         = "top"
constraints = ["constraints/artix7.xdc"]

[targets.cyclone10]
vendor      = "intel"
device      = "10CL025YU256C8G"
top         = "top"
constraints = ["constraints/cyclone10.sdc"]

[targets.trion_t20]
vendor            = "efinix"
device            = "T20F256"
top               = "top"
interface_script  = "constraints/trion_t20.peri.xml"

[targets.ice40]
vendor      = "yosyshq"
device      = "iCE40HX8K"
top         = "top"
constraints = ["constraints/ice40.pcf"]

[toolchain]
vivado   = ">=2023.1, <2026"
quartus  = "~23.1"
efinity  = "*"

[toolchain.lock]
vivado   = "2025.2.0"
quartus  = "23.1.1"
efinity  = "2025.2.0"

[ip.fifo_gen]
vendor  = "xilinx"
name    = "fifo_generator"
version = "13.2"
config  = "ip/fifo_gen.xci"

[build]
parallel_jobs       = 8
incremental_compile = true
cache_dir           = ".fpga-cache"

[sim]
tool    = "iverilog"
top_tb  = "tb_top"
plusargs = ["+DUMP_WAVES=1"]
```

## Related Documentation

- [fpga/binary_spec.md](binary_spec.md) — hestia-fpga-cli binary specification
- [fpga/vendor_adapter.md](vendor_adapter.md) — VendorAdapter trait
- [fpga/state_machines.md](state_machines.md) — Build state machine
- [../rtl/config_schema.md](../rtl/config_schema.md) — rtl.toml schema