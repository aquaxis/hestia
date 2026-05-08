# Hestia システム向け FPGA 基板開発 指示テンプレート

> 本文書は、Hestia AI エージェントシステムを使って **Efinix TZ75 FPGA + RISC-V SoC + リアルタイム OS** を搭載した評価基板を開発するための **指示テンプレート（サンプル）** です。ユーザは本文書をベースに `.aiprj/instructions.md` を作成し、Hestia の `/setup_ai` → `/update_ai` → `/ai`(exec_job) フローに乗せることで、要件定義から PCB 設計・FPGA 実装・RTOS ファームウェアまでを統合的に駆動できます。

---

## TL;DR

- **目的**: Efinix Titanium 75 を中心に、HyperRAM / 1GbE / USB-PD / MIPI-CSI / QSPI / I2C / 8bit LED / GPIO 拡張を備えた評価基板を **回路図 → アートワーク → FPGA RTL → RISC-V SoC → RTOS → PC ↔ UART 通信** まで一貫開発する。
- **駆動エンジン**: Hestia の `pcb` / `fpga` / `rtl` / `hal` / `apps` / `debug` の 6 conductor + 配下 sub-agent。
- **使い方**: 章 4 のハードウェア要求仕様を確定 → 章 5 で部品選定 → 章 6 で電源試算 → 章 7〜13 を `agent-cli send <persona> "..."` で順次 dispatch → 章 14 のチェックリストで完了確認。
- **重要追加項目**: JTAG / bitstream 用 SPI Flash / 電源シーケンサ / クロック源 / ESD 保護 / PCB 層構成 / I2C 拡張 / RTC / microSD / USB-Serial / 機械寸法 / CI/CD は ADD-1〜ADD-21 として網羅。

---

## 1. プロジェクト概要

| 項目 | 内容 |
|------|------|
| プロジェクト名 | Hestia FPGA Eval Board r0 |
| ターゲット FPGA | Efinix Titanium 75（`Ti75H486C4I7` 主、`Ti75H361` 副） |
| 目的 | RISC-V SoC + RTOS 評価、MIPI / 1GbE / HyperRAM 動作確認 |
| 想定電力 | USB-PD 9V/3A=27W プロファイル（標準）、5V/3A=15W（最低） |
| 想定基板 | 4 層〜6 層、100 × 80 mm 程度、ねじ穴 4 個、USB-C / RJ45 / MIPI を片側集約 |
| 想定読者 | 本指示文を `.aiprj/instructions.md` として Hestia に渡すユーザ |
| 想定期間 | 仕様策定 1w → 回路図 2w → アートワーク 2w → FPGA / RTL 4w → RTOS 2w → 検証 2w（約 13 週） |

---

## 2. 前提：Hestia システムのエージェント階層

本指示文はすべて Hestia の persona 群に対する `agent-cli send` で実行されます。

```
ai-conductor
├── pcb-conductor
│   ├── pcb-designer       … ハードウェア仕様策定
│   ├── pcb-schematic      … 回路図作成（KiCad sch）
│   ├── pcb-layout         … PCB アートワーク（KiCad pcb）
│   ├── pcb-emi-analyzer   … EMI / SI / PI 解析
│   └── pcb-tester         … DRC / ERC / 実機通電試験
├── fpga-conductor
│   ├── fpga-designer      … FPGA アーキ仕様
│   ├── fpga-synthesizer   … 合成（Efinity）
│   ├── fpga-implementer   … P&R / bitstream 生成
│   ├── fpga-floorplanner  … floorplan / IO 配置
│   ├── fpga-programmer    … 実機書込み
│   └── fpga-tester        … 実機テスト
├── rtl-conductor
│   ├── rtl-designer       … RTL 仕様（RISC-V SoC）
│   ├── rtl-coder          … RTL コーディング（SystemVerilog）
│   ├── rtl-formal-verifier… 形式検証
│   └── rtl-tester         … テストベンチ / cocotb
├── hal-conductor
│   ├── hal-designer       … BSP / HAL 仕様
│   ├── hal-coder          … BSP / ドライバ実装
│   └── hal-validator      … HAL テスト
├── apps-conductor
│   ├── apps-designer      … アプリ / RTOS 仕様
│   ├── apps-coder         … アプリ実装（タスク・コマンド処理）
│   ├── apps-builder       … クロスビルド / フラッシュ
│   └── apps-tester        … SIL / HIL / QEMU テスト
└── debug-conductor
    ├── debug-designer
    ├── debug-programmer
    ├── debug-analyzer
    ├── debug-coverage-analyzer
    └── debug-session-manager
```

### 2.1 dispatch の基本形

```bash
# conductor に投げる場合（自動で sub-agent を spawn）
agent-cli send pcb "Hestia FPGA Eval Board r0 のハードウェア仕様を策定してください。詳細は .aiprj/instructions.md 章 4-6 を参照。"

# sub-agent に直接投げる場合
agent-cli send pcb-schematic "回路図ドラフト v0.1 をレビューしてください。"
```

### 2.2 推奨リポジトリ構造（ADD-19）

```
<repo-root>/
├── .aiprj/                  ← Hestia 設定 / 指示 / ログ
├── hw/
│   ├── sch/                 ← KiCad schematic
│   ├── pcb/                 ← KiCad pcb
│   ├── bom/                 ← BOM CSV
│   ├── gerbers/             ← 製造データ
│   └── mech/                ← 機械図面 / 3D STEP
├── rtl/                     ← RISC-V SoC RTL
│   ├── src/
│   ├── sim/
│   └── constraints/
├── fw/
│   ├── bsp/                 ← HAL / BSP
│   ├── app/                 ← RTOS タスク / コマンド処理
│   └── samples/host_cli/    ← PC 側 CLI
└── docs/                    ← schematic PDF / user manual
```

---

## 3. プロジェクトゴール

### 3.1 完成イメージ

- USB-C ケーブル 1 本で電源 + デバッグ + UART を確保。
- bitstream は SPI NOR Flash から自動ロード（電源投入後 数秒で RTOS 起動 / heartbeat LED 点滅開始）。
- PC からターミナルで `ping` → `pong` が返ってくる。
- `led 0xAA` で 8bit LED が `10101010` 表示される。
- `task list` で RTOS 上のタスクが列挙される。
- 拡張ヘッダ（PMOD 互換）から I2C センサや SPI 機器を増設可能。

### 3.2 ブロック図（言語化）

