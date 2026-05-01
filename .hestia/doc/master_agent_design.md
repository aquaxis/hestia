# ai-conductor 詳細設計 — メタオーケストレーター

**対象領域**: ai-conductor（メタオーケストレーター）
**ソース**: 設計仕様書 §3（745-1240行目）

---

## 概要

ai-conductor は HESTIA の最上位 conductor であり、フロントエンド（VSCode / Tauri / CLI）からの唯一の入口として機能する。配下の全 conductor（rtl / fpga / asic / pcb / hal / apps / debug / rag）を統括し、タスクの分解・振り分け、ヘルスチェック、スキル管理、コンテナ管理の4つの中核機能を提供する。

---

## 1. 4中核機能

| 機能 | 役割 | 関連節 |
|------|------|--------|
| **タスク分解・振り分け** | フロントエンドからの自然言語または構造化指示を理解し、タスクを分解して配下の適切な conductor に振り分ける | §3.3 task-router / §3.3.1 / §3.5 WorkflowEngine / §3.6 SpecDriven |
| **ヘルスチェック** | 全 conductor を定期ポーリングし、Online / Offline / Degraded / Upgrading 状態を集約管理。異常時は自動再起動またはフロントエンドへエスカレーション | §3.1 ai-core/health_check.rs / §3.2 ConductorStatus / §3.3.2 |
| **スキル管理** | SkillRegistry に専門スキル（HDL 生成、制約生成、テストベンチ生成等）をプラグイン登録し、配下 conductor の agent-cli persona に提供 | §3.1 skill-system/ / §3.7 |
| **コンテナ管理** | `container.toml` 宣言に基づく Containerfile 自動生成・ビルド・差分更新・プロビジョニング・レジストリ管理（コンテナ実行を選択した場合のみ） | §3.1 container-manager/ / §3.8 / §12 |

加えて補助機能として、持続可能アップグレード（§3.4 UpgradeManager）/ DAG ベースワークフロー（§3.5 WorkflowEngine）/ 仕様書駆動開発（§3.6 SpecDriven）/ LLM バックエンド切替（§20 agent-cli エンドポイント設定）を提供する。

---

## 2. クレート構成

```
ai-conductor/
├── Cargo.toml
├── crates/
│   ├── ai-core/                    # ConductorManager、ヘルスチェック
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── conductor_manager.rs # 全 conductor ライフサイクル管理
│   │       └── health_check.rs      # 定期的ヘルスチェック
│   ├── conductor-client/           # agent-cli IPC クライアント
│   │   └── src/
│   │       ├── lib.rs              # ConductorClient
│   │       └── transport.rs        # Unix Socket トランスポート
│   ├── upgrade-manager/            # 持続可能アップグレード管理
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── version_policy.rs   # セマンティックバージョニング
│   │       ├── rollout.rs          # 段階的ロールアウト
│   │       └── rollback.rs         # 自動ロールバック
│   ├── workflow-engine/            # DAG ベースワークフローエンジン
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── dag.rs              # DAG 定義・実行
│   │       └── pipeline.rs         # クロス conductor パイプライン
│   ├── spec-driven/                # 仕様書駆動開発エンジン
│   │   └── src/
│   │       ├── lib.rs
│   │       └── parser.rs           # SpecParser → DesignSpec
│   ├── skill-system/               # スキルプラグインシステム
│   │   └── src/
│   │       ├── lib.rs              # SkillRegistry
│   │       └── skill.rs            # Skill トレイト
│   ├── multi-agent/                # 階層的エージェント管理
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── agent_manager.rs    # エージェント起動・停止・監視
│   │       ├── message_broker.rs   # メッセージルーティング
│   │       └── session.rs          # セッション管理
│   ├── agent-communication/        # メッセージブローカー
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── protocol.rs         # メッセージプロトコル定義
│   │       └── message.rs          # AgentMessage フォーマット
│   ├── agent-monitoring/           # リアルタイム監視
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── live_view.rs        # リアルタイム表示
│   │       └── health_check.rs     # エージェントヘルスチェック
│   └── container-manager/          # コンテナ管理
│       └── src/
│           ├── lib.rs
│           ├── builder.rs          # Containerfile 自動生成・ビルド
│           ├── registry.rs         # コンテナイメージレジストリ
│           ├── updater.rs          # イメージ差分更新
│           ├── provisioner.rs      # ツールプロビジョニング
│           └── tool_updater.rs     # ツールアップデート管理
├── ai-cli/                         # Rust 製 CLI（hestia-ai-cli）
└── conductor-sdk/                  # 共通 SDK（transport/message/agent/config/error）
```

