# hal-conductor レジスタマップ定義

**対象 Conductor**: hal-conductor
**ソース**: 設計仕様書 §8（2175-2280行目付近）

## 概要

hal-conductor は SystemRDL / IP-XACT / 独自 TOML スキーマからレジスタ定義を読み込み、内部表現（RegisterMap）に変換する。この RegisterMap を基に多言語コード生成・バリデーション・差分表示を行う。

## 入力フォーマット

| フォーマット | 識別子 | 説明 |
|-------------|--------|------|
| SystemRDL | `systemrdl` | IEEE 1685 準拠のレジスタ記述言語 |
| IP-XACT | `ipxact` | IEEE 1685 XML ベースのIP 記述形式 |
| TOML | `toml` | Hestia 独自の TOML ベース定義 |

## HalToolAdapter トレイト

```rust
#[async_trait]
pub trait HalToolAdapter: Send + Sync {
    fn id(&self) -> &str;
    fn supported_inputs(&self) -> &[RegisterFormat];   // SystemRDL / IP-XACT / TOML
    fn supported_outputs(&self) -> &[OutputLang];      // C / Rust / Python / Markdown / SVD

    async fn parse(&self, src: &Path) -> Result<RegisterMap, AdapterError>;
    async fn validate(&self, map: &RegisterMap) -> Result<ValidationReport, AdapterError>;
    async fn generate(&self, map: &RegisterMap, target: OutputLang, out: &Path)
        -> Result<PathBuf, AdapterError>;
}
```

## RegisterMap モデル

レジスタマップの内部表現（`register_map.rs` で定義）。

### 主要データ構造

- **RegisterMap**: レジスタブロックのコレクション（ベースアドレス、バスプロトコル）
- **RegisterBlock**: レジスタのグループ（オフセット、サイズ）
- **Register**: 個別レジスタ（アドレス、幅、アクセス権、リセット値）
- **RegisterField**: ビットフィールド（ビット幅、オフセット、アクセス権、列挙値）

### バリデーション項目

| チェック項目 | 説明 |
|-------------|------|
| アドレス重複 | 複数レジスタのアドレス範囲が重複していないか |
| バス境界 | レジスタがバス幅（例: 32bit）の境界に配置されているか |
| 型整合性 | フィールド幅がレジスタ幅を超過していないか |
| アクセス権 | RW/RO/WO/RESERVED の組み合わせが矛盾していないか |
| メモリマップ | メモリマップ定義との整合性 |

## メモリマップ管理

`memory_map.rs` でアドレス空間を管理し、重複検出を行う。複数レジスタブロックの配置を検証し、アドレス空間の整合性を保つ。

## バスプロトコル定義

`bus_protocol.rs` でサポートするバスプロトコルを定義。

| プロトコル | 識別子 | 説明 |
|-----------|--------|------|
| AXI4-Lite | `axi4-lite` | 低スループット向け軽量 AXI |
| AXI4 | `axi4` | 高性能メモリマップドインターフェース |
| Wishbone B4 | `wishbone-b4` | オープンソース SoC バス |
| AHB-Lite | `ahb-lite` | ARM 高帯域幅バス |

## 主要アダプター

| アダプター | 役割 | 入力 | 出力 |
|-----------|------|------|------|
| `peakrdl` | SystemRDL 多言語生成 | SystemRDL | C / Markdown / HTML |
| `peakrdl-rust` | Rust ドライバ生成 | SystemRDL | Rust（embedded-hal 互換） |
| `ipyxact` | IP-XACT パース | IP-XACT XML | 内部レジスタモデル |
| `csr2regs` | CSR レジスタ生成 | TOML / YAML | C / SystemVerilog |
| `cmsis-svd-gen` | CMSIS SVD 生成 | 内部モデル | SVD XML |
| `svd2rust-bridge` | SVD → Rust crate | SVD | Rust（svd2rust 互換） |

## 関連ドキュメント

- [hal/binary_spec.md](binary_spec.md) — hestia-hal-cli バイナリ仕様
- [hal/codegen.md](codegen.md) — 多言語コード生成
- [hal/config_schema.md](config_schema.md) — hal.toml スキーマ
- [hal/state_machines.md](state_machines.md) — ビルドステートマシン