```
[USB-C] ─→ [USB-PD コントローラ TPS65987] ─→ [DC/DC × 複数] ─→ FPGA / 周辺
                              │
                              └→ [USB Data] ─→ [FT2232H] ─→ JTAG / UART → FPGA
[RJ45] ── [Magnetics] ── [PHY RTL8211F] ── (RGMII) ── [TZ75 FPGA]
[MIPI-CSI コネクタ] ─ (D-PHY 2-lane) ─ [TZ75]
[HyperRAM × 1] ── (HyperBus 1.8V 200MHz) ── [TZ75]
[microSD] ── (SDIO 4-bit) ── [TZ75]
[I2C 拡張] ── [RTC DS3231 / EEPROM / sensor] ── [TZ75]
[8bit LED] / [PWR/FAULT/heartbeat LED] / [GPIO 32本ヘッダ] ── [TZ75]
[bitstream SPI Flash MX25L25645G] ── (QSPI) ── [TZ75 config 専用]
[QSPI 外部ペリフェラル（Flash / EEPROM 等）] ── (QSPI) ── [TZ75 SoC 汎用 QSPI]
[電源シーケンサ TPS3895/LM3880] ── [DC/DC enable]
```

---

## 4. ハードウェア要求仕様

`pcb-designer` および `fpga-designer` に並走で投げる仕様確定タスク。

### 4.1 要求仕様表

| カテゴリ | 要求 | 目標値 / 備考 | 必須/任意 |
|---------|------|-------------|---------|
| FPGA | Efinix TZ75 | `Ti75H486C4I7`（主）/ `Ti75H361`（副） | 必須 |
| Memory | HyperRAM | 64 Mbit、1.8V、200 MHz、`S27KL0641DABHI020` 1〜2 個 | 必須 |
| Ethernet | 1GbE | RJ45 + マグネティクス + RGMII PHY | 必須 |
| 電源 | USB-C / USB-PD | 9V/3A=27W プロファイル既定、5V/3A 互換 | 必須 |
| MIPI-CSI | カメラ入力 | 2-lane（4-lane 上限拡張）、Hirose FH52 系 | 必須 |
| GPIO | 拡張ヘッダ | PMOD 互換 × 2、3.3V LVCMOS、計 32 本以上 | 必須 |
| LED（ユーザ） | 8bit 個別制御 | 緑または白、3.3V 経由抵抗 | 必須 |
| LED（ステータス）（ADD-10）| PWR / FAULT / heartbeat | 3 個、PWR 緑 / FAULT 赤 / heartbeat 黄 | 必須 |
| JTAG（ADD-1, ADD-8, ADD-17）| FPGA / RISC-V 共用 | オンボード FT2232H + 外部 10pin/20pin ヘッダ | 必須 |
| QSPI Flash（ADD-2）| bitstream 保存（FPGA config 専用） | 32 MB QSPI NOR、`MX25L25645G` | 必須 |
| QSPI（SoC 汎用） | SoC からアクセスする外部 QSPI バス | 拡張ヘッダから外部 QSPI Flash / EEPROM を接続可（容量・型番はユーザ選択）、3.3V LVCMOS、最大 50 MHz | 必須 |
| 電源シーケンサ（ADD-3）| 多レール起動順制御 | `LM3880` または `TPS3895` × 必要数 | 必須 |
| クロック（ADD-4）| FPGA / Ethernet / MIPI | 25 MHz xtal × 2、27 MHz xtal × 1、または `Si5351A` | 必須 |
| ESD 保護（ADD-5）| 外部 IO 全て | USB / RJ45 / MIPI / GPIO に TVS | 必須 |
| PCB 層数（ADD-6）| 高速差動 | 6 層推奨（最低 4 層） | 必須 |
| I2C 拡張（ADD-12）| 周辺接続 | 3.3V、400kHz、ヘッダ + RTC + EEPROM | 必須 |
| RTC（ADD-13）| 時刻同期 | `DS3231`（I2C） + コイン電池ホルダ | 必須 |
| microSD（ADD-14）| ストレージ | SDIO 4-bit、push-push スロット | 必須 |
| USB-Serial（ADD-15）| PC ↔ UART | FT2232H 兼用（ADD-17）、または CP2102N 別ピン | 必須 |
| 機械寸法（ADD-16）| 筐体組込み | 100 × 80 mm、4 隅 M3 ねじ穴、コネクタ片側集約 | 必須 |
| 監視 / テスト（ADD-11）| 主要レール / クロック | 各電源・クロック・リセットに TP | 推奨 |

### 4.2 dispatch 例

```bash
agent-cli send pcb-designer "
.aiprj/instructions.md 章 4 のハードウェア要求仕様表に従い、Hestia FPGA Eval Board r0 の
<workspace>/pcb-designer/{requirements,design,tasks}.md を作成してください。

考慮:
- FPGA: Efinix Ti75H486C4I7 を主候補。pin 数充足と MIPI / SerDes 配置を検討
- 必須項目（必須欄が必須）と推奨/任意項目を明確に分離
- ADD-1〜ADD-17 のうちハードウェア起因項目を全て要求仕様に含める
"

agent-cli send fpga-designer "
同じ要求仕様（章 4）を FPGA 視点で評価し、Ti75H486 で十分かを LE / DSP / BRAM / SerDes / GPIO ピン
数の観点から判断してください。不足時は Ti120 / Ti180 への昇格を提案してください。
"
```

---

## 5. 部品選定の指示

`pcb-designer` + `pcb-schematic` に部品選定を委譲します。

### 5.1 主要部品候補表

| カテゴリ | 候補例 | 選定基準 |
|---------|-------|--------|
| FPGA | `Ti75H486C4I7` / `Ti75H361` | LE 数、MIPI lane 数、I/O 数、入手性 |
| HyperRAM | `S27KL0641DABHI020` (64Mbit/1.8V/200MHz) | 容量、Hz、電圧、HyperBus 互換性 |
| Ethernet PHY | `RTL8211F-CG` / `KSZ9031RNX` | RGMII、温度範囲、入手性 |
| RJ45 + Mag | Pulse `JK0-0177NL` / Bel Fuse `0826-1X1T-21-F` | 統合マグネティクス、LED 表示 |
| USB-PD | TI `TPS65987DDH` / Cypress `CYPD3175` | PD3.0、シンク能力、I2C 制御 |
| USB-C コネクタ | Amphenol `12401610E4#2A` / GCT `USB4105-GF-A` | 24-pin フル、シールド、SMD |
| DC/DC コア | TI `TPS62810` (4A, 0.6V〜) | 効率、応答性、サイズ |
| DC/DC IO 用 | TI `TPS54824` (8A) | 電流余裕 |
| LDO アナログ | TI `TPS7A47` (低ノイズ) | PHY / MIPI 用ノイズ抑制 |
| 電源シーケンサ（ADD-3） | TI `LM3880` (3 レール) または `TPS3895` × 数 | レール順序保証 |
| クロック（ADD-4） | 25 MHz xtal `ABM10W-25.000MHZ-K4Z-T3` × 2 + 27 MHz `ECS-270-8-30B-CKM-TR` | 周波数精度、ジッタ |
| プログラマブル PLL | Silicon Labs `Si5351A-B-GT` | 必要なら MIPI / Ethernet 周波数生成 |
| 設定 Flash（ADD-2） | Macronix `MX25L25645GMI-08G` (32MB) | 容量、QSPI、Efinix 推奨 |
| RTC（ADD-13） | Maxim `DS3231SN#` (±2ppm) | 高精度、I2C |
| ESD（ADD-5） | TI `TPD4S009DRYR` (USB)、`TPD4E1U06DCKR` (汎用)、Onsemi `ESD7L5.0DT5G` (RJ45) | クランプ電圧、容量 |
| USB-Serial / JTAG（ADD-15, ADD-17） | FTDI `FT2232HL` (Channel A=JTAG, Channel B=UART) | OpenOCD 対応、Linux ドライバ標準 |
| microSD（ADD-14） | Hirose `DM3AT-SF-PEJM5` push-push | SDIO 4-bit、活線挿抜 |
| MIPI コネクタ | Hirose `FH52-15S-0.5SH` 15-pin 0.5mm FPC | Raspberry Pi 互換 |
| LED（ユーザ）| 0805 緑 LED + 1kΩ × 8 | 個別制御 |
| LED（ステータス、ADD-10） | Kingbright `APT2012LSECK` (緑/赤) × 3 | PWR / FAULT / heartbeat |
| ヘッダ・コネクタ | 2.54mm × 2 列 PMOD（GPIO）/ 10pin Cortex Debug（JTAG）| 拡張性 |
| コイン電池ホルダ（RTC） | Renata `MS621` / Keystone 1051 | RTC バックアップ |

