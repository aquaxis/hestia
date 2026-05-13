# debug-conductor Message Method List

**Target Conductor**: debug-conductor
**Source**: Design Specification §10.4 (around lines 2477-2493), §14 (around lines 3492-3630)

## Transport

All communication is unified via agent-cli native IPC. Peer name: `debug`.

## debug.* Method List

### Session Management

| Method | Direction | Description |
|---------|------|------|
| `debug.connect` | Request | Connect to target device (JTAG / SWD) |
| `debug.disconnect` | Request | Disconnect from target device |
| `debug.reset` | Request | Target reset (Hardware / Software / System) |
| `debug.status` | Request | Get session status |

### Breakpoints

| Method | Direction | Description |
|---------|------|------|
| `debug.setBreakpoint` | Request | Set breakpoint (Source / Address / Symbol 3 modes) |
| `debug.removeBreakpoint` | Request | Remove breakpoint |

### Execution Control

| Method | Direction | Description |
|---------|------|------|
| `debug.run` | Request | Execute target program |
| `debug.pause` | Request | Pause |
| `debug.stepOver` | Request | Step over execution |
| `debug.stepInto` | Request | Step into execution |

### Memory Access

| Method | Direction | Description |
|---------|------|------|
| `debug.readMemory` | Request | Memory read |
| `debug.writeMemory` | Request | Memory write |

### Capture

| Method | Direction | Description |
|---------|------|------|
| `debug.startCapture` | Request | Start waveform capture |
| `debug.stopCapture` | Request | Stop waveform capture |
| `debug.read_signals` | Request | Read signals |
| `debug.set_trigger` | Request | Set trigger condition |

### Programming

| Method | Direction | Description |
|---------|------|------|
| `debug.program` | Request | Firmware programming (SVF / JAM / probe-rs / OpenOCD) |

### Notifications

| Method | Direction | Description |
|---------|------|------|
| `debug.sessionStateChanged` | Notification | Session state change notification |
| `debug.breakpointHit` | Notification | Breakpoint hit notification |
| `debug.captureComplete` | Notification | Capture complete notification |

## conductor-core Common

| Method | Direction | Description |
|---------|------|------|
| `system.health.v1` | Request | Health check |
| `system.readiness` | Request | Readiness check |

## Related Documentation

- [debug/binary_spec.md](binary_spec.md) — hestia-debug-cli binary specification
- [debug/error_types.md](error_types.md) — debug-conductor error codes
- [debug/debug_protocols.md](debug_protocols.md) — JTAG/SWD protocols
- [debug/state_machines.md](state_machines.md) — Session management state machine