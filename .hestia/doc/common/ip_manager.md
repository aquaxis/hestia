# IP Manager

**対象領域**: common — IP コア管理
**ソース**: 設計仕様書 §13.4

## 概要

IP コアの登録・検索・バージョン解決・ライセンス管理・依存関係解決を提供する共有サービス。`petgraph` の DAG ベース解決アルゴリズム（トポロジカルソート）で多段依存を解く。agent-cli peer `ip-manager` として提供される。

## 主要型

### IpCore

```rust
pub struct IpCore {
    pub id: String,                // "com.vendor.name"
    pub version: String,           // semver
    pub vendor: String,
    pub library: String,
    pub device_families: Vec<String>,
    pub supported_languages: Vec<String>,
    pub dependencies: Vec<IpDependency>,
    pub files: Vec<IpFile>,
    pub parameters: Vec<IpParameter>,
}
```

### IpDependency

```rust
pub struct IpDependency {
    pub ip_id: String,
    pub version_req: VersionReq,    // semver VersionReq
    pub optional: bool,
}
```

### IpFile

```rust
pub struct IpFile {
    pub path: String,
    pub file_type: IpFileType,     // rtl | testbench | doc | constraint
    pub language: IpLanguage,       // verilog | vhdl | その他
}
```

## 依存関係解決アルゴリズム

`petgraph` の DAG で IP コア間の依存関係を構築し、トポロジカルソートで解決順序を決定する。

```
IpCore A (depends on B, C)
  ├── IpCore B (depends on D)
  └── IpCore C (depends on D)
      └── IpCore D (no dependencies)

トポロジカルソート結果: [D, B, C, A]
```

循環依存を検出した場合はエラー（DAG ではない）。

## ライセンス分類

| 分類 | 対象ライセンス | 扱い |
|------|-------------|------|
| `Oss` | MIT / Apache-2.0 / BSD / GPL / ISC / CC0 | 自由に利用・公開可能 |
| `VendorProprietary` | FlexLM・seat 制限 | `terms_accepted=true` 必須、社内利用のみ |
| `Unknown` | 不明 | **拒否**（取り込み不可）|

## バージョン解決

semver に基づくバージョン制約の解決:

| 制約 | 意味 |
|------|------|
| `>=0.40` | 0.40 以上 |
| `^1.0.0` | 1.x.x（互換性維持）|
| `~1.2.0` | 1.2.x（パッチ更新のみ）|
| `=2025.2` | 厳密一致 |

## クレート構成

```
ip-manager/
├── Cargo.toml
└── src/
    ├── lib.rs              # IpCore, IpRegistry
    ├── resolver.rs         # DAG 解決（petgraph）
    ├── license.rs          # ライセンス分類・検証
    └── version.rs          # semver バージョン解決
```

## 関連ドキュメント

- [constraint_bridge.md](constraint_bridge.md) — 制約ファイル変換
- [database_schema.md](database_schema.md) — ip_registry スキーマ
- [observability.md](observability.md) — 監視