### 5.2 dispatch 例

```bash
agent-cli send pcb-schematic "
章 5 の部品選定候補表を BOM 案 v0.1 に展開してください。

要求:
- 各カテゴリで第 1 候補・第 2 候補を選定理由とともに 1 行記載
- DigiKey / Mouser の在庫数と単価を記載（API 不要、最新値の調査で良い）
- 代替部品の互換性（pin compatible / footprint compatible）を明記
- 出力先: <workspace>/pcb-schematic/bom_v0_1.csv

最終確定はユーザレビュー後とする。
"
```

### 5.3 部品選定のセルフチェック

- [ ] FPGA は Ti75H486 を主候補に決定し LE / I/O / MIPI 数で要求充足を確認
- [ ] 全部品が産業温度範囲（-40〜85℃）対応か、もしくは商用範囲で足りるかを判断
- [ ] BGA 0.5mm pitch を扱う基板製造業者の能力（最小ビア径、層数）を確認
- [ ] ESD / TVS が USB / RJ45 / MIPI / GPIO 全外部 IO に配置されている
- [ ] RoHS 準拠 / 入手難部品の代替が用意されている

---

## 6. 電源設計と消費電力検討

`pcb-designer` + `pcb-schematic` に電源設計を委譲します。

### 6.1 電源レール定義（Ti75H486 想定）

| レール | 電圧 | 想定電流 | 用途 | 推定電力 | 供給 |
|-------|------|---------|------|---------|------|
| VCC | 0.85V | 1.5A | FPGA core | 1.28W | TPS62810 |
| VCCAUX | 1.8V | 0.3A | FPGA aux / config | 0.54W | TPS62810 |
| VCCIO_HRAM | 1.8V | 0.3A | HyperRAM IO | 0.54W | 同上レール共用可 |
| VCCIO_GPIO | 3.3V | 0.4A | LED / GPIO / I2C / SPI | 1.32W | TPS62810 |
| VCCIO_MIPI | 1.2V | 0.2A | MIPI D-PHY | 0.24W | TPS62810 |
| 3.3V_PHY | 3.3V | 0.3A | Ethernet PHY | 0.99W | LDO TPS7A47（低ノイズ）|
| 3.3V_AUX | 3.3V | 0.3A | USB-PD / FT2232H / RTC | 0.99W | TPS62810 |
| 1.8V_AUX | 1.8V | 0.1A | RTC backup（コイン電池）/ Flash IO | 0.18W | LDO |

合計推定 **~6.1W**。USB-PD 9V/3A=27W に対し約 23% 利用、4 倍以上の余裕。標準プロファイルは 9V を選択し、5V/3A=15W は最低許容（一部周辺を制限する想定）。

### 6.2 電源シーケンス（ADD-3）

Ti75H486 の起動順序ルール（Efinix Ti75 データシート Power-up Sequence 章準拠）:

```
T0:        VCC (0.85V)        を立ち上げ
T0+1ms:    VCCAUX (1.8V)      を立ち上げ
T0+2ms:    VCCIO_HRAM (1.8V)  / VCCIO_MIPI (1.2V)
T0+3ms:    VCCIO_GPIO (3.3V)  / 3.3V_PHY / 3.3V_AUX

立ち下げは逆順。
```

実装: `LM3880` 1 個または `TPS3895` × 2〜3 を組合わせ、各 DC/DC の `EN` ピンを順次アサート。`pcb-designer` に「LM3880 1 個でレール順序を 4 段階に制御し、残りは EN 派生で制御してください」と指示。

### 6.3 消費電力検討の dispatch 例

```bash
agent-cli send pcb-designer "
章 6.1 のレール表を出発点とし、Efinity Power Estimator（Efinity 内蔵）の出力に基づいて
Ti75H486 のコア消費電力を再見積りしてください。

入力:
- RTL 構成: RV32IMC + 周辺 (UART×2 / GPIO / Ethernet MAC / HyperBus / MIPI-CSI 2-lane)
- 動作周波数: core 100MHz, RGMII 125MHz, MIPI 200MHz
- 温度: 25℃ TJ, 85℃ TJ の 2 ケース

出力:
- <workspace>/pcb-designer/power_budget_v0_1.md（レール別 / 用途別 / 温度別）
- USB-PD 9V/3A プロファイルでの利用率レポート
- ディレーティング（× 1.5）後の DC/DC 選定推奨
"
```

### 6.4 デカップリング戦略（ADD-7）

- VCC（0.85V）: BGA 真下に 0402 100nF を 8〜12 個 + 大容量 22uF MLCC × 4
- VCCIO_*: ボール群ごとに 100nF + 1uF + 10uF を最短経路で
- 全ボリューム: バルク 100uF タンタル / ポリマ × 1 を入力近傍

`pcb-layout` への指示で「PI シミュレーションで電源インピーダンスを 100kHz〜1GHz で確認、目標 < 30 mΩ」を明記。

### 6.5 検証コマンド / 期待結果

```bash
# 電源シーケンス測定（pcb-tester）
oscilloscope_trigger.py --channels VCC VCCAUX VCCIO_GPIO VCCIO_HRAM
# 期待: 上記 6.2 の順序を満たす（順序逆転 0 件）
```

---

## 7. 回路図設計

`pcb-schematic` に投げます。

### 7.1 ネットリスト方針

