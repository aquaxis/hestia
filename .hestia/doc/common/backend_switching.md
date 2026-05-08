# LLM バックエンド切替 / Engine 切替

**対象領域**: common — peer 駆動エンジン
**ソース**: 設計仕様書 §20 / Phase 113

## 概要

HESTIA の peer 駆動には 2 段階の切替が存在する:

1. **Engine 切替（Phase 113）** — peer を起動するバイナリそのものを `agent-cli` か `claude-cli-shim` から選択。`.hestia/config.toml` の `[engine]` セクションで指定。
2. **LLM バックエンド切替** — engine が `agent-cli` の場合、agent-cli のバックエンド LLM を 4 種類から選択（`[agent_cli]` セクション）。`claude-cli-shim` engine の場合は Claude Code (Anthropic API) 単一固定。

## Engine（Phase 113）

| Engine | `[engine] type` 値 | バイナリ | 用途 |
|--------|--------------------|----------|------|
| agent-cli（既定） | `"agent_cli"` または未設定 | `agent-cli` | 4 種 LLM backend 対応、従来挙動 |
| claude-cli-shim | `"claude_cli_shim"` | `claude-cli-shim` | Claude Code (`claude` CLI) を子プロセスで保持する wrapper、案 C |

```toml
[engine]
# "agent_cli" (既定、後方互換) | "claude_cli_shim"
type = "agent_cli"
binary = ""           # 省略時は type に応じた既定 path
registry_path = ""    # 省略時は engine 既定 (~/.local/share/<engine>/registry)
log_path = ""         # 省略時は engine 既定 (~/.local/share/<engine>/logs)
```

`[engine]` 未設定時は `type = "agent_cli"` 既定で従来挙動と完全互換。

`type = "claude_cli_shim"` を選ぶと、hestia は `claude-cli-shim run` を spawn し、shim が内部で `claude --input-format stream-json --output-format stream-json --print` を子プロセスとして保持する。registry / log は agent-cli 互換 schema で別ディレクトリに記録される。

## 対応バックエンド（agent-cli engine 配下のみ）

| バックエンド | `backend` 値 | 特徴 |
|------------|-------------|------|
| Anthropic Claude | `"claude"` | 既定。高精度 Tool Use |
| OpenAI Codex | `"codex"` | OpenAI API 互換 |
| Ollama | `"ollama"` | ローカル実行、オフライン対応 |
| llama.cpp | `"llama_cpp"` | OpenAI 互換エンドポイント |

**注**: `type = "claude_cli_shim"` の場合、本表は無関係（Claude Code 経由で Anthropic API に直結）。

## `[agent_cli]` スキーマ

```toml
[agent_cli]
backend = "claude"                            # "claude" | "codex" | "ollama" | "llama_cpp"
binary_path = ""                              # 空 = $PATH 解決 / フルパス指定可
anthropic_base_url = ""                       # 空 = Anthropic 公式 / OpenAI 互換 API の URL
anthropic_api_key_env = "ANTHROPIC_API_KEY"   # API キーを格納するホスト環境変数名
model = "claude-opus-4-7"                     # LLM モデル識別子
max_tokens = 4096                             # 既定の応答上限トークン数
registry_dir = ""                             # agent-cli IPC レジストリ（空 = $XDG_RUNTIME_DIR/agent-cli）
```

## Rust 型

```rust
pub struct AgentCliSection {
    pub backend: String,            // default: "claude"
    pub binary_path: String,        // default: ""
    pub anthropic_base_url: String, // default: ""
    pub anthropic_api_key_env: String, // default: "ANTHROPIC_API_KEY"
    pub model: String,             // default: "claude-opus-4-7"
    pub max_tokens: u32,          // default: 4096
    pub registry_dir: String,     // default: ""
}
```

## 環境変数フォワーディング（FR-CFG-07）

