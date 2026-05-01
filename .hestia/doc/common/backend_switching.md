# LLM バックエンド切替

**対象領域**: common — agent-cli エンドポイント
**ソース**: 設計仕様書 §20

## 概要

HESTIA の 9 conductor および各サブエージェントは agent-cli プロセスとして実行される。agent-cli のバックエンド LLM は `config.toml` の `[agent_cli]` セクションで設定し、4 種類のバックエンドから選択可能。

## 対応バックエンド

| バックエンド | `backend` 値 | 特徴 |
|------------|-------------|------|
| Anthropic Claude | `"claude"` | 既定。高精度 Tool Use |
| OpenAI Codex | `"codex"` | OpenAI API 互換 |
| Ollama | `"ollama"` | ローカル実行、オフライン対応 |
| llama.cpp | `"llama_cpp"` | OpenAI 互換エンドポイント |

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