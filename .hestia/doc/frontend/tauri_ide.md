# Tauri デスクトップアプリ

**対象領域**: frontend — デスクトップ IDE
**ソース**: 設計仕様書 §16.2

## 概要

Tauri（Rust + React）ベースのデスクトップアプリ。VSCode 拡張と同等の機能をネイティブデスクトップ環境で提供する。

## 基本設定

| 項目 | 値 |
|------|-----|
| バージョン | 0.1.0 |
| 識別子 | `dev.hestia.ide` |
| 設定ファイル | `tauri.conf.json` |

## ウィンドウ構成

| ウィンドウ | サイズ | 用途 |
|----------|-------|------|
| main | 1440 x 900 | メインエディタ・conductor 管理パネル |
| waveform | 1200 x 600 | 波形ビューア（WASM / ネイティブ）|
| settings | 800 x 600 | 設定画面 |

## セキュリティ

### Content Security Policy (CSP)

```
connect-src 'self' ipc: ws://localhost:*
```

- `connect-src 'self'`: 同一オリジンからの通信を許可
- `ipc:`: Tauri IPC チャンネル
- `ws://localhost:*`: 開発時の HMR（Hot Module Replacement）

## バンドルターゲット

| ターゲット | フォーマット |
|----------|------------|
| Debian / Ubuntu | `.deb` |
| RHEL / Fedora | `.rpm` |
| Linux 汎用 | `.AppImage` |

## Shell プラグイン

Tauri Shell プラグイン経由で以下の 10 コマンドを呼び出し可能:

| コマンド | 用途 |
|---------|------|
| `hestia` | 統合ランナー |
| `hestia-ai-cli` | ai-conductor CLI |
| `hestia-rtl-cli` | rtl-conductor CLI |
| `hestia-fpga-cli` | fpga-conductor CLI |
| `hestia-asic-cli` | asic-conductor CLI |
| `hestia-pcb-cli` | pcb-conductor CLI |
| `hestia-hal-cli` | hal-conductor CLI |
| `hestia-apps-cli` | apps-conductor CLI |
| `hestia-debug-cli` | debug-conductor CLI |
| `hestia-rag-cli` | rag-conductor CLI |

## UI コンポーネント

`hestia-ui`（§16.3）のコンポーネントを流用し、Tauri 固有のテーマ変数に追従して表示を統一。

## 関連ドキュメント

- [vscode_extension.md](vscode_extension.md) — VSCode 拡張
- [ui_components.md](ui_components.md) — UI コンポーネントライブラリ
- [agent_cli_client.md](agent_cli_client.md) — agent-cli クライアント仕様