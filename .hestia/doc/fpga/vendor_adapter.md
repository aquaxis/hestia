# fpga-conductor VendorAdapter トレイト

**対象 Conductor**: fpga-conductor
**ソース**: 設計仕様書 §5.2（1489-1540行目付近）, §5.5-5.7（1643-1729行目付近）

## VendorAdapter トレイト定義

全てのアダプターが実装すべき統一インターフェース。

```rust
#[async_trait::async_trait]
pub trait VendorAdapter: Send + Sync + 'static {
    // --- 必須: 自己記述 ---
    fn manifest(&self) -> &AdapterManifest;
    fn capabilities(&self) -> CapabilitySet;

    // --- 必須: コアフロー ---
    async fn synthesize(&self, ctx: &BuildContext) -> Result<StepResult, AdapterError>;
    async fn implement(&self, ctx: &BuildContext) -> Result<StepResult, AdapterError>;
    async fn generate_bitstream(&self, ctx: &BuildContext) -> Result<StepResult, AdapterError>;

    // --- オプション (デフォルト: CapabilityUnsupported を返す) ---
    async fn timing_analysis(&self, ctx: &BuildContext) -> Result<TimingReport, AdapterError>;
    async fn start_debug_session(&self, ctx: &BuildContext) -> Result<DebugSession, AdapterError>;
    async fn hls_compile(&self, ctx: &BuildContext) -> Result<StepResult, AdapterError>;
    async fn program_device(&self, ctx: &ProgramContext) -> Result<(), AdapterError>;
    async fn simulate(&self, ctx: &SimContext) -> Result<SimResult, AdapterError>;

    // --- ログ診断 (デフォルト: None) ---
    fn parse_log_line(&self, line: &str) -> Option<Diagnostic> { None }
}
```

## AdapterManifest

```rust
pub struct AdapterManifest {
    pub id:                String,        // "com.xilinx.vivado"
    pub name:              String,        // "AMD Vivado"
    pub version:           String,        // "0.5.1" (アダプター自身のバージョン)
    pub vendor:            String,        // "AMD/Xilinx"
    pub api_version:       u32,           // ABI 互換チェック用
    pub supported_devices: Vec<String>,   // glob: ["xc7*", "xcvu*", "xck*"]
    pub capabilities:      CapabilitySet,
    pub release_notes_url: Option<String>, // WatcherAgent が使用
}
```

## CapabilitySet

| フィールド | 型 | 説明 |
|-----------|---|------|
| `synthesis` | bool | 合成機能 |
| `implementation` | bool | 配置配線機能 |
| `bitstream` | bool | bitstream 生成機能 |
| `timing_analysis` | bool | タイミング解析機能 |
| `on_chip_debug` | bool | オンチップデバッグ機能 |
| `device_program` | bool | デバイスプログラミング機能 |
| `hls` | bool | HLS 機能 |
| `simulation` | bool | シミュレーション機能 |
| `ip_catalog` | bool | IP カタログ機能 |

## Vivado アダプター実装（§5.5）

AMD Vivado 向けアダプター。TCL スクリプト自動生成（minijinja テンプレート）+ バッチモード実行 + リアルタイムログパース。

- 合成: `vivado -mode batch -source synth.tcl`
- ログパース: 正規表現 `^(ERROR|WARNING|INFO):\s+\[(\w+)\s+([\d-]+)\]\s+(.+)$`
- テンプレート: `vivado_synth.tcl.j2` / `vivado_impl.tcl.j2` / `vivado_bit.tcl.j2`

## Quartus アダプター実装（§5.6）

Intel Quartus Prime 向けアダプター。QPF/QSF ファイル自動生成 + `quartus_sh --flow compile` 実行。

- 合成: .qpf（プロジェクトファイル）+ .qsf（設定ファイル）生成
- 実行: `quartus_sh --flow compile <project>.qpf`

## Efinity アダプター実装（§5.7）

Efinix Efinity 向けアダプター。インターフェーススクリプト（XML）生成 + ビルドスクリプト（Python）生成 + Efinity 同梱 Python で実行。

- インターフェース: `interface.peri.xml`（Rust serde で生成）
- ビルド: `build.py`（Rust テンプレートエンジンで生成）
- 実行: Efinity 同梱 `python3/bin/python3`（外部 Python に依存しない）

## ScriptAdapter（adapter.toml による拡張）

新しいベンダーツールを追加する際、Rust コードの変更なしに `adapter.toml` を書くだけでアダプターを追加可能（原則2: ゼロ変更での拡張）。adapter.toml にはコマンド、ログパースルール、レポート抽出ルールを正規表現で定義する。

## アダプター種別

| 種別 | 説明 |
|------|------|
| ScriptAdapter | adapter.toml ベース（コード変更不要） |
| DynamicAdapter | dlopen による動的読み込み |
| RemoteAdapter | gRPC 経由のリモートアダプター |

## 関連ドキュメント

- [fpga/config_schema.md](config_schema.md) — fpga.toml スキーマ
- [fpga/state_machines.md](state_machines.md) — ビルドステートマシン
- [fpga/error_types.md](error_types.md) — fpga-conductor エラーコード
- [../rtl/rtl_tool_adapter.md](../rtl/rtl_tool_adapter.md) — RtlToolAdapter トレイト
- [../asic/tool_adapter.md](../asic/tool_adapter.md) — AsicToolAdapter トレイト