# debug-conductor メッセージメソッド一覧

**対象 Conductor**: debug-conductor
**ソース**: 設計仕様書 §10.4（2477-2493行目付近）, §14（3492-3630行目付近）

## トランスポート

全通信は agent-cli ネイティブ IPC に統一。peer 名 `debug`。

## debug.* メソッド一覧

### セッション管理

| メソッド | 方向 | 説明 |
|---------|------|------|
| `debug.connect` | Request | ターゲットデバイスへ接続（JTAG / SWD） |
| `debug.disconnect` | Request | ターゲットデバイスから切断 |
| `debug.reset` | Request | ターゲットリセット（Hardware / Software / System） |
| `debug.status` | Request | セッション状態取得 |

### ブレークポイント

| メソッド | 方向 | 説明 |
|---------|------|------|
| `debug.setBreakpoint` | Request | ブレークポイント設定（Source / Address / Symbol 3方式） |
| `debug.removeBreakpoint` | Request | ブレークポイント削除 |

### 実行制御

| メソッド | 方向 | 説明 |
|---------|------|------|
| `debug.run` | Request | ターゲットプログラム実行 |
| `debug.pause` | Request | 一時停止 |
| `debug.stepOver` | Request | ステップオーバー実行 |
| `debug.stepInto` | Request | ステップイン実行 |

### メモリアクセス

| メソッド | 方向 | 説明 |
|---------|------|------|
| `debug.readMemory` | Request | メモリ読み出し |
| `debug.writeMemory` | Request | メモリ書き込み |

### キャプチャ

| メソッド | 方向 | 説明 |
|---------|------|------|
| `debug.startCapture` | Request | 波形キャプチャ開始 |
| `debug.stopCapture` | Request | 波形キャプチャ停止 |
| `debug.read_signals` | Request | 信号読み取り |
| `debug.set_trigger` | Request | トリガ条件設定 |

### プログラミング

| メソッド | 方向 | 説明 |
|---------|------|------|
| `debug.program` | Request | ファームウェア書込（SVF / JAM / probe-rs / OpenOCD） |

### 通知

| メソッド | 方向 | 説明 |
|---------|------|------|
| `debug.sessionStateChanged` | Notification | セッション状態変化通知 |
| `debug.breakpointHit` | Notification | ブレークポイント到達通知 |
| `debug.captureComplete` | Notification | キャプチャ完了通知 |

## conductor-core 共通

| メソッド | 方向 | 説明 |
|---------|------|------|
| `system.health.v1` | Request | ヘルスチェック |
| `system.readiness` | Request | 準備状態確認 |

## 関連ドキュメント

- [debug/binary_spec.md](binary_spec.md) — hestia-debug-cli バイナリ仕様
- [debug/error_types.md](error_types.md) — debug-conductor エラーコード
- [debug/debug_protocols.md](debug_protocols.md) — JTAG/SWD プロトコル
- [debug/state_machines.md](state_machines.md) — セッション管理ステートマシン