- ネット名はオール大文字 + アンダースコア（例: `USB_VBUS`、`FPGA_VCC`、`MIPI0_D0_P` / `MIPI0_D0_N`）
- 差動ペアは `_P` / `_N` でペアリング
- バス信号は `[7:0]` 表記（`LED[7:0]` 等）
- 電源ネットは色分け（KiCad 慣習）

### 7.2 シート構成（KiCad）

| シート | 内容 |
|------|------|
| `top.sch` | ブロック図、外部接続マップ |
| `power.sch` | USB-PD / DC/DC / シーケンサ / バルク (ADD-3, ADD-7) |
| `fpga.sch` | TZ75 ピン配置、デカップリング、電源接続 |
| `clock.sch` | xtal × 25 MHz / 27 MHz、Si5351 (任意) (ADD-4) |
| `config.sch` | bitstream SPI Flash + JTAG ヘッダ (ADD-1, ADD-2, ADD-17) |
| `memory.sch` | HyperRAM × 1〜2 + デカップリング |
| `ethernet.sch` | RJ45 + Magnetics + PHY + ESD (ADD-5) |
| `usb.sch` | USB-C コネクタ + USB-PD + FT2232H + ESD |
| `mipi.sch` | MIPI-CSI コネクタ + ESD |
| `gpio_led.sch` | 拡張ヘッダ、ユーザ LED 8bit、ステータス LED 3 個 (ADD-10) |
| `i2c_aux.sch` | RTC DS3231、I2C 拡張ヘッダ、EEPROM、コイン電池 (ADD-12, ADD-13) |
| `sd.sch` | microSD コネクタ (ADD-14) |
| `testpoints.sch` | 主要ネットの TP (ADD-11) |

### 7.3 dispatch 例

```bash
agent-cli send pcb-schematic "
章 7.2 のシート構成で KiCad 7.x プロジェクト hw/sch/board.kicad_pro を作成し、
章 5 の BOM v0.1 を symbol へ反映してください。

指示:
- 全外部 IO に ESD / TVS を配置 (ADD-5)
- 電源シーケンスを LM3880 で 4 段階構築 (ADD-3)
- ステータス LED を 3 個（PWR/FAULT/heartbeat）配置 (ADD-10)
- TP を電源各レール / 25MHz xtal / リセット信号に最低 12 個 (ADD-11)
- ERC をクリーンに pass する状態で commit
出力: hw/sch/ 一式 + ERC レポート
"
```

### 7.4 検証コマンド / 期待結果

```bash
kicad-cli sch erc --severity-error hw/sch/board.kicad_sch
# 期待: ERC violations: 0
kicad-cli sch export pdf -o docs/board_schematic.pdf hw/sch/board.kicad_sch
# 期待: PDF 生成、章ごとに整理されている
```

---

## 8. PCB アートワーク

`pcb-layout` + `pcb-emi-analyzer` に投げます。

### 8.1 層構成（ADD-6）

| 層 | 用途（6 層案） |
|---|------|
| L1 | 信号（高速含む）、コネクタ |
| L2 | GND（リファレンス） |
| L3 | 電源プレーン（VCCIO_GPIO / 3.3V 系） |
| L4 | 電源プレーン（VCC / VCCAUX / 1.8V 系） |
| L5 | GND（リファレンス） |
| L6 | 信号（低速 / GPIO / I2C / 制御） |

差動ペア（MIPI / RGMII / HyperBus / USB / SDIO）は L1 を主、L6 を予備。

### 8.2 配線方針

- **MIPI D-PHY**: 100Ω 差動、長さマッチング ±5mil、コネクタからの最短経路
- **RGMII**: 50Ω SE、長さマッチング TXC/TXD、RXC/RXD
- **HyperBus**: 短距離（< 50mm 推奨）、長さマッチング厳格、リファレンスプレーン GND 連続
- **USB 2.0/3.0**: 90Ω 差動、bend 最小化、ESD 後にスタブ最短
- **SDIO**: CLK は他線より長め（スキュー対策）
- **電源**: スターポイント GND、バルクは入力近傍、デカップリングは BGA 真下

### 8.3 機械寸法（ADD-16）

- 基板外形: 100 × 80 mm（暫定）
- 4 隅に M3 ねじ穴（内径 3.2mm、ランド 6mm、5mm キーアウト）
- USB-C / RJ45 / MIPI コネクタ: 片側集約（短辺）
- PMOD ヘッダ: 反対側
- 高さ制限: 表面 12mm（コネクタ含む）/ 裏面 3mm

### 8.4 dispatch 例

```bash
agent-cli send pcb-layout "
hw/sch/board.kicad_sch を入力に hw/pcb/board.kicad_pcb を 6 層構成で作成してください。

制約:
- 100 × 80 mm 矩形、4 隅 M3 ねじ穴
- 章 8.1 の層構成
- 章 8.2 の配線方針（差動長マッチ、リファレンス GND 連続）
- DRC violations: 0 を目指す（IPC class 2、最小ビア 0.2mm/0.4mm）

出力: hw/pcb/ 一式 + DRC レポート + 3D 確認用 STEP
"

agent-cli send pcb-emi-analyzer "
hw/pcb/board.kicad_pcb の電源インテグリティ（PI）を解析し、各レール 100kHz〜1GHz で
インピーダンス < 30mΩ を満たすかをレポートしてください。
不足箇所はバルク追加またはレイアウト変更を提案してください。
"
```

### 8.5 検証コマンド / 期待結果

```bash
kicad-cli pcb drc --severity-error hw/pcb/board.kicad_pcb
# 期待: DRC violations: 0
kicad-cli pcb export gerbers -o hw/gerbers/ hw/pcb/board.kicad_pcb
kicad-cli pcb export step -o hw/mech/board.step hw/pcb/board.kicad_pcb
# 期待: gerber 一式、3D STEP 出力
```

---

## 9. FPGA 回路設計

`fpga-designer` → `fpga-synthesizer` → `fpga-implementer` → `fpga-floorplanner` の順で。

### 9.1 内部構成（Ti75H486 想定）

