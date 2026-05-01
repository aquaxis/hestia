# apps-conductor HIL/SIL テスト

**対象 Conductor**: apps-conductor
**ソース**: 設計仕様書 §9（2281-2400行目付近）

## テストモード概要

| モード | 説明 | 実行環境 | 速度 | 精度 |
|-------|------|---------|------|------|
| SIL | Software-in-the-Loop | QEMU エミュレーション | 高速 | 機能検証レベル |
| HIL | Hardware-in-the-Loop | 実機 + debug-conductor | 実時間 | サイクル精度 |
| QEMU | QEMU テスト専用 | QEMU 単体 | 高速 | 機能検証レベル |

## SIL（Software-in-the-Loop）テスト

QEMU エミュレーションを使用したソフトウェアレベルテスト。

### 特徴

- ハードウェア不要（QEMU で完結）
- 高速実行（実時間より速い場合もある）
- CI/CD パイプラインに組み込み可能
- 機能的正確性の検証に適する

### 対応ターゲット

| ターゲット | QEMU コマンド |
|-----------|-------------|
| ARM Cortex-M | `qemu-system-arm -machine <board>` |
| RISC-V 32bit | `qemu-system-riscv32 -machine <board>` |

### テストフロー

```
1. QEMU 起動（ファームウェア ELF をロード）
2. テストシナリオ実行（シリアル出力 / GDB 接続）
3. 結果判定（終了コード / 出力文字列 / メモリ状態）
4. テストレポート生成
```

### apps-conductor 連携

- アダプター: `qemu-system`
- apps.toml: `[test] mode = "sil"` または `mode = "qemu"`
- GDB リモートデバッグ対応

## HIL（Hardware-in-the-Loop）テスト

実機ハードウェアと debug-conductor（§10）を組み合わせたハードウェアレベルテスト。

### 特徴

- 実機ハードウェアでテスト（真の実行環境）
- サイクル精度の検証が可能
- ペリフェラル・割り込みの実動作確認
- タイミング制約の検証

### テストフロー

```
1. debug-conductor 経由でプローブ接続（ST-Link / J-Link 等）
2. ファームウェア書込（probe-rs / OpenOCD）
3. ターゲット実行・RTT ログ取得
4. テストシナリオ実行（GPIO / UART / SPI 等の実ペリフェラル操作）
5. 結果判定・テストレポート生成
```

### apps-conductor 連携

- アダプター: `probe-rs` / `openocd-bridge`
- apps.toml: `[test] mode = "hil"` / `probe = "stlink-v3"`
- debug-conductor（§10）と連携してデバッグセッション管理

## クロステスト（QEMU + Cycle-Accurate Co-Simulation）

QEMU とサイクル精度シミュレータを組み合わせたハイブリッドテスト。機能検証（QEMU）とタイミング検証（サイクル精度）を同時に実施。

## テストレポート

| 項目 | 説明 |
|------|------|
| テスト名 | テストケース識別子 |
| 結果 | PASS / FAIL / SKIP |
| 実行時間 | テスト実行所要時間 |
| カバレッジ | コードカバレッジ（gcov / llvm-cov） |
| ログ | RTT ログ / シリアル出力 |
| メモリ使用量 | スタック使用量・ヒープ使用量 |

## CI/CD 統合

SIL テストは共有サービス層 / CI/CD API（§13）経由で GitHub Actions / GitLab CI 上で自動実行可能。HIL テストは実機接続が必要なため、ローカルまたは専用 CI ランナーで実行する。

## 関連ドキュメント

- [apps/binary_spec.md](binary_spec.md) — hestia-apps-cli バイナリ仕様
- [apps/state_machines.md](state_machines.md) — ビルドステートマシン
- [apps/toolchain.md](toolchain.md) — 主要アダプター
- [apps/rtos.md](rtos.md) — RTOS サポート
- [../debug/binary_spec.md](../debug/binary_spec.md) — debug-conductor CLI