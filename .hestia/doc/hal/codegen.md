# hal-conductor 多言語コード生成

**対象 Conductor**: hal-conductor
**ソース**: 設計仕様書 §8（2175-2280行目付近）

## 概要

hal-conductor は RegisterMap から複数言語のドライバスケルトン・レジスタアクセス API・メモリマップ定義を自動生成する。`codegen.rs` に実装される。

## 対応出力言語

| 出力言語 | 識別子 | 説明 |
|---------|--------|------|
| C | `c` | C ヘッダファイル（レジスタアクセスマクロ・構造体定義） |
| Rust | `rust` | Rust crate（embedded-hal 互換のドライバ） |
| Python | `python` | Python モジュール（MMIO アクセスラッパー） |
| Markdown | `markdown` | レジスタドキュメント |
| SVD | `svd` | CMSIS SVD XML（デバッガ・IDE 連携用） |

## C ヘッダ生成

生成内容:
- レジスタベースアドレスマクロ（`#define SOC_BASE 0x10000000`）
- レジスタオフセットマクロ（`#define REG_CTRL_OFFSET 0x00`）
- レジスタ構造体定義（ビットフィールド対応）
- 読み書きヘルパーマクロ（`REG_READ` / `REG_WRITE`）

出力パス: hal.toml `[outputs] c_header` で指定

## Rust Crate 生成

生成内容:
- `embedded-hal` トレイト互換のドライバ構造体
- MMIO レジスタアクセス（`read()` / `write()` / `modify()`）
- 型安全なビットフィールド操作
- PAC（Peripheral Access Crate）形式

出力パス: hal.toml `[outputs] rust_crate` で指定

関連アダプター:
- `peakrdl-rust`: SystemRDL → Rust（embedded-hal 互換）
- `svd2rust-bridge`: SVD → Rust（svd2rust 互換）

## Python モジュール生成

生成内容:
- MMIO アクセスクラス（`/dev/mem` または UIO 経由）
- レジスタフィールドプロパティ（`@property` デコレータ）
- 列挙型マッピング

出力パス: hal.toml `[outputs] python_module` で指定

## SVD 生成

CMSIS SVD（System View Description）XML 形式。デバッガ・IDE がレジスタ情報を表示するために使用する。

生成内容:
- `<peripheral>` エレメント（ベースアドレス・サイズ）
- `<register>` エレメント（オフセット・アクセス権・リセット値）
- `<field>` エレメント（ビット幅・オフセット・列挙値）

出力パス: hal.toml `[outputs] svd` で指定

関連アダプター:
- `cmsis-svd-gen`: 内部モデル → SVD XML

## Markdown ドキュメント生成

生成内容:
- レジスタブロック概要
- レジスタテーブル（アドレス・名前・アクセス権・リセット値）
- ビットフィールド図

出力パス: hal.toml `[outputs] documentation` で指定

## 並列コード生成

出力言語が複数ある場合、coder サブエージェント（`hal-coder-c` / `hal-coder-rust` / `hal-coder-python` / `hal-coder-svd`）が言語ごとに並列起動し、同時にコード生成を行う。

## 下流連携

### apps-conductor（§9）

生成された C ヘッダ / Rust crate / Python モジュールは apps-conductor の `[hal] import = "..."` で取り込まれる。

### debug-conductor（§10）

SVD ファイルは debug-conductor が再利用し、ライブデバッグ時のレジスタ表示・編集 UI に活用する。

### asic-conductor / fpga-conductor

`export-rtl` で出力した SystemVerilog テンプレートを、対応する conductor の `[sources]` に直接渡せる。

## 関連ドキュメント

- [hal/register_map.md](register_map.md) — レジスタマップ定義
- [hal/binary_spec.md](binary_spec.md) — hestia-hal-cli バイナリ仕様
- [hal/config_schema.md](config_schema.md) — hal.toml [outputs] セクション
- [../apps/config_schema.md](../apps/config_schema.md) — apps.toml [hal] セクション