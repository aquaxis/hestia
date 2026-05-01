# apps-conductor ビルドステートマシン

**対象 Conductor**: apps-conductor
**ソース**: 設計仕様書 §9.3（2317-2323行目付近）

## 8 状態ビルドステートマシン

```
Idle → Resolving → Compiling → Linking → SizeChecking → Flashing → Testing → Done
                                       ↓ (メモリ超過 / リンクエラー / テスト失敗)
                                   Failed → Diagnosing → 修正提案
```

## 状態定義

| 状態 | 説明 | 主要処理 |
|------|------|---------|
| Idle | 初期状態 | — |
| Resolving | ツールチェーン・RTOS・HAL 依存関係解決中 | バージョン確認・パス検証 |
| Compiling | クロスコンパイル実行中 | arm-gcc / riscv-gcc / cargo によるコンパイル |
| Linking | リンク実行中 | リンカスクリプト適用・バイナリ生成 |
| SizeChecking | バイナリサイズチェック中 | Flash/RAM 使用量確認・制限内判定 |
| Flashing | フラッシュ書込中 | probe-rs / OpenOCD 経由でデバイスに書込 |
| Testing | テスト実行中 | SIL（QEMU）/ HIL（実機）/ 単体テスト |
| Done | 正常完了 | テストレポート・カバレッジ出力 |
| Failed | エラー発生 | エラー詳細 + 修正提案 |
| Diagnosing | 原因分析中 | 修正提案（メモリ最適化・リンク設定変更等） |

## 状態遷移ルール

| 遷移 | トリガー | 条件 |
|------|---------|------|
| Idle → Resolving | ビルド開始 | — |
| Resolving → Compiling | 依存関係解決完了 | 全ツールチェーン利用可能 |
| Resolving → Failed | 依存関係解決失敗 | ツールチェーン未検出等 |
| Compiling → Linking | コンパイル完了 | コンパイル成功 |
| Compiling → Failed | コンパイル失敗 | — |
| Linking → SizeChecking | リンク完了 | リンク成功 |
| Linking → Failed | リンク失敗 | 未解決シンボル等 |
| SizeChecking → Flashing | サイズチェック通過 | Flash/RAM 使用量制限内 |
| SizeChecking → Failed | サイズチェック失敗 | メモリオーバーフロー |
| Flashing → Testing | フラッシュ書込完了 | — |
| Flashing → Failed | フラッシュ失敗 | プローブ接続エラー等 |
| Testing → Done | テスト完了 | 全テスト通過（または警告のみ） |
| Testing → Failed | テスト失敗 | テスト不合格 |
| Failed → Diagnosing | 診断開始 | — |

## テストモード

| モード | 説明 | 実行環境 |
|-------|------|---------|
| SIL | Software-in-the-Loop | QEMU エミュレーション |
| HIL | Hardware-in-the-Loop | 実機 + debug-conductor（§10） |
| QEMU | QEMU テスト専用 | QEMU 単体 |

## 関連ドキュメント

- [apps/binary_spec.md](binary_spec.md) — hestia-apps-cli バイナリ仕様
- [apps/error_types.md](error_types.md) — apps 固有エラー型
- [apps/toolchain.md](toolchain.md) — 主要アダプター
- [apps/hil_sil.md](hil_sil.md) — HIL/SIL テスト