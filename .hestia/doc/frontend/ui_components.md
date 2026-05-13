# UI Component Library

**Target Domain**: frontend — UI design system
**Source**: Design Specification §16.3

## Overview

A standalone UI component library distributed as `hestia-ui` (React + TypeScript). Shared between the VSCode extension and the Tauri desktop app.

## Component List

| Component | Purpose |
|-----------|---------|
| `ConductorStatusCard` | Display status for each conductor (Online / Offline / Degraded / Upgrading) |
| `AgentList` | List, start, and stop sub-agents |
| `SpecViewer` | Structured display and editing of specifications (DesignSpec) |
| `LogViewer` | Real-time streaming display of structured logs |
| `WaveformViewer` | VCD / FST / GHW / EVCD waveform display (WASM rendering) |
| `ConfigPanel` | Form-based editing of configuration files (config.toml, fpga.toml, etc.) |
| `TaskProgress` | Progress display for build / workflow tasks |

## Design System

### Brand Colors

| Color | Code | Purpose |
|-------|------|---------|
| Primary (akane) | `#e84d2c` | Actions and accents |
| Secondary (deep green) | `#2d8f5e` | Success and affirmation |

### Functional Colors

| Color | Purpose |
|-------|---------|
| success | Success and completion |
| warning | Warnings |
| error | Errors and failures |
| info | Informational notices |

## Theme Adaptation

Follows VSCode / Tauri theme variables, automatically adapting to dark / light themes:

- VSCode: `--vscode-editor-background` / `--vscode-list-hoverBackground`, etc.
- Tauri: Follows OS theme via Tauri theme variables

## Distribution

- npm package: `hestia-ui`
- Both VSCode extension and Tauri app import the same components

## Related Documentation

- [vscode_extension.md](vscode_extension.md) — VSCode extension
- [tauri_ide.md](tauri_ide.md) — Tauri desktop app
- [wasm_waveform_viewer.md](../common/wasm_waveform_viewer.md) — WASM waveform viewer