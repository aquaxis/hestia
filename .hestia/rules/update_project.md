---
name: Hestia Agent Update Guidelines
description: hestia agent が update_ai サイクルで従う更新規約。`.aiprj/rules/update_project.md` を agent 文脈に解釈変更したもの（Phase 81 P-3）。
---

# Hestia Agent Update Guidelines (Phase 81)

このファイルは hestia agent が **update_ai サイクル**（上位から「現状反映」依頼を受けたとき、または上位指示と現行 `<workspace>/requirements.md` の内容差分検出時、Phase 89 用語統一）で参照する規約です。プロジェクト管理 AI 用の `.aiprj/rules/update_project.md` とは独立した hestia 文脈の実体です。

---

## Article 1: 更新トリガーの判定

agent は以下のいずれかを検出した場合に update_ai サイクルへ遷移します:

1. 上位 conductor から `agent-cli send` で「update / refresh / reload」相当の prompt 受信
2. 上位指示と現行 `<workspace>/requirements.md` の内容差分検出
3. 自身の責務範囲内で workspace 内成果物の不整合を検出（Article 3 参照）

（Phase 89 用語統一により旧 mtime 監視ベースのトリガーは廃止、上記 3 系統に整理）

判定は exec_job サイクル内でも周期的に行います（毎 prompt 受信時）。

---

## Article 2: 更新対象（Phase 91 — 3 文書遵守）

agent はペルソナ責務範囲内で **3 文書 + workspace 内成果物** を更新します。Phase 91 で全階層に 3 文書遵守義務が明示化されています:

| ペルソナ階層 | 主な更新対象（3 文書 + 成果物） |
|------------|---------------------------|
| ai-conductor | (1) `<workspace>/{requirements,design,tasks}.md` の 3 文書 / (2) `<root>/.hestia/run_log/<run-id>.json` aggregate / `<workspace>/agent.log` |
| domain conductor | (1) 3 文書 / (2) 担当ドメインの成果物 (`<root>/<domain>/...`) / handler 経由で生成された artifact。Phase 91 で domain planner 廃止に伴いタスク管理は本 conductor が直接担当 |
| sub-agent | (1) 3 文書 / (2) 担当モジュールの設計 / 実装 / テスト成果物 |

`.aiprj/AI_PRJ_*.md` 等のプロジェクト管理 AI 用文書は **更新しません**（責務外）。3 文書 skip は禁止（Phase 91 遵守必須化）。

---

## Article 3: 整合性の維持と halt 報告

更新作業中に既存成果物との不整合（Article 2 of `exec_job.md` 参照）を検出した場合、agent は次の優先順で対応します:

1. **責務範囲内かつ自明な不整合**: 自動修正し、修正内容を agent.log に明示
2. **責務範囲内だが影響範囲が広い**: 上位に halt 理由付きで報告し、修正方針の判断を仰ぐ
3. **責務範囲外**: 修正を試みず、上位に halt 理由付きで報告

「責務範囲内かつ自明」の判断基準は、ペルソナの `name` / `description` フィールドおよび `.hestia/personas/<self>.md` 本文の責務定義に明記された範囲のみです。

---

## Article 4: 進捗の記録 (Phase 49 mirror 継承)

更新作業の進捗は agent-cli 構造化 JSONL → mirror → workspace agent.log 経路で自動記録されます。agent は明示的な進捗 fs_write を行いません（`exec_job.md` Article 3 と同じ）。

---

## Article 5: 失敗時の透明な報告 (Phase 50 継承)

更新が失敗した場合、`exec_job.md` Article 5 と同じ 3 点セット（理由・次アクション候補・関連ログ抜粋）で上位に報告します。「更新できませんでした」だけの報告は禁止。

---

## Article 6: バージョン管理（autonomous）

agent は workspace 内成果物のバージョン管理（git commit / tag 等）を **行いません**。git 操作はプロジェクト管理 AI および user の責務領域です。agent は workspace 配下の最新状態を保つことのみを責務とし、履歴管理は上位に委譲します。

---

## Article 7: `.aiprj/` 直接参照の禁止 (Phase 81 新規)

hestia agent は `.aiprj/` ディレクトリへの直接参照を行いません。更新規約は `.hestia/rules/update_project.md`（本ファイル）を参照対象とします。

---

## Article 8: 自己実行ループとの整合 (Phase 57b/68/71 継承)

update_ai サイクルは setup_ai → exec_job → update_ai → close_ai の 4 サイクルの 1 つです。update_ai 完了後は通常 exec_job または idle に遷移します。各サイクル間の遷移ロジックは persona 内の「起動時の `.hestia/rules/` 自己実行規約」節に記載されています。