```
┌─────────────────────────────────────────────────────────────┐
│  TZ75 (Ti75H486)                                            │
│                                                             │
│  ┌────────────┐  ┌────────────┐  ┌────────────────┐         │
│  │ RV32IMC    │  │ AXI4 Inter │  │ HyperBus Ctrl  │         │
│  │ (RISC-V)   │←─│ connect    │─→│ (1.8V/200MHz)  │─→ HyperRAM
│  └────────────┘  └─┬─┬─┬─┬─┬──┘  └────────────────┘         │
│                    │ │ │ │ │                                │
│           ┌────────┘ │ │ │ └────────┐                       │
│           ↓          ↓ ↓ ↓          ↓                       │
│        ┌──────┐ ┌──────┐ ┌──────┐ ┌──────────┐ ┌──────────┐ │
│        │ UART │ │ GPIO │ │Timer │ │ Eth MAC  │ │ MIPI-CSI │ │
│        │ ×2   │ │ 32b  │ │ ×2   │ │ (RGMII)  │ │ RX 2-lane│ │
│        └──────┘ └──────┘ └──────┘ └──────────┘ └──────────┘ │
│        ┌──────┐ ┌──────┐ ┌──────┐ ┌──────────┐              │
│        │ I2C  │ │ SPI  │ │ SDIO │ │ PLIC +   │              │
│        │ ×2   │ │ ×1   │ │      │ │ CLINT    │              │
│        └──────┘ └──────┘ └──────┘ └──────────┘              │
│                                                             │
│  Clock: 25MHz xtal → PLL → core 100MHz / RGMII 125MHz       │
│                          / MIPI 200MHz / HyperBus 200MHz    │
└─────────────────────────────────────────────────────────────┘
```

### 9.2 リソース見積り（要件 §3.FR-7 対応）

| ブロック | LE | DSP | BRAM | 備考 |
|--------|----|----|----|------|
| RV32IMC コア | ~10K | 0〜4 | 4〜8 | RISC-V standard config |
| AXI4 interconnect | ~3K | 0 | 0 | 4 master / 8 slave |
| HyperBus controller | ~5K | 0 | 2 | Cypress HyperBus IP |
| Ethernet MAC | ~8K | 0 | 4 | RGMII + checksum offload |
| MIPI-CSI 2-lane RX | ~6K | 0 | 0 | D-PHY hard IP 利用 |
| 周辺 (UART/GPIO/Timer/I2C/SPI/QSPI/SDIO/PLIC/CLINT) | ~8K | 0 | 2 | |
| **合計** | **~40K** | **〜8** | **〜20** | TZ75 の 75K LE で **~53% 利用、47% 余裕** |

### 9.3 dispatch 例

```bash
agent-cli send fpga-designer "
.aiprj/instructions.md 章 9 と章 10 を入力に <workspace>/fpga-designer/{requirements,design,tasks}.md
を作成してください。

要求:
- FPGA: Efinix Ti75H486C4I7
- クロック計画: 25MHz xtal → PLL × 1 → core 100MHz / RGMII 125MHz / MIPI 200MHz / HyperBus 200MHz
- IO 制約: 章 8 の物理配置と矛盾しない pinout
- ブロック構成: 章 9.1 の図に従う
- ピン残数で QSPI / I2C 拡張ヘッダ等の周辺配置を最適化
"

agent-cli send fpga-synthesizer "
RTL（章 10 で生成）と <workspace>/fpga-designer/design.md の制約をもとに、
Efinity v2024.x で合成してください。

出力:
- bitstream（hw/fpga/build/board.hex）
- リソースレポート、タイミングレポート、消費電力推定
- 失敗時は rtl-conductor へ修正依頼
"

agent-cli send fpga-floorplanner "
MIPI / HyperBus / RGMII の高速 IO バンクを集中配置してください。
JTAG / config Flash 用 SPI ピンは bank 0 / dedicated に固定。
出力: hw/fpga/constraints/board.pdc
"
```

### 9.4 検証コマンド / 期待結果

```bash
# 合成（Efinity プロジェクト）
efx_run --flow compile --project hw/fpga/board.xml
# 期待: timing slack > 0、LUT < 75% 利用、bitstream 生成

# 実機書込み
agent-cli send fpga-programmer "FT2232H 経由で hw/fpga/build/board.hex を SPI Flash に書込み"
# 期待: 書込み成功、再起動後 LED heartbeat 点滅
```

---

## 10. RISC-V SoC 実装

`rtl-designer` → `rtl-coder` → `rtl-formal-verifier` → `rtl-tester` の順で。

### 10.1 ISA / コア選定

本テンプレートでは特定の RISC-V IP を指定しない。以下の要件を満たす任意の
RISC-V コア（事前生成済 Verilog として提供されるもの）を選定し、SystemVerilog
wrapper 経由で SoC に統合する:

- **ISA**: RV32IMC（M-mode + U-mode、no MMU、no FPU）
- **バス**: AXI4-lite（周辺）+ AXI4（HyperRAM 高速側）
- **割り込み**: PLIC + CLINT 互換
- **サイズ目安**: TZ75 の 75K LE に対し ~1500〜10000 LUT4 程度（リソース余裕 30% 以上を確保）
- **言語**: 入力は Verilog（SystemVerilog 互換）。生成元が他の HDL の場合は事前に Verilog 化したものを取り込む。

> **HDL 制約**: rtl-coder が手書きする RTL は **SystemVerilog 単一**とする。
> RISC-V コア IP は事前生成済の Verilog 形式を採用し、Hestia 内では
> SystemVerilog wrapper 経由で SoC に統合する。

### 10.2 SoC 仕様（ADD-9 ウォッチドッグ含む）

```
ISA:        RV32IMC, no MMU, no FPU
Privilege:  M-mode + U-mode (RTOS 用)
Bus:        AXI4-lite (周辺) + AXI4 (HyperRAM 高速側)
割り込み:   PLIC (32 source) + CLINT (mtime)
Watchdog:   IP 内蔵 1 個（RTOS が定期 kick、4 秒で reset）
Boot ROM:   16 KB (block RAM、起動時 SPI Flash → HyperRAM コピーロジック)
SRAM:       32 KB (block RAM、stack / IRQ 退避用)
HyperRAM:   8 MB (外付け、RTOS + アプリ実行領域)
```

### 10.3 メモリマップ

| アドレス | 領域 | サイズ |
|---------|-----|------|
| `0x0000_0000` | BootROM | 16 KB |
| `0x1000_0000` | SRAM | 32 KB |
| `0x2000_0000` | HyperRAM | 8 MB |
| `0x4000_0000` | UART0 | 4 KB |
| `0x4000_1000` | UART1 | 4 KB |
| `0x4000_2000` | GPIO | 4 KB |
| `0x4000_3000` | LED (8bit) | 4 KB |
| `0x4000_4000` | Timer0 / Timer1 | 8 KB |
| `0x4000_6000` | I2C0 / I2C1 | 8 KB |
| `0x4000_8000` | SPI0 | 4 KB |
| `0x4000_9000` | SDIO | 4 KB |
| `0x4000_A000` | QSPI Flash Controller | 4 KB |
| `0x4001_0000` | Ethernet MAC | 16 KB |
| `0x4002_0000` | MIPI-CSI RX | 16 KB |
| `0x4003_0000` | Watchdog | 4 KB |
| `0x0C00_0000` | CLINT (mtime/mtimecmp) | 64 KB |
| `0x0F00_0000` | PLIC | 64 MB |

### 10.4 ブートシーケンス

