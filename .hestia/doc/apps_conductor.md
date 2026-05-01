# アプリケーションソフトウェア開発オーケストレーター

**対象領域**: apps-conductor
**ソース**: 設計仕様書 §9（2281-2400行目）

---

## 概要

`apps-conductor` はターゲットハードウェア上で動作するアプリケーションソフトウェア（ファームウェア / RTOS アプリ / ベアメタルアプリケーション）の開発フローを抽象化する conductor である。クロスコンパイラ・RTOS・リンカスクリプト・HIL（Hardware-in-the-Loop）/ SIL（Software-in-the-Loop）テストを統合し、`hal-conductor` が生成したドライバを活用してアプリケーションロジックをビルド・検証する。

---

## クレート構成

```
crates/apps-conductor/
├── src/
│   ├── lib.rs              # Conductor 本体・agent-cli メッセージハンドラ
│   ├── adapter.rs          # AppsToolAdapter トレイト
│   ├── toolchain.rs        # クロスコンパイラ管理（gcc-arm / riscv-gnu / llvm）
│   ├── rtos.rs             # RTOS 統合（FreeRTOS / Zephyr / embassy-rs）
│   ├── linker.rs           # リンカスクリプト管理
│   ├── target.rs           # ターゲット定義（CPU / メモリレイアウト）
│   ├── hil.rs              # HIL / SIL テスト統合
│   └── fsm_states.rs       # ビルドステートマシン
└── Cargo.toml
```

---

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

---

## ビルドステートマシン（8状態）

```
Idle → Resolving → Compiling → Linking → SizeChecking → Flashing → Testing → Done
                                       ↓ (メモリ超過 / リンクエラー / テスト失敗)
                                   Failed → Diagnosing → 修正提案
```

---

## 統一プロジェクトフォーマット（apps.toml）

```toml
[project]
name = "sensor_node_fw"
language = "rust"                       # "c" | "cpp" | "rust"
target = "thumbv7em-none-eabihf"

[toolchain]
compiler = "arm-none-eabi-gcc"          # or "cargo" for Rust
version = "14.2.1"

[rtos]
kernel = "embassy-rs"                   # "freertos" | "zephyr" | "embassy-rs" | "bare-metal"
version = "0.4"

[memory]
flash_origin  = 0x08000000
flash_length  = "256K"
ram_origin    = 0x20000000
ram_length    = "64K"
linker_script = "memory.x"

[hal]
import = "build/hal/rust/soc-hal"       # hal-conductor の生成物を import

[test]
mode  = "hil"                           # "sil" | "hil" | "qemu"
probe = "stlink-v3"                     # debug-conductor が提供
```

---

## 主要アダプター

| アダプター | 役割 | 対象 |
|----------|------|------|
| `arm-gcc` | ARM Cortex-M ビルド | C / C++ |
| `riscv-gcc` | RISC-V ビルド | C / C++ |
| `cargo-embed` | Rust 組み込みビルド | Rust |
| `cargo-binutils` | バイナリサイズ解析 | Rust |
| `west-zephyr` | Zephyr RTOS ビルド | C |
| `freertos-builder` | FreeRTOS 統合 | C / C++ |
| `embassy-builder` | embassy-rs（async Rust）| Rust |
| `qemu-system` | QEMU SIL テスト | C / C++ / Rust |
| `probe-rs` | フラッシュ書込 / RTT ログ | Rust |
| `openocd-bridge` | OpenOCD 連携（debug-conductor 経由）| C / C++ |

---

## 上流・下流連携

- **上流（hal-conductor）**: hal-conductor が生成した HAL モジュール（C ヘッダ / Rust crate / Python モジュール）を `[hal] import = "..."` で取り込む
- **横断（debug-conductor）**: フラッシュ書込 / RTT ログ / GDB セッションを debug-conductor 経由で実施
- **テスト**: SIL モード（QEMU）/ HIL モード（実機 + debug-conductor）/ クロステスト（QEMU + cycle-accurate co-sim）の3モードをサポート
- **CI/CD**: 共有サービス層 / CI/CD API 経由で GitHub Actions / GitLab CI 上で再現性ビルドを実施
- **ai-conductor 連携**: ai-conductor がアプリケーション仕様書から apps.toml の自動生成を支援（Spec-Driven）

---

## 公開 method

`apps.build.v1` / `apps.flash.v1` / `apps.test.v1` / `apps.size.v1` / `apps.debug.v1`（debug-conductor との bridging）の5系統。agent-cli IPC 上の構造化 JSON ペイロードとして送信される。

---

## サブエージェント構成

apps-conductor は **planner / designer / coder（複数）/ builder / tester** の5種類のサブエージェントを持ち、機能モジュール単位に複数の coder を並列割当することでアプリケーション開発を効率化する。各サブエージェントは独立した agent-cli プロセスとして起動され、`agent-cli send <peer>` IPC で apps-conductor 本体（peer 名 `apps`）と協調する。

| サブエージェント | peer 名 | 役割 | 多重度 |
|----------------|---------|------|-------|
| **planner** | `apps-planner` | アプリ開発プランニング（RTOS 選定、メモリレイアウト、タスク分割、HAL 取り込み戦略）| 1 |
| **designer** | `apps-designer` | アプリ詳細仕様（タスク構成、IPC プロトコル、状態遷移、エラー処理ポリシー）| 1 |
| **coder** | `apps-coder-{module}` | 機能モジュール単位のアプリケーションコード（C / C++ / Rust）実装 | **N**（モジュール数だけ動的並列起動、最大16）|
| **builder** | `apps-builder` | クロスコンパイル / リンカスクリプト適用 / バイナリサイズ最適化 | 1 |
| **tester** | `apps-tester` | SIL（QEMU）/ HIL（実機 + debug-conductor）/ 単体テスト実行 + カバレッジ集計 | 1 |

**フロー**: planner → designer → coder（モジュール並列）→ builder → tester の順次実行。大規模アプリ（モジュール数 >= 4）では coder を最大16並列まで自動拡張。

---

## 関連ドキュメント

- [master_agent_design.md](master_agent_design.md) — ai-conductor 詳細設計
- [hal_conductor.md](hal_conductor.md) — HAL 生成オーケストレーター（上流）
- [debug_conductor.md](debug_conductor.md) — デバッグ環境オーケストレーター（横断）
- [rtl_conductor.md](rtl_conductor.md) — RTL 設計フローオーケストレーター