**注**: 旧 `rag-engine` クレート（TypeScript + LangChain）と `rag-ingest` モジュール（Rust）は **rag-conductor**（独立 Conductor）に分離された。ai-conductor からは agent-cli IPC の `rag` peer に対して `rag.*` 構造化メッセージを送信して呼び出す。

---

## 3. ConductorManager（MetaOrchestrator）

```rust
pub struct ConductorManager {
    conductors: Arc<RwLock<HashMap<ConductorId, ConductorInfo>>>,
    pub config: OrchestratorConfig,
}

pub enum ConductorId {
    Ai,          // agent-cli peer "ai"          (self / loopback ヘルスチェック用)
    Rtl,         // agent-cli peer "rtl"         (RTL 上流)
    Fpga,        // agent-cli peer "fpga"
    Asic,        // agent-cli peer "asic"
    Pcb,         // agent-cli peer "pcb"
    Hal,         // agent-cli peer "hal"          (HAL 生成)
    Apps,        // agent-cli peer "apps"         (アプリ FW)
    Debug,       // agent-cli peer "debug"
    Rag,         // agent-cli peer "rag"          (旧 ai-conductor::rag-engine から分離)
}

pub enum ConductorStatus {
    Online,     // 正常稼働中
    Offline,    // 停止中
    Degraded,   // 劣化状態（一部機能制限あり）
    Upgrading,  // アップグレード中
}
```

---

## 4. メタオーケストレーション機能

ai-conductor 自身が agent-cli プロセス（peer 名 `ai`）として起動され、以下の機能を提供する。下流 conductor との通信は agent-cli ネイティブ IPC のみを使用する。

```
ai-conductor (= agent-cli process / peer name "ai")
    │
    ├── task-router ───── フロントエンド指示の理解・タスク分解・振り分け
    ├── health-checker ── 全 conductor の定期ヘルスチェック
    ├── conductor-router ─── 下流 conductor への agent-cli IPC ルーティング
    │   ├── rtl-conductor         (peer "rtl")
    │   ├── fpga-conductor        (peer "fpga")
    │   ├── asic-conductor        (peer "asic")
    │   ├── pcb-conductor         (peer "pcb")
    │   ├── hal-conductor         (peer "hal")
    │   ├── apps-conductor        (peer "apps")
    │   ├── debug-conductor       (peer "debug")
    │   └── rag-conductor         (peer "rag")
    ├── conductor-startup ─── 起動順序オーケストレーション
    │   ├── Group 0: ai-conductor（最高優先度、直列）
    │   └── Group 1: rtl / fpga / asic / pcb / hal / apps / debug / rag（8 並列、ai readiness 後）
    ├── upgrade-manager ─── 持続可能アップグレード
    ├── workflow-engine ─── DAG ベースワークフロー
    ├── spec-driven ─── 仕様書駆動開発
    ├── skill-system ─── スキルプラグイン
    ├── backend-switching ─── LLM バックエンド切替
    └── container-manager ─── コンテナライフサイクル管理
```

---

## 5. タスク振り分けフロー

フロントエンドから ai-conductor への指示は自然言語または構造化 JSON ペイロードとして受領される。`task-router` は agent-cli の LLM 推論を活用してタスクを分解し、依存関係を解析した上で配下 conductor へ dispatch する。

```
[Frontend (VSCode/Tauri/CLI)]
       │
       │ agent-cli send ai <payload>
       ▼
[ai-conductor: task-router]
       │
       │ Step 1. 意図理解 (intent classification)
       │    - 自然言語 → 設計タスク種別判定
       │    - 構造化 JSON → method 名前空間で直接判定
       │
       │ Step 2. タスク分解 (task decomposition)
       │    - 単一 conductor で完結 → そのまま dispatch
       │    - 複数 conductor 横断   → workflow-engine に委譲し DAG 化
       │    - 仕様書ベース          → spec-driven で DesignSpec 生成 → DAG 化
       │
       │ Step 3. 振り分け (routing via conductor-router)
       │    - 適切な peer に agent-cli send <peer> <payload>
       │
       ▼
[配下 conductor]
       │ 結果応答（同じ trace_id で agent-cli send ai <result>）
       ▼
[ai-conductor: 結果集約 → フロントエンドへ通知]
```

**タスク分解・振り分け例:**

| 入力例 | 分解後タスク | 振り分け先 |
|--------|-----------|----------|
| "Vivado で artix7 用にビルドして" | `fpga.build.v1.start { target: "artix7" }` | fpga-conductor |
| "RTL を lint して合成可能か確認して" | `rtl.lint.v1` → `rtl.handoff.v1 { target: "fpga" }` | rtl-conductor → fpga-conductor |
| "FPGA プロトタイプから ASIC 化して GDSII まで作って" | DAG: `rtl.handoff` → `asic.synth` → ... → `asic.gdsii` | workflow-engine 経由 / 複数 conductor |
| `{"method":"meta.dualBuild.v1", "params":{...}}` | DAG: `fpga.build` ‖ `asic.synth` → `meta.collect` | workflow-engine |