```
1. POR → BootROM 起動 (0x00000000)
2. SPI Flash から 0x00010000 オフセットの RTOS image を HyperRAM (0x20000000) へ DMA コピー
3. CRC32 チェック OK ならば 0x20000000 へ jump
4. RTOS 初期化 → タスク作成 → スケジューラ start
```

### 10.5 dispatch 例

```bash
agent-cli send rtl-designer "
章 10 の SoC 仕様（ISA, バス, メモリマップ, ブート）に従い
<workspace>/rtl-designer/{requirements,design,tasks}.md を作成してください。

参考: 選定する RISC-V コアの standard config をベースに、AXI4-lite 周辺と AXI4 HyperRAM を統合したトップを設計
"

agent-cli send rtl-coder "
rtl/src/ に以下を実装してください:
- soc_top.sv（トップラッパ）
- riscv_core_wrapper.sv（RISC-V コア IP の Verilog ラップ）
- axi_interconnect.sv
- bootrom.sv（init コード hex を localparam で）
- hyperram_ctrl.sv
- uart_16550.sv ×2
- gpio.sv / led.sv / timer.sv / i2c.sv / spi.sv / qspi.sv / sdio.sv
- watchdog.sv (ADD-9)
- plic.sv / clint.sv
- eth_mac_wrapper.sv (RGMII)
- mipi_csi_rx.sv

各モジュールに cocotb テストを rtl/sim/<module>/ に配置
"

agent-cli send rtl-formal-verifier "
plic / clint / watchdog / axi_interconnect の安全特性（deadlock 無し、acknowledge 必ず返る、
watchdog kick 後 timeout 内なら reset 起こさない）を SymbiYosys で形式検証してください。
"

agent-cli send rtl-tester "
全モジュールの cocotb テスト + RV32 ISA test (riscv-tests rv32ui-p-*) を Verilator で実行し、
カバレッジ > 90% を目標にレポートしてください。
"
```

### 10.6 検証

```bash
# シミュレーション
make -C rtl/sim verilator
# 期待: 全 cocotb テスト pass、ISA test pass

# 合成（章 9 の fpga-synthesizer に流す）
agent-cli send fpga-synthesizer "rtl/src/ を入力に Efinity で合成"
```

---

## 11. リアルタイム OS

`hal-designer` / `hal-coder` / `hal-validator` + `apps-designer` / `apps-coder` / `apps-builder` に分担。

### 11.1 RTOS 候補比較

| 候補 | RV32IMC port | ライセンス | フットプリント | コミュニティ | 採否 |
|-----|--------------|-----------|------------|-----------|----|
| **FreeRTOS 10.6.x** ★主候補 | あり | MIT | 6〜10 KB | 大 | 採用 |
| Zephyr 3.6.x | あり | Apache-2.0 | 30〜100 KB | 大 | 副候補 |
| RT-Thread | あり | Apache-2.0 | 8〜20 KB | 中（中国） | 副 |
| NuttX | あり | Apache-2.0 | 30〜80 KB | 中 | 不採用 |

選定理由: RTOS フットプリント小、ライセンス互換、リソース余裕考慮、ドライバ自作前提。

### 11.2 BSP / ドライバ構成（ADD-12, ADD-13, ADD-14, ADD-9 含む）

```
fw/bsp/
├── crt0.S                     ← リセットベクタ、スタック初期化、jump main
├── linker.ld                  ← 0x0000_0000 ROM, 0x1000_0000 SRAM, 0x2000_0000 HyperRAM
├── clock.c / clock.h          ← PLL 設定読み出し
├── hyperram_init.c            ← HyperRAM キャリブレーション
├── uart.c / uart.h            ← UART0 (debug) / UART1 (RTOS console)
├── gpio.c / led.c             ← LED / GPIO レジスタ
├── i2c.c                      ← I2C0 / I2C1（DS3231 / EEPROM 用）
├── rtc.c                      ← DS3231 ドライバ（ADD-13）
├── sdio.c                     ← microSD（FATFS 連携）（ADD-14）
├── eth.c                      ← lwIP 統合（任意）
├── timer.c                    ← FreeRTOS tick 用 mtime
├── plic.c / clint.c
├── watchdog.c                 ← FreeRTOS タスクから定期 kick（ADD-9）
└── irq.c                      ← 割り込みディスパッチ
```

### 11.3 タスク構成

| タスク | 優先度 | スタック | 役割 |
|-------|-------|--------|------|
| heartbeat | 1 | 256 W | 1Hz でステータス LED トグル（heartbeat） |
| uart_rx | 4 | 512 W | UART1 受信 → コマンドキュー push |
| cmd_proc | 3 | 1024 W | キュー pop → parse → dispatch |
| uart_tx | 3 | 512 W | 応答キュー pop → UART1 送信 |
| watchdog_kick | 2 | 128 W | 1Hz で watchdog kick |
| eth_lwip | 2 | 1024 W | lwIP main loop（任意） |

### 11.4 dispatch 例

```bash
agent-cli send hal-designer "
章 11 の構成に従い fw/bsp/ の HAL / BSP 仕様を策定してください。
出力: <workspace>/hal-designer/{requirements,design,tasks}.md

考慮:
- メモリマップ（章 10.3）
- 全レジスタアクセスは volatile pointer 経由
- 割り込みハンドラは naked + IRQ context
"

agent-cli send hal-coder "
章 11.2 の各ファイルを fw/bsp/ に実装してください。

制約:
- C99、依存ライブラリなし（newlib-nano は許可）
- リエントラント
- HyperRAM 初期化前は SRAM/ROM のみで動作
- 各ドライバに mock ヘッダ（fw/bsp/test/mocks/）を提供
"

agent-cli send hal-validator "
fw/bsp/ の各ドライバを QEMU + Verilator co-simulation でテストしてください。
カバレッジ > 80%。
"

agent-cli send apps-designer "
章 11.3 のタスク構成を <workspace>/apps-designer/design.md に展開してください。
コマンドプロトコル（章 12）と紐付けてください。
"

agent-cli send apps-coder "
fw/app/ に main.c + tasks/ を実装してください。

タスク:
- main.c: HAL init → FreeRTOS 起動 → タスク生成 → vTaskStartScheduler()
- tasks/heartbeat.c
- tasks/uart_rx.c
- tasks/cmd_proc.c（章 12 のコマンド集を実装）
- tasks/uart_tx.c
- tasks/watchdog.c
- (任意) tasks/eth_lwip.c
"

agent-cli send apps-builder "
fw/ を riscv32-unknown-elf-gcc でビルドし、
- ELF / map / sym / hex / bin
- size レポート
- objdump 全体
を artifacts/ に出力。さらに hex を SPI Flash 書込み形式（offset 0x10000）に整形。
"
```

---

## 12. PC ↔ UART 通信

`apps-coder` + `apps-tester` に。

### 12.1 物理層（ADD-15, ADD-17）

