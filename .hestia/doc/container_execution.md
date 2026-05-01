# コンテナ実行戦略

**対象領域**: コンテナ実行・管理
**ソース**: 設計仕様書 §11（2551-2641行目付近）, §12（2644-3185行目付近）

---

## 1. Podman コンテナ戦略（§11）

> **コンテナは必須ではない**。ローカルに直接インストールされたツール（Vivado / Quartus / KiCad 等）も使用可能であり、本章はコンテナ実行を選択した場合の戦略を記述する。ローカル実行とコンテナ実行はツールごと・ビルドごとに混在可能であり、いずれを選択しても fpga.lock / asic.lock による再現性を保証する。

本章は **Linux ホスト OS 前提**（TC-10）。Podman rootless が依存する user namespace / cgroup / SELinux は Linux カーネル機能であり、本章の設計はすべて Linux 環境上での動作を想定する（ローカル実行のみを選択する場合、本章の Podman 関連設定は適用対象外となる）。

### 1.1 設計方針

- **デーモンレス**: システムサービス不要（Docker の dockerd に依存しない）
- **UID 一致**: `--userns=keep-id` でファイル権限問題を回避
- **SELinux 対応**: `--security-opt=label=type:container_runtime_t`
- **権限昇格防止**: `--security-opt=no-new-privileges`
- **ライセンス保護**: ベンダーライセンスファイルの読み取り専用マウント
- **ネットワーク隔離**: ビルドコンテナは `--network=none` で外部通信を遮断
- **知的財産保護**: HDL ソース、ビットストリームのコンテナ外流出防止

---

## 2. podman-runtime クレート（§11.2）

```rust
pub struct PodmanRuntime {
    socket: PathBuf,  // /run/user/1000/podman/podman.sock
}

impl PodmanRuntime {
    /// バッチビルド用コンテナ起動
    pub async fn run_build(
        &self, image: &str, project_dir: &Path, cmd: &[&str],
    ) -> Result<ExitStatus, RuntimeError> {
        let mut args = vec![
            "run", "--rm",
            "--userns=keep-id",
            "--security-opt=no-new-privileges",
            "--network=none",
            "-v", &format!("{}:/workspace:Z", project_dir.display()),
            "-v", &format!("{}:/reports:Z", project_dir.join("reports").display()),
        ];
        if self.needs_jtag(image) {
            args.extend(["--device", "/dev/bus/usb"]);
        }
        args.extend([image]);
        args.extend(cmd);
        Command::new("podman").args(args).status().await
    }

    /// GUI 起動 (X11/Wayland フォワード)
    pub async fn run_gui(&self, image: &str) -> Result<Child, RuntimeError> {
        let display = std::env::var("DISPLAY").unwrap_or(":0".into());
        let xauth = std::env::var("XAUTHORITY")
            .unwrap_or_else(|_| format!("{}/.Xauthority", std::env::var("HOME").unwrap()));

        Command::new("podman").args([
            "run", "--rm", "--userns=keep-id",
            "-e", &format!("DISPLAY={display}"),
            "-v", "/tmp/.X11-unix:/tmp/.X11-unix:ro",
            "-v", &format!("{xauth}:{xauth}:ro"),
            "-e", &format!("XAUTHORITY={xauth}"),
            "--security-opt=label=type:container_runtime_t",
            image,
        ]).spawn()
    }
}
```

---

## 3. コンテナ運用パターン（§11.3）

**表 HD-008: コンテナ運用パターン**

| パターン | 用途 | 設定 |
|---------|------|------|
| バッチビルド | 合成・配置配線 | `--rm`, `--network=none`, プロジェクトディレクトリマウント |
| GUI 起動 | Vivado GUI 等 | X11/Wayland フォワード、DISPLAY 環境変数 |
| systemd サービス | 常時起動コンテナ | Podman Quad 定義（.container ファイル） |
| デバイスアクセス | JTAG プログラミング | `--device /dev/bus/usb` |

---

## 4. 対応コンテナイメージ（§11.4）

対応する 8 種類のコンテナイメージの一覧を以下に示す。ベース OS 別の構成、主要ツール、ライセンス取扱、Containerfile サンプルなどの**詳細は §12.2「Containerfile 自動生成」の表 HD-021 を参照**。

**表 HD-009: 対応コンテナイメージ一覧（要約、詳細は §12.2 表 HD-021 参照）**

