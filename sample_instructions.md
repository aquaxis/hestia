# FPGA Board Development Instruction Template for Hestia System

> This document is an **instruction template (sample)** for developing an evaluation board with Efinix TZ75 FPGA + RISC-V SoC + Real-Time OS using the Hestia AI agent system. Users can create `.aiprj/instructions.md` based on this document and run it through Hestia's `/setup_ai` -> `/update_ai` -> `/ai`(exec_job) flow to drive everything from requirements definition to PCB design, FPGA implementation, RTOS firmware, and PC-UART communication in an integrated manner.

---

## TL;DR

- **Goal**: Develop an evaluation board centered on the Efinix Titanium 75 with HyperRAM / 1GbE / USB-PD / MIPI-CSI / QSPI / I2C / 8-bit LED / GPIO expansion, covering the full range from **schematic -> artwork -> FPGA RTL -> RISC-V SoC -> RTOS -> PC-UART communication**.
- **Driving Engine**: Hestia's `pcb` / `fpga` / `rtl` / `hal` / `apps` / `debug` conductors and their sub-agents.
- **How to Use**: Finalize the hardware requirements in Section 4 -> Component selection in Section 5 -> Power estimation in Section 6 -> Dispatch Sections 7-13 sequentially via `agent-cli send <persona> "..."` -> Confirm completion with the Section 14 checklist.
- **Key Additions**: JTAG / bitstream SPI Flash / power sequencer / clock source / ESD protection / PCB layer stackup / I2C expansion / RTC / microSD / USB-Serial / mechanical dimensions / CI/CD are all covered as ADD-1 through ADD-21.

---

## 1. Project Overview

| Item | Content |
|------|---------|
| Project Name | Hestia FPGA Eval Board r0 |
| Target FPGA | Efinix Titanium 75 (`Ti75H486C4I7` primary, `Ti75H361` secondary) |
| Purpose | RISC-V SoC + RTOS evaluation, MIPI / 1GbE / HyperRAM verification |
| Estimated Power | USB-PD 9V/3A=27W profile (standard), 5V/3A=15W (minimum) |
| Estimated Board | 4 to 6 layers, approximately 100 x 80 mm, 4 mounting holes, USB-C / RJ45 / MIPI concentrated on one side |
| Target Audience | Users who provide this instruction document as `.aiprj/instructions.md` to Hestia |
| Estimated Timeline | Spec definition 1w -> Schematic 2w -> Artwork 2w -> FPGA / RTL 4w -> RTOS 2w -> Verification 2w (approx. 13 weeks) |

---

## 2. Prerequisites: Hestia System Agent Hierarchy

All instructions in this document are executed via `agent-cli send` to Hestia's personas.

```
ai-conductor
├── pcb-conductor
│   ├── pcb-designer       ... Hardware specification definition
│   ├── pcb-schematic      ... Schematic creation (KiCad sch)
│   ├── pcb-layout         ... PCB artwork (KiCad pcb)
│   ├── pcb-emi-analyzer   ... EMI / SI / PI analysis
│   └── pcb-tester         ... DRC / ERC / power-on testing
├── fpga-conductor
│   ├── fpga-designer      ... FPGA architecture specification
│   ├── fpga-synthesizer   ... Synthesis (Efinity)
│   ├── fpga-implementer   ... P&R / bitstream generation
│   ├── fpga-floorplanner  ... Floorplan / IO placement
│   ├── fpga-programmer    ... Board programming
│   └── fpga-tester        ... Board testing
├── rtl-conductor
│   ├── rtl-designer       ... RTL specification (RISC-V SoC)
│   ├── rtl-coder          ... RTL coding (SystemVerilog)
│   ├── rtl-formal-verifier... Formal verification
│   └── rtl-tester         ... Testbenches / cocotb
├── hal-conductor
│   ├── hal-designer       ... BSP / HAL specification
│   ├── hal-coder          ... BSP / driver implementation
│   └── hal-validator      ... HAL testing
├── apps-conductor
│   ├── apps-designer      ... Application / RTOS specification
│   ├── apps-coder         ... Application implementation (tasks, command processing)
│   ├── apps-builder       ... Cross-build / flash
│   └── apps-tester        ... SIL / HIL / QEMU testing
└── debug-conductor
    ├── debug-designer
    ├── debug-programmer
    ├── debug-analyzer
    ├── debug-coverage-analyzer
    └── debug-session-manager
```

### 2.1 Basic Dispatch Pattern

```bash
# When dispatching to a conductor (sub-agents are spawned automatically)
agent-cli send pcb "Please define the hardware specifications for the Hestia FPGA Eval Board r0. See .aiprj/instructions.md Sections 4-6 for details."

# When dispatching directly to a sub-agent
agent-cli send pcb-schematic "Please review the schematic draft v0.1."
```

### 2.2 Recommended Repository Structure (ADD-19)

```
<repo-root>/
├── .aiprj/                  <-- Hestia config / instructions / logs
├── hw/
│   ├── sch/                 <-- KiCad schematic
│   ├── pcb/                 <-- KiCad pcb
│   ├── bom/                 <-- BOM CSV
│   ├── gerbers/             <-- Manufacturing data
│   └── mech/                <-- Mechanical drawings / 3D STEP
├── rtl/                     <-- RISC-V SoC RTL
│   ├── src/
│   ├── sim/
│   └── constraints/
├── fw/
│   ├── bsp/                 <-- HAL / BSP
│   ├── app/                 <-- RTOS tasks / command processing
│   └── samples/host_cli/    <-- PC-side CLI
└── docs/                    <-- Schematic PDF / user manual
```

---

## 3. Project Goals

### 3.1 Completion Image

- A single USB-C cable provides power + debug + UART.
- Bitstream auto-loads from SPI NOR Flash (RTOS boots / heartbeat LED starts blinking within a few seconds of power-on).
- `ping` from the PC terminal returns `pong`.
- `led 0xAA` sets the 8-bit LED to `10101010`.
- `task list` enumerates RTOS tasks.
- Expansion headers (PMOD-compatible) allow adding I2C sensors and SPI devices.

### 3.2 Block Diagram (Text Description)

