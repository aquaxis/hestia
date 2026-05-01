# apps-conductor エラー型

**対象 Conductor**: apps-conductor
**ソース**: 設計仕様書 §9（2281-2400行目付近）, §14.3（3565-3581行目付近）

## エラーカテゴリ

apps-conductor は HESTIA 共通エラーコード（-32000 〜 -32099）およびリクエスト標準エラー（-32600 〜 -32603）を使用する。apps 固有のエラーは共通範囲内で細分化される。

## ビルド エラー

| エラー | 説明 |
|-------|------|
| CROSS_COMPILATION_FAILED | クロスコンパイル失敗（arm-gcc / riscv-gcc / cargo） |
| TOOLCHAIN_NOT_FOUND | 指定されたツールチェーン未検出 |
| TOOLCHAIN_VERSION_MISMATCH | ツールチェーンバージョン不整合 |
| COMPILATION_ERROR | ソースコードコンパイルエラー |
| LINK_ERROR | リンクエラー（未解決シンボル・重複定義等） |

## メモリ エラー

| エラー | 説明 |
|-------|------|
| MEMORY_OVERFLOW_FLASH | Flash 領域オーバーフロー |
| MEMORY_OVERFLOW_RAM | RAM 領域オーバーフロー |
| LINKER_SCRIPT_ERROR | リンカスクリプトエラー |
| SIZE_CHECK_FAILED | バイナリサイズチェック失敗 |

## RTOS エラー

| エラー | 説明 |
|-------|------|
| RTOS_NOT_FOUND | 指定された RTOS が未インストール |
| RTOS_VERSION_INCOMPATIBLE | RTOS バージョン非互換 |
| FREERTOS_CONFIG_ERROR | FreeRTOS 設定エラー |
| ZEPHYR_WEST_FAILED | Zephyr west コマンド失敗 |

## フラッシュ エラー

| エラー | 説明 |
|-------|------|
| FLASH_FAILED | フラッシュ書込失敗 |
| PROBE_NOT_FOUND | デバッグプローブ未検出 |
| PROBE_CONNECTION_ERROR | プローブ接続エラー |
| TARGET_NOT_RESPONDING | ターゲットデバイス応答なし |

## テスト エラー

| エラー | 説明 |
|-------|------|
| TEST_FAILED | テスト実行失敗 |
| QEMU_LAUNCH_FAILED | QEMU 起動失敗 |
| HIL_CONNECTION_FAILED | HIL テスト接続失敗 |
| TEST_TIMEOUT | テストタイムアウト |

## HAL 連携 エラー

| エラー | 説明 |
|-------|------|
| HAL_IMPORT_NOT_FOUND | HAL モジュール import 先未検出 |
| HAL_VERSION_MISMATCH | HAL バージョン不整合 |

## ビルドステートマシン エラー

ビルド中の各ステート（Resolving / Compiling / Linking / SizeChecking / Flashing / Testing）で失敗した場合、`Failed` 状態に遷移し `Diagnosing` で修正提案を生成する。

## 関連ドキュメント

- [apps/message_methods.md](message_methods.md) — apps.* メソッド一覧
- [apps/state_machines.md](state_machines.md) — ビルドステートマシン
- [apps/toolchain.md](toolchain.md) — 主要アダプター
- [../common/error_registry.md](../common/error_registry.md) — HESTIA 共通エラーレジストリ