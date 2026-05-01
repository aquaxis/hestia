# rtl-conductor RtlToolAdapter トレイト

**対象 Conductor**: rtl-conductor
**ソース**: 設計仕様書 §4.2（1262-1278行目付近）, §4.5（1312-1327行目付近）

## RtlToolAdapter トレイト定義

全ての RTL ツールアダプターが実装すべき統一インターフェース。

```rust
#[async_trait]
pub trait RtlToolAdapter: Send + Sync {
    fn id(&self) -> &str;
    fn supported_languages(&self) -> &[HdlLanguage];   // SystemVerilog / VHDL / Chisel / SpinalHDL / Amaranth
    fn capabilities(&self) -> RtlCapability;            // Lint | Sim | Formal | Transpile

    async fn lint(&self, project: &RtlProject) -> Result<LintReport, AdapterError>;
    async fn simulate(&self, project: &RtlProject, tb: &TestBench) -> Result<SimReport, AdapterError>;
    async fn formal_verify(&self, project: &RtlProject, props: &[Property])
        -> Result<FormalReport, AdapterError>;
    async fn transpile(&self, src: &Path, target_lang: HdlLanguage)
        -> Result<PathBuf, AdapterError>;
}
```

## RtlCapability

アダプターがサポートする機能フラグ。

| 値 | 説明 |
|---|------|
| `Lint` | Lint / フォーマット / 静的解析 |
| `Sim` | シミュレーション（サイクル精度 / 動作） |
| `Formal` | 形式検証（プロパティベース） |
| `Transpile` | HDL 言語間トランスパイル |

## 対応言語（HdlLanguage）

| 言語 | 識別子 |
|------|--------|
| SystemVerilog | `systemverilog` |
| Verilog | `verilog` |
| VHDL | `vhdl` |
| Chisel | `chisel` |
| SpinalHDL | `spinalhdl` |
| Amaranth | `amaranth` |
| MyHDL | `myhdl` |

## 主要アダプター一覧

| アダプター | 役割 | 対応言語 | Capability |
|----------|------|---------|-----------|
| `verilator-lint` | Lint | SystemVerilog / Verilog | Lint |
| `verible` | Format / Lint | SystemVerilog | Lint |
| `verilator` | サイクル精度シミュレーション | SystemVerilog / Verilog | Sim |
| `iverilog` | シミュレーション | Verilog | Sim |
| `ghdl` | VHDL シミュレーション | VHDL | Sim |
| `symbiyosys` | 形式検証（プロパティ） | SystemVerilog | Formal |
| `riscof` | RISC-V 命令セット適合性 | RV32I/M/A/F/D/C | Formal |
| `cocotb` | Python テストベンチ | 全言語 | Sim |
| `chisel-bridge` | Chisel → Verilog | Chisel | Transpile |
| `spinalhdl-bridge` | SpinalHDL → Verilog | SpinalHDL | Transpile |
| `amaranth-bridge` | Amaranth → Verilog | Amaranth (Python) | Transpile |

## クレート構成

```
crates/rtl-conductor/
├── src/
│   ├── adapter.rs      # RtlToolAdapter トレイト
│   ├── language.rs     # HDL 言語識別 / トランスパイル管理
│   ├── lint.rs         # Lint / フォーマット / 静的解析
│   ├── simulation.rs   # シミュレーション統合
│   ├── formal.rs       # 形式検証統合
│   ├── repository.rs   # RTL モジュール台帳
│   └── handoff.rs      # 下流 conductor へのハンドオフ
```

## 関連ドキュメント

- [rtl/config_schema.md](config_schema.md) — rtl.toml スキーマ（[adapters] セクション）
- [rtl/state_machines.md](state_machines.md) — ビルドステートマシン
- [rtl/message_methods.md](message_methods.md) — rtl.* メソッド一覧
- [../fpga/vendor_adapter.md](../fpga/vendor_adapter.md) — FPGA VendorAdapter トレイト