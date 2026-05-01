# 通信仕様（Agent Communication）

**対象領域**: Hestia 全体
**ソース**: 設計仕様書 §2.3（通信アーキテクチャ）, §14（インターフェース定義）, §20（agent-cli エンドポイント設定）

---

## 1. 通信アーキテクチャ概要

全 conductor は [`agent-cli`](https://github.com/aquaxis/agent-cli) プロセスとして起動された AI エージェントであり、フロントエンドおよび conductor 間のすべての通信は **agent-cli ネイティブ IPC** に統一される。

レガシー JSON-RPC 2.0 over Unix Domain Socket（`/var/run/hestia/*.sock`）は廃止し、`agent-cli send <peer> <payload>` API および共有レジストリ（`$XDG_RUNTIME_DIR/agent-cli/`）による peer 探索で接続する。

```
┌─────────────────────────────────────────────────────────────────┐
│           通信プロトコルスタック (agent-cli 単一チャンネル)         │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  ━━━ agent-cli ネイティブ IPC（唯一の通信手段）━━━                │
│                                                                 │
│  全 9 conductor + フロントエンドクライアントが peer 参加          │
│      レジストリ: $XDG_RUNTIME_DIR/agent-cli/  (パーミッション 0700)│
│      探索:       agent-cli list                                  │
│      送信:       agent-cli send <peer> <payload>                 │
│                  または REPL 内コマンド /send <peer> <payload>    │
│                                                                 │
│      peer 名 (= ConductorId 文字列):                             │
│        ai / rtl / fpga / asic / pcb /                           │
│        hal / apps / debug / rag                                  │
│                                                                 │
│      フロントエンドの peer 名: vscode / tauri / cli (任意)       │
│                                                                 │
│  ━━━ ペイロード形式（同一チャンネルで併存可）━━━                   │
│                                                                 │
│  (a) 構造化メッセージ — JSON ペイロード                           │
│      method 名前空間規約に従う（§14 メッセージング仕様）           │
│                                                                 │
│  (b) 自然言語メッセージ — プレーンテキスト                         │
│      自由形式、CoT 文脈共有、エージェント本来の協調モデル          │
│                                                                 │
│  ━━━ 共有サービス層（横断ツール、agent-cli の peer として実装）━━━│
│                                                                 │
│  共有サービス peer 名: lsp / constraint-bridge / ip-manager /    │
│                       cicd / observability / waveform / mcp     │
│                                                                 │
│  ━━━ 外部アダプター（agent-cli IPC の境界外）━━━                  │
│                                                                 │
│  Remote アダプター: gRPC (proto3)                               │
│      サービス: VendorAdapterService                              │
│      （ベンダーツール等、外部システムとの境界に限り gRPC を使用） │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. ペイロード形式

### 2.1 構造化リクエスト / 応答 / 通知

```json
// Request payload
{
  "method": "fpga.build.v1.synthesize",
  "params": { ... },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}

// Success Response payload
{
  "result": { ... },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}

// Error Response payload
{
  "error": { "code": -32200, "message": "...", "data": { ... } },
  "id": "msg_2026-05-01T12:00:00Z_abc123",
  "trace_id": "trace_xyz789"
}

// Notification payload（id なし、応答なし）
{
  "method": "agent.status_update",
  "params": { ... },
  "trace_id": "trace_xyz789"
}

// Batch payload（同順応答）
[ { "method":"...", "params":{}, "id":"msg_1" },
  { "method":"...", "params":{}, "id":"msg_2" } ]
```

- `id` は `msg_{ISO8601 timestamp}_{random}` 形式
- `trace_id` はワークフロー横断のトレース ID
- レガシー JSON-RPC 2.0 の `"jsonrpc": "2.0"` フィールドは不要

### 2.2 ペイロード判定

受信側 agent-cli persona は payload の先頭を判定:
- `{` で始まる → JSON 構造化メッセージとしてツール呼出に変換
- それ以外 → 自然言語として LLM に直接渡す

### 2.3 ペイロード選択指針

| 通信種別 | 推奨ペイロード | 理由 |
|---------|-------------|------|
| フロントエンドからの構造化操作 | (a) 構造化 JSON | 型安全、エラーコード規約、SDK 互換性 |
| conductor 間の構造化ツール呼出 | (a) 構造化 JSON | 再現性、トレース ID 連鎖、Sled 状態永続化との整合 |
| conductor 間の自然言語協調 | (b) 自然言語テキスト | 自由形式、CoT 文脈共有 |
| 進捗・CoT・思考過程の共有 | (b) 自然言語テキスト | 軽量伝搬、observability log 連携 |
| エラーエスカレーション | (b) 自然言語で ai-conductor に集約 → (a) 構造化通知でフロントエンドへ |
| イベント通知 | (a) 構造化 JSON（id なし = 通知）| 購読 / フィルタ可能、UI 即時反映 |

---

## 3. メソッド名前空間

### 3.1 命名規則

`{domain}.{method_group}.{version_prefix}.{action}`（例: `fpga.build.v1.synthesize`）

簡略形 `{domain}.{action}` も同義（v1 既定）。

### 3.2 バージョニング

- `ApiVersion { major, minor }`
- 必須パラメータ追加・既存型変更・メソッド削除は `major` バンプ
- 任意パラメータ／応答フィールド追加は後方互換
- 廃止予告: `DeprecationNotice { deprecated_since, removal_scheduled, replacement }`

### 3.3 ドメイン一覧

| ドメイン | 例 |
|---------|----|
| `ai.*`   | `ai.spec.init` / `ai.spec.update` / `ai.spec.review` / `ai.exec` / `agent_spawn` / `agent_list` |
| `fpga.*` | `fpga.synthesize` / `fpga.implement` / `fpga.bitstream` / `fpga.simulate` / `fpga.program` |
| `asic.*` | `asic.synthesize` / `asic.floorplan` / `asic.place` / `asic.cts` / `asic.route` / `asic.gdsii` / `asic.drc` / `asic.lvs` |
| `pcb.*`  | `pcb.generate_schematic` / `pcb.run_drc` / `pcb.run_erc` / `pcb.generate_bom` / `pcb.place_components` / `pcb.route_traces` / `pcb.generate_output` / `pcb.ai_synthesize` / `pcb.status` |
| `debug.*`| `debug.connect` / `debug.disconnect` / `debug.program` / `debug.start_capture` / `debug.stop_capture` / `debug.read_signals` / `debug.set_trigger` / `debug.reset` / `debug.status` |
| `rag.*`  | `rag.ingest` / `rag.search` / `rag.cleanup` / `rag.status` |
| `meta.*` | `meta.dualBuild` / `meta.boardWithFpga` ほかクロス Conductor ワークフロー |
| `system.*` | `system.readiness` / `system.health` / `system.shutdown` |

---

## 4. エラーコード規約

| 範囲 | 領域 |
|------|------|
| `-32700` | Parse Error（JSON ペイロードのパース失敗）|
| `-32600` 〜 `-32603` | リクエスト標準エラー（Invalid Request / Method not found / Invalid params / Internal）|
| `-32000` 〜 `-32099` | HESTIA 共通（Timeout / NotFound / AlreadyExists / PermissionDenied / InvalidState 等）|
| `-32100` 〜 `-32199` | ai-conductor（Orchestration / Agent mgmt / Spec-driven / Version tracking / LLM）|
| `-32200` 〜 `-32299` | fpga-conductor（Synthesis / Implementation / Bitstream / Timing / Debug / HLS / Device / Simulation / Constraints / Adapter）|
| `-32300` 〜 `-32399` | asic-conductor（RTL Synth / Floorplan / Placement / CTS / Routing / 他）|
| `-32400` 〜 `-32499` | pcb-conductor（Schematic / DRC・ERC / BOM・Placement / Gerber / AI Synthesis / KG / Constraint Verify）|
| `-32500` 〜 `-32599` | debug-conductor（JTAG / SWD / Session / Waveform / Programming / Signal / Trigger / Reset / Protocol）|
| `-32600` 〜 `-32699` | rag-conductor（Ingest / PDF / Web / Quality gate / Chunk・Embed / Vector・Search / License・PII / Scheduler / Cache）|

エラー応答 `data` には `tool` / `exit_code` / `log_path` / `errors[]` / `retry_possible` / `suggested_action` を含める。

---

## 5. conductor-core 共通 API

```rust
pub trait ConductorRpc {
    // プロジェクト管理
    async fn project_open(&self, path: String) -> ProjectInfo;
    async fn project_targets(&self) -> Vec<Target>;
    async fn project_files(&self) -> FileTree;

    // ビルド
    async fn build_start(&self, target: String, steps: Vec<BuildStep>) -> BuildJobId;
    async fn build_cancel(&self, job_id: BuildJobId) -> ();
    async fn build_status(&self, job_id: BuildJobId) -> BuildStatus;

    // レポート
    async fn report_timing(&self, job_id: BuildJobId) -> TimingReport;
    async fn report_resource(&self, job_id: BuildJobId) -> ResourceReport;
    async fn report_messages(&self, job_id: BuildJobId) -> Vec<AnnotatedMessage>;

    // プログラミング
    async fn program_targets(&self) -> Vec<ProgramTarget>;
    async fn program_flash(&self, target: String, bitfile: String) -> ();

    // ツールチェーン
    async fn toolchain_list(&self) -> Vec<ToolInstall>;
    async fn toolchain_install(&self, id: String) -> InstallProgress;
    async fn toolchain_select(&self, target: String, version: String) -> ();

    // エージェント
    async fn agent_status(&self) -> AgentSystemStatus;
    async fn agent_patch_list(&self) -> Vec<PatchProposal>;
    async fn agent_apply_patch(&self, patch_id: String) -> ();
    async fn agent_reject_patch(&self, patch_id: String, reason: String) -> ();

    // コンテナ
    async fn container_list(&self) -> Vec<ContainerInfo>;
    async fn container_start(&self, id: String) -> ();
    async fn container_stop(&self, id: String) -> ();
    async fn container_update(&self, id: String) -> UpdateResult;

    // システム
    async fn system_readiness(&self) -> ReadinessStatus;
    async fn system_health(&self) -> HealthStatus;
}
```

---

## 6. agent-cli エンドポイント設定

### 6.1 概要

各 conductor は agent-cli プロセスとして起動される。agent-cli は Claude Code 等価のツール/思考機能を提供する Rust 製スタンドアロン CLI バイナリ。バックエンド LLM は `config.toml::[agent_cli]` で設定。

4種類のバックエンドから選択可能:
- Anthropic Claude（既定）
- OpenAI Codex
- Ollama（ローカル）
- llama.cpp（OpenAI 互換）

### 6.2 `[agent_cli]` スキーマ

```toml
[agent_cli]
backend = "claude"                            # "claude" | "codex" | "ollama" | "llama_cpp"
binary_path = ""                              # 空 = $PATH 解決 / フルパス指定可
anthropic_base_url = ""                       # 空 = Anthropic 公式
anthropic_api_key_env = "ANTHROPIC_API_KEY"   # API キーを格納する環境変数名
model = "claude-opus-4-7"                     # LLM モデル識別子
max_tokens = 4096                             # 応答上限トークン数
registry_dir = ""                             # エージェント間 IPC レジストリ（空 = $XDG_RUNTIME_DIR/agent-cli）
```

### 6.3 環境変数フォワーディング

1. `config.toml` を読む
2. `anthropic_api_key_env` で指定された環境変数をホストから取得（未設定 → fail-fast）
3. `anthropic_base_url` が空でなければ `ANTHROPIC_BASE_URL` を子プロセスに inject
4. API キーを `ANTHROPIC_API_KEY` として子プロセスに inject
5. `agent-cli` を子プロセスとして起動

### 6.4 セキュリティ考慮

- **平文 API キー禁止**: `config.toml` に直接 API キーを書かない
- **環境変数経由のみ**: `anthropic_api_key_env` で環境変数名を指定
- **未設定時の明示エラー**: fail-fast で起動前に失敗
- **ログ出力時の masking**: API キー値そのものを出力せず長さのみ表示
- **IPC レジストリ**: パーミッション 0700 で他ユーザーからのなりすまし防止

---

## 7. 実装イメージ

各 conductor は単一プロセスとして起動:

```bash
agent-cli run --persona-file ./.hestia/personas/<conductor>.md --name <conductor>
```

persona ファイルで「構造化メッセージハンドラ tool」と「自然言語応答 tool」を宣言する。フロントエンド（VSCode / Tauri / CLI）も agent-cli の peer として参加し、`agent-cli send ai <payload>` で ai-conductor に接続する。

### 設計上のメリット

- 通信路が agent-cli IPC 1 系統に集約され、運用・障害切り分けが単純化
- peer モデルにより全エージェントが対等に discoverable
- 構造化呼出と自然言語が同チャンネルで併存
- 開発者は `agent-cli list` / `agent-cli send` で任意 conductor と直接対話可能
- 各 conductor の LLM バックエンドは個別または一括切替可能

---

## 関連ドキュメント

- [architecture_overview.md](architecture_overview.md) — アーキテクチャ概要
- [glossary.md](glossary.md) — 用語集
- [common/agent_cli_messaging.md](common/agent_cli_messaging.md) — agent-cli メッセージング詳細
- [common/api_versioning.md](common/api_versioning.md) — API バージョニング詳細
- [common/error_registry.md](common/error_registry.md) — エラーコード一覧