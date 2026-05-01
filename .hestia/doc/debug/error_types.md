# debug-conductor エラーコード

**対象 Conductor**: debug-conductor
**ソース**: 設計仕様書 §14.3（3565-3581行目付近）

## エラーコード範囲

debug-conductor のエラーコードは **-32500 〜 -32599** の範囲を使用する。

## エラーカテゴリ

### JTAG

| コード | 名称 | 説明 |
|-------|------|------|
| -32500 | JTAG_CONNECTION_FAILED | JTAG 接続失敗 |
| -32501 | JTAG_TAP_NOT_DETECTED | TAP デバイス未検出 |
| -32502 | JTAG_IR_DR_ERROR | IR/DR スキャンチェーンエラー |
| -32503 | JTAG_RESET_FAILED | JTAG リセット（TRST/SRST）失敗 |

### SWD

| コード | 名称 | 説明 |
|-------|------|------|
| -32510 | SWD_CONNECTION_FAILED | SWD 接続失敗 |
| -32511 | SWD_DP_READ_FAILED | Debug Port レジスタ読み出し失敗 |
| -32512 | SWD_DP_WRITE_FAILED | Debug Port レジスタ書き込み失敗 |
| -32513 | SWD_AP_READ_FAILED | Access Port レジスタ読み出し失敗 |
| -32514 | SWD_AP_WRITE_FAILED | Access Port レジスタ書き込み失敗 |
| -32515 | SWD_PARITY_ERROR | SWD パリティエラー |

### Session

| コード | 名称 | 説明 |
|-------|------|------|
| -32520 | SESSION_CREATE_FAILED | セッション作成失敗 |
| -32521 | SESSION_NOT_FOUND | 指定されたセッションが存在しない |
| -32522 | SESSION_ALREADY_CONNECTED | 既に接続済み |
| -32523 | SESSION_DISCONNECTED | 予期せぬ切断 |

### Waveform

| コード | 名称 | 説明 |
|-------|------|------|
| -32530 | CAPTURE_START_FAILED | 波形キャプチャ開始失敗 |
| -32531 | CAPTURE_STOP_FAILED | 波形キャプチャ停止失敗 |
| -32532 | CAPTURE_BUFFER_OVERFLOW | キャプチャバッファオーバーフロー |
| -32533 | VCD_PARSE_ERROR | VCD ファイルパースエラー |
| -32534 | FST_PARSE_ERROR | FST ファイルパースエラー |

### Programming

| コード | 名称 | 説明 |
|-------|------|------|
| -32540 | PROGRAM_FAILED | ファームウェア書込失敗 |
| -32541 | PROGRAM_VERIFY_FAILED | 書込検証失敗 |
| -32542 | PROGRAM_UNSUPPORTED_FORMAT | 未対応のファームウェアフォーマット |

### Signal / Trigger

| コード | 名称 | 説明 |
|-------|------|------|
| -32550 | SIGNAL_NOT_FOUND | 指定された信号が存在しない |
| -32551 | TRIGGER_CONDITION_INVALID | トリガ条件が不正 |
| -32552 | TRIGGER_TIMEOUT | トリガ待機タイムアウト |

### Reset

| コード | 名称 | 説明 |
|-------|------|------|
| -32555 | RESET_FAILED | リセット失敗 |
| -32556 | RESET_TIMEOUT | リセット応答タイムアウト |

### Protocol

| コード | 名称 | 説明 |
|-------|------|------|
| -32560 | PROTOCOL_DECODE_ERROR | プロトコルデコードエラー |
| -32561 | PROTOCOL_UNSUPPORTED | 未対応のプロトコル |

## 関連ドキュメント

- [debug/message_methods.md](message_methods.md) — debug.* メソッド一覧
- [debug/debug_protocols.md](debug_protocols.md) — JTAG/SWD プロトコル
- [debug/state_machines.md](state_machines.md) — セッション管理ステートマシン
- [../common/error_registry.md](../common/error_registry.md) — HESTIA 共通エラーレジストリ