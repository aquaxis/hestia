# データベーススキーマ

**対象領域**: common — データ永続化
**ソース**: 設計仕様書 §18.9

## 概要

HESTIA は SQLite と sled の2種類のデータストアを使い分ける。SQLite はリレーショナルな構造化データ、sled は高スループットな KV データに使用する。

## SQLite スキーマ

用途: 構造化データの永続化。軽量組み込み DB。

### compat-matrix

ツールバージョン互換性マトリクス。

```sql
CREATE TABLE compat_matrix (
    id          INTEGER PRIMARY KEY,
    tool_name   TEXT NOT NULL,
    version     TEXT NOT NULL,
    target      TEXT NOT NULL,
    compatible  BOOLEAN NOT NULL,
    tested_at   TEXT NOT NULL,
    notes       TEXT
);
```

### spec_history

仕様書の変更履歴。

```sql
CREATE TABLE spec_history (
    id          INTEGER PRIMARY KEY,
    spec_path   TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    author      TEXT,
    changed_at  TEXT NOT NULL,
    change_type TEXT NOT NULL  -- 'created' | 'updated' | 'reviewed'
);
```

### work_log

作業ログのメタデータ。

```sql
CREATE TABLE work_log (
    id          INTEGER PRIMARY KEY,
    conductor   TEXT NOT NULL,
    task_id     TEXT NOT NULL,
    category    TEXT NOT NULL,  -- 'design_case' | 'bugfix_case' | 'build_log' | ...
    outcome     TEXT NOT NULL,  -- 'success' | 'failure' | 'partial'
    duration_secs INTEGER,
    created_at  TEXT NOT NULL
);
```

### ip_registry

IP コアの登録情報。

```sql
CREATE TABLE ip_registry (
    id          TEXT PRIMARY KEY,  -- 'com.vendor.name'
    version     TEXT NOT NULL,
    vendor      TEXT NOT NULL,
    license     TEXT NOT NULL,     -- 'Oss' | 'VendorProprietary' | 'Unknown'
    device_families TEXT,          -- JSON array
    updated_at  TEXT NOT NULL
);
```

### container_images

コンテナイメージ管理。

```sql
CREATE TABLE container_images (
    id          INTEGER PRIMARY KEY,
    image_name  TEXT NOT NULL,
    tag         TEXT NOT NULL,
    digest      TEXT,
    size_bytes  INTEGER,
    signed      BOOLEAN DEFAULT 0,
    built_at    TEXT NOT NULL
);
```

### prompt-archive/index.db

プロンプトアーカイブのインデックス。

```sql
CREATE TABLE prompts (
    prompt_id    TEXT PRIMARY KEY,
    trace_id     TEXT NOT NULL,
    agent_id     TEXT NOT NULL,
    timestamp    TEXT NOT NULL,
    model_name   TEXT NOT NULL,
    template_id  TEXT,
    status       TEXT NOT NULL,
    tokens_input  INTEGER,
    tokens_output INTEGER,
    latency_ms   INTEGER,
    file_path    TEXT NOT NULL
);
CREATE INDEX idx_prompt_trace ON prompts(trace_id);
CREATE INDEX idx_prompt_template ON prompts(template_id);
```

## sled スキーマ

用途: 高スループット KV ストア。zstd 圧縮、cache 1 GiB。

| KV コレクション | キー | 値 | 用途 |
|----------------|------|----|------|
| `messages` | `trace_id` | JSON | メッセージ履歴 |
| `agent_state` | `agent_id` | JSON | エージェント状態スナップショット |
| `task_queue` | `task_id` | JSON | タスクキュー |
| `rag_cache` | `query_hash` | JSON | RAG クエリキャッシュ |
| `version_matrix` | `tool@version` | JSON | バージョン互換性情報 |
| `workflow_state` | `workflow_id` | JSON | ワークフロー実行状態 |

## 関連ドキュメント

- [observability.md](observability.md) — 監視・メトリクス
- [ip_manager.md](ip_manager.md) — IP Manager（ip_registry 利用）
- [cicd_api.md](cicd_api.md) — CI/CD API（work_log 利用）