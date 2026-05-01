# 仕様書駆動開発（Spec-Driven Development）

**対象領域**: 仕様書駆動開発（SDD）
**ソース**: 設計仕様書 §1.3.1（79-94行目付近）, §3.6（1078-1094行目付近）, §19.1（4021-4099行目付近）

---

## 1. 概念

自然言語で記述された仕様書から、設計データ（HDL コード、制約ファイル、回路図、テストベンチ等）を AI が自動生成する開発手法である。

---

## 2. 必要性

ハードウェア設計では仕様書から HDL コードへの翻訳に多大な工数がかかる。仕様の曖昧さや解釈の揺れが設計ミスの原因となる。AI による自動変換で工数を削減し、仕様と実装の乖離を防止する。

---

## 3. 機能概要

1. エンジニアが自然言語で仕様を記述する（`REQ:` / `CON:` / `IF:` プレフィックス）
2. `SpecParser` が仕様を構造化された `DesignSpec` に変換する
3. AI エンジンが `DesignSpec` から HDL コード・制約ファイル・テストベンチを生成する
4. 生成されたコードは自動検証パイプラインで品質を担保する

```
仕様書（自然言語）→ SpecParser → DesignSpec → AI 生成エンジン → HDL / 制約 / テストベンチ
```

---

## 4. SDD プロセス（§19.1 詳細）

```
┌─────────────────────────┐
│ 1. 仕様記述（自然言語）  │  REQ: クロック周波数 100 MHz
│   REQ: / CON: / IF:      │  CON: リソース LUT < 5000
│                          │  IF: AXI4-Lite slave
└────────────┬─────────────┘
             ▼
┌─────────────────────────┐
│ 2. SpecParser            │  入力: テキスト
│   構文解析 → AST          │  出力: DesignSpec { requirements: [...],
│                          │                     constraints: [...],
│                          │                     interfaces: [...] }
└────────────┬─────────────┘
             ▼
┌─────────────────────────┐
│ 3. AI 生成エンジン        │  スキル呼び出し:
│   HDL 生成 / 制約生成 /   │    - HDL 生成（FR-AI-CONCEPT-02）
│   テストベンチ生成        │    - 制約生成（XDC / SDC）
│                          │    - テストベンチ生成（FR-AI-PRAC-02）
└────────────┬─────────────┘
             ▼
┌─────────────────────────┐
│ 4. 自動検証              │  - HDL 静的解析（svls / veridian）
│   構文 / 型 / 合成可能性   │  - 合成ドライラン（小構成）
│                          │  - テストベンチ実行（Verilator）
└────────────┬─────────────┘
             ▼
┌─────────────────────────┐
│ 5. 逆方向検証（仕様差分） │  生成された HDL から仕様を再抽出し、
│   実装 → 仕様 の再導出    │  元 DesignSpec との差分をレポート
└─────────────────────────┘
```

---

## 5. SpecParser による仕様 → DesignSpec 変換（§3.6）

`SpecParser` は自然言語仕様書を構造化された `DesignSpec` に変換する。`REQ:` / `CON:` / `IF:` プレフィックスで要件・制約・インターフェースを自動解析する。

```rust
pub struct SpecParser;

impl SpecParser {
    pub fn parse(spec_text: &str) -> Result<DesignSpec, SpecError> {
        // REQ: で始まる行 → 要件
        // CON: で始まる行 → 制約
        // IF:  で始まる行 → インターフェース定義
        // 必須要件が1件以上なければエラー
    }
}
```

### DesignSpec AST（主要フィールド）

