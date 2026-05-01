# セキュリティ設計

**対象領域**: セキュリティ
**ソース**: 設計仕様書 §1.2 原則4, §11（Podman コンテナ戦略）, §12.5（成果物署名・SBOM）, §12.8（運用ルール）, §20.4

---

## 1. セキュリティ設計方針

### 原則4: セキュリティ

ツール実行はコンテナ実行とローカル実行のいずれも選択可能とし、用途・運用に応じて使い分ける。コンテナ実行を選択した場合は Podman rootless により非特権実行を実現し、`--network=none` でネットワーク隔離、`--security-opt=no-new-privileges` で権限昇格を防止する。ローカル実行を選択した場合はホスト OS のユーザー権限・SELinux/AppArmor 設定に従う。いずれの実行モードでも知的財産（HDL ソース、ビットストリーム）の保護を徹底する。

---

## 2. Podman rootless による非特権実行

Podman はデーモンレスで動作し、root 権限を必要としない（Docker の dockerd に依存しない）。コンテナはユーザー空間で実行され、ホストの root 権限への影響を排除する。

- **デーモンレス**: システムサービス不要。Docker のように常駐デーモン（dockerd）を必要としない
- **rootless 実行**: ユーザー namespace を利用し、コンテナプロセスをホストユーザー権限で実行
- **Linux ホスト前提**: user namespace / cgroup / SELinux は Linux カーネル機能に依存

---

## 3. ネットワーク隔離（--network=none）

ビルドコンテナは `--network=none` で起動し、外部通信を完全に遮断する。これにより以下のリスクを防止する:

- ビルドプロセス中の意図しない外部通信
- 知的財産のコンテナ外流出
- サプライチェーン攻撃による不正なデータ送信

`podman-runtime::run_build()` では、バッチビルド用コンテナ起動時に必ず `--network=none` を指定する。

---

## 4. 権限昇格防止（--security-opt=no-new-privileges）

`--security-opt=no-new-privileges` を指定し、コンテナ内プロセスがホスト側の root 権限に昇格することを防止する。これにより以下を防止する:

- setuid / setgid バイナリを利用した権限昇格
- コンテナブレイクアウトによるホスト権限の奪取

---

## 5. SELinux 対応（label=type:container_runtime_t）

`--security-opt=label=type:container_runtime_t` を指定し、SELinux が有効なホストでコンテナプロセスに適切なセキュリティコンテキストを割り当てる。GUI 起動（`run_gui()`）時に使用し、X11/Wayland フォワーディングと SELinux を両立させる。

---

## 6. UID 一致（--userns=keep-id）

`--userns=keep-id` により、コンテナ内の UID をホストのユーザー UID と一致させる。これにより以下の問題を回避する:

- コンテナ内で生成されたファイルがホスト側で root 所有となる問題
- ホスト側プロジェクトディレクトリの読み書き権限問題
- ビルド成果物のホスト側アクセス不可能問題

---

## 7. ライセンス保護（読み取り専用マウント）

ベンダーライセンスファイルは読み取り専用（`:ro`）でマウントする。具体的には以下の保護措置を講じる:

- インストーラ・ライセンスファイルはイメージに焼き込まない（`--secret` でマウントのみ）
- 実行時ライセンス（FlexLM 等）は `podman run -e LM_LICENSE_FILE=...` またはマウントで注入
- `.hestia/secure/` ディレクトリはモード 0700、`.gitignore` で Git 管理外

---

## 8. 知的財産保護（HDL ソース・ビットストリームの流出防止）

いずれの実行モード（コンテナ / ローカル）でも知的財産（HDL ソース、ビットストリーム）の保護を徹底する:

- **コンテナ実行時**: `--network=none` でネットワーク遮断、マウント範囲をプロジェクトディレクトリに限定
- **ローカル実行時**: ホスト OS のユーザー権限・SELinux/AppArmor 設定に従う
- **自己学習蓄積時**: HDL ソース / ビットストリーム本体は RAG に格納せず、メタデータと要約のみ蓄積

---

## 9. 成果物署名・SBOM

### 9.1 ビルド後の処理パイプライン

コンテナイメージのビルド後、以下のパイプラインで署名と SBOM 生成を実行する:

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

### 9.2 SBOM 生成（syft）

`syft` により SPDX 形式および CycloneDX 形式の SBOM を生成する:

```bash
# SPDX 形式 SBOM
syft ghcr.io/hestia/fpga/oss:latest -o spdx-json > .hestia/containers/fpga/oss/sbom.spdx.json

# CycloneDX 形式
syft ghcr.io/hestia/fpga/oss:latest -o cyclonedx-json > .hestia/containers/fpga/oss/sbom.cdx.json
```