| イメージ | 主な用途 |
|---------|--------|
| fpga/vivado:2025.2 | AMD/Xilinx FPGA の合成・配置配線（Vivado 2025.2）|
| fpga/quartus:25.1 | Intel/Altera FPGA の合成・配置配線（Quartus Prime Pro 25.1）|
| fpga/efinity:2025.2 | Efinix Trion/Titanium FPGA（Efinity 2025.2）|
| fpga/radiant:2024.2 | Lattice LIFCL/LFD2NX/iCE40 FPGA（Radiant 2024.2）|
| fpga/oss:latest | OSS FPGA フロー（Yosys + nextpnr + icestorm + Verilator）|
| asic/openlane:latest | RTL-to-GDSII 自動化（OpenLane 2 + OpenROAD + Magic + Netgen）|
| pcb/kicad:latest | PCB 設計（KiCad + SKiDL + Freerouting）|
| debug/tools:latest | デバッグ・ロジック解析（OpenOCD + sigrok + PulseView + pyOCD）|

---

## 5. container-manager（§12）

> **container-manager はコンテナ実行を選択した場合のみ使用される**。ローカルにインストールされたツールを使用する場合、本章の機能は適用されず、adapter 経由でローカル実行パスを直接呼び出す。コンテナ実行とローカル実行はツールごとに混在可能（例: Vivado はコンテナ、KiCad はローカル）。

### 5.1 モジュール構成（§12.1）

**表 HD-010: container-manager モジュール構成**

| モジュール | 役割 |
|-----------|------|
| builder | Containerfile 自動生成・ビルド（ツール定義ファイルに基づく） |
| registry | コンテナイメージのレジストリ管理（ローカル/リモート） |
| updater | イメージの差分更新・バージョン管理・自動リビルド |
| provisioner | ツール定義ファイルに基づくプロビジョニング（パッケージインストール） |
| tool_updater | ツールバージョン検出・互換性チェック・段階的更新 |

### 5.2 Containerfile 自動生成（§12.2, FR-CTN-11）

`container.toml`（§3.8 リファレンス）を入力として、minijinja テンプレートエンジンで Containerfile（Dockerfile 互換の Podman ビルド仕様）を自動生成する。

**生成フロー**:

```
container.toml ──▶ builder::parse ──▶ Container Spec (Rust 構造体)
                                        │
                                        ▼
                                   Containerfile テンプレート（minijinja）
                                        │
                                        ▼
                                   Containerfile （テキスト）
                                        │
                                        ▼
                                   podman build（§12.3）
```

**表 HD-021: 8 イメージ × 主要ビルドパラメータ**

| イメージ | ベース | 主要ツール | ライセンス取扱 |
|---------|-------|----------|--------------|
| `fpga/vivado:2025.2` | `registry.access.redhat.com/ubi9/ubi:9.5` | AMD Vivado 2025.2 | FlexLM ライセンスサーバ参照（`LM_LICENSE_FILE`）|
| `fpga/quartus:25.1` | `registry.access.redhat.com/ubi9/ubi:9.5` | Intel Quartus Prime Pro 25.1 | QPF/QSF、FlexLM |
| `fpga/efinity:2025.2` | `docker.io/library/ubuntu:24.04` | Efinix Efinity 2025.2 | Efinity 同梱 Python |
| `fpga/radiant:2024.2` | `registry.access.redhat.com/ubi9/ubi:9.5` | Lattice Radiant 2024.2 | FlexLM |
| `fpga/oss:latest` | `docker.io/library/ubuntu:24.04` | Yosys + nextpnr + icestorm + Verilator | 不要（OSS） |
| `asic/openlane:latest` | `docker.io/library/ubuntu:24.04` | OpenLane 2 + Yosys + OpenROAD + Magic + Netgen | 不要（OSS） |
| `pcb/kicad:latest` | `docker.io/library/ubuntu:24.04` | KiCad + SKiDL + Freerouting | 不要（OSS） |
| `debug/tools:latest` | `docker.io/library/ubuntu:24.04` | OpenOCD + sigrok + PulseView + pyOCD | 不要（OSS）|

### 5.3 プロビジョニング（§12.4, FR-CTN-13）

`container.toml` の `[tools.*]` 宣言から自動的にパッケージマネージャのコマンドに翻訳する。

