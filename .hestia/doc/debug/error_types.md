# debug-conductor Error Codes

**Target Conductor**: debug-conductor
**Source**: Design Specification §14.3 (around lines 3565-3581)

## Error Code Range

debug-conductor error codes use the range **-32500 to -32599**.

## Error Categories

### JTAG

| Code | Name | Description |
|-------|------|------|
| -32500 | JTAG_CONNECTION_FAILED | JTAG connection failed |
| -32501 | JTAG_TAP_NOT_DETECTED | TAP device not detected |
| -32502 | JTAG_IR_DR_ERROR | IR/DR scan chain error |
| -32503 | JTAG_RESET_FAILED | JTAG reset (TRST/SRST) failed |

### SWD

| Code | Name | Description |
|-------|------|------|
| -32510 | SWD_CONNECTION_FAILED | SWD connection failed |
| -32511 | SWD_DP_READ_FAILED | Debug Port register read failed |
| -32512 | SWD_DP_WRITE_FAILED | Debug Port register write failed |
| -32513 | SWD_AP_READ_FAILED | Access Port register read failed |
| -32514 | SWD_AP_WRITE_FAILED | Access Port register write failed |
| -32515 | SWD_PARITY_ERROR | SWD parity error |

### Session

| Code | Name | Description |
|-------|------|------|
| -32520 | SESSION_CREATE_FAILED | Session creation failed |
| -32521 | SESSION_NOT_FOUND | Specified session does not exist |
| -32522 | SESSION_ALREADY_CONNECTED | Already connected |
| -32523 | SESSION_DISCONNECTED | Unexpected disconnection |

### Waveform

| Code | Name | Description |
|-------|------|------|
| -32530 | CAPTURE_START_FAILED | Waveform capture start failed |
| -32531 | CAPTURE_STOP_FAILED | Waveform capture stop failed |
| -32532 | CAPTURE_BUFFER_OVERFLOW | Capture buffer overflow |
| -32533 | VCD_PARSE_ERROR | VCD file parse error |
| -32534 | FST_PARSE_ERROR | FST file parse error |

### Programming

| Code | Name | Description |
|-------|------|------|
| -32540 | PROGRAM_FAILED | Firmware programming failed |
| -32541 | PROGRAM_VERIFY_FAILED | Programming verification failed |
| -32542 | PROGRAM_UNSUPPORTED_FORMAT | Unsupported firmware format |

### Signal / Trigger

| Code | Name | Description |
|-------|------|------|
| -32550 | SIGNAL_NOT_FOUND | Specified signal does not exist |
| -32551 | TRIGGER_CONDITION_INVALID | Invalid trigger condition |
| -32552 | TRIGGER_TIMEOUT | Trigger wait timeout |

### Reset

| Code | Name | Description |
|-------|------|------|
| -32555 | RESET_FAILED | Reset failed |
| -32556 | RESET_TIMEOUT | Reset response timeout |

### Protocol

| Code | Name | Description |
|-------|------|------|
| -32560 | PROTOCOL_DECODE_ERROR | Protocol decode error |
| -32561 | PROTOCOL_UNSUPPORTED | Unsupported protocol |

## Related Documentation

- [debug/message_methods.md](message_methods.md) — debug.* method list
- [debug/debug_protocols.md](debug_protocols.md) — JTAG/SWD protocols
- [debug/state_machines.md](state_machines.md) — Session management state machine
- [../common/error_registry.md](../common/error_registry.md) — HESTIA common error registry