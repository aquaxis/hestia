# Error Code Complete Listing

**Domain**: common — Error Codes
**Source**: Design Specification §14.3

## Overview

Defines the complete range of error codes for HESTIA's structured messages. It reuses the numbering scheme from legacy JSON-RPC 2.0 while establishing HESTIA-specific extension ranges.

## Error Code Range Listing

| Range | Domain | Details |
|------|------|------|
| `-32700` | Parse Error | JSON payload parse failure |
| `-32600` ~ `-32603` | Standard request errors | Reused from legacy JSON-RPC 2.0 |
| `-32000` ~ `-32099` | HESTIA common | Timeout / NotFound / AlreadyExists, etc. |
| `-32100` ~ `-32199` | ai-conductor | Orchestration / agent management / specification / version tracking / LLM |
| `-32200` ~ `-32299` | fpga-conductor | Synthesis / place-and-route / bitstream / timing / debug / HLS / device / simulation / constraints / adapter |
| `-32300` ~ `-32399` | asic-conductor | RTL synthesis / floorplan / placement / CTS / routing / other |
| `-32400` ~ `-32499` | pcb-conductor | Schematic / DRC/ERC / BOM/placement / Gerber / AI synthesis / KG / constraint verification |
| `-32500` ~ `-32599` | debug-conductor | JTAG / SWD / session / waveform / programming / signals / trigger / reset / protocol |
| `-32600` ~ `-32699` | rag-conductor | Ingest / PDF / Web / quality gate / chunk-embedding / vector-search / license-PII / scheduler / cache |

## Standard Errors (-32600 ~ -32603)

| Code | Name | Meaning |
|-------|------|------|
| `-32600` | Invalid Request | Invalid request format |
| `-32601` | Method not found | Undefined method |
| `-32602` | Invalid params | Invalid parameters |
| `-32603` | Internal error | Internal error |

## HESTIA Common Errors (-32000 ~ -32099)

| Code | Name | Meaning |
|-------|------|------|
| `-32000` | Internal | Internal error (generic)|
| `-32001` | Timeout | Timeout |
| `-32002` | NotFound | Resource not found |
| `-32003` | AlreadyExists | Resource duplicate |
| `-32004` | PermissionDenied | Insufficient permissions |
| `-32005` | InvalidState | Invalid state |
| `-32006` | ServiceUnavailable | Service unavailable |

## ai-conductor Errors (-32100 ~ -32199)

| Code Range | Sub-domain |
|----------|---------|
| `-32100` ~ `-32119` | Orchestration |
| `-32120` ~ `-32139` | Agent management |
| `-32140` ~ `-32159` | Specification-driven |
| `-32160` ~ `-32179` | Version tracking |
| `-32180` ~ `-32199` | LLM |

## fpga-conductor Errors (-32200 ~ -32299)

| Code Range | Sub-domain |
|----------|---------|
| `-32200` ~ `-32209` | Synthesis |
| `-32210` ~ `-32219` | Place and route |
| `-32220` ~ `-32229` | Bitstream |
| `-32230` ~ `-32239` | Timing |
| `-32240` ~ `-32249` | Debug |
| `-32250` ~ `-32259` | HLS |
| `-32260` ~ `-32269` | Device |
| `-32270` ~ `-32279` | Simulation |
| `-32280` ~ `-32289` | Constraints |
| `-32290` ~ `-32299` | Adapter |

## Error Response Format

```json
{
  "error": {
    "code": -32200,
    "message": "Synthesis failed",
    "data": {
      "tool": "vivado",
      "exit_code": 1,
      "log_path": ".hestia/logs/vivado_synth.log",
      "errors": ["[Synth 8-439] ..."],
      "retry_possible": true,
      "suggested_action": "Check HDL syntax and constraint files"
    }
  },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}
```

## Related Documents

- [error_handling_strategy.md](error_handling_strategy.md) — Error handling strategy
- [agent_message.md](agent_message.md) — Message payload format
- [api_versioning.md](api_versioning.md) — Method namespace