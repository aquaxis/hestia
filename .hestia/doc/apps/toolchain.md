# apps-conductor 主要ツールチェーンアダプター

**対象 Conductor**: apps-conductor
**ソース**: 設計仕様書 §9.5（2356-2369行目付近）

## AppsToolAdapter トレイト

```rust
#[async_trait]
pub trait AppsToolAdapter: Send + Sync {
    fn id(&self) -> &str;
    fn target_arch(&self) -> &[TargetArch];     // arm-cortex-m / riscv32imac / xtensa-esp32
    fn supported_languages(&self) -> &[AppLanguage];  // C / C++ / Rust

    async fn build(&self, project: &AppProject) -> Result<Artifact, AdapterError>;
    async fn flash(&self, artifact: &Artifact, target: &Target) -> Result<(), AdapterError>;
    async fn test(&self, project: &AppProject, mode: TestMode) -> Result<TestReport, AdapterError>;
    async fn size_report(&self, artifact: &Artifact) -> Result<SizeReport, AdapterError>;
}
```

## 主要アダプター一覧

### クロスコンパイラ

| アダプター | 役割 | 対象言語 | 対象アーキテクチャ |
|----------|------|---------|------------------|
| `arm-gcc` | ARM Cortex-M ビルド | C / C++ | arm-cortex-m |
| `riscv-gcc` | RISC-V ビルド | C / C++ | riscv32imac |
| `cargo-embed` | Rust 組み込みビルド | Rust | arm / riscv |
| `cargo-binutils` | バイナリサイズ解析 | Rust | 全アーキテクチャ |

### RTOS ビルド

| アダプター | 役割 | 対象言語 | 対象 RTOS |
|----------|------|---------|----------|
| `west-zephyr` | Zephyr RTOS ビルド | C | Zephyr |
| `freertos-builder` | FreeRTOS 統合 | C / C++ | FreeRTOS |
| `embassy-builder` | embassy-rs（async Rust）ビルド | Rust | Embassy |

### テスト・デバッグ

| アダプター | 役割 | 対象言語 | 説明 |
|----------|------|---------|------|
| `qemu-system` | QEMU SIL テスト | C / C++ / Rust | エミュレーションベーステスト |
| `probe-rs` | フラッシュ書込 / RTT ログ | Rust | Rust エコシステムのプローブツール |
| `openocd-bridge` | OpenOCD 連携 | C / C++ | debug-conductor 経由で使用 |

## クレート構成

```
crates/apps-conductor/
├── src/
│   ├── lib.rs              # Conductor 本体
│   ├── adapter.rs          # AppsToolAdapter トレイト
│   ├── toolchain.rs        # クロスコンパイラ管理
│   ├── rtos.rs             # RTOS 統合
│   ├── linker.rs           # リンカスクリプト管理
│   ├── target.rs           # ターゲット定義
│   ├── hil.rs              # HIL / SIL テスト統合
│   └── fsm_states.rs       # ビルドステートマシン
```

## TargetArch 対応一覧

| アーキテクチャ | 識別子 | 説明 |
|-------------|--------|------|
| ARM Cortex-M | `arm-cortex-m` | STM32 / nRF / LPC 等 |
| RISC-V 32bit | `riscv32imac` | SiFive / ESP32-C 等 |
| Xtensa ESP32 | `xtensa-esp32` | ESP32 / ESP32-S2 / S3 |

## 関連ドキュメント

- [apps/binary_spec.md](binary_spec.md) — hestia-apps-cli バイナリ仕様
- [apps/rtos.md](rtos.md) — RTOS サポート
- [apps/hil_sil.md](hil_sil.md) — HIL/SIL テスト
- [apps/config_schema.md](config_schema.md) — apps.toml スキーマ