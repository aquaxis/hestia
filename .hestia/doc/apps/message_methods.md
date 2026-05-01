# apps-conductor メッセージメソッド一覧

**対象 Conductor**: apps-conductor
**ソース**: 設計仕様書 §9.7（2379-2381行目付近）, §14（3492-3630行目付近）

## トランスポート

全通信は agent-cli ネイティブ IPC に統一。peer 名 `apps`。

## apps.* メソッド一覧

### ビルドフロー

| メソッド | 方向 | 説明 |
|---------|------|------|
| `apps.build.v1` | Request | クロスコンパイル・ビルド実行 |
| `apps.flash.v1` | Request | ターゲットデバイスへのファームウェア書込 |

### テスト

| メソッド | 方向 | 説明 |
|---------|------|------|
| `apps.test.v1` | Request | テスト実行（SIL / HIL / QEMU モード選択） |

### 解析

| メソッド | 方向 | 説明 |
|---------|------|------|
| `apps.size.v1` | Request | バイナリサイズレポート取得（セクション別サイズ） |

### デバッグ

| メソッド | 方向 | 説明 |
|---------|------|------|
| `apps.debug.v1` | Request | debug-conductor（§10）との bridging（GDB セッション開始） |

### conductor-core 共通

| メソッド | 方向 | 説明 |
|---------|------|------|
| `system.health.v1` | Request | ヘルスチェック |
| `system.readiness` | Request | 準備状態確認 |

## ペイロード例

### apps.build.v1 リクエスト

```json
{
  "method": "apps.build.v1",
  "params": {
    "project": "sensor_node_fw",
    "target": "thumbv7em-none-eabihf",
    "compiler": "arm-none-eabi-gcc"
  },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}
```

### apps.test.v1 リクエスト

```json
{
  "method": "apps.test.v1",
  "params": {
    "mode": "hil",
    "probe": "stlink-v3"
  },
  "id": "msg_2026-05-01T12:00:00Z_def456"
}
```

## 上流・下流連携

- **上流（hal-conductor §8）**: hal-conductor が生成した HAL モジュール（C ヘッダ / Rust crate / Python モジュール）を `[hal] import = "..."` で取り込む
- **横断（debug-conductor §10）**: フラッシュ書込 / RTT ログ / GDB セッションを debug-conductor 経由で実施
- **テスト**: SIL（QEMU）/ HIL（実機 + debug-conductor）/ クロステスト（QEMU + cycle-accurate co-sim）の 3 モードをサポート
- **CI/CD**: 共有サービス層 / CI/CD API（§13）経由で GitHub Actions / GitLab CI 上で再現性ビルド
- **ai-conductor 連携**: §3 ai-conductor がアプリケーション仕様書から apps.toml の自動生成を支援（Spec-Driven）

## 関連ドキュメント

- [apps/binary_spec.md](binary_spec.md) — hestia-apps-cli バイナリ仕様
- [apps/error_types.md](error_types.md) — apps 固有エラー型
- [apps/toolchain.md](toolchain.md) — 主要アダプター
- [apps/rtos.md](rtos.md) — RTOS サポート
- [../common/agent_cli_messaging.md](../common/agent_cli_messaging.md) — agent-cli メッセージング仕様