1. `config.toml` を読む（`HestiaConfig::from_toml_file`）
2. `anthropic_api_key_env` で指定された環境変数をホストから取得（未設定 / 空 → fail-fast）
3. `anthropic_base_url` が空でなければ子プロセスに `ANTHROPIC_BASE_URL` を inject
4. API キーを子プロセスに `ANTHROPIC_API_KEY` として inject
5. `tokio::process::Command::spawn` で agent-cli 子プロセス起動

ヘルパー: `AgentCliSection::build_env() -> Result<Vec<(String, String)>, AgentCliEnvError>`

## セキュリティ考慮

- **平文 API キー禁止**: `config.toml` に直接キーを書かない
- **環境変数経由のみ**: 1Password CLI / direnv / systemd EnvironmentFile / GPG 等の secret backend から解決
- **未設定時 fail-fast**: `AgentCliEnvError::MissingApiKeyEnv` で起動前に失敗
- **ログ出力 masking**: `ANTHROPIC_API_KEY=<set, len=N>` 形式で長さのみ表示
- **レジストリパーミッション**: `0700` で他ユーザーからの peer 探索防止

## 利用例

### Anthropic Claude（既定）

```toml
[agent_cli]
backend = "claude"
anthropic_api_key_env = "ANTHROPIC_API_KEY"
model = "claude-opus-4-7"
max_tokens = 4096
```

### Ollama（ローカル）

```toml
[agent_cli]
backend = "ollama"
anthropic_base_url = "http://localhost:11434/v1/"
anthropic_api_key_env = "OLLAMA_API_KEY"
model = "glm-5.1:cloud"
max_tokens = 8192
```

### OpenAI Codex / llama.cpp / LM Studio

- **Codex**: `backend = "codex"` + `model = "gpt-4.1"` + `anthropic_base_url = "https://api.openai.com/v1/"`
- **llama.cpp**: `backend = "llama_cpp"` + `anthropic_base_url = "http://localhost:8080/v1/"`
- **LM Studio**: `backend = "llama_cpp"` + `anthropic_base_url = "http://localhost:1234/v1/"`

### claude-cli-shim engine（Phase 113、Claude Code wrapper）

```toml
[engine]
type = "claude_cli_shim"
# binary = "/home/hidemi/.local/bin/claude-cli-shim"   # 省略時 PATH 解決
# registry_path = "/custom/path"                       # 共有時のみ指定
# log_path = "/custom/path"
```

前提:
- `claude` CLI がインストール済み（`which claude`）
- `ANTHROPIC_API_KEY` が環境変数に設定済み
- `claude-cli-shim` バイナリが PATH に存在（`cargo build` 後 `target/debug/claude-cli-shim`）

cross-engine 通信が必要な場合は `[agent_cli]` `[engine] registry_path` を共有 path にして agent-cli と registry を同期させる（peer 名衝突に注意）。

## テスト戦略

`project-model::config` 配下に 8 件の単体テスト + 3 件の統合テスト:

1. `agent_cli_section_defaults` — Default 値検証
2. `agent_cli_section_parses_with_defaults_when_omitted` — 省略時の Default 補完
3. `agent_cli_section_round_trip_with_custom_values` — Ollama 設定の TOML round-trip
4. `default_template_includes_agent_cli` — default_template 組み込み検証
5. `build_env_anthropic_official_default` — 空 base_url 時の inject 検証
6. `build_env_ollama_includes_base_url` — Ollama 設定の 2 件 inject 検証
7. `build_env_missing_api_key_returns_error` / `build_env_empty_api_key_returns_error` — fail-fast 検証
8. `backend_enum_parse` — 4 種バックエンドパース検証

## 関連ドキュメント

- [agent_cli_messaging.md](agent_cli_messaging.md) — agent-cli メッセージング仕様
- [sub_agent_lifecycle.md](sub_agent_lifecycle.md) — サブエージェント起動・終了管理
- [error_registry.md](error_registry.md) — エラーコード規約