# UpgradeManager（持続可能アップグレード）

**対象領域**: common — バージョン管理
**ソース**: 設計仕様書 §3.4, §1.3.7

## 概要

ベンダーツールのバージョンアップに対して AI エージェントが自動的にパッチを生成・検証・適用し、人間の介入を最小化する持続可能アップグレード機構。セマンティックバージョニングに基づく段階的ロールアウトと自動ロールバックを提供する。

## 4 エージェント構成

各エージェントは独立した agent-cli プロセスとして動作する:

| エージェント | Peer 名 | 役割 |
|------------|---------|------|
| WatcherAgent | `ai-watcher` | 6 時間ごとにベンダーサイトを監視し、新バージョンを検出 |
| ProbeAgent | `ai-probe` | 標準テストプロジェクト群で新バージョンのテストビルドを実行、非互換を検出 |
| PatcherAgent | `ai-patcher` | agent-cli の Tool Use 機能を活用し、パッチを自動生成 |
| ValidatorAgent | `ai-validator` | サンドボックス環境でパッチを検証、信頼度スコアを算出 |

### PatcherAgent の Tool Use（6 種）

| ツール | 機能 |
|-------|------|
| `read_adapter_manifest()` | adapter.toml の内容を取得 |
| `read_error_log()` | ビルドエラーの詳細を取得 |
| `search_breaking_changes()` | 既知の非互換変更を検索 |
| `read_vendor_changelog()` | リリースノートを取得 |
| `propose_patch()` | パッチ案を提出 |
| `trigger_validation()` | 検証を実行 |

## HumanReviewGate

信頼度スコアに基づき自動適用 or 手動レビューを判定:

```
PatcherAgent → ValidatorAgent → 信頼度スコア算出
                                    │
                                    ├── 信頼度 高 → 自動適用
                                    └── 信頼度 低 → HumanReviewGate（人間レビュー）
```

## 互換性判定

| バージョン変更 | 互換性 | 要求戦略 |
|--------------|--------|---------|
| `1.0.0` → `1.1.0` | 互換 | Production 可 |
| `1.0.0` → `1.0.1` | 互換 | Production 可 |
| `1.0.0` → `2.0.0` | 非互換 | Canary または Staging 必須 |

## 段階的ロールアウト

| 戦略 | 説明 | 使用場面 |
|------|------|---------|
| `Canary` | 少数環境に先行展開 | メジャーバージョン変更時 |
| `Staging` | ステージング環境で検証後に本番展開 | マイナーバージョン更新 |
| `Production` | 本番環境に直接展開 | パッチリリース |

## 自動ロールバック

```rust
pub struct RollbackConfig {
    pub auto_rollback: bool,     // 自動ロールバック有効化
    pub timeout_secs: u64,       // タイムアウト（既定: 300 秒）
    pub max_retries: u32,        // 最大リトライ回数（既定: 3 回）
}
```

ロールバックは以下の条件で発火:
- ヘルスチェック連続失敗（§3.3.2）
- テストスイートの回帰
- Observability メトリクスの閾値超過

## フロー全体

```
検出（WatcherAgent）→ テスト（ProbeAgent）→ パッチ生成（PatcherAgent）
  → 検証（ValidatorAgent）→ 判定（HumanReviewGate）
  → 段階的適用（Canary → Staging → Production）
  → 異常時ロールバック
```

## クレート構成

```
upgrade-manager/
└── src/
    ├── lib.rs
    ├── version_policy.rs   # セマンティックバージョニング
    ├── rollout.rs          # 段階的ロールアウト
    └── rollback.rs         # 自動ロールバック
```

## 関連ドキュメント

- [health_check_orchestration.md](health_check_orchestration.md) — ヘルスチェック
- [backend_switching.md](backend_switching.md) — LLM バックエンド切替
- [sub_agent_lifecycle.md](sub_agent_lifecycle.md) — サブエージェント管理