---

## 6. ヘルスチェック機能

`health-checker` は全 conductor の生存・正常性を定期的に確認し、ConductorManager の `ConductorStatus` を更新する。異常検出時は自動再起動を試み、回復不能な場合は upgrade-manager または人間（フロントエンド通知）にエスカレーションする。

- **ポーリング間隔**: 既定 30 秒（`[health] interval_secs` で変更可）
- **方式**: `agent-cli send <peer> '{"method":"system.health.v1","id":"hc_<ts>"}'`
- **応答パターン**:
  - 3 秒以内に "online" 応答 → Online
  - タイムアウト (3 秒) → Offline
  - "degraded" 応答 → Degraded
  - "upgrading" 応答 → Upgrading
- **状態変化時アクション**:
  - Online → Offline / Degraded → 自動再起動試行 (max 3)
  - 連続 3 回失敗 → フロントエンド通知
  - Upgrading → Online → upgrade-manager に成功通知
  - 任意 → 状態履歴を sled に永続化

**ヘルスチェック設定例（container.toml `[health]` セクション）:**

```toml
[health]
cmd = "vivado -version || true"
interval_secs = 30
timeout_secs = 3
max_retries = 3
escalate_on_fail = true
restart_on_fail = true
```

---

## 7. UpgradeManager 詳細

セマンティックバージョニングに基づく互換性評価・段階的ロールアウト・自動ロールバックを提供する。

### 7.1 互換性判定

| バージョン変更 | 互換性 | 要求される戦略 |
|--------------|--------|-------------|
| `1.0.0` → `1.1.0` | 互換 | Production 可 |
| `1.0.0` → `1.0.1` | 互換 | Production 可 |
| `1.0.0` → `2.0.0` | 非互換 | Canary または Staging 必須 |

### 7.2 段階的ロールアウト戦略

| 戦略 | 説明 | 使用場面 |
|------|------|---------|
| `Canary` | 少数環境に先行展開 | メジャーバージョン変更時 |
| `Staging` | ステージング環境で検証後に本番展開 | マイナーバージョン更新 |
| `Production` | 本番環境に直接展開 | パッチリリース |

### 7.3 エージェントチェーン

```
WatcherAgent → ProbeAgent → PatcherAgent → ValidatorAgent
```

- **WatcherAgent**: ベンダーツールのリリースノートを監視・検知
- **ProbeAgent**: リリースノートから変更内容を分析・影響評価
- **PatcherAgent**: Anthropic SDK の Tool Use 機能を活用し、エージェントループ内でパッチを自動生成
- **ValidatorAgent**: 生成されたパッチの検証を実施

### 7.4 RollbackConfig

```rust
pub struct RollbackConfig {
    pub auto_rollback: bool,     // 自動ロールバック有効化
    pub timeout_secs: u64,       // タイムアウト（デフォルト: 300秒）
    pub max_retries: u32,        // 最大リトライ回数（デフォルト: 3回）
}
```

---

## 8. WorkflowEngine 詳細

DAG ベースのクロス conductor パイプラインエンジンである。カーンのアルゴリズムによるトポロジカルソートで実行順序を決定し、sled で状態を永続化する。

```rust
pub struct WorkflowStep {
    pub id: String,              // ステップ ID
    pub name: String,            // ステップ名
    pub conductor: String,       // 対象 conductor
    pub method: String,          // 実行する agent-cli メッセージ method
    pub params: Option<Value>,   // パラメータ
    pub depends_on: Vec<String>, // 依存ステップ ID（DAG 構造）
    pub status: StepStatus,      // 現在の状態
}
```

**ダイヤモンド型依存関係の例:**

```
        [A: FPGA 合成]
       /              \
[B: ASIC 合成]    [C: PCB 設計]
       \              /
        [D: 統合検証]
```

---

## 9. SpecDriven（仕様書駆動開発）詳細

自然言語仕様書から設計データを自動生成する。`REQ:` / `CON:` / `IF:` プレフィックスで要件・制約・インターフェースを自動解析する。

```rust
pub struct SpecParser;

impl SpecParser {
    pub fn parse(spec_text: &str) -> Result<DesignSpec, SpecError> {
        // REQ: で始まる行 → 要件
        // CON: で始まる行 → 制約
        // IF:  で始まる行 → インターフェース定義
        // 必須要件が1件以上なければエラー
    }
}
```

**フロー**: `仕様書（自然言語）→ SpecParser → DesignSpec → AI 生成エンジン → HDL / 制約 / テストベンチ`

