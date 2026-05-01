# ai-conductor SkillSystem 詳細

**対象 Conductor**: ai-conductor
**ソース**: 設計仕様書 §3.7（1095-1108行目付近）, §1.3.2（95-114行目付近）

## 概要

SkillRegistry に専門スキルを登録し、AI エージェント（agent-cli プロセスとして起動、§20 参照）が呼び出す。スキルは agent-cli のペルソナファイル（YAML+Markdown）と組み合わせて conductor ごとのメインエージェント・サブエージェントの能力を定義する。

## SkillRegistry

スキルの登録・管理・解決を行うレジストリ。ai-conductor の `skill-system/` クレートに実装される。

```
skill-system/
└── src/
    ├── lib.rs      # SkillRegistry
    └── skill.rs    # Skill トレイト
```

## Skill トレイト

全スキルが実装すべきトレイト。

```rust
pub trait Skill {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, input: &SkillInput) -> Result<SkillOutput, SkillError>;
}
```

カスタムスキルは `Skill` トレイトを実装して SkillRegistry に登録する。

## デフォルトスキル（5種）

| スキル | 入力 | 出力 | 説明 |
|--------|------|------|------|
| **HDL 生成** | 仕様書 / ブロック図 | SystemVerilog / Verilog / VHDL ソースコード | AI による HDL コード自動生成 |
| **制約生成** | ターゲットデバイス / タイミング要件 | XDC / SDC / PCF 制約ファイル | デバイス固有制約の自動生成 |
| **テストベンチ生成** | HDL モジュール定義 | テストベンチ + アサーション + カバレッジ | 検証環境スケルトン自動生成 |
| **回路図設計** | 自然言語仕様 / KG | SKiDL コード / KiCad ネットリスト | AI 駆動回路図合成（pcb-conductor と連携） |
| **レビュー** | HDL コード / 回路図 | レビュー結果 + 修正提案 | 設計品質レビューと改善提案 |

## スキル呼び出しフロー

```
1. SkillRegistry にスキルを登録（Skill トレイト実装）
2. ai-conductor が必要なスキルを持つ Agent を動的に起動
3. Agent はスキルを実行し、結果を ai-conductor に報告
```

## スキルと conductor の対応

| Conductor | 利用スキル |
|-----------|-----------|
| ai-conductor | 全スキル（オーケストレーション） |
| rtl-conductor | HDL 生成、テストベンチ生成、レビュー |
| fpga-conductor | HDL 生成、制約生成、テストベンチ生成 |
| asic-conductor | HDL 生成、制約生成、テストベンチ生成 |
| pcb-conductor | 回路図設計、レビュー |
| hal-conductor | HDL 生成（SystemVerilog テンプレート） |
| apps-conductor | レビュー |

## agent-cli ペルソナとの統合

スキルは agent-cli のペルソナファイルと組み合わせて使用される。各サブエージェントのペルソナファイル（`.hestia/personas/<name>.md`）にスキル利用の指示が含まれる。

```yaml
# ペルソナファイル例 (ai-planner.md)
skills:
  - hdl_generation
  - constraint_generation
  - testbench_generation
  - documentation_generation
```

## 拡張性

新しいスキルの追加は `Skill` トレイトを実装して SkillRegistry に登録するだけで完了する。コアコードの変更は不要（原則2: ゼロ変更での拡張）。

## 関連ドキュメント

- [ai/agent_hierarchy.md](agent_hierarchy.md) — サブエージェント構成
- [ai/message_methods.md](message_methods.md) — ai.* メソッド一覧
- [ai/workflow_engine.md](workflow_engine.md) — WorkflowEngine 詳細
- [../spec_driven_development.md](../spec_driven_development.md) — 仕様書駆動開発概要