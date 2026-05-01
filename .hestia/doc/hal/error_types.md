# hal-conductor エラー型

**対象 Conductor**: hal-conductor
**ソース**: 設計仕様書 §8（2175-2280行目付近）, §14.3（3565-3581行目付近）

## エラーカテゴリ

hal-conductor は HESTIA 共通エラーコード（-32000 〜 -32099）およびリクエスト標準エラー（-32600 〜 -32603）を使用する。HAL 固有のエラーは共通範囲内で細分化される。

## パース エラー

| エラー | 説明 |
|-------|------|
| PARSE_FORMAT_UNSUPPORTED | 未対応の入力フォーマット |
| PARSE_SYNTAX_ERROR | SystemRDL / IP-XACT / TOML 構文エラー |
| PARSE_FILE_NOT_FOUND | レジスタ定義ファイル未検出 |
| PARSE_SCHEMA_MISMATCH | スキーマバージョン不整合 |

## バリデーション エラー

| エラー | 説明 |
|-------|------|
| VALIDATION_ADDRESS_OVERLAP | アドレス重複検出 |
| VALIDATION_BUS_BOUNDARY_VIOLATION | バス境界違反（例: 32bit 境界を跨ぐレジスタ） |
| VALIDATION_TYPE_MISMATCH | 型不整合（フィールド幅 > レジスタ幅等） |
| VALIDATION_ACCESS_CONFLICT | アクセス権矛盾（書込み専用フィールドに初期値等） |
| VALIDATION_RESERVED_FIELD_WRITE | 予約フィールドへの書込み定義 |

## コード生成 エラー

| エラー | 説明 |
|-------|------|
| CODEGEN_TARGET_UNSUPPORTED | 未対応の出力言語 |
| CODEGEN_TEMPLATE_ERROR | テンプレート処理エラー |
| CODEGEN_OUTPUT_WRITE_ERROR | 出力ファイル書き込みエラー |
| CODEGEN_RUST_CRATE_BUILD_FAILED | Rust crate ビルド失敗 |

## 差分 エラー

| エラー | 説明 |
|-------|------|
| DIFF_BASELINE_NOT_FOUND | ベースラインバージョン未検出 |
| DIFF_INCOMPATIBLE_VERSIONS | 比較不可能なバージョン |

## バスプロトコル エラー

| エラー | 説明 |
|-------|------|
| BUS_PROTOCOL_UNSUPPORTED | 未対応のバスプロトコル |
| BUS_WIDTH_MISMATCH | データ/アドレス幅とレジスタ定義の不整合 |

## ビルドステートマシン エラー

ビルド中の各ステート（Parsing / Validating / Generating / Reporting）で失敗した場合、`Failed` 状態に遷移し `Diagnosing` で修正提案を生成する。

## 関連ドキュメント

- [hal/message_methods.md](message_methods.md) — hal.* メソッド一覧
- [hal/state_machines.md](state_machines.md) — ビルドステートマシン
- [hal/register_map.md](register_map.md) — レジスタマップ定義
- [../common/error_registry.md](../common/error_registry.md) — HESTIA 共通エラーレジストリ