```
[USB-C] ─→ [USB-PD Controller TPS65987] ─→ [DC/DC x multiple] ─→ FPGA / peripherals
                              |
                              └→ [USB Data] ─→ [FT2232H] ─→ JTAG / UART → FPGA
[RJ45] ── [Magnetics] ── [PHY RTL8211F] ── (RGMII) ── [TZ75 FPGA]
[MIPI-CSI Connector] ─ (D-PHY 2-lane) ─ [TZ75]
[HyperRAM x 1] ── (HyperBus 1.8V 200MHz) ── [TZ75]
[microSD] ── (SDIO 4-bit) ── [TZ75]
[I2C Expansion] ── [RTC DS3231 / EEPROM / sensor] ── [TZ75]
[8-bit LED] / [PWR/FAULT/heartbeat LED] / [GPIO 32-pin header] ── [TZ75]
[Bitstream SPI Flash MX25L25645G] ── (QSPI) ── [TZ75 config dedicated]
[QSPI External Peripherals (Flash / EEPROM etc.)] ── (QSPI) ── [TZ75 SoC general-purpose QSPI]
[Power Sequencer TPS3895/LM3880] ── [DC/DC enable]
```

---

## 4. Hardware Requirements Specification

Dispatch the specification finalization task to `pcb-designer` and `fpga-designer` in parallel.

### 4.1 Requirements Table