```rust
pub struct DesignSpec {
    pub metadata:    SpecMetadata,        // source_path, author, timestamp
    pub requirements: Vec<Requirement>,   // REQ: 行の構造化
    pub constraints:  Vec<Constraint>,    // CON: 行の構造化
    pub interfaces:   Vec<InterfaceDecl>, // IF: 行の構造化
    pub free_text:    String,             // 非プレフィックス本文
}

pub struct Requirement {
    pub id:          String,   // 自動採番（REQ-001 など）
    pub description: String,   // 本文
    pub priority:    Priority, // MUST / SHOULD / MAY
    pub tags:        Vec<String>,
}

pub struct Constraint {
    pub id:       String,   // CON-001
    pub kind:     ConstraintKind,  // Timing / Resource / Power / Area
    pub text:     String,
    pub numeric:  Option<NumericConstraint>, // e.g. LUT < 5000
}

pub struct InterfaceDecl {
    pub id:       String,            // IF-001
    pub name:     String,            // AXI4-Lite / APB / Wishbone
    pub role:     InterfaceRole,     // Master / Slave
    pub signals:  Vec<SignalDecl>,
}
```

AST は `serde` で JSON 化し、`action-log` に保存される。

---

## 6. AI 生成エンジンによる HDL / 制約 / テストベンチ生成

AI 生成エンジンは `DesignSpec` を入力として、スキルシステム（§1.3.2）経由で以下の生成を行う:

- **HDL 生成スキル**: SystemVerilog / Verilog / VHDL ソースコードの自動生成
- **制約生成スキル**: XDC / SDC / PCF 制約ファイルの自動生成
- **テストベンチ生成スキル**: テストベンチスケルトン + アサーション + カバレッジの自動生成

---

## 7. 自動検証パイプライン

生成されたコードは以下の自動検証パイプラインで品質を担保する:

1. **HDL 静的解析**: svls / veridian による構文・型チェック
2. **合成ドライラン**: 小構成での合成可能性検証
3. **テストベンチ実行**: Verilator によるシミュレーション実行
4. **逆方向検証**: 生成された HDL から仕様を再抽出し、元 DesignSpec との差分をレポート

逆方向検証で差分が検出された場合、`Feedback Loop`（§19.10）を通じて仕様 or 生成物を修正する。

---

## 8. REQ / CON / IF プレフィックス規約

仕様書内の行頭プレフィックスにより、SpecParser が自動的に構造化データを抽出する:

| プレフィックス | 対応構造体 | 内容 |
|--------------|----------|------|
| `REQ:` | `Requirement` | 要件定義。優先度（MUST / SHOULD / MAY）を付与可能 |
| `CON:` | `Constraint` | 設計制約。種別（Timing / Resource / Power / Area）を自動判定 |
| `IF:` | `InterfaceDecl` | インターフェース定義。名前・ロール・信号一覧を構造化 |

`SpecParser` は必須要件（MUST）が 1 件以上なければエラーを返す。

---

## 9. 運用ルール

- 仕様書は `.hestia/specs/<project-name>.md` に配置
- `SpecParser` は必須要件（MUST）が 1 件以上なければエラー
- 生成物（HDL / 制約 / テストベンチ）は `.hestia/generated/<project-name>/` に配置し、対応する `DesignSpec` のハッシュをメタデータとして埋め込む
- 逆方向検証で差分が検出された場合、`Feedback Loop`（§19.10）を通じて仕様 or 生成物を修正

---

## 10. 実装箇所

`ai-conductor/crates/spec-driven/`（既存）+ `spec-driven/src/parser.rs` 拡張。

---

## 関連ドキュメント

- [Hestia Flow](hestia_flow.md) — AI 活用9概念の体系的解説（SDD は §1.3.1 に該当）
- [アーキテクチャ概要](architecture_overview.md) — 設計原則7「AI 活用」の位置づけ
- [共有サービス](shared_services.md) — RAG による仕様書関連情報の検索・注入
- [セキュリティ](security.md) — 生成物の知的財産保護
- `.hestia/doc/ai/skills_system.md` — スキルシステム（HDL 生成 / 制約生成 / テストベンチ生成スキル）
- `.hestia/doc/ai/workflow_engine.md` — ワークフローエンジン（SDD を含むワークフローの自動実行）