- USB-C 経由で FT2232H → Channel A=JTAG（FPGA / RISC-V 用）、Channel B=UART（RTOS console）
- 速度: 115200 bps を既定、921600 bps まで拡張可能
- 設定: 8N1、ハードウェアフロー制御 OFF

### 12.2 プロトコル

テキストコマンド方式（LF 終端、ASCII）を既定。最低限のコマンド集:

| コマンド | 例 | 応答 | 説明 |
|---------|----|----|----|
| `ping` | `ping\n` | `pong\n` | 動作確認 |
| `version` | `version\n` | `Hestia FPGA r0 v0.1.0\n` | バージョン |
| `uptime` | `uptime\n` | `uptime: 12345 ticks\n` | RTOS tick 数 |
| `task list` | `task list\n` | 表形式の応答 | タスク列挙 |
| `led <bits>` | `led 0xAA\n` | `OK\n` | LED 8bit 設定 |
| `mem r <addr>` | `mem r 0x40003000\n` | `0x000000AA\n` | メモリ読出し |
| `mem w <addr> <val>` | `mem w 0x40003000 0xFF\n` | `OK\n` | メモリ書込み |
| `i2c scan <bus>` | `i2c scan 0\n` | アドレス一覧 | I2C デバイススキャン |
| `rtc get` / `rtc set` | `rtc get\n` | `2026-05-07 12:34:56\n` | RTC アクセス |
| `reset` | `reset\n` | （応答なし、再起動）| ソフトリセット |

将来拡張: SLIP / COBS フレーミングでバイナリ転送モード追加可。

### 12.3 ホスト CLI（Python pyserial 例）

```python
# samples/host_cli/cli.py
import serial, sys, argparse

def main():
    p = argparse.ArgumentParser()
    p.add_argument('--port', default='/dev/ttyUSB1')
    p.add_argument('--baud', type=int, default=115200)
    p.add_argument('cmd', nargs='+')
    args = p.parse_args()
    with serial.Serial(args.port, args.baud, timeout=2) as s:
        s.write((' '.join(args.cmd) + '\n').encode())
        print(s.read_until(b'\n').decode().rstrip())

if __name__ == '__main__':
    main()
```

使用例:
```bash
python samples/host_cli/cli.py --port /dev/ttyUSB1 ping
# → pong
python samples/host_cli/cli.py led 0xAA
# → OK
```

### 12.4 ホスト CLI（Rust 例、samples/host_cli/Cargo.toml + src/main.rs）

```toml
[package]
name = "hestia-fpga-cli"
version = "0.1.0"
edition = "2021"

[dependencies]
serialport = "4"
clap = { version = "4", features = ["derive"] }
```

```rust
// samples/host_cli/src/main.rs
use std::time::Duration;
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[arg(long, default_value = "/dev/ttyUSB1")]
    port: String,
    #[arg(long, default_value_t = 115200)]
    baud: u32,
    cmd: Vec<String>,
}

fn main() -> anyhow::Result<()> {
    let a = Args::parse();
    let mut p = serialport::new(&a.port, a.baud)
        .timeout(Duration::from_secs(2))
        .open()?;
    let line = format!("{}\n", a.cmd.join(" "));
    p.write_all(line.as_bytes())?;
    let mut buf = [0u8; 256];
    let n = p.read(&mut buf)?;
    print!("{}", std::str::from_utf8(&buf[..n])?);
    Ok(())
}
```

### 12.5 dispatch 例

```bash
agent-cli send apps-coder "
fw/app/tasks/cmd_proc.c に章 12.2 のコマンド集を実装し、
samples/host_cli/cli.py と samples/host_cli/src/main.rs を作成してください。

要求:
- パーサは strtok ベース（バッファ 256B）
- 不明コマンドは 'ERR: unknown' 応答
- メモリ書込みは 0x4000_0000 以降のみ許可（BootROM 保護）
"

agent-cli send apps-tester "
ホストから 1000 回 ping を送信し、応答時間平均と取りこぼし率を測定してください。
基準: 平均 < 5ms、loss = 0
"
```

---

## 13. 検証 / インテグレーション

| 段階 | 担当 persona | 内容 | 検証コマンド |
|------|-----------|------|----------|
| ボードレベル | `pcb-tester` | 通電試験、電源シーケンス計測、温度分布 | オシロ + サーモグラフィ |
| FPGA レベル | `fpga-tester` | bitstream 書込み、LED ブリンク、UART loopback | `agent-cli send fpga-tester` |
| RTL レベル | `rtl-tester` | cocotb / Verilator UVM、RV32 ISA test | `make -C rtl/sim verilator` |
| RTOS レベル | `apps-tester` | 起動時間、タスク switch 計測、UART throughput | ホスト CLI から測定 |
| 統合 | `debug-session-manager` | E2E、ログ集約、カバレッジ統合 | `agent-cli send debug` |

### 13.1 CI/CD（ADD-20）

```yaml
# .github/workflows/hw_fw.yml（例）
jobs:
  hw_drc:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: kicad-cli sch erc --severity-error hw/sch/board.kicad_sch
      - run: kicad-cli pcb drc --severity-error hw/pcb/board.kicad_pcb
  rtl_sim:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: make -C rtl/sim verilator
  fw_build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - run: make -C fw/app
      - run: make -C samples/host_cli
```

`agent-cli send debug-session-manager "上記 CI を整備し、PR 毎に DRC / ERC / RTL sim / FW build を回す"` で依頼。

### 13.2 実機統合シナリオ

1. USB-C 接続 → PWR LED 緑点灯、heartbeat LED 1Hz 点滅
2. ホストから `python cli.py ping` → `pong` 受信
3. `cli.py led 0xAA` → ユーザ LED が `10101010` 表示
4. `cli.py task list` → 5〜6 タスクが列挙
5. `cli.py i2c scan 0` → DS3231 のアドレス `0x68` がヒット
6. `cli.py rtc get` → 現在時刻取得
7. （任意）Ethernet 接続 → `ping <board-ip>` 応答
8. 24 時間連続稼働 → watchdog reset 0 回、ハングなし

---

## 14. 完了条件チェックリスト

### 14.1 ハードウェア（章 4〜8）

