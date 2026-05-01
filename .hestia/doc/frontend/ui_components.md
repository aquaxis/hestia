# UI コンポーネントライブラリ

**対象領域**: frontend — UI デザインシステム
**ソース**: 設計仕様書 §16.3

## 概要

`hestia-ui`（React + TypeScript）として独立配布される UI コンポーネントライブラリ。VSCode 拡張と Tauri デスクトップアプリで共通利用する。

## コンポーネント一覧

| コンポーネント | 用途 |
|-------------|------|
| `ConductorStatusCard` | 各 conductor のステータス表示（Online / Offline / Degraded / Upgrading）|
| `AgentList` | サブエージェント一覧表示・起動・終了操作 |
| `SpecViewer` | 仕様書（DesignSpec）の構造化表示・編集 |
| `LogViewer` | 構造化ログのリアルタイムストリーム表示 |
| `WaveformViewer` | VCD / FST / GHW / EVCD 波形表示（WASM レンダリング）|
| `ConfigPanel` | 設定（config.toml / fpga.toml 等）のフォーム編集 |
| `TaskProgress` | ビルド / ワークフロータスクの進捗表示 |

## デザインシステム

### ブランド色

| 色 | コード | 用途 |
|----|-------|------|
| プライマリ（akane） | `#e84d2c` | アクション・アクセント |
| セカンダリ（deep green） | `#2d8f5e` | 成功・肯定 |

### 機能色

| 色 | 用途 |
|----|------|
| success | 成功・完了 |
| warning | 警告 |
| error | エラー・失敗 |
| info | 情報通知 |

## テーマ追従

VSCode / Tauri のテーマ変数に追従し、ダーク / ライトテーマに自動対応:

- VSCode: `--vscode-editor-background` / `--vscode-list-hoverBackground` 等
- Tauri: Tauri テーマ変数経由で OS テーマに追従

## 配布形式

- npm パッケージ: `hestia-ui`
- VSCode 拡張と Tauri アプリの両方で同一コンポーネントを import

## 関連ドキュメント

- [vscode_extension.md](vscode_extension.md) — VSCode 拡張
- [tauri_ide.md](tauri_ide.md) — Tauri デスクトップアプリ
- [wasm_waveform_viewer.md](../common/wasm_waveform_viewer.md) — WASM 波形ビューア