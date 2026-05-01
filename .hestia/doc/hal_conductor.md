# HAL 生成オーケストレーター

**対象領域**: hal-conductor
**ソース**: 設計仕様書 §8（2175-2280行目）

---

## 概要

`hal-conductor` は HDL（RTL）と Application Software の境界に位置する Hardware Abstraction Layer (HAL) の生成・管理を担う conductor である。SystemRDL / IP-XACT / 独自スキーマからレジスタ定義を読み込み、複数言語（C / Rust / Python）のドライバスケルトン・レジスタアクセス API・メモリマップ定義を自動生成する。`rtl-conductor` が定義したバスインターフェースと、`apps-conductor` が利用する高水準ドライバを橋渡しする。

---

## クレート構成

```
crates/hal-conductor/
├── src/
│   ├── lib.rs              # Conductor 本体・agent-cli メッセージハンドラ
│   ├── adapter.rs          # HalToolAdapter トレイト
│   ├── register_map.rs     # レジスタマップモデル（フィールド/ビット/アクセス権）
│   ├── codegen.rs          # 多言語コード生成（C / Rust / Python）
│   ├── memory_map.rs       # アドレス空間管理・重複検出
│   ├── bus_protocol.rs     # バスプロトコル定義（AXI / Wishbone / AHB）
│   └── fsm_states.rs       # ビルドステートマシン
└── Cargo.toml
```

---

## HalToolAdapter トレイト

統一インターフェース。全ての HAL ツールアダプターが実装すべきトレイトである。

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

---

## ビルドステートマシン（5状態）

```
Idle → Parsing → Validating → Generating → Reporting → Done
                          ↓ (バス境界違反 / アドレス重複 / 型不整合)
                      Failed → Diagnosing → 修正提案
```

---

## 統一プロジェクトフォーマット（hal.toml）

```toml
[project]
name = "soc_hal"
input_format = "systemrdl"             # "systemrdl" | "ipxact" | "toml"

[sources]
register_definitions = ["regs/**/*.rdl"]
memory_map = "config/memory_map.toml"

[bus]
protocol = "axi4-lite"                  # "axi4-lite" | "axi4" | "wishbone-b4" | "ahb-lite"
data_width = 32
addr_width = 32

[outputs]
c_header = "build/hal/inc/soc_hal.h"
rust_crate = "build/hal/rust/soc-hal"
python_module = "build/hal/python/soc_hal.py"
documentation = "build/hal/docs/registers.md"
svd = "build/hal/svd/soc_hal.svd"
```

---

## 主要アダプター

| アダプター | 役割 | 入力 | 出力 |
|-----------|------|------|------|
| `peakrdl` | SystemRDL 多言語生成 | SystemRDL | C / Markdown / HTML |
| `peakrdl-rust` | Rust ドライバ生成 | SystemRDL | Rust（embedded-hal 互換）|
| `ipyxact` | IP-XACT パース | IP-XACT XML | 内部レジスタモデル |
| `csr2regs` | CSR レジスタ生成 | TOML / YAML | C / SystemVerilog |
| `cmsis-svd-gen` | CMSIS SVD 生成 | 内部モデル | SVD XML |
| `svd2rust-bridge` | SVD → Rust crate | SVD | Rust（svd2rust 互換）|

---

## 上流・下流連携

- **上流（rtl-conductor）**: rtl-conductor が定義したバスインターフェース宣言を入力に取り、レジスタマップを SystemRDL 形式でエクスポート可能。`hal.handoff` イベントで rtl-conductor からトリガーされる
- **下流（apps-conductor）**: 生成された C ヘッダ / Rust crate / Python モジュールを apps-conductor の `[hal] import = "..."` で取り込む
- **横断（debug-conductor）**: 同一レジスタマップを debug-conductor が再利用し、ライブデバッグ時のレジスタ表示・編集 UI に活用
- **横断（asic-conductor / fpga-conductor）**: レジスタブロックの SystemVerilog テンプレート出力を、対応する conductor の `[sources]` に直接渡せる

---

## 公開 method

`hal.parse.v1` / `hal.validate.v1` / `hal.generate.v1` / `hal.export.v1`（rtl/asic 向けエクスポート）/ `hal.diff.v1`（レジスタマップ差分）の5系統。agent-cli IPC 上の構造化 JSON ペイロードとして送信される。

---

## サブエージェント構成

hal-conductor は **planner / designer / coder（複数）/ validator** の4種類のサブエージェントを持ち、レジスタマップから多言語ドライバ生成フローを分担する。各サブエージェントは独立した agent-cli プロセスとして起動され、`agent-cli send <peer>` IPC で hal-conductor 本体（peer 名 `hal`）と協調する。

| サブエージェント | peer 名 | 役割 | 多重度 |
|----------------|---------|------|-------|
| **planner** | `hal-planner` | HAL 生成プランニング（レジスタブロック分割、バスプロトコル選定、出力言語決定）| 1 |
| **designer** | `hal-designer` | HAL 詳細仕様（レジスタフィールド、アクセス権、メモリマップ、SystemRDL/IP-XACT スキーマ）| 1 |
| **coder** | `hal-coder-{lang}` | 言語ごとのドライバコード生成（`hal-coder-c` / `hal-coder-rust` / `hal-coder-python` / `hal-coder-svd`）| **N**（出力言語数だけ並列起動）|
| **validator** | `hal-validator` | レジスタマップ検証（アドレス重複 / 型整合性 / バス境界チェック / プロトコル準拠）| 1 |

**フロー**: planner → designer → coder（言語ごとに並列）→ validator の順次実行。出力言語が C / Rust / Python の3つなら coder は3並列起動。

---

## 関連ドキュメント

- [master_agent_design.md](master_agent_design.md) — ai-conductor 詳細設計
- [rtl_conductor.md](rtl_conductor.md) — RTL 設計フローオーケストレーター（上流）
- [apps_conductor.md](apps_conductor.md) — アプリケーションソフトウェア開発オーケストレーター（下流）
- [fpga_conductor.md](fpga_conductor.md) — FPGA 設計フローオーケストレーター（横断）
- [asic_conductor.md](asic_conductor.md) — ASIC 設計フローオーケストレーター（横断）
- [debug_conductor.md](debug_conductor.md) — デバッグ環境オーケストレーター（横断）