- [ ] FPGA が Efinix Ti75H486C4I7 で確定し、リソース余裕 > 30%
- [ ] HyperRAM × 1 個以上が 1.8V/200MHz で動作
- [ ] 1GbE が RGMII PHY 経由でリンクアップ
- [ ] USB-PD 9V/3A プロファイル動作確認
- [ ] MIPI-CSI 2-lane 動作確認
- [ ] GPIO 拡張ヘッダ 32 本以上配線
- [ ] ユーザ LED 8bit が個別制御可
- [ ] JTAG ヘッダ実装（ADD-1）
- [ ] bitstream SPI Flash 実装（ADD-2）
- [ ] 電源シーケンサ実装（ADD-3）
- [ ] クロック源実装（ADD-4）
- [ ] ESD 保護実装（ADD-5）
- [ ] PCB 6 層構成決定（ADD-6）
- [ ] デカップリング戦略反映（ADD-7）
- [ ] ステータス LED 3 個実装（ADD-10）
- [ ] テストポイント 12 個以上（ADD-11）
- [ ] I2C 拡張バス実装（ADD-12）
- [ ] RTC 実装（ADD-13）
- [ ] microSD 実装（ADD-14）
- [ ] USB-Serial（FT2232H）実装（ADD-15, ADD-17）
- [ ] 機械寸法 100×80mm + ねじ穴 4 個（ADD-16）
- [ ] schematic ERC violations: 0
- [ ] PCB DRC violations: 0

### 14.2 FPGA / RTL（章 9〜10）

- [ ] timing slack > 0、LUT 利用率 < 75%
- [ ] RV32IMC コアが riscv-tests rv32ui-p-* 全 pass
- [ ] cocotb テストカバレッジ > 90%
- [ ] watchdog 動作確認（ADD-9）
- [ ] BootROM → SPI Flash → HyperRAM ブートシーケンス成功

### 14.3 ファームウェア（章 11〜12）

- [ ] FreeRTOS 起動成功
- [ ] heartbeat / uart_rx / cmd_proc / uart_tx / watchdog_kick タスク動作
- [ ] 章 12.2 のコマンド集すべて応答
- [ ] ホスト CLI（Python / Rust）動作
- [ ] 24 時間連続稼働ハングなし

### 14.4 成果物（ADD-18, ADD-19, ADD-20, ADD-21）

- [ ] schematic PDF（`docs/board_schematic.pdf`）
- [ ] BOM CSV（`hw/bom/board_bom.csv`）
- [ ] gerber zip（`hw/gerbers/`）
- [ ] pick-and-place ファイル
- [ ] 機械図面 + 3D STEP（`hw/mech/`）
- [ ] bitstream（`hw/fpga/build/board.hex`）
- [ ] RTOS image（`fw/app/build/firmware.hex`）
- [ ] ユーザマニュアル（`docs/user_manual.md`）
- [ ] CI/CD pipeline 動作（DRC / ERC / RTL sim / FW build）
- [ ] USB-PD 認証 / CE / FCC EMC 検討メモ（任意、`docs/compliance.md`）

### 14.5 セルフチェック（指示テンプレートとしての品質）

- [ ] 文書全体が日本語
- [ ] persona 名（pcb-designer 等）はすべて `.hestia/personas/` の実在ファイルに対応
- [ ] 部品型番がすべて実在型番
- [ ] 上位指示の 19 考慮事項 + ADD-1〜ADD-21 がすべて文書中に網羅されている
- [ ] 各章末に検証コマンド・期待結果がある（ある場合）

---

## 付録 A: persona 拡張提案（必要時のみ）

現行の Hestia persona は本タスクに必要な機能を概ね充足する。以下は追加が **想定される** 範囲（実装は別タスク、本指示文では提案にとどめる）:

- `pcb-mfg`: ガーバー製造データ・組立図・量産連携専門
- `pcb-compliance`: EMC / RoHS / USB-IF 認証専門
- `rtl-perf-tuner`: RTL のクリティカルパス自動最適化
- `apps-bootloader`: BootROM / 2nd-stage loader 専門

これらは本指示文には含めず、必要に応じて別途 `.hestia/personas/` に追加する（既存 persona の改変は行わない方針）。

## 付録 B: パッケージ選定の補足

Efinix Ti75 のパッケージ選択肢:

| パッケージ | ボール | ピッチ | I/O 数 | MIPI lane | 推奨用途 |
|----------|------|------|------|----------|--------|
| `Ti75H361` | 361-ball BGA | 0.65mm | ~250 | ~16 | 量産機（基板コスト優先）|
| `Ti75H486` | 486-ball BGA | 0.5mm | ~360 | ~24 | 評価機（拡張余地優先）★主候補 |

本テンプレートは `Ti75H486C4I7` を前提として記述。量産化時に `Ti75H361` への移行を検討する場合、PMOD 拡張や任意機能を見直す必要がある。

## 付録 C: 参考リンク

- Efinix Titanium 75 Datasheet: <https://www.efinixinc.com/products-titanium-overview-Ti75.html>
- Cypress / Infineon HyperRAM: <https://www.infineon.com/cms/en/product/memories/hyperram/>
- USB-PD 3.1 Specification: <https://www.usb.org/document-library/usb-power-delivery>
- FreeRTOS RV32 Port: <https://www.freertos.org/Documentation/03-Libraries/02-FreeRTOS-libraries/00-Overview>
- KiCad 7+: <https://www.kicad.org/>
- Hestia persona 一覧: 本リポジトリの `.hestia/personas/*.md`（50 件）

---

## 付録 D: 上位指示 19 考慮事項 × ADD-* 対応マトリクス

| 上位指示項目 | 配置章 |
|-----------|------|
| 基板設計について言及する | 章 7 / 章 8 |
| FPGA: Efinix TZ75 | 章 4 / 章 5 / 章 9 |
| Memory: HyperRAM | 章 4 / 章 5 / 章 9 / 章 10 |
| Ethernet: 1GbE | 章 4 / 章 5 / 章 9 / 章 11 |
| 電源: USB-PD | 章 4 / 章 5 / 章 6 |
| MIPI-CSI: 付けたい | 章 4 / 章 5 / 章 9 |
| QSPI: 設定用 Flash（FPGA config 専用） | 章 3 / 章 4 |
| QSPI: SoC 汎用バス（外部 QSPI 接続） | 章 3 / 章 4 / 章 10 |
| I2C: 主要バス | 章 4 / 章 7 / 章 10 |
| GPIO: それなりに | 章 4 / 章 7 / 章 9 |
| LED: 8bit | 章 4 / 章 7 / 章 11 |
| 電源: 部品選定 | 章 5 / 章 6 |
| 要求仕様から部品選定 | 章 4 → 章 5 全体 |
| 電源: 消費電力検討 | 章 6 |
| 回路図 | 章 7 |
| アートワーク | 章 8 |
| FPGA の回路を作成 | 章 9 |
| RISC-V を FPGA に実装 | 章 10 |
| RISC-V にリアルタイム OS | 章 11 |
| PC と UART で通信 | 章 12 |
| ADD-1〜ADD-21（調査追加）| 章 4〜14 に分散統合（要件 §3.FR-11、設計 §3.11 マッピング表参照）|

---

> 本サンプル指示文は完了です。ユーザはこの内容を `.aiprj/instructions.md` に転記または改変し、`/setup_ai` → `/update_ai` → `/ai`(exec_job) の Hestia フローを順次実行してください。
