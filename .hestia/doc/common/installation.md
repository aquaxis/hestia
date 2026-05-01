# ビルド手順

**対象領域**: common — インストール
**ソース**: 設計仕様書 §15.4

## 概要

HESTIA の Rust ワークスペースのビルド・テスト手順を定める。全バイナリは `cargo build --release` で一括ビルド可能。

## 前提条件

| 要件 | バージョン |
|------|----------|
| Rust ツールチェーン | stable（最新推奨）|
| Cargo | Rust 同梱 |
| Linux ホスト OS | カーネル 5.x 以降推奨 |
| Podman | コンテナ実行時のみ |

## ビルド手順

### 全バイナリ一括ビルド

```bash
cd .hestia/tools
cargo build --release
```

9 conductor + 10 CLI の計 19 バイナリが生成される。

### 特定 conductor のみビルド

```bash
cargo build --release -p hestia-ai-conductor
cargo build --release -p hestia-rtl-conductor
cargo build --release -p hestia-fpga-conductor
cargo build --release -p hestia-asic-conductor
cargo build --release -p hestia-pcb-conductor
cargo build --release -p hestia-hal-conductor
cargo build --release -p hestia-apps-conductor
cargo build --release -p hestia-debug-conductor
cargo build --release -p hestia-rag-conductor
```

### 特定クレートのみビルド

```bash
cargo build --release -p container-manager
cargo build --release -p conductor-sdk
```

## テスト実行

### 全テスト

```bash
cargo test
```

### 特定 conductor のテスト

```bash
cargo test -p hestia-fpga-conductor
```

### 特定クレートのテスト

```bash
cargo test -p container-manager
```

## 生成物の配置

ビルド成果物は `.hestia/tools/target/release/` に配置される:

```
.hestia/tools/target/release/
├── hestia-ai-conductor
├── hestia-rtl-conductor
├── hestia-fpga-conductor
├── ...
├── hestia                # 統合ランナー CLI
├── hestia-ai-cli
├── hestia-fpga-cli
└── ...
```

## デバッグビルド

```bash
cargo build                # debug ビルド
cargo test -- --nocapture  # 標準出力を表示
RUST_LOG=debug cargo run -p hestia-fpga-conductor  # ログレベル指定
```

## 関連ドキュメント

- [cargo_workspace.md](cargo_workspace.md) — ワークスペース構成
- [conductor_startup.md](conductor_startup.md) — デーモン起動順序
- [error_handling_strategy.md](error_handling_strategy.md) — エラー処理