# デーモン起動順序

**対象領域**: common — 起動・運用
**ソース**: 設計仕様書 §15.5

## 概要

HESTIA の 9 conductor は ai-conductor を最優先で起動し、readiness 確認後に残り 8 conductor を並列起動する。この2段階の起動順序により、メタオーケストレータが稼働していることを前提とした残り conductor の初期化が保証される。

## 起動グループ

### Group 0（直列・最高優先度）

ai-conductor を単独起動し、`system.health.v1` が `status="online"` を返すまで待機する。

```bash
# Group 0
hestia-ai-conductor &
# system.health.v1 が status="online" を返すまで待機
hestia status --conductor ai
```

### Group 1（8 並列）

ai-conductor の readiness 確認後、以下 8 conductor を並列起動する。

```bash
# Group 1（8 並列）
hestia-rtl-conductor &
hestia-fpga-conductor &
hestia-asic-conductor &
hestia-pcb-conductor &
hestia-hal-conductor &
hestia-apps-conductor &
hestia-debug-conductor &
hestia-rag-conductor &
```

## systemd ユーザーユニット（推奨）

```bash
# /etc/systemd/user/hestia-{ai,rtl,fpga,asic,pcb,hal,apps,debug,rag}.service
systemctl --user start hestia-ai hestia-rtl hestia-fpga hestia-asic \
  hestia-pcb hestia-hal hestia-apps hestia-debug hestia-rag
```

systemd ユニットを使用することで、以下を自動化:

- 起動依存関係の宣言（ai-conductor → 残り 8 conductor）
- 障害時の自動再起動
- ログ管理（journald 連携）

## readiness チェック詳細

ai-conductor が起動完了したかどうかは以下で判定:

```bash
agent-cli send ai '{"method":"system.health.v1"}'
```

応答例:

```json
{
  "result": {
    "status": "online",
    "uptime_secs": 5,
    "tools_ready": ["task-router", "health-checker", "skill-system"],
    "active_jobs": 0
  }
}
```

3 秒以内に `"online"` 応答がなければ待機を継続。

## 関連ドキュメント

- [health_check_orchestration.md](health_check_orchestration.md) — ヘルスチェック詳細
- [installation.md](installation.md) — ビルド手順
- [sub_agent_lifecycle.md](sub_agent_lifecycle.md) — サブエージェント起動・終了管理