# apps-conductor RTOS サポート

**対象 Conductor**: apps-conductor
**ソース**: 設計仕様書 §9（2281-2400行目付近）

## サポート RTOS 一覧

| RTOS | 識別子 | 言語 | ビルドツール | 説明 |
|------|--------|------|------------|------|
| Zephyr | `zephyr` | C | west | Linux Foundation 主導のスケーラブル RTOS |
| FreeRTOS | `freertos` | C / C++ | CMake / Make | AWS 主導のリアルタイムカーネル |
| Embassy | `embassy-rs` | Rust | cargo | async/await ベースの Rust 組み込みフレームワーク |
| Bare Metal | `bare-metal` | — | — | RTOS なしのベアメタル実行 |

## Zephyr / west

### 特徴

- スケーラブル：MCU から MPU まで対応
- west メタツールによるマルチリポジトリ管理
- Device Tree によるハードウェア記述
- KConfig による機能選択

### ビルドフロー

```
west init → west update → west build -b <board> → west flash
```

### apps-conductor 連携

- アダプター: `west-zephyr`
- apps.toml: `[rtos] kernel = "zephyr"`
- west.yml は apps-conductor が管理・自動生成可能

## FreeRTOS

### 特徴

- AWS が主導する広く使われる RTOS
- タスク・キュー・セマフォ・ミューテックスの基本カーネル機能
- 多数のデモ・ポート定義

### ビルドフロー

```
CMake / Make によるビルド → .elf / .bin 生成
```

### apps-conductor 連携

- アダプター: `freertos-builder`
- apps.toml: `[rtos] kernel = "freertos"`
- FreeRTOSConfig.h は apps-conductor が [memory] セクションから自動生成可能

## Embassy (embassy-rs)

### 特徴

- Rust の async/await を活用した非同期組み込みフレームワーク
- ゼロコスト抽象化: コンパイル時タスクディスパッチ
- embedded-hal 互換ドライバエコシステム
- probe-rs によるフラッシュ・デバッグ統合

### ビルドフロー

```
cargo build --target <target> → probe-rs flash / cargo-embed
```

### apps-conductor 連携

- アダプター: `embassy-builder` / `cargo-embed`
- apps.toml: `[rtos] kernel = "embassy-rs"`
- Cargo.toml の依存関係管理は apps-conductor が支援

## RTOS 選定フロー

ai-conductor（§3）は、アプリケーション仕様書から最適な RTOS を提案する:

| 条件 | 推奨 RTOS |
|------|----------|
| Rust 言語指定 | Embassy |
| マルチプラットフォーム対応 | Zephyr |
| 軽量・最小フットプリント | FreeRTOS |
| リアルタイム性重視 | FreeRTOS / Zephyr |
| Rust async 活用 | Embassy |

## 関連ドキュメント

- [apps/binary_spec.md](binary_spec.md) — hestia-apps-cli バイナリ仕様
- [apps/toolchain.md](toolchain.md) — 主要アダプター
- [apps/config_schema.md](config_schema.md) — apps.toml [rtos] セクション
- [apps/hil_sil.md](hil_sil.md) — HIL/SIL テスト