# Tauri Desktop App

**Target Domain**: frontend — desktop IDE
**Source**: Design Specification §16.2

## Overview

A Tauri (Rust + React) based desktop app. Provides equivalent functionality to the VSCode extension in a native desktop environment.

## Basic Settings

| Item | Value |
|------|-------|
| Version | 0.1.0 |
| Identifier | `dev.hestia.ide` |
| Configuration file | `tauri.conf.json` |

## Window Configuration

| Window | Size | Purpose |
|--------|------|---------|
| main | 1440 x 900 | Main editor and conductor management panel |
| waveform | 1200 x 600 | Waveform viewer (WASM / native) |
| settings | 800 x 600 | Settings screen |

## Security

### Content Security Policy (CSP)

```
connect-src 'self' ipc: ws://localhost:*
```

- `connect-src 'self'`: Allow communication from the same origin
- `ipc:`: Tauri IPC channel
- `ws://localhost:*`: HMR (Hot Module Replacement) during development

## Bundle Targets

| Target | Format |
|--------|--------|
| Debian / Ubuntu | `.deb` |
| RHEL / Fedora | `.rpm` |
| Linux (generic) | `.AppImage` |

## Shell Plugin

The following 10 commands can be invoked via the Tauri Shell plugin:

| Command | Purpose |
|---------|---------|
| `hestia` | Unified runner |
| `hestia-ai-cli` | ai-conductor CLI |
| `hestia-rtl-cli` | rtl-conductor CLI |
| `hestia-fpga-cli` | fpga-conductor CLI |
| `hestia-asic-cli` | asic-conductor CLI |
| `hestia-pcb-cli` | pcb-conductor CLI |
| `hestia-hal-cli` | hal-conductor CLI |
| `hestia-apps-cli` | apps-conductor CLI |
| `hestia-debug-cli` | debug-conductor CLI |
| `hestia-rag-cli` | rag-conductor CLI |

## UI Components

Reuses components from `hestia-ui` (§16.3), with display unified by adhering to Tauri-specific theme variables.

## Related Documentation

- [vscode_extension.md](vscode_extension.md) — VSCode extension
- [ui_components.md](ui_components.md) — UI component library
- [agent_cli_client.md](agent_cli_client.md) — agent-cli client specification