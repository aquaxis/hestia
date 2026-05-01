# VSCode 設定スキーマ

**対象領域**: frontend — VSCode 拡張設定
**ソース**: 設計仕様書 §16.1

## 概要

VSCode 拡張 `hestia-vscode` が提供する `hestia.*` 設定スキーマ。ユーザーが `settings.json` で Hestia の動作をカスタマイズするために使用する。

## 設定一覧（hestia.*）

| 設定キー | 型 | 既定値 | 説明 |
|---------|-----|-------|------|
| `hestia.agentCliRegistryDir` | string | `$XDG_RUNTIME_DIR/agent-cli/` | agent-cli IPC レジストリディレクトリ（空時は agent-cli 既定値）|
| `hestia.autoConnect` | boolean | true | 起動時の自動接続 |
| `hestia.reconnectInterval` | number | 5000 | 再接続間隔（ms）|
| `hestia.requestTimeout` | number | 30000 | リクエストタイムアウト（ms）|
| `hestia.ai.model` | string | `"claude-sonnet-4-6"` | AI モデル選択（`claude-sonnet-4-6` / `claude-opus-4-7` / `claude-haiku-4-5`）|
| `hestia.ai.maxTokens` | number | 4096 | AI 応答上限トークン数 |
| `hestia.ai.apiKeyEnv` | string | `"ANTHROPIC_API_KEY"` | API キー環境変数名 |
| `hestia.ai.baseUrl` | string | `""` | OpenAI 互換 API エンドポイント |

## ConductorId との対応

peer 名（ConductorId）は設定キーと 1:1 対応:

| ConductorId | 対応 conductor |
|------------|---------------|
| `ai` | ai-conductor |
| `rtl` | rtl-conductor |
| `fpga` | fpga-conductor |
| `asic` | asic-conductor |
| `pcb` | pcb-conductor |
| `hal` | hal-conductor |
| `apps` | apps-conductor |
| `debug` | debug-conductor |
| `rag` | rag-conductor |

## 設定例（settings.json）

```json
{
  "hestia.agentCliRegistryDir": "/run/user/1000/agent-cli",
  "hestia.autoConnect": true,
  "hestia.requestTimeout": 60000,
  "hestia.ai.model": "claude-opus-4-7",
  "hestia.ai.maxTokens": 8192,
  "hestia.ai.apiKeyEnv": "ANTHROPIC_API_KEY"
}
```

## 関連ドキュメント

- [vscode_extension.md](vscode_extension.md) — VSCode 拡張
- [agent_cli_client.md](agent_cli_client.md) — agent-cli クライアント仕様
- [backend_switching.md](../common/backend_switching.md) — LLM バックエンド切替