公開 method: `ai.spec.init` / `ai.spec.update` / `ai.spec.review`

---

## 10. SkillSystem（スキルプラグイン）詳細

SkillRegistry に専門スキルを登録し、AI エージェント（agent-cli プロセス）が呼び出す。スキルは agent-cli のペルソナファイル（YAML+Markdown）と組み合わせて conductor ごとのメインエージェント・サブエージェントの能力を定義する。

**デフォルトスキル:**

| スキル | 説明 |
|--------|------|
| HDL 生成 | SystemVerilog / Verilog / VHDL コード自動生成 |
| 制約生成 | XDC / SDC / PCF 制約ファイル自動生成 |
| テストベンチ生成 | テストベンチスケルトン + アサーション自動生成 |

カスタムスキルは `Skill` トレイトを実装して SkillRegistry に登録する。

---

## 11. container.toml リファレンス

各 conductor が使用するコンテナ環境を宣言的に定義するファイルである。

| セクション | 必須 | 説明 |
|-----------|------|------|
| `[container]` | 必須 | コンテナ基本設定（名前、ベースイメージ、対象 conductor） |
| `[tools.*]` | 任意 | インストールするツール定義 |
| `[env]` | 任意 | 環境変数 |
| `[[volumes]]` | 任意 | ボリュームマウント定義 |
| `[health]` | 任意 | ヘルスチェック設定 |
| `[update]` | 任意 | アップデートポリシー |

**container.toml サンプル:**

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

---

## 12. upgrade.toml リファレンス

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

---

## 13. サブエージェント構成

ai-conductor はタスク振り分けと SpecDriven を支援するために、配下に **2 種類のサブエージェント** を持つ。各サブエージェントは独立した agent-cli プロセスとして起動され、`agent-cli send <peer>` IPC で ai-conductor 本体（peer 名 `ai`）と協調する。

| サブエージェント | peer 名 | 役割 | 多重度 | persona ファイル |
|----------------|---------|------|-------|-----------------|
| **planner** | `ai-planner` | フロントエンド指示タスクの分解、実行プランニング（DAG 化、依存関係解析、配下 conductor への dispatch 戦略）を作成 | 1（高負荷時 N 並列可）| `.hestia/personas/ai-planner.md` |
| **designer** | `ai-designer` | フロントエンド指示に基づき全体仕様（DesignSpec、HW/SW 統合の上位設計、conductor 間連携契約）を作成 | 1 | `.hestia/personas/ai-designer.md` |

**起動と協調フロー:**

```
[Frontend (VSCode/Tauri/CLI)]
       │ agent-cli send ai <payload>
       ▼
[ai-conductor (peer "ai")]
       │
       ├── agent-cli send ai-planner '{"method":"plan.v1.create",...}'
       │       ↓ プラン応答（DAG / Step リスト / 配下 conductor 割当案）
       │
       ├── agent-cli send ai-designer '{"method":"design.v1.create",...}'
       │       ↓ DesignSpec 応答（上位仕様 / conductor 間連携契約）
       │
       ▼
[ai-conductor: planner + designer の出力を統合 → conductor-router で dispatch]
```

**スケーリングと寿命:**

- planner / designer はいずれも常駐型で、ai-conductor の寿命と同期して起動・停止される
- 高負荷時には planner を複数 instance 起動可能（peer 名 `ai-planner-1`, `ai-planner-2` ...）
- `agent-cli list` で discoverable、health-checker の対象に含まれる

**起動コマンド例:**

```bash
agent-cli run --persona-file ./.hestia/personas/ai-planner.md  --name ai-planner  &
agent-cli run --persona-file ./.hestia/personas/ai-designer.md --name ai-designer &
```

---

## 関連ドキュメント

- [ai_conductor.md](ai_conductor.md) — ai-conductor 全体概要（要約版）
- [rtl_conductor.md](rtl_conductor.md) — RTL 設計フローオーケストレーター
- [fpga_conductor.md](fpga_conductor.md) — FPGA 設計フローオーケストレーター
- [asic_conductor.md](asic_conductor.md) — ASIC 設計フローオーケストレーター
- [pcb_conductor.md](pcb_conductor.md) — PCB 設計フローオーケストレーター
- [hal_conductor.md](hal_conductor.md) — HAL 生成オーケストレーター
- [apps_conductor.md](apps_conductor.md) — アプリケーションソフトウェア開発オーケストレーター
- [debug_conductor.md](debug_conductor.md) — デバッグ環境オーケストレーター
- [rag_conductor.md](rag_conductor.md) — 知識基盤オーケストレーター
- [architecture_overview.md](architecture_overview.md) — 全体アーキテクチャ概要