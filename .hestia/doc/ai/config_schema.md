# ai-conductor 設定スキーマ

**対象 Conductor**: ai-conductor
**ソース**: 設計仕様書 §3.8（1109-1164行目付近）, §3.9（1189-...行目付近）

## container.toml

各 conductor が使用するコンテナ環境を宣言的に定義するファイル。コンテナ実行を選択した場合のみ使用（ローカル実行時は不要）。

### セクション一覧

| セクション | 必須 | 説明 |
|-----------|------|------|
| `[container]` | 必須 | コンテナ基本設定（名前、ベースイメージ、対象 conductor） |
| `[tools.*]` | 任意 | インストールするツール定義 |
| `[env]` | 任意 | 環境変数 |
| `[[volumes]]` | 任意 | ボリュームマウント定義 |
| `[health]` | 任意 | ヘルスチェック設定 |
| `[update]` | 任意 | アップデートポリシー |

### `[container]` セクション

| フィールド | 型 | 説明 |
|-----------|---|------|
| `name` | string | コンテナ名 |
| `base_image` | string | ベースイメージ（例: `ubuntu:24.04`） |
| `conductor` | string | 対象 conductor 名 |

### `[tools.*]` セクション

| フィールド | 型 | 説明 |
|-----------|---|------|
| `name` | string | ツール表示名 |
| `version` | string | バージョン制約（semver 例: `>=2025.1`） |
| `install_script` | string | インストールスクリプト |
| `version_cmd` | string | バージョン確認コマンド |

### `[health]` セクション

| フィールド | 型 | 既定値 | 説明 |
|-----------|---|-------|------|
| `cmd` | string | — | ヘルスチェックコマンド |
| `interval_secs` | integer | 60 | ポーリング間隔（秒） |
| `timeout_secs` | integer | 3 | 応答タイムアウト（秒） |
| `max_retries` | integer | 3 | 連続リトライ回数 |
| `escalate_on_fail` | boolean | true | 連続失敗時にフロントエンドへ通知 |
| `restart_on_fail` | boolean | true | 自動再起動試行 |

### `[update]` セクション

| フィールド | 型 | 説明 |
|-----------|---|------|
| `auto` | boolean | 自動アップデート有効化 |
| `schedule` | string | cron スケジュール |
| `rollback_on_failure` | boolean | 失敗時自動ロールバック |

### 設定例

```toml
[container]
name = "vivado-build"
base_image = "ubuntu:24.04"
conductor = "fpga"

[tools.vivado]
name = "AMD Vivado"
version = ">=2025.1"
install_script = "apt-get update && apt-get install -y wget && ..."
version_cmd = "vivado -version"

[tools.yosys]
name = "Yosys"
version = ">=0.40"
install_script = "apt-get install -y yosys"
version_cmd = "yosys --version"

[env]
XILINX_ROOT = "/opt/Xilinx"
PATH = "/opt/Xilinx/Vivado/2025.2/bin:$PATH"

[[volumes]]
host = "/workspace"
container = "/workspace"
options = "Z"

[[volumes]]
host = "/opt/Xilinx/license"
container = "/opt/Xilinx/license"
options = "ro"

[health]
cmd = "vivado -version || true"
interval_secs = 60

[update]
auto = true
schedule = "0 3 * * 0"
rollback_on_failure = true
```

## upgrade.toml

持続可能アップグレード（§3.4 UpgradeManager）の設定ファイル。

### セクション一覧

| セクション | 説明 |
|-----------|------|
| `[upgrade]` | アップグレード基本設定 |
| `[strategy.major]` | メジャーバージョン戦略 |
| `[strategy.minor]` | マイナーバージョン戦略 |
| `[strategy.patch]` | パッチバージョン戦略 |
| `[rollback]` | ロールバック設定 |

### `[upgrade]` セクション

| フィールド | 型 | 説明 |
|-----------|---|------|
| `check_interval_hours` | integer | 新バージョン確認間隔（時間） |
| `auto_upgrade` | boolean | 自動アップグレード有効化 |
| `notification_email` | string | 通知先メールアドレス |

### `[strategy.*]` セクション

| 戦略タイプ | 説明 | 使用場面 |
|-----------|------|---------|
| `canary` | 少数環境に先行展開 | メジャーバージョン変更時 |
| `staging` | ステージング環境で検証後に本番展開 | マイナーバージョン更新 |
| `production` | 本番環境に直接展開 | パッチリリース |

### `[rollback]` セクション

| フィールド | 型 | 既定値 | 説明 |
|-----------|---|-------|------|
| `auto` | boolean | true | 自動ロールバック有効化 |
| `timeout_secs` | integer | 300 | タイムアウト（秒） |
| `max_retries` | integer | 3 | 最大リトライ回数 |

### 設定例

```toml
[upgrade]
check_interval_hours = 6
auto_upgrade = true
notification_email = "team@example.com"

[strategy.major]
type = "canary"
canary_percentage = 10

[strategy.minor]
type = "staging"

[strategy.patch]
type = "production"

[rollback]
auto = true
timeout_secs = 300
max_retries = 3
```

## 関連ドキュメント

- [ai/binary_spec.md](binary_spec.md) — hestia-ai-cli バイナリ仕様
- [ai/error_types.md](error_types.md) — ai-conductor エラーコード
- [ai/state_machines.md](state_machines.md) — タスク状態遷移
- [../common/container_manager.md](../container/container_manager.md) — コンテナ管理詳細