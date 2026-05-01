# asic-conductor ツールアダプター

**対象 Conductor**: asic-conductor
**ソース**: 設計仕様書 §6.4（1842-1865行目付近）, §6.6（1886-1893行目付近）, §6.7（1894-1919行目付近）

## AsicToolAdapter トレイト

ASIC 固有のツールアダプターインターフェース。FPGA の VendorAdapter と異なり、物理設計ステップ（フロアプラン、CTS、寄生抽出等）を網羅する。

```rust
#[async_trait]
pub trait AsicToolAdapter: Send + Sync + 'static {
    fn manifest(&self) -> &AdapterManifest;
    fn capabilities(&self) -> &AsicCapabilitySet;

    // コアフロー（7ステップ）
    async fn synthesize(&self, ctx: &AsicBuildContext) -> Result<StepResult, AdapterError>;
    async fn floorplan(&self, ctx: &AsicBuildContext) -> Result<StepResult, AdapterError>;
    async fn place(&self, ctx: &AsicBuildContext) -> Result<StepResult, AdapterError>;
    async fn cts(&self, ctx: &AsicBuildContext) -> Result<StepResult, AdapterError>;
    async fn route(&self, ctx: &AsicBuildContext) -> Result<StepResult, AdapterError>;
    async fn extract(&self, ctx: &AsicBuildContext) -> Result<StepResult, AdapterError>;
    async fn generate_gdsii(&self, ctx: &AsicBuildContext) -> Result<StepResult, AdapterError>;

    // サインオフ
    async fn timing_signoff(&self, ctx: &AsicBuildContext) -> Result<TimingReport, AdapterError>;
    async fn drc(&self, ctx: &AsicBuildContext) -> Result<SignoffResult, AdapterError>;
    async fn lvs(&self, ctx: &AsicBuildContext) -> Result<SignoffResult, AdapterError>;
}
```

## AsicCapabilitySet

AsicToolAdapter がサポートする機能セット。各ステップごとに対応可否を示す。

## AsicCapabilityRouter（ルーティング戦略）

アダプター選択のルーティング戦略。

| 戦略 | 説明 |
|------|------|
| `PreferOpenLane` | OpenLane2 が対応可能なステップは OpenLane2 に委譲する（既定） |
| `StepOptimal` | 各ステップごとに最適なアダプターを個別選択する |
| `Explicit` | asic.toml で明示的に指定されたアダプターを使用する |

## SignoffChecker

テープアウト前の最終検証を担当する。

### SignoffResult

```rust
pub struct SignoffResult {
    pub tool: SignoffTool,
    pub check_type: CheckType,     // DRC or LVS
    pub passed: bool,
    pub violations: Vec<Violation>,
    pub summary: SignoffSummary,
}

pub struct Violation {
    pub rule: String,              // 違反ルール名
    pub description: String,       // 違反の説明
    pub location: Option<GdsCoord>,// GDSII 座標
    pub severity: ViolationSeverity,
}
```

### サインオフツール

| ツール | 検証種別 | 説明 |
|--------|---------|------|
| Magic | DRC | レイアウト DRC エンジン |
| Netgen | LVS | SPICE レベルの回路比較 |
| KLayout | DRC + LVS | スクリプタブルなレイアウト検証 |

## 主要クレート構成

```
asic-conductor/
├── crates/
│   ├── conductor-core/             # agent-cli persona・main.rs
│   ├── project-model/              # asic.toml パーサー
│   ├── plugin-registry/            # ツール登録・解決（AsicToolAdapter トレイト）
│   ├── adapter-openlane/           # OpenLane 2 統合
│   ├── adapter-yosys/              # Yosys 論理合成
│   ├── adapter-openroad/           # OpenROAD 配置配線
│   ├── pdk-manager/                # PDK 管理
│   ├── podman-runtime/             # コンテナ管理
│   └── conductor-sdk/              # 共有 SDK
```

## 関連ドキュメント

- [asic/config_schema.md](config_schema.md) — asic.toml スキーマ
- [asic/state_machines.md](state_machines.md) — ASIC ビルドステートマシン
- [asic/error_types.md](error_types.md) — asic-conductor エラーコード
- [../fpga/vendor_adapter.md](../fpga/vendor_adapter.md) — FPGA VendorAdapter トレイト