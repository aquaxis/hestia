# fpga-conductor ビルドステートマシン

**対象 Conductor**: fpga-conductor
**ソース**: 設計仕様書 §5.3（1542-1575行目付近）

## ビルドステートマシン

```
Idle
 │  build_start(target, steps)
 ▼
Resolving            ← VersionSelector でツールチェーン確定
 │                      fpga.toml の [toolchain] セクション参照
 │                      CompatibilityMatrix で最良バージョン選択
 ▼
ContainerStarting    ← Podman コンテナ起動
 │                      --userns=keep-id --network=none
 │                      ベンダーツールイメージ選択
 ▼
Synthesizing         ← adapter.synthesize(ctx)
 │  成功                TCL/QSF/Python スクリプト自動生成
 │                      リアルタイムログパース
 ▼
Implementing         ← adapter.implement(ctx)
 │  成功                配置配線・タイミング制約適用
 ▼
Bitstreamming        ← adapter.generate_bitstream(ctx)
 │  成功                ビットストリーム/JED/BIN 生成
 ▼
Success              → reports/ にタイミング・リソースレポート保存
                        fpga.lock 更新
```

## 状態定義

| 状態 | 説明 | 主要処理 |
|------|------|---------|
| Idle | 初期状態 | — |
| Resolving | ツールチェーンバージョン解決中 | VersionSelector、CompatibilityMatrix |
| ContainerStarting | Podman コンテナ起動中 | PodmanRuntime（コンテナ実行時のみ） |
| Synthesizing | RTL 合成実行中 | adapter.synthesize、TCL/QSF スクリプト自動生成、リアルタイムログパース |
| Implementing | 配置配線実行中 | adapter.implement、タイミング制約適用 |
| Bitstreamming | bitstream 生成中 | adapter.generate_bitstream、ビットストリーム/JED/BIN 生成 |
| Success | ビルド成功 | レポート保存、fpga.lock 更新 |

## 失敗時処理

```
各ステップで失敗 → SelfHealingPipeline.on_build_failure()
                    ↓
              CompatibilityMatrix で診断
                    ↓
              既知パッチあり → 自動適用/通知
              未知エラー    → PatcherAgent 起動
```

SelfHealingPipeline はビルド失敗時に CompatibilityMatrix を参照して診断し、既知パッチがあれば自動適用または通知する。未知エラーの場合は PatcherAgent（TypeScript + Anthropic SDK）を起動し、Tool Use 機能でパッチを生成する。

## 複数ターゲット並列ビルド

複数 target（artix7 / cyclone10 / trion 等）を同時ビルドする場合、各ターゲットごとに独立したステートマシンインスタンスが動作し、Synthesizing / Implementing ステップはターゲットごとに並列実行される。

## fpga.lock による再現性保証

ビルド成功時、使用したツールバージョン・コンテナイメージハッシュ・ビルドパラメータを fpga.lock に記録し、同一環境での再現性を保証する。

## 関連ドキュメント

- [fpga/binary_spec.md](binary_spec.md) — hestia-fpga-cli バイナリ仕様
- [fpga/error_types.md](error_types.md) — fpga-conductor エラーコード
- [fpga/vendor_adapter.md](vendor_adapter.md) — VendorAdapter トレイト
- [fpga/config_schema.md](config_schema.md) — fpga.toml スキーマ