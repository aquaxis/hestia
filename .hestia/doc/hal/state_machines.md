# hal-conductor ビルドステートマシン

**対象 Conductor**: hal-conductor
**ソース**: 設計仕様書 §8.3（2210-2216行目付近）

## 5 状態ビルドステートマシン

```
Idle → Parsing → Validating → Generating → Reporting → Done
                          ↓ (バス境界違反 / アドレス重複 / 型不整合)
                      Failed → Diagnosing → 修正提案
```

## 状態定義

| 状態 | 説明 | 入力 | 出力 |
|------|------|------|------|
| Idle | 初期状態 | ビルド開始コマンド | — |
| Parsing | レジスタ定義ファイルのパース中 | SystemRDL / IP-XACT / TOML | RegisterMap |
| Validating | レジスタマップのバリデーション中 | RegisterMap | ValidationReport |
| Generating | 多言語コード生成中 | RegisterMap + 出力言語指定 | C ヘッダ / Rust crate / Python モジュール / SVD |
| Reporting | 結果レポート生成中 | 各ステップ結果 | 統合レポート |
| Done | 正常完了 | — | 全成果物出力 |
| Failed | エラー発生 | エラー情報 | エラー詳細 + 修正提案 |
| Diagnosing | 原因分析中 | エラー情報 | 修正提案 |

## 状態遷移ルール

| 遷移 | トリガー | 条件 |
|------|---------|------|
| Idle → Parsing | パース開始 | — |
| Parsing → Validating | パース完了 | 構文エラーなし |
| Parsing → Failed | パース失敗 | 構文エラー検出 |
| Validating → Generating | バリデーション完了 | アドレス重複・型不整合・バス境界違反なし |
| Validating → Failed | バリデーション失敗 | 違反検出 |
| Generating → Reporting | 生成完了 | 全出力言語の生成成功 |
| Generating → Failed | 生成失敗 | 一部出力の生成エラー |
| Reporting → Done | レポート完了 | — |
| Failed → Diagnosing | 診断開始 | — |

## 失敗パターン

| 失敗ステート | 原因 | 修正提案 |
|-------------|------|---------|
| Parsing 失敗 | 構文エラー | エラー位置の特定・修正案提示 |
| Validating 失敗 | バス境界違反 | アドレス整列の調整案 |
| Validating 失敗 | アドレス重複 | 重複レジスタの統合・アドレス再割当案 |
| Validating 失敗 | 型不整合 | フィールド幅の調整案 |
| Generating 失敗 | テンプレートエラー | テンプレート修正案 |

## 多言語並列生成

出力言語が C / Rust / Python / SVD の複数ある場合、Generating ステップでは各言語の生成を並列実行可能（coder サブエージェントが言語ごとに並列起動）。

## 関連ドキュメント

- [hal/binary_spec.md](binary_spec.md) — hestia-hal-cli バイナリ仕様
- [hal/error_types.md](error_types.md) — HAL 固有エラー型
- [hal/register_map.md](register_map.md) — レジスタマップ定義
- [hal/codegen.md](codegen.md) — 多言語コード生成