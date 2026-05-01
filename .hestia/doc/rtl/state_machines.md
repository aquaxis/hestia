# rtl-conductor ビルドステートマシン

**対象 Conductor**: rtl-conductor
**ソース**: 設計仕様書 §4.3（1280-1286行目付近）

## 7 状態ビルドステートマシン

```
Idle → Resolving → Linting → Compiling → Simulating → FormalChecking → Reporting → Done
                                                  ↓ (失敗時)
                                              Failed → Diagnosing → 修正提案
```

## 状態定義

| 状態 | 説明 | 入力 | 出力 |
|------|------|------|------|
| Idle | 初期状態 | ビルド開始コマンド | — |
| Resolving | 依存関係解決中（アダプター選択、ツールチェーン確認） | rtl.toml | 解決済みアダプター情報 |
| Linting | Lint / フォーマット / 静的解析実行中 | HDL ソース | LintReport（警告・エラー） |
| Compiling | コンパイル中（シミュレーション用ビルド等） | HDL ソース + テストベンチ | コンパイル済みシミュレーションモデル |
| Simulating | シミュレーション実行中 | テストベンチ + コンパイル済みモデル | SimReport（パス/フェイル、カバレッジ） |
| FormalChecking | 形式検証実行中 | プロパティ定義 | FormalReport（証明結果） |
| Reporting | 結果集約・レポート生成中 | 各ステップ結果 | 統合レポート |
| Done | 正常完了 | — | 全レポート出力 |
| Failed | エラー発生 | エラー情報 | エラー詳細 + 修正提案 |
| Diagnosing | 原因分析中 | エラー情報 | 修正提案（パッチ / 設定変更案） |

## 状態遷移ルール

| 遷移 | トリガー | 条件 |
|------|---------|------|
| Idle → Resolving | ビルド開始コマンド受領 | — |
| Resolving → Linting | 依存関係解決完了 | 全アダプター利用可能 |
| Linting → Compiling | Lint 完了 | 致命的 Lint エラーなし |
| Linting → Failed | Lint 失敗 | 致命的エラー検出 |
| Compiling → Simulating | コンパイル完了 | コンパイル成功 |
| Compiling → Failed | コンパイル失敗 | — |
| Simulating → FormalChecking | シミュレーション完了 | シミュレーション成功または警告のみ |
| Simulating → Failed | シミュレーション失敗 | アサーション違反等 |
| FormalChecking → Reporting | 形式検証完了 | — |
| Reporting → Done | レポート生成完了 | — |
| Failed → Diagnosing | 診断開始 | — |

## 並列実行

Simulating と FormalChecking は互いに依存しないため、並列実行が可能。ただし、設計仕様書では順次実行（Simulating → FormalChecking）として定義されている。

## 関連ドキュメント

- [rtl/binary_spec.md](binary_spec.md) — hestia-rtl-cli バイナリ仕様
- [rtl/error_types.md](error_types.md) — RTL 固有エラー型
- [rtl/rtl_tool_adapter.md](rtl_tool_adapter.md) — RtlToolAdapter トレイト
- [rtl/handoff.md](handoff.md) — 下流連携ハンドオフ