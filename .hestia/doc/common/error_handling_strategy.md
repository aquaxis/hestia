# エラー処理戦略

**対象領域**: common — エラー処理
**ソース**: 設計仕様書 §18.9, §14.3

## 概要

HESTIA は Rust のエラー処理エコシステムである `thiserror` と `anyhow` を用途に応じて使い分ける。ライブラリクレートでは `thiserror` で型安全なエラーを定義し、バイナリクレートでは `anyhow` で柔軟なエラー処理を行う。エラーコード規約（§14.3）と整合させる。

## thiserror / anyhow 分離方針

| 用途 | クレート | 選択 | 理由 |
|------|---------|------|------|
| ライブラリ | `conductor-sdk`, `adapter-core`, `project-model` 等 | `thiserror` | 呼び出し元がエラー種別で分岐可能、型安全 |
| バイナリ | `hestia-fpga-conductor`, `hestia-ai-cli` 等 | `anyhow` | エラー伝搬の簡素化、トップレベルで一括処理 |

## エラー型設計パターン

### ライブラリ側（thiserror）

```rust
#[derive(Debug, thiserror::Error)]
pub enum ConductorError {
    #[error("Tool not found: {name}")]
    ToolNotFound { name: String },

    #[error("Build failed: exit code {exit_code}")]
    BuildFailed { exit_code: i32 },

    #[error("Timeout after {secs}s")]
    Timeout { secs: u64 },

    #[error("JSON-RPC error {code}: {message}")]
    Rpc { code: i32, message: String },
}
```

### バイナリ側（anyhow）

```rust
fn main() -> anyhow::Result<()> {
    let config = HestiaConfig::from_toml_file(path)?;
    // ? 演算子でシンプルに伝搬
    conductor.run().await?;
    Ok(())
}
```

## エラーコード規約との整合

構造化メッセージのエラー応答（§14.3）に変換する際、ライブラリの `thiserror` 型をエラーコードにマッピングする:

```rust
impl From<ConductorError> for ErrorResponse {
    fn from(err: ConductorError) -> Self {
        match err {
            ConductorError::ToolNotFound { .. } => ErrorResponse { code: -32209, .. },
            ConductorError::BuildFailed { .. }  => ErrorResponse { code: -32201, .. },
            ConductorError::Timeout { .. }      => ErrorResponse { code: -32001, .. },
            ConductorError::Rpc { code, .. }    => ErrorResponse { code, .. },
        }
    }
}
```

## エラー応答 data フィールド規約

全エラー応答の `data` に以下を含める:

| フールド | 型 | 説明 |
|---------|-----|------|
| `tool` | string | 発生元ツール名 |
| `exit_code` | int | プロセス終了コード |
| `log_path` | string | ログファイルパス |
| `errors[]` | array | エラー詳細リスト |
| `retry_possible` | bool | リトライ可否 |
| `suggested_action` | string | 推奨対応 |

## 関連ドキュメント

- [error_registry.md](error_registry.md) — エラーコード全一覧
- [agent_message.md](agent_message.md) — メッセージペイロード形式
- [observability.md](observability.md) — 監視・ログ