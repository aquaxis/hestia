# ヘルスチェックオーケストレーション

**対象領域**: common — 運用監視
**ソース**: 設計仕様書 §3.3.2

## 概要

ai-conductor の `health-checker` が全 conductor の生存・正常性を定期ポーリングし、ConductorStatus を集約管理する。異常検出時は自動再起動を試み、回復不能な場合は人間（フロントエンド通知）にエスカレーションする。

## ポーリング仕様

| パラメータ | 既定値 | 設定場所 |
|----------|-------|---------|
| ポーリング間隔 | 30 秒 | `[health] interval_secs` |
| 応答タイムアウト | 3 秒 | `[health] timeout_secs` |
| 自動再起動上限 | 3 回 | `[health] max_retries` |

## ConductorStatus

| 状態 | 意味 | 遷移条件 |
|------|------|---------|
| `Online` | 正常稼働中 | 3 秒以内に `"online"` 応答 |
| `Offline` | 停止中 | タイムアウト（3 秒）|
| `Degraded` | 劣化状態（一部機能制限） | `"degraded"` 応答 |
| `Upgrading` | アップグレード中 | `"upgrading"` 応答 |

## ヘルスチェックフロー

```
ai-conductor::health-checker (tokio interval task)
    │
    │ 30 秒間隔
    │
    ▼
For each peer in [rtl, fpga, asic, pcb, hal, apps, debug, rag]:
    │
    │ agent-cli send <peer> '{"method":"system.health.v1","id":"hc_<ts>"}'
    │
    ▼
応答パターン → ConductorStatus 更新:
    - 3 秒以内に "online" 応答   → Online
    - タイムアウト (3 秒)         → Offline
    - "degraded" 応答             → Degraded
    - "upgrading" 応答            → Upgrading
    │
    ▼
状態変化時アクション:
    - Online → Offline / Degraded → Observability log + 自動再起動 (max 3)
    - 連続 3 回失敗              → フロントエンド通知
    - Upgrading → Online         → upgrade-manager に成功通知
    - 任意                       → 状態履歴を sled に永続化
```

## 応答仕様（system.health.v1）

```json
// Request
{
  "method": "system.health.v1",
  "id": "hc_2026-05-01T12:00:00Z_abc123",
  "trace_id": "health_loop_20260501T120000"
}

// Response
{
  "result": {
    "status": "online",
    "uptime_secs": 12345,
    "tools_ready": ["vivado", "yosys"],
    "load": { "cpu_pct": 12, "mem_mb": 512 },
    "active_jobs": 3,
    "last_error": null
  },
  "id": "hc_2026-05-01T12:00:00Z_abc123",
  "trace_id": "health_loop_20260501T120000"
}
```

## 設定例

```toml
[health]
cmd = "vivado -version || true"   # ローカル実行モードでの簡易確認
interval_secs = 30                # ポーリング間隔
timeout_secs = 3                  # 応答タイムアウト
max_retries = 3                   # 連続リトライ回数
escalate_on_fail = true           # 連続失敗時にフロントエンドへ通知
restart_on_fail = true            # 自動再起動試行
```

## 関連ドキュメント

- [conductor_startup.md](conductor_startup.md) — デーモン起動順序
- [update_mechanism.md](update_mechanism.md) — UpgradeManager
- [observability.md](observability.md) — 監視・メトリクス