**翻訳マトリクス**:

| ツール定義 | apt（Debian / Ubuntu）| dnf / yum（RHEL / UBI）| その他 |
|----------|---------------------|---------------------|------|
| `install_method = "apt"` | `apt-get install -y ${package}` | 変換時にエラー | — |
| `install_method = "dnf"` | 変換時にエラー | `dnf install -y ${package}` | — |
| `install_method = "tarball"` | `wget -O - ${url} \| tar -xz -C ${prefix}` | 同左 | — |
| `install_method = "install_script"` | `bash -c "${install_script}"` | 同左 | ベンダー独自インストーラ |
| `install_method = "pip"` | `pip install --no-cache-dir ${package}` | 同左 | Python パッケージ |
| `install_method = "cargo"` | `cargo install ${package}` | 同左 | Rust パッケージ |

**ベンダーライセンス保護**:

- インストーラ・ライセンスファイルはイメージに焼き込まない（`--secret` でマウントのみ）
- 実行時ライセンス（FlexLM 等）は `podman run -e LM_LICENSE_FILE=...` or マウントで注入
- `.hestia/secure/` ディレクトリはモード 0700、`.gitignore` で Git 管理外

**プロビジョニング検証**:

1. インストール完了後、各ツールの `version_cmd` を実行
2. `HEALTHCHECK` 命令で定期検証（`interval=60s`）
3. 失敗時は `action-log` に `prov.failed` を記録、リトライ（最大 3 回、指数バックオフ）

### 5.4 成果物署名・SBOM（§12.5, FR-CTN-14）

**ビルド後の処理パイプライン**:

```
podman build → イメージ生成 → syft で SBOM 生成 → grype で脆弱性スキャン
                                  │                   │
                                  ▼                   ▼
                                sbom.spdx.json       vuln.json
                                  │                   │
                                  └──▶ 評価ゲート ◀───┘
                                        │
                                        ▼
                                   cosign sign（keyless）
                                        │
                                        ▼
                                   podman push（§12.6）
```

**syft による SBOM 生成**: SPDX 形式および CycloneDX 形式の両方に対応。

**grype による脆弱性スキャン**: `--fail-on high --only-fixed` で Critical/High の修正可能脆弱性が存在する場合はビルド失敗とする。

**cosign によるイメージ署名（keyless OIDC）**: GitHub Actions 上でキーレス署名を実行し、SBOM attachment を添付する。

**署名なしイメージの拒否ポリシー**: `podman-runtime::run_build()` の前に `cosign verify` を実行、検証失敗時は `SignatureVerificationError` で起動拒否。開発環境のみ `HESTIA_ALLOW_UNSIGNED=1` で回避可。

### 5.5 レジストリ管理（§12.6, FR-CTN-15）

**表 HD-022: 対応レジストリと用途**

| レジストリ | 用途 | 認証 |
|-----------|-----|------|
| `localhost:5000` | ローカル開発 | なし |
| `ghcr.io/hestia/*` | 公式公開（OSS イメージのみ）| GitHub OIDC |
| `quay.io/hestia/*` | ミラー（OSS イメージのみ）| Robot account |
| プライベート Harbor | 商用ツール入りイメージ | mTLS + Robot account |

**タグ規約**:

```
<registry>/<org>/<conductor>/<tool>:<version>
<registry>/<org>/<conductor>/<tool>:<YYYYMMDD>   # 日付タグ
<registry>/<org>/<conductor>/<tool>:cache        # ビルドキャッシュ専用
```

**Retention ポリシー**:

- `latest` / `<version>` タグ: 永続
- 日付タグ（`YYYYMMDD`）: 直近 30 日分のみ保持、以降は `skopeo delete` で削除
- `cache` タグ: 最新のみ保持
- 自動削除: `skopeo` + cron / GitHub Actions Schedule

**レート制限対策**: Docker Hub のプル制限（匿名 100 pulls/6h）を回避するためプライベートミラー（Harbor）をプルスルーキャッシュとして使用。GitHub Actions では GHCR をプル元にして制限回避。

### 5.6 CI/CD 統合・監視（§12.7, FR-CTN-12 / FR-CTN-14 の延長）

GitHub Actions ワークフロー例（`.github/workflows/container-build.yaml`）:

