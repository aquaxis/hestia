# asic-conductor ビルドステートマシン

**対象 Conductor**: asic-conductor
**ソース**: 設計仕様書 §6.5（1868-1884行目付近）

## 13 状態ビルドステートマシン

| 状態 | 進捗 | 説明 |
|------|------|------|
| `Idle` | 0% | 初期状態 |
| `PdkResolving` | 3% | PDK バージョン解決・パス検証中 |
| `Synthesizing` | 10% | 論理合成実行中 (Yosys) |
| `Floorplanning` | 20% | フロアプラン作成中 |
| `Placing` | 30% | セル配置実行中 |
| `CTS` | 45% | クロックツリー合成実行中 |
| `Routing` | 60% | 配線実行中 |
| `Extraction` | 70% | 寄生抽出実行中 |
| `TimingSignoff` | 75% | タイミングサインオフ検証中 |
| `DRC` | 80% | デザインルールチェック実行中 |
| `LVS` | 90% | レイアウト対回路図検証実行中 |
| `GDSII` | 95% | GDSII ストリーム生成中 |
| `Success` | 100% | ビルド成功 |

## 状態遷移図

```
Idle
  │  build_start
  ▼
PdkResolving ─────── PDK 未インストール → Failed
  │
  ▼
Synthesizing ─────── Yosys 合成失敗 → Failed
  │
  ▼
Floorplanning ────── フロアプラン失敗 → Failed
  │
  ▼
Placing ──────────── 配置失敗 → Failed
  │
  ▼
CTS ──────────────── クロックツリー合成失敗 → Failed
  │
  ▼
Routing ──────────── 配線失敗 → Failed
  │
  ▼
Extraction ────────── 寄生抽出失敗 → Failed
  │
  ▼
TimingSignoff ─────── タイミング違反 → AI 修復提案 or Failed
  │
  ▼
DRC ───────────────── DRC 違反 → AI 修復提案 or Failed
  │
  ▼
LVS ───────────────── LVS ミスマッチ → Failed
  │
  ▼
GDSII ─────────────── GDSII 生成失敗 → Failed
  │
  ▼
Success
```

## 失敗時の AI エージェント連携

| ステップ | AI 連携 | 説明 |
|---------|---------|------|
| TimingSignoff | タイミング違反自動修復 | 制約緩和またはバッファ挿入を提案 |
| DRC | DRC 違反自動修復 | 違反パターンに基づきレイアウト修正パッチを生成 |
| Floorplanning | フロアプラン最適化 | 配置密度・配線混雑度に基づき改善提案 |

## OpenLane 2 Step-based Execution との連携

asic-conductor は OpenLane 2 の Python ベース Step-based Execution を活用し、各工程の個別再実行が可能。`advance` コマンドで特定ステップから再開できる。

## asic.lock による再現性保証

ビルド成功時、使用した PDK バージョン・ツールバージョン・ビルドパラメータを asic.lock に記録し、同一環境での再現性を保証する。

## 関連ドキュメント

- [asic/binary_spec.md](binary_spec.md) — hestia-asic-cli バイナリ仕様
- [asic/error_types.md](error_types.md) — asic-conductor エラーコード
- [asic/tool_adapter.md](tool_adapter.md) — AsicToolAdapter トレイト
- [../fpga/state_machines.md](../fpga/state_machines.md) — FPGA ビルドステートマシン