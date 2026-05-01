# rtl-conductor エラー型

**対象 Conductor**: rtl-conductor
**ソース**: 設計仕様書 §14.3（3565-3581行目付近）, §4（1241-1397行目付近）

## エラーカテゴリ

rtl-conductor は HESTIA 共通エラーコード（-32000 〜 -32099）およびリクエスト標準エラー（-32600 〜 -32603）を使用する。RTL 固有のエラーは共通範囲内で細分化される。

## Lint エラー

| エラー | 説明 |
|-------|------|
| LINT_ADAPTER_NOT_FOUND | 指定された Lint アダプターが未登録 |
| LINT_EXECUTION_FAILED | Lint ツール実行失敗（Verilator/Verible 等のプロセスエラー） |
| LINT_PARSE_ERROR | Lint 出力のパース失敗 |
| LINT_VIOLATIONS_FOUND | Lint 違反検出（警告・エラー） |

## シミュレーション エラー

| エラー | 説明 |
|-------|------|
| SIM_ADAPTER_NOT_FOUND | 指定されたシミュレーションアダプターが未登録 |
| SIM_COMPILATION_FAILED | テストベンチ / RTL コンパイル失敗 |
| SIM_RUNTIME_ERROR | シミュレーション実行時エラー（アサーション失敗等） |
| SIM_TIMEOUT | シミュレーション実行タイムアウト |
| SIM_TESTBENCH_NOT_FOUND | 指定されたテストベンチが存在しない |

## 形式検証 エラー

| エラー | 説明 |
|-------|------|
| FORMAL_ADAPTER_NOT_FOUND | 形式検証アダプター未登録 |
| FORMAL_PROOF_FAILED | 形式検証プロパティの証明失敗 |
| FORMAL_TIMEOUT | 形式検証タイムアウト |
| FORMAL_PROPERTY_INVALID | プロパティ定義不正 |

## トランスパイル エラー

| エラー | 説明 |
|-------|------|
| TRANSPILE_UNSUPPORTED_LANGUAGE | 未対応のソース/ターゲット言語 |
| TRANSPILE_COMPILATION_FAILED | トランスパイル元のコンパイル失敗 |
| TRANSPILE_OUTPUT_ERROR | トランスパイル結果の出力エラー |

## ハンドオフ エラー

| エラー | 説明 |
|-------|------|
| HANDOFF_TARGET_UNKNOWN | 不明なハンドオフ先（fpga/asic/hal 以外） |
| HANDOFF_ARTIFACT_MISSING | 指定された成果物が存在しない |
| HANDOFF_DOWNSTREAM_UNREACHABLE | 下流 conductor が到達不可 |

## ビルドステートマシン エラー

ビルド中の各ステート（Linting / Compiling / Simulating / FormalChecking / Reporting）で失敗した場合、`Failed` 状態に遷移し `Diagnosing` で修正提案を生成する。

## 共通エラーコード参照

| 範囲 | 領域 |
|------|------|
| -32700 | Parse Error |
| -32600 〜 -32603 | リクエスト標準エラー |
| -32000 〜 -32099 | HESTIA 共通（Timeout / NotFound / AlreadyExists / PermissionDenied / InvalidState 等） |

## 関連ドキュメント

- [rtl/message_methods.md](message_methods.md) — rtl.* メソッド一覧
- [rtl/state_machines.md](state_machines.md) — ビルドステートマシン
- [rtl/rtl_tool_adapter.md](rtl_tool_adapter.md) — RtlToolAdapter トレイト
- [../common/error_registry.md](../common/error_registry.md) — HESTIA 共通エラーレジストリ