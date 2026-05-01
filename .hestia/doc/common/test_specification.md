# テスト仕様

**対象領域**: common — テスト戦略
**ソース**: 設計仕様書 §15.4, §19.2, §20.7

## 概要

HESTIA のテスト戦略は3層（単体 / 統合 / E2E）で構成される。各層の目的・範囲・実行方法を定義する。

## テスト層構成

### 単体テスト

| 項目 | 説明 |
|------|------|
| 目的 | 個々の関数・メソッドの正確性検証 |
| 範囲 | クレート内のロジック・パーサ・変換・データモデル |
| 実行方法 | `cargo test -p <crate>` |
| CI トリガ | 全 PR / push |

主要な単体テスト例:
- `project-model::config` — 8 件の `[agent_cli]` 設定テスト
- `constraint-bridge` — 各フォーマットのパース / 生成テスト
- `ip-manager` — semver 解決・DAG 構築テスト
- `error_registry` — エラーコード範囲検証

### 統合テスト

| 項目 | 説明 |
|------|------|
| 目的 | 複数クレート間の連携・IPC 通信の検証 |
| 範囲 | conductor 間通信・DB 読み書き・コンテナビルド |
| 実行方法 | `cargo test -p integration-tests` |
| CI トリガ | merge to main |

主要な統合テスト例:
- `integration-tests::agent_cli_config` — 3 件（parse / Default 一致 / build_env）
- conductor 間 `agent-cli send` 通信テスト
- SQLite / sled 読み書き整合性テスト
- container-manager ビルドパイプラインテスト

### E2E テスト

| 項目 | 説明 |
|------|------|
| 目的 | エンドツーエンドのユーザーシナリオ検証 |
| 範囲 | CLI → conductor → ツール実行 → 結果確認 |
| 実行方法 | `cargo test -p e2e-tests` / 手動 |
| CI トリガ | リリース前 / スケジュール |

主要な E2E テスト例:
- `hestia init` → `hestia start fpga` → `hestia fpga build` → 結果確認
- Ollama 実起動 → agent-cli spawn → ping
- `hestia rag ingest` → `hestia rag search` → 結果確認

## TDD プラクティス（§19.2）

各モジュールについて「テストベンチ Phase → 実装 Phase」の 2 フェーズを必須化:

1. テストベンチを先行生成（AI または手動）
2. テスト実行 → FAIL（DUT 未実装）
3. 設計実装
4. テスト実行 → PASS/FAIL
5. カバレッジ < 95% では次ステップに進まない

## テストカバレッジ目標

| 層 | カバレッジ目標 |
|----|-------------|
| 単体テスト | 80% 以上 |
| 統合テスト | 主要パス 100% |
| E2E テスト | ユーザーシナリオ 100% |

## CI/CD 統合

- 全 PR で単体テスト + 統合テスト自動実行
- リリースブランチで E2E テスト自動実行
- テスト失敗時は `action-log` に記録

## 関連ドキュメント

- [installation.md](installation.md) — ビルド手順
- [cargo_workspace.md](cargo_workspace.md) — ワークスペース構成
- [cicd_api.md](cicd_api.md) — CI/CD API