### 9.3 脆弱性スキャン（grype）

`grype` によりコンテナイメージの脆弱性スキャンを実行する:

```bash
grype ghcr.io/hestia/fpga/oss:latest \
  --fail-on high \
  --only-fixed \
  -o json > .hestia/containers/fpga/oss/vuln.json
```

**評価ゲート（grype 閾値）**:

| 重大度 | 閾値 | 挙動 |
|-------|-----|------|
| Critical | いずれも 0 | **ビルド失敗**（push 不可）|
| High（修正可能）| 0 | **ビルド失敗** |
| High（修正不可）| 記録 | 警告 + 例外承認（Review Agent 経由）|
| Medium 以下 | 記録 | 通常 push 可 |

### 9.4 イメージ署名（cosign keyless OIDC）

`cosign` によるキーレス署名を実行する:

```bash
# GitHub Actions 上でのキーレス署名
COSIGN_EXPERIMENTAL=1 cosign sign ghcr.io/hestia/fpga/oss:latest \
  --attachment sbom.spdx.json

# SBOM attestation
cosign attach sbom --sbom .hestia/containers/fpga/oss/sbom.spdx.json \
  ghcr.io/hestia/fpga/oss:latest
```

**署名なしイメージの拒否ポリシー**:

- `podman-runtime::run_build()` の前に `cosign verify` を実行
- 署名検証失敗時はコンテナ起動を拒否（`SignatureVerificationError`）
- 開発環境のみ `HESTIA_ALLOW_UNSIGNED=1` で回避可（本番は常に検証）

---

## 10. 運用ルール（§12.8）

1. **週次ビルド**: `cron: '0 2 * * 1'`（毎週月曜 02:00 UTC）でベースイメージ + 依存更新を取り込み再ビルド
2. **パッチバージョン（例: 2.4.1 → 2.4.2）**: 自動ビルド + 自動 push（`updater` モジュールによる差分検出）
3. **マイナーバージョン（例: 2.4.x → 2.5.0）**: 自動ビルド後に Review Agent 経由で 1 人承認必須
4. **メジャーバージョン**: 手動トリガ、UpgradeManager の Canary 戦略で段階的展開
5. **失敗時**: `action-log` に `container.build.failed` を記録、PatcherAgent に通知
6. **イメージサイズ上限**: OSS 5GB、商用 20GB。超過時は Review Agent が分割検討
7. **BuildKit secret 管理**: `.hestia/secure/` は read-only、`0700` 権限、Git 管理外
8. **アップストリーム脆弱性（CVE）通知**: `grype` 週次スキャン結果の変化を検知、Critical 発生時は即座にセキュリティチームへエスカレーション

---

## 11. API キー保護（§20.4）

### 11.1 平文 API キー禁止

`config.toml` に直接 `anthropic_api_key = "sk-..."` のように書かない。`security-validation::secrets::audit_text`（31 tests）が API キー直書きを既に検出する仕組みと整合し、本機能はその「入力ガード版」として機能する。

### 11.2 環境変数経由のみ

必ず `anthropic_api_key_env` で環境変数名を指定し、ホスト側で 1Password CLI / `direnv` / systemd EnvironmentFile / GPG 等の secret backend から解決する。

### 11.3 未設定時の明示エラー

環境変数が未設定 / 空の場合、`AgentCliSection::build_env()` は `AgentCliEnvError::MissingApiKeyEnv` を返し、`hestia-runner` / `ai-conductor` は `-32602 Invalid params` で起動前に失敗する（fail-fast）。

### 11.4 ログ出力時のマスキング

子プロセス起動時のログには API キー値そのものを出力せず、`ANTHROPIC_API_KEY=<set, len=N>` のような形式で長さのみ表示する。

### 11.5 agent-cli IPC レジストリ

`registry_dir`（既定 `$XDG_RUNTIME_DIR/agent-cli`）はパーミッション 0700 で作成し、他ユーザーから peer 探索・なりすましを防止する。

---

## 関連ドキュメント

- [アーキテクチャ概要](architecture_overview.md) — 全体設計原則（原則4の概要）
- [コンテナ実行](container_execution.md) — Podman コンテナ戦略・container-manager の詳細
- [共有サービス](shared_services.md) — Observability によるメトリクス監視
- [Hestia Flow](hestia_flow.md) — AI 活用概念におけるセキュリティ関連事項
- `.hestia/doc/container/security_hardening.md` — コンテナセキュリティ強化の詳細