- **トリガ**: `.hestia/containers/**` / `.hestia/container-manager/**` の push、および毎週月曜 02:00 UTC のスケジュール
- **マトリクス**: fpga/oss / asic/openlane / pcb/kicad / debug/tools の 4 イメージ
- **パーミッション**: `contents: read`, `packages: write`, `id-token: write`（cosign keyless）
- **ステップ**: Install → Build → SBOM → Vulnerability scan → Login to GHCR → Push → Sign

**ObservabilityLayer 連携**（§19.8 / §13.4）:

| メトリクス | 型 | 意味 |
|----------|----|-----|
| `hestia_container_build_total{image,status}` | Counter | ビルド回数（成功 / 失敗）|
| `hestia_container_build_duration_seconds{image,stage}` | Histogram | ステージ別所要時間 |
| `hestia_container_image_size_bytes{image,tag}` | Gauge | イメージサイズ |
| `hestia_container_vuln_total{image,severity}` | Gauge | 脆弱性件数 |
| `hestia_container_signature_verified{image}` | Gauge | 署名検証成功（1/0）|

### 5.7 運用ルール（§12.8）

1. **週次ビルド**: `cron: '0 2 * * 1'`（毎週月曜 02:00 UTC）でベースイメージ + 依存更新を取り込み再ビルド
2. **パッチバージョン**: 自動ビルド + 自動 push
3. **マイナーバージョン**: 自動ビルド後に Review Agent 経由で 1 人承認必須
4. **メジャーバージョン**: 手動トリガ、UpgradeManager の Canary 戦略で段階的展開
5. **失敗時**: `action-log` に `container.build.failed` を記録、PatcherAgent に通知
6. **イメージサイズ上限**: OSS 5GB、商用 20GB
7. **BuildKit secret 管理**: `.hestia/secure/` は read-only、`0700` 権限、Git 管理外
8. **CVE 通知**: `grype` 週次スキャン結果の変化を検知、Critical 発生時は即座にセキュリティチームへエスカレーション

---

## 6. 実装クレート構成（§12.9 要約）

container-manager は `ai-conductor/crates/container-manager/` に配置:

```
container-manager/
├── Cargo.toml
└── src/
    ├── lib.rs
    ├── builder/
    │   ├── mod.rs            # Containerfile 生成 + podman build 呼び出し
    │   ├── templates/*.j2    # minijinja テンプレート
    │   └── parser.rs         # container.toml → ContainerSpec
    ├── registry/
    │   └── mod.rs            # push/pull/retention（skopeo ラッパー）
    ├── updater/
    │   └── mod.rs            # 差分更新 / 自動リビルド
    ├── provisioner/
    │   └── mod.rs            # [tools.*] → インストールコマンド翻訳
    ├── tool_updater/
    │   └── mod.rs            # バージョン検出・semver マッチ
    ├── signer/
    │   └── mod.rs            # cosign ラッパー
    ├── sbom/
    │   └── mod.rs            # syft / grype ラッパー
    └── observability.rs      # メトリクス送信
```

---

## 7. ローカル実行との混在可能性

コンテナ実行とローカル実行はツールごとに混在可能である（例: Vivado はコンテナ、KiCad はローカル）。ローカルに直接インストールされたツールは adapter 経由でローカル実行パスを直接呼び出す。いずれを選択しても fpga.lock / asic.lock による再現性を保証する。

---

## 関連ドキュメント

- [セキュリティ](security.md) — コンテナセキュリティ（rootless / ネットワーク隔離 / 権限昇格防止 / 成果物署名 / API キー保護）
- [アーキテクチャ概要](architecture_overview.md) — 全体アーキテクチャにおけるコンテナ層の位置づけ
- [共有サービス](shared_services.md) — Observability によるメトリクス監視
- `.hestia/doc/container/container_manager.md` — container-manager 詳細仕様
- `.hestia/doc/container/security_hardening.md` — コンテナセキュリティ強化の詳細
- `.hestia/doc/container/vivado_container.md` — Vivado コンテナ詳細
- `.hestia/doc/container/quartus_container.md` — Quartus コンテナ詳細
- `.hestia/doc/container/efinity_container.md` — Efinity コンテナ詳細
- `.hestia/doc/container/radiant_container.md` — Radiant コンテナ詳細
- `.hestia/doc/container/oss_container.md` — OSS コンテナ詳細