| Category | Requirement | Target Value / Notes | Required/Optional |
|---------|-------------|---------------------|-------------------|
| FPGA | Efinix TZ75 | `Ti75H486C4I7` (primary) / `Ti75H361` (secondary) | Required |
| Memory | HyperRAM | 64 Mbit, 1.8V, 200 MHz, `S27KL0641DABHI020` 1-2 units | Required |
| Ethernet | 1GbE | RJ45 + magnetics + RGMII PHY | Required |
| Power | USB-C / USB-PD | 9V/3A=27W profile default, 5V/3A compatible | Required |
| MIPI-CSI | Camera input | 2-lane (4-lane max expansion), Hirose FH52 series | Required |
| GPIO | Expansion header | PMOD-compatible x 2, 3.3V LVCMOS, 32+ pins total | Required |
| LED (User) | 8-bit individual control | Green or white, via 3.3V resistor | Required |
| LED (Status) (ADD-10)| PWR / FAULT / heartbeat | 3 units, PWR green / FAULT red / heartbeat yellow | Required |
| JTAG (ADD-1, ADD-8, ADD-17)| FPGA / RISC-V shared | On-board FT2232H + external 10-pin/20-pin header | Required |
| QSPI Flash (ADD-2)| Bitstream storage (FPGA config dedicated) | 32 MB QSPI NOR, `MX25L25645G` | Required |
| QSPI (SoC general-purpose) | External QSPI bus accessible from SoC | Connectable to external QSPI Flash / EEPROM via expansion header (capacity and part number at user's choice), 3.3V LVCMOS, max 50 MHz | Required |
| Power Sequencer (ADD-3)| Multi-rail boot order control | `LM3880` or `TPS3895` x as needed | Required |
| Clock (ADD-4)| FPGA / Ethernet / MIPI | 25 MHz xtal x 2, 27 MHz xtal x 1, or `Si5351A` | Required |
| ESD Protection (ADD-5)| All external IOs | TVS on USB / RJ45 / MIPI / GPIO | Required |
| PCB Layer Count (ADD-6)| High-speed differential | 6 layers recommended (4 minimum) | Required |
| I2C Expansion (ADD-12)| Peripheral connection | 3.3V, 400kHz, header + RTC + EEPROM | Required |
| RTC (ADD-13)| Time synchronization | `DS3231` (I2C) + coin cell holder | Required |
| microSD (ADD-14)| Storage | SDIO 4-bit, push-push slot | Required |
| USB-Serial (ADD-15)| PC-UART | FT2232H shared (ADD-17), or CP2102N on separate pins | Required |
| Mechanical Dimensions (ADD-16)| Enclosure integration | 100 x 80 mm, 4 corners M3 mounting holes, connectors on one side | Required |
| Monitor / Test (ADD-11)| Major rails / clocks | Test points on each power, clock, and reset rail | Recommended |

### 4.2 Dispatch Example

```bash
agent-cli send pcb-designer "
Based on the hardware requirements table in Section 4 of .aiprj/instructions.md, create
<workspace>/pcb-designer/{requirements,design,tasks}.md for the Hestia FPGA Eval Board r0.

Considerations:
- FPGA: Efinix Ti75H486C4I7 as primary candidate. Verify pin count sufficiency and MIPI / SerDes placement
- Clearly separate required items from recommended/optional items
- Include all hardware-related items from ADD-1 through ADD-17 in the requirements
"

agent-cli send fpga-designer "
Evaluate the same requirements (Section 4) from an FPGA perspective, and determine whether
Ti75H486 is sufficient in terms of LE / DSP / BRAM / SerDes / GPIO pin count. If insufficient,
propose upgrading to Ti120 / Ti180.
"
```

---

## 5. Component Selection Instructions

Delegate component selection to `pcb-designer` + `pcb-schematic`.

### 5.1 Primary Component Candidates Table

| Category | Candidate Example | Selection Criteria |
|---------|-------------------|--------------------|
| FPGA | `Ti75H486C4I7` / `Ti75H361` | LE count, MIPI lane count, I/O count, availability |
| HyperRAM | `S27KL0641DABHI020` (64Mbit/1.8V/200MHz) | Capacity, frequency, voltage, HyperBus compatibility |
| Ethernet PHY | `RTL8211F-CG` / `KSZ9031RNX` | RGMII, temperature range, availability |
| RJ45 + Mag | Pulse `JK0-0177NL` / Bel Fuse `0826-1X1T-21-F` | Integrated magnetics, LED indicators |
| USB-PD | TI `TPS65987DDH` / Cypress `CYPD3175` | PD3.0, sink capability, I2C control |
| USB-C Connector | Amphenol `12401610E4#2A` / GCT `USB4105-GF-A` | 24-pin full, shielded, SMD |
| DC/DC Core | TI `TPS62810` (4A, 0.6V+) | Efficiency, transient response, size |
| DC/DC IO | TI `TPS54824` (8A) | Current headroom |
| LDO Analog | TI `TPS7A47` (low noise) | Noise suppression for PHY / MIPI |
| Power Sequencer (ADD-3) | TI `LM3880` (3 rails) or `TPS3895` x needed | Rail order guarantee |
| Clock (ADD-4) | 25 MHz xtal `ABM10W-25.000MHZ-K4Z-T3` x 2 + 27 MHz `ECS-270-8-30B-CKM-TR` | Frequency accuracy, jitter |
| Programmable PLL | Silicon Labs `Si5351A-B-GT` | If needed, generate MIPI / Ethernet frequencies |
| Config Flash (ADD-2) | Macronix `MX25L25645GMI-08G` (32MB) | Capacity, QSPI, Efinix recommended |
| RTC (ADD-13) | Maxim `DS3231SN#` (+/-2ppm) | High accuracy, I2C |
| ESD (ADD-5) | TI `TPD4S009DRYR` (USB), `TPD4E1U06DCKR` (general), Onsemi `ESD7L5.0DT5G` (RJ45) | Clamp voltage, capacitance |
| USB-Serial / JTAG (ADD-15, ADD-17) | FTDI `FT2232HL` (Channel A=JTAG, Channel B=UART) | OpenOCD support, standard Linux driver |
| microSD (ADD-14) | Hirose `DM3AT-SF-PEJM5` push-push | SDIO 4-bit, hot-plug |
| MIPI Connector | Hirose `FH52-15S-0.5SH` 15-pin 0.5mm FPC | Raspberry Pi compatible |
| LED (User) | 0805 green LED + 1kOhm x 8 | Individual control |
| LED (Status, ADD-10) | Kingbright `APT2012LSECK` (green/red) x 3 | PWR / FAULT / heartbeat |
| Headers / Connectors | 2.54mm x 2-row PMOD (GPIO) / 10-pin Cortex Debug (JTAG) | Expandability |
| Coin Cell Holder (RTC) | Renata `MS621` / Keystone 1051 | RTC backup |

### 5.2 Dispatch Example

```bash
agent-cli send pcb-schematic "
Expand the component candidate table in Section 5 into BOM draft v0.1.

Requirements:
- For each category, list the primary and secondary candidates with selection reasons on one line
- Include DigiKey / Mouser stock quantities and unit prices (no API needed, approximate latest values are fine)
- Note pin-compatible / footprint-compatible alternatives for each component
- Output to: <workspace>/pcb-schematic/bom_v0_1.csv

Final approval is subject to user review.
"
```

### 5.3 Component Selection Self-Check

- [ ] FPGA is confirmed as Ti75H486 primary candidate with LE / I/O / MIPI count meeting requirements
- [ ] All components are confirmed for industrial temperature range (-40 to 85C), or commercial range is acceptable
- [ ] PCB manufacturer capability for BGA 0.5mm pitch (minimum via diameter, layer count) is confirmed
- [ ] ESD / TVS is placed on all external IOs: USB / RJ45 / MIPI / GPIO
- [ ] RoHS compliance / alternatives for hard-to-source components are identified

---

## 6. Power Design and Power Consumption Estimation

Delegate power design to `pcb-designer` + `pcb-schematic`.

### 6.1 Power Rail Definition (Ti75H486 assumed)

| Rail | Voltage | Estimated Current | Usage | Estimated Power | Supply |
|------|---------|-------------------|-------|----------------|--------|
| VCC | 0.85V | 1.5A | FPGA core | 1.28W | TPS62810 |
| VCCAUX | 1.8V | 0.3A | FPGA aux / config | 0.54W | TPS62810 |
| VCCIO_HRAM | 1.8V | 0.3A | HyperRAM IO | 0.54W | Shared with VCCAUX rail |
| VCCIO_GPIO | 3.3V | 0.4A | LED / GPIO / I2C / SPI | 1.32W | TPS62810 |
| VCCIO_MIPI | 1.2V | 0.2A | MIPI D-PHY | 0.24W | TPS62810 |
| 3.3V_PHY | 3.3V | 0.3A | Ethernet PHY | 0.99W | LDO TPS7A47 (low noise) |
| 3.3V_AUX | 3.3V | 0.3A | USB-PD / FT2232H / RTC | 0.99W | TPS62810 |
| 1.8V_AUX | 1.8V | 0.1A | RTC backup (coin cell) / Flash IO | 0.18W | LDO |

Total estimated **~6.1W**. Approximately 23% utilization of USB-PD 9V/3A=27W, with over 4x headroom. Standard profile is 9V; 5V/3A=15W is the minimum (with some peripheral restrictions assumed).

### 6.2 Power Sequencing (ADD-3)

Ti75H486 boot sequence rules (per Efinix Ti75 datasheet Power-up Sequence chapter):

```
T0:        VCC (0.85V)        ramp up
T0+1ms:    VCCAUX (1.8V)      ramp up
T0+2ms:    VCCIO_HRAM (1.8V)  / VCCIO_MIPI (1.2V)
T0+3ms:    VCCIO_GPIO (3.3V)  / 3.3V_PHY / 3.3V_AUX

Power-down is in reverse order.
```

Implementation: Combine 1x `LM3880` or 2-3x `TPS3895`, sequentially asserting each DC/DC's `EN` pin. Instruct `pcb-designer` to "Use 1x LM3880 to control rail order in 4 stages, with remaining rails controlled by EN-derived signals."

### 6.3 Power Consumption Estimation Dispatch Example

```bash
agent-cli send pcb-designer "
Using the rail table in Section 6.1 as a starting point, re-estimate the Ti75H486 core
power consumption based on the Efinity Power Estimator output.

Inputs:
- RTL configuration: RV32IMC + peripherals (UARTx2 / GPIO / Ethernet MAC / HyperBus / MIPI-CSI 2-lane)
- Operating frequency: core 100MHz, RGMII 125MHz, MIPI 200MHz
- Temperature: 25C TJ, 85C TJ (2 cases)

Outputs:
- <workspace>/pcb-designer/power_budget_v0_1.md (by rail / by usage / by temperature)
- USB-PD 9V/3A profile utilization report
- DC/DC selection recommendation after derating (x 1.5)
"
```

### 6.4 Decoupling Strategy (ADD-7)

- VCC (0.85V): 8-12x 0402 100nF directly under BGA + 4x 22uF bulk MLCC
- VCCIO_*: 100nF + 1uF + 10uF per ball group on shortest path
- All bulk: 1x 100uF tantalum / polymer near input

Instruct `pcb-layout` to "Verify power impedance from 100kHz to 1GHz in PI simulation, target < 30 mOhm."

### 6.5 Verification Commands / Expected Results

```bash
# Power sequence measurement (pcb-tester)
oscilloscope_trigger.py --channels VCC VCCAUX VCCIO_GPIO VCCIO_HRAM
# Expected: Sequence in Section 6.2 is met (0 order violations)
```

---

## 7. Schematic Design

Dispatch to `pcb-schematic`.

### 7.1 Net Naming Convention

- Net names use ALL_CAPS with underscores (e.g., `USB_VBUS`, `FPGA_VCC`, `MIPI0_D0_P` / `MIPI0_D0_N`)
- Differential pairs use `_P` / `_N` suffix
- Bus signals use `[7:0]` notation (e.g., `LED[7:0]`)
- Power nets are color-coded (KiCad convention)

### 7.2 Sheet Structure (KiCad)

| Sheet | Content |
|-------|---------|
| `top.sch` | Block diagram, external connection map |
| `power.sch` | USB-PD / DC/DC / sequencer / bulk (ADD-3, ADD-7) |
| `fpga.sch` | TZ75 pin assignment, decoupling, power connections |
| `clock.sch` | xtal x 25 MHz / 27 MHz, Si5351 (optional) (ADD-4) |
| `config.sch` | Bitstream SPI Flash + JTAG header (ADD-1, ADD-2, ADD-17) |
| `memory.sch` | HyperRAM x 1-2 + decoupling |
| `ethernet.sch` | RJ45 + Magnetics + PHY + ESD (ADD-5) |
| `usb.sch` | USB-C connector + USB-PD + FT2232H + ESD |
| `mipi.sch` | MIPI-CSI connector + ESD |
| `gpio_led.sch` | Expansion header, 8-bit user LED, 3x status LED (ADD-10) |
| `i2c_aux.sch` | RTC DS3231, I2C expansion header, EEPROM, coin cell (ADD-12, ADD-13) |
| `sd.sch` | microSD connector (ADD-14) |
| `testpoints.sch` | Test points on major nets (ADD-11) |

### 7.3 Dispatch Example

```bash
agent-cli send pcb-schematic "
Create a KiCad 7.x project hw/sch/board.kicad_pro with the sheet structure in Section 7.2,
and reflect the BOM v0.1 from Section 5 into the symbols.

Instructions:
- Place ESD / TVS on all external IOs (ADD-5)
- Build 4-stage power sequencing with LM3880 (ADD-3)
- Place 3x status LEDs (PWR/FAULT/heartbeat) (ADD-10)
- Place at least 12 test points on power rails / 25MHz xtal / reset signals (ADD-11)
- Commit with a clean ERC pass
Output: hw/sch/ complete set + ERC report
"
```

### 7.4 Verification Commands / Expected Results

```bash
kicad-cli sch erc --severity-error hw/sch/board.kicad_sch
# Expected: ERC violations: 0
kicad-cli sch export pdf -o docs/board_schematic.pdf hw/sch/board.kicad_sch
# Expected: PDF generated, organized by section
```

---

## 8. PCB Artwork

Dispatch to `pcb-layout` + `pcb-emi-analyzer`.

### 8.1 Layer Stackup (ADD-6)

| Layer | Usage (6-layer proposal) |
|-------|--------------------------|
| L1 | Signal (including high-speed), connectors |
| L2 | GND (reference) |
| L3 | Power plane (VCCIO_GPIO / 3.3V rails) |
| L4 | Power plane (VCC / VCCAUX / 1.8V rails) |
| L5 | GND (reference) |
| L6 | Signal (low-speed / GPIO / I2C / control) |

Differential pairs (MIPI / RGMII / HyperBus / USB / SDIO) primarily on L1, with L6 as backup.

### 8.2 Routing Guidelines

- **MIPI D-PHY**: 100 Ohm differential, length matching +/-5mil, shortest path from connector
- **RGMII**: 50 Ohm SE, length matching TXC/TXD, RXC/RXD
- **HyperBus**: Short distance (< 50mm recommended), strict length matching, continuous GND reference plane
- **USB 2.0/3.0**: 90 Ohm differential, minimize bends, shortest stub after ESD
- **SDIO**: CLK slightly longer than other lines (skew mitigation)
- **Power**: Star-point GND, bulk near input, decoupling directly under BGA

### 8.3 Mechanical Dimensions (ADD-16)

- Board outline: 100 x 80 mm (tentative)
- 4 corners M3 mounting holes (inner diameter 3.2mm, land 6mm, 5mm keepout)
- USB-C / RJ45 / MIPI connectors: concentrated on one side (short edge)
- PMOD headers: opposite side
- Height restriction: top side 12mm (including connectors) / bottom side 3mm

### 8.4 Dispatch Example

```bash
agent-cli send pcb-layout "
Create hw/pcb/board.kicad_pcb from hw/sch/board.kicad_sch with a 6-layer stackup.

Constraints:
- 100 x 80 mm rectangle, 4 corners M3 mounting holes
- Layer stackup per Section 8.1
- Routing guidelines per Section 8.2 (differential length matching, continuous GND reference)
- Target DRC violations: 0 (IPC class 2, minimum via 0.2mm/0.4mm)

Output: hw/pcb/ complete set + DRC report + 3D STEP for verification
"

agent-cli send pcb-emi-analyzer "
Analyze the power integrity (PI) of hw/pcb/board.kicad_pcb and report whether
each rail meets impedance < 30mOhm from 100kHz to 1GHz.
Propose bulk additions or layout changes where insufficient.
"
```

### 8.5 Verification Commands / Expected Results

```bash
kicad-cli pcb drc --severity-error hw/pcb/board.kicad_pcb
# Expected: DRC violations: 0
kicad-cli pcb export gerbers -o hw/gerbers/ hw/pcb/board.kicad_pcb
kicad-cli pcb export step -o hw/mech/board.step hw/pcb/board.kicad_pcb
# Expected: gerber set, 3D STEP output
```

---

## 9. FPGA Circuit Design

In order: `fpga-designer` -> `fpga-synthesizer` -> `fpga-implementer` -> `fpga-floorplanner`.

### 9.1 Internal Architecture (Ti75H486 assumed)

```
┌─────────────────────────────────────────────────────────────┐
│  TZ75 (Ti75H486)                                            │
│                                                             │
│  ┌────────────┐  ┌────────────┐  ┌────────────────┐         │
│  │ RV32IMC    │  │ AXI4 Inter │  │ HyperBus Ctrl  │         │
│  │ (RISC-V)   │<-│ connect    │->│ (1.8V/200MHz)  │-> HyperRAM
│  └────────────┘  └─┬─┬─┬─┬─┬──┘  └────────────────┘         │
│                    │ │ │ │ │                                │
│           ┌────────┘ │ │ │ └────────┐                       │
│           ↓          ↓ ↓ ↓          ↓                       │
│        ┌──────┐ ┌──────┐ ┌──────┐ ┌──────────┐ ┌──────────┐ │
│        │ UART │ │ GPIO │ │Timer │ │ Eth MAC  │ │ MIPI-CSI │ │
│        │ x2   │ │ 32b  │ │ x2   │ │ (RGMII)  │ │ RX 2-lane│ │
│        └──────┘ └──────┘ └──────┘ └──────────┘ └──────────┘ │
│        ┌──────┐ ┌──────┐ ┌──────┐ ┌──────────┐              │
│        │ I2C  │ │ SPI  │ │ SDIO │ │ PLIC +   │              │
│        │ x2   │ │ x1   │ │      │ │ CLINT    │              │
│        └──────┘ └──────┘ └──────┘ └──────────┘              │
│                                                             │
│  Clock: 25MHz xtal -> PLL -> core 100MHz / RGMII 125MHz    │
│                          / MIPI 200MHz / HyperBus 200MHz    │
└─────────────────────────────────────────────────────────────┘
```

### 9.2 Resource Estimation (Requirements Section 3.FR-7)

| Block | LE | DSP | BRAM | Notes |
|-------|----|----|------|-------|
| RV32IMC core | ~10K | 0-4 | 4-8 | RISC-V standard config |
| AXI4 interconnect | ~3K | 0 | 0 | 4 master / 8 slave |
| HyperBus controller | ~5K | 0 | 2 | Cypress HyperBus IP |
| Ethernet MAC | ~8K | 0 | 4 | RGMII + checksum offload |
| MIPI-CSI 2-lane RX | ~6K | 0 | 0 | D-PHY hard IP utilized |
| Peripherals (UART/GPIO/Timer/I2C/SPI/QSPI/SDIO/PLIC/CLINT) | ~8K | 0 | 2 | |
| **Total** | **~40K** | **~8** | **~20** | **~53% utilization of TZ75's 75K LE, 47% headroom** |

### 9.3 Dispatch Example

```bash
agent-cli send fpga-designer "
Using Sections 9 and 10 of .aiprj/instructions.md as input, create
<workspace>/fpga-designer/{requirements,design,tasks}.md.

Requirements:
- FPGA: Efinix Ti75H486C4I7
- Clock plan: 25MHz xtal -> PLL x 1 -> core 100MHz / RGMII 125MHz / MIPI 200MHz / HyperBus 200MHz
- IO constraints: Pinout consistent with physical placement in Section 8
- Block configuration: Per diagram in Section 9.1
- Optimize peripheral placement (QSPI / I2C expansion header etc.) based on remaining pins
"

agent-cli send fpga-synthesizer "
Based on RTL (generated in Section 10) and <workspace>/fpga-designer/design.md constraints,
synthesize with Efinity v2024.x.

Output:
- bitstream (hw/fpga/build/board.hex)
- Resource report, timing report, power estimation
- If synthesis fails, request fix from rtl-conductor
"

agent-cli send fpga-floorplanner "
Concentrate high-speed IO banks for MIPI / HyperBus / RGMII.
Fix JTAG / config Flash SPI pins to bank 0 / dedicated.
Output: hw/fpga/constraints/board.pdc
"
```

### 9.4 Verification Commands / Expected Results

```bash
# Synthesis (Efinity project)
efx_run --flow compile --project hw/fpga/board.xml
# Expected: timing slack > 0, LUT < 75% utilization, bitstream generated

# Board programming
agent-cli send fpga-programmer "Write hw/fpga/build/board.hex to SPI Flash via FT2232H"
# Expected: Write successful, LED heartbeat blinking after restart
```

---

## 10. RISC-V SoC Implementation

In order: `rtl-designer` -> `rtl-coder` -> `rtl-formal-verifier` -> `rtl-tester`.

### 10.1 ISA / Core Selection

This template does not specify a particular RISC-V IP. Select any RISC-V core that meets the following requirements, provided as a pre-generated Verilog, and integrate it into the SoC via a SystemVerilog wrapper:

- **ISA**: RV32IMC (M-mode + U-mode, no MMU, no FPU)
- **Bus**: AXI4-lite (peripherals) + AXI4 (HyperRAM high-speed side)
- **Interrupts**: PLIC + CLINT compatible
- **Size guideline**: Approximately 1,500-10,000 LUT4s relative to TZ75's 75K LE (maintain 30%+ resource headroom)
- **Language**: Input is Verilog (SystemVerilog compatible). If generated from other HDL, convert to Verilog before integration.

> **HDL Constraint**: RTL hand-written by rtl-coder shall use **SystemVerilog exclusively**.
> RISC-V core IPs shall be adopted in pre-generated Verilog format and integrated into the SoC
> via a SystemVerilog wrapper within Hestia.

### 10.2 SoC Specification (Including ADD-9 Watchdog)

```
ISA:        RV32IMC, no MMU, no FPU
Privilege:  M-mode + U-mode (for RTOS)
Bus:        AXI4-lite (peripherals) + AXI4 (HyperRAM high-speed side)
Interrupts: PLIC (32 source) + CLINT (mtime)
Watchdog:   1 internal IP (RTOS kicks periodically, resets after 4 seconds)
Boot ROM:   16 KB (block RAM, SPI Flash -> HyperRAM copy logic at boot)
SRAM:       32 KB (block RAM, stack / IRQ save area)
HyperRAM:   8 MB (external, RTOS + application execution area)
```

### 10.3 Memory Map

| Address | Region | Size |
|---------|--------|------|
| `0x0000_0000` | BootROM | 16 KB |
| `0x1000_0000` | SRAM | 32 KB |
| `0x2000_0000` | HyperRAM | 8 MB |
| `0x4000_0000` | UART0 | 4 KB |
| `0x4000_1000` | UART1 | 4 KB |
| `0x4000_2000` | GPIO | 4 KB |
| `0x4000_3000` | LED (8-bit) | 4 KB |
| `0x4000_4000` | Timer0 / Timer1 | 8 KB |
| `0x4000_6000` | I2C0 / I2C1 | 8 KB |
| `0x4000_8000` | SPI0 | 4 KB |
| `0x4000_9000` | SDIO | 4 KB |
| `0x4000_A000` | QSPI Flash Controller | 4 KB |
| `0x4001_0000` | Ethernet MAC | 16 KB |
| `0x4002_0000` | MIPI-CSI RX | 16 KB |
| `0x4003_0000` | Watchdog | 4 KB |
| `0x0C00_0000` | CLINT (mtime/mtimecmp) | 64 KB |
| `0x0F00_0000` | PLIC | 64 MB |

### 10.4 Boot Sequence

```
1. POR -> BootROM starts (0x00000000)
2. DMA copy RTOS image from SPI Flash offset 0x00010000 to HyperRAM (0x20000000)
3. If CRC32 check passes, jump to 0x20000000
4. RTOS initialization -> create tasks -> start scheduler
```

### 10.5 Dispatch Example

```bash
agent-cli send rtl-designer "
Based on the SoC specification in Section 10 (ISA, bus, memory map, boot),
create <workspace>/rtl-designer/{requirements,design,tasks}.md.

Reference: Design a top-level SoC integrating AXI4-lite peripherals and AXI4 HyperRAM
based on the selected RISC-V core's standard config
"

agent-cli send rtl-coder "
Implement the following in rtl/src/:
- soc_top.sv (top wrapper)
- riscv_core_wrapper.sv (Verilog wrapper for RISC-V core IP)
- axi_interconnect.sv
- bootrom.sv (init code hex in localparam)
- hyperram_ctrl.sv
- uart_16550.sv x2
- gpio.sv / led.sv / timer.sv / i2c.sv / spi.sv / qspi.sv / sdio.sv
- watchdog.sv (ADD-9)
- plic.sv / clint.sv
- eth_mac_wrapper.sv (RGMII)
- mipi_csi_rx.sv

Place cocotb tests for each module in rtl/sim/<module>/
"

agent-cli send rtl-formal-verifier "
Formally verify safety properties of plic / clint / watchdog / axi_interconnect
(no deadlock, acknowledgment always returned, watchdog does not reset if kicked
within timeout) using SymbiYosys.
"

agent-cli send rtl-tester "
Run cocotb tests for all modules + RV32 ISA test (riscv-tests rv32ui-p-*)
on Verilator, targeting coverage > 90%. Report results.
"
```

### 10.6 Verification

```bash
# Simulation
make -C rtl/sim verilator
# Expected: All cocotb tests pass, ISA tests pass

# Synthesis (feed to fpga-synthesizer in Section 9)
agent-cli send fpga-synthesizer "Synthesize rtl/src/ with Efinity"
```

---

## 11. Real-Time OS

Split between `hal-designer` / `hal-coder` / `hal-validator` and `apps-designer` / `apps-coder` / `apps-builder`.

### 11.1 RTOS Candidate Comparison

| Candidate | RV32IMC Port | License | Footprint | Community | Decision |
|-----|--------------|-----------|------------|-----------|----|
| **FreeRTOS 10.6.x** (primary) | Available | MIT | 6-10 KB | Large | Adopted |
| Zephyr 3.6.x | Available | Apache-2.0 | 30-100 KB | Large | Secondary |
| RT-Thread | Available | Apache-2.0 | 8-20 KB | Medium (China) | Secondary |
| NuttX | Available | Apache-2.0 | 30-80 KB | Medium | Not adopted |

Selection rationale: Small RTOS footprint, license compatibility, resource headroom, driver self-implementation assumed.

### 11.2 BSP / Driver Configuration (Including ADD-12, ADD-13, ADD-14, ADD-9)

```
fw/bsp/
├── crt0.S                     <- Reset vector, stack init, jump main
├── linker.ld                  <- 0x0000_0000 ROM, 0x1000_0000 SRAM, 0x2000_0000 HyperRAM
├── clock.c / clock.h          <- PLL config readout
├── hyperram_init.c            <- HyperRAM calibration
├── uart.c / uart.h            <- UART0 (debug) / UART1 (RTOS console)
├── gpio.c / led.c             <- LED / GPIO registers
├── i2c.c                      <- I2C0 / I2C1 (DS3231 / EEPROM)
├── rtc.c                      <- DS3231 driver (ADD-13)
├── sdio.c                     <- microSD (FATFS integration) (ADD-14)
├── eth.c                      <- lwIP integration (optional)
├── timer.c                    <- FreeRTOS tick via mtime
├── plic.c / clint.c
├── watchdog.c                 <- Periodic kick from FreeRTOS task (ADD-9)
└── irq.c                      <- Interrupt dispatch
```

### 11.3 Task Configuration

| Task | Priority | Stack | Role |
|------|----------|-------|------|
| heartbeat | 1 | 256 W | Toggle status LED at 1Hz (heartbeat) |
| uart_rx | 4 | 512 W | UART1 receive -> push to command queue |
| cmd_proc | 3 | 1024 W | Queue pop -> parse -> dispatch |
| uart_tx | 3 | 512 W | Pop from response queue -> UART1 transmit |
| watchdog_kick | 2 | 128 W | Kick watchdog at 1Hz |
| eth_lwip | 2 | 1024 W | lwIP main loop (optional) |

### 11.4 Dispatch Example

```bash
agent-cli send hal-designer "
Based on the configuration in Section 11, define the HAL / BSP specification for fw/bsp/.
Output: <workspace>/hal-designer/{requirements,design,tasks}.md

Considerations:
- Memory map (Section 10.3)
- All register access via volatile pointers
- Interrupt handlers use naked + IRQ context
"

agent-cli send hal-coder "
Implement each file in Section 11.2 under fw/bsp/.

Constraints:
- C99, no library dependencies (newlib-nano allowed)
- Reentrant
- Must work using SRAM/ROM only before HyperRAM initialization
- Provide mock headers for each driver (fw/bsp/test/mocks/)
"

agent-cli send hal-validator "
Test each driver in fw/bsp/ using QEMU + Verilator co-simulation.
Target coverage > 80%.
"

agent-cli send apps-designer "
Expand the task configuration in Section 11.3 into
<workspace>/apps-designer/design.md.
Map it to the command protocol (Section 12).
"

agent-cli send apps-coder "
Implement main.c + tasks/ under fw/app/.

Tasks:
- main.c: HAL init -> FreeRTOS start -> create tasks -> vTaskStartScheduler()
- tasks/heartbeat.c
- tasks/uart_rx.c
- tasks/cmd_proc.c (implement command set from Section 12)
- tasks/uart_tx.c
- tasks/watchdog.c
- (optional) tasks/eth_lwip.c
"

agent-cli send apps-builder "
Cross-build fw/ using riscv32-unknown-elf-gcc, outputting:
- ELF / map / sym / hex / bin
- size report
- full objdump
to artifacts/. Also format hex for SPI Flash writing (offset 0x10000).
"
```

---

## 12. PC-UART Communication

Dispatch to `apps-coder` + `apps-tester`.

### 12.1 Physical Layer (ADD-15, ADD-17)

- USB-C via FT2232H -> Channel A=JTAG (for FPGA / RISC-V), Channel B=UART (RTOS console)
- Speed: 115200 bps default, expandable to 921600 bps
- Configuration: 8N1, hardware flow control OFF

### 12.2 Protocol

Text command format (LF-terminated, ASCII) is the default. Minimum command set:

| Command | Example | Response | Description |
|---------|---------|----------|-------------|
| `ping` | `ping\n` | `pong\n` | Alive check |
| `version` | `version\n` | `Hestia FPGA r0 v0.1.0\n` | Version |
| `uptime` | `uptime\n` | `uptime: 12345 ticks\n` | RTOS tick count |
| `task list` | `task list\n` | Tabular response | List tasks |
| `led <bits>` | `led 0xAA\n` | `OK\n` | Set 8-bit LED |
| `mem r <addr>` | `mem r 0x40003000\n` | `0x000000AA\n` | Read memory |
| `mem w <addr> <val>` | `mem w 0x40003000 0xFF\n` | `OK\n` | Write memory |
| `i2c scan <bus>` | `i2c scan 0\n` | Address list | I2C device scan |
| `rtc get` / `rtc set` | `rtc get\n` | `2026-05-07 12:34:56\n` | RTC access |
| `reset` | `reset\n` | (No response, restart) | Soft reset |

Future expansion: Binary transfer mode can be added via SLIP / COBS framing.

### 12.3 Host CLI (Python pyserial example)

```python
# samples/host_cli/cli.py
import serial, sys, argparse

def main():
    p = argparse.ArgumentParser()
    p.add_argument('--port', default='/dev/ttyUSB1')
    p.add_argument('--baud', type=int, default=115200)
    p.add_argument('cmd', nargs='+')
    args = p.parse_args()
    with serial.Serial(args.port, args.baud, timeout=2) as s:
        s.write((' '.join(args.cmd) + '\n').encode())
        print(s.read_until(b'\n').decode().rstrip())

if __name__ == '__main__':
    main()
```

Usage example:
```bash
python samples/host_cli/cli.py --port /dev/ttyUSB1 ping
# -> pong
python samples/host_cli/cli.py led 0xAA
# -> OK
```

### 12.4 Host CLI (Rust example, samples/host_cli/Cargo.toml + src/main.rs)

```toml
[package]
name = "hestia-fpga-cli"
version = "0.1.0"
edition = "2021"

[dependencies]
serialport = "4"
clap = { version = "4", features = ["derive"] }
```

```rust
// samples/host_cli/src/main.rs
use std::time::Duration;
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "/dev/ttyUSB1")]
    port: String,
    #[arg(long, default_value_t = 115200)]
    baud: u32,
    cmd: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let a = Args::parse();
    let mut p = serialport::new(&a.port, a.baud)
        .timeout(Duration::from_secs(2))
        .open()?;
    let line = format!("{}\n", a.cmd.join(" "));
    p.write_all(line.as_bytes())?;
    let mut buf = [0u8; 256];
    let n = p.read(&mut buf)?;
    print!("{}", std::str::from_utf8(&buf[..n])?);
    Ok(())
}
```

### 12.5 Dispatch Example

```bash
agent-cli send apps-coder "
Implement the command set from Section 12.2 in fw/app/tasks/cmd_proc.c,
and create samples/host_cli/cli.py and samples/host_cli/src/main.rs.

Requirements:
- Parser based on strtok (256B buffer)
- Unknown commands respond with 'ERR: unknown'
- Memory writes are allowed only for addresses >= 0x4000_0000 (BootROM protection)
"

agent-cli send apps-tester "
Send 1000 pings from the host and measure average response time and drop rate.
Benchmark: average < 5ms, loss = 0
"
```

---

## 13. Verification / Integration

| Stage | Responsible Persona | Content | Verification Command |
|-------|---------------------|---------|---------------------|
| Board level | `pcb-tester` | Power-on test, power sequence measurement, temperature distribution | Oscilloscope + thermography |
| FPGA level | `fpga-tester` | Bitstream programming, LED blink, UART loopback | `agent-cli send fpga-tester` |
| RTL level | `rtl-tester` | cocotb / Verilator UVM, RV32 ISA test | `make -C rtl/sim verilator` |
| RTOS level | `apps-tester` | Boot time, task switch measurement, UART throughput | Measure from host CLI |
| Integration | `debug-session-manager` | E2E, log aggregation, coverage integration | `agent-cli send debug` |

### 13.1 CI/CD (ADD-20)

```yaml
# .github/workflows/hw_fw.yml (example)
jobs:
  hw_drc:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: kicad-cli sch erc --severity-error hw/sch/board.kicad_sch
      - run: kicad-cli pcb drc --severity-error hw/pcb/board.kicad_pcb
  rtl_sim:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: make -C rtl/sim verilator
  fw_build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: make -C fw/app
      - run: make -C samples/host_cli
```

Dispatch with: `agent-cli send debug-session-manager "Set up the above CI to run DRC / ERC / RTL sim / FW build on every PR"`

### 13.2 Board Integration Scenario

1. Connect USB-C -> PWR LED lights green, heartbeat LED blinks at 1Hz
2. `python cli.py ping` from host -> receive `pong`
3. `cli.py led 0xAA` -> user LED displays `10101010`
4. `cli.py task list` -> 5-6 tasks listed
5. `cli.py i2c scan 0` -> DS3231 address `0x68` found
6. `cli.py rtc get` -> current time retrieved
7. (Optional) Connect Ethernet -> `ping <board-ip>` responds
8. 24-hour continuous operation -> 0 watchdog resets, no hangs

---

## 14. Completion Criteria Checklist

### 14.1 Hardware (Sections 4-8)

- [ ] FPGA confirmed as Efinix Ti75H486C4I7 with > 30% resource headroom
- [ ] HyperRAM x 1+ operates at 1.8V/200MHz
- [ ] 1GbE links up via RGMII PHY
- [ ] USB-PD 9V/3A profile operation confirmed
- [ ] MIPI-CSI 2-lane operation confirmed
- [ ] GPIO expansion header with 32+ pins routed
- [ ] 8-bit user LED individually controllable
- [ ] JTAG header implemented (ADD-1)
- [ ] Bitstream SPI Flash implemented (ADD-2)
- [ ] Power sequencer implemented (ADD-3)
- [ ] Clock source implemented (ADD-4)
- [ ] ESD protection implemented (ADD-5)
- [ ] PCB 6-layer stackup determined (ADD-6)
- [ ] Decoupling strategy reflected (ADD-7)
- [ ] 3x status LEDs implemented (ADD-10)
- [ ] 12+ test points (ADD-11)
- [ ] I2C expansion bus implemented (ADD-12)
- [ ] RTC implemented (ADD-13)
- [ ] microSD implemented (ADD-14)
- [ ] USB-Serial (FT2232H) implemented (ADD-15, ADD-17)
- [ ] Mechanical dimensions 100x80mm + 4 mounting holes (ADD-16)
- [ ] Schematic ERC violations: 0
- [ ] PCB DRC violations: 0

### 14.2 FPGA / RTL (Sections 9-10)

- [ ] Timing slack > 0, LUT utilization < 75%
- [ ] RV32IMC core passes all riscv-tests rv32ui-p-*
- [ ] cocotb test coverage > 90%
- [ ] Watchdog operation confirmed (ADD-9)
- [ ] BootROM -> SPI Flash -> HyperRAM boot sequence successful

### 14.3 Firmware (Sections 11-12)

- [ ] FreeRTOS boots successfully
- [ ] heartbeat / uart_rx / cmd_proc / uart_tx / watchdog_kick tasks running
- [ ] All commands in Section 12.2 respond correctly
- [ ] Host CLI (Python / Rust) working
- [ ] 24-hour continuous operation with no hangs

### 14.4 Deliverables (ADD-18, ADD-19, ADD-20, ADD-21)

- [ ] Schematic PDF (`docs/board_schematic.pdf`)
- [ ] BOM CSV (`hw/bom/board_bom.csv`)
- [ ] Gerber zip (`hw/gerbers/`)
- [ ] Pick-and-place file
- [ ] Mechanical drawing + 3D STEP (`hw/mech/`)
- [ ] Bitstream (`hw/fpga/build/board.hex`)
- [ ] RTOS image (`fw/app/build/firmware.hex`)
- [ ] User manual (`docs/user_manual.md`)
- [ ] CI/CD pipeline operational (DRC / ERC / RTL sim / FW build)
- [ ] USB-PD certification / CE / FCC EMC consideration memo (optional, `docs/compliance.md`)

### 14.5 Self-Check (Quality as Instruction Template)

- [ ] Entire document is in English
- [ ] All persona names (pcb-designer, etc.) correspond to actual files in `.hestia/personas/`
- [ ] All component part numbers are real part numbers
- [ ] All 19 considerations + ADD-1 through ADD-21 from the higher-level instructions are covered in the document
- [ ] Each section ending has verification commands and expected results where applicable

---

## Appendix A: Persona Extension Proposals (Only If Needed)

The current Hestia personas largely cover the functionality needed for this task. The following are **potential** extensions that may be added (implementation is a separate task; this instruction document only proposes them):

- `pcb-mfg`: Gerber manufacturing data, assembly drawings, mass production coordination specialist
- `pcb-compliance`: EMC / RoHS / USB-IF certification specialist
- `rtl-perf-tuner`: RTL critical path auto-optimization specialist
- `apps-bootloader`: BootROM / 2nd-stage loader specialist

These are not included in this instruction document; they can be added to `.hestia/personas/` as needed (no modification of existing personas).

## Appendix B: Package Selection Supplement

Efinix Ti75 package options:

| Package | Balls | Pitch | I/O Count | MIPI Lanes | Recommended Use |
|----------|-------|-------|------------|------------|-----------------|
| `Ti75H361` | 361-ball BGA | 0.65mm | ~250 | ~16 | Mass production (board cost priority) |
| `Ti75H486` | 486-ball BGA | 0.5mm | ~360 | ~24 | Evaluation board (expansion priority) (primary candidate) |

This template is written assuming `Ti75H486C4I7`. If migrating to `Ti75H361` for mass production, PMOD expansion and optional features should be reviewed.

## Appendix C: Reference Links

- Efinix Titanium 75 Datasheet: <https://www.efinixinc.com/products-titanium-overview-Ti75.html>
- Cypress / Infineon HyperRAM: <https://www.infineon.com/cms/en/product/memories/hyperram/>
- USB-PD 3.1 Specification: <https://www.usb.org/document-library/usb-power-delivery>
- FreeRTOS RV32 Port: <https://www.freertos.org/Documentation/03-Libraries/02-FreeRTOS-libraries/00-Overview>
- KiCad 7+: <https://www.kicad.org/>
- Hestia persona list: `.hestia/personas/*.md` (50 items) in this repository

---

## Appendix D: Higher-Level 19 Considerations x ADD-* Mapping Matrix

| Higher-Level Consideration | Section |
|-----------|---------|
| Mention board design | Section 7 / Section 8 |
| FPGA: Efinix TZ75 | Section 4 / Section 5 / Section 9 |
| Memory: HyperRAM | Section 4 / Section 5 / Section 9 / Section 10 |
| Ethernet: 1GbE | Section 4 / Section 5 / Section 9 / Section 11 |
| Power: USB-PD | Section 4 / Section 5 / Section 6 |
| MIPI-CSI: Want to add | Section 4 / Section 5 / Section 9 |
| QSPI: Config Flash (FPGA config dedicated) | Section 3 / Section 4 |
| QSPI: SoC general-purpose bus (external QSPI connection) | Section 3 / Section 4 / Section 10 |
| I2C: Main bus | Section 4 / Section 7 / Section 10 |
| GPIO: Reasonable amount | Section 4 / Section 7 / Section 9 |
| LED: 8-bit | Section 4 / Section 7 / Section 11 |
| Power: Component selection | Section 5 / Section 6 |
| Requirements to component selection | Section 4 -> Section 5 overall |
| Power: Power consumption estimation | Section 6 |
| Schematic | Section 7 |
| Artwork | Section 8 |
| Create FPGA circuit | Section 9 |
| Implement RISC-V in FPGA | Section 10 |
| Real-time OS on RISC-V | Section 11 |
| PC-UART communication | Section 12 |
| ADD-1 through ADD-21 (investigation additions) | Distributed and integrated into Sections 4-14 (see Requirements Section 3.FR-11, Design Section 3.11 mapping table) |

---

> This sample instruction document is complete. Users should copy or modify this content into `.aiprj/instructions.md` and execute the Hestia flow sequentially: `/setup_ai` -> `/update_ai` -> `/ai` (exec_job).