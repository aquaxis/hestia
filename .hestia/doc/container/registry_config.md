# レジストリ管理

**対象領域**: container — レジストリ
**ソース**: 設計仕様書 §12.6

## 概要

コンテナイメージのレジストリ管理。ローカル開発用レジストリとリモートレジストリ（GHCR / Quay / プライベート Harbor）を使い分け、タグ規約と Retention ポリシーで運用効率化する。

## 対応レジストリ

| レジストリ | 用途 | 認証 |
|-----------|-----|------|
| `localhost:5000` | ローカル開発 | なし |
| `ghcr.io/hestia/*` | 公式公開（OSS イメージのみ）| GitHub OIDC |
| `quay.io/hestia/*` | ミラー（OSS イメージのみ）| Robot account |
| プライベート Harbor | 商用ツール入りイメージ | mTLS + Robot account |

## タグ規約

```
<registry>/<org>/<conductor>/<tool>:<version>
<registry>/<org>/<conductor>/<tool>:<YYYYMMDD>   # 日付タグ
<registry>/<org>/<conductor>/<tool>:cache        # ビルドキャッシュ専用
```

例:
- `ghcr.io/hestia/fpga/vivado:2025.2`
- `ghcr.io/hestia/fpga/oss:20260423`
- `ghcr.io/hestia/fpga/oss:cache`

## push / pull フロー

```bash
# 認証
podman login ghcr.io -u ${GITHUB_USER} --password-stdin < token

# push
podman push ghcr.io/hestia/fpga/oss:latest
podman push ghcr.io/hestia/fpga/oss:20260423

# プルスルーキャッシュ（Harbor 経由）
podman pull harbor.internal/docker-proxy/docker.io/library/ubuntu:24.04
```

## Retention ポリシー

| タグ種別 | 保持期間 | 削除方法 |
|---------|---------|---------|
| `latest` / `<version>` | 永続 | 手動 |
| 日付タグ（`YYYYMMDD`）| 直近 30 日分のみ | `skopeo delete` + cron |
| `cache` | 最新のみ | 自動 |

```bash
# retention スクリプト例
skopeo list-tags docker://ghcr.io/hestia/fpga/oss | \
  jq -r '.Tags[] | select(test("^[0-9]{8}$"))' | \
  while read tag; do
    tag_date=$(date -d "${tag:0:4}-${tag:4:2}-${tag:6:2}" +%s 2>/dev/null) || continue
    cutoff=$(date -d "30 days ago" +%s)
    if [ "$tag_date" -lt "$cutoff" ]; then
      skopeo delete docker://ghcr.io/hestia/fpga/oss:$tag
    fi
  done
```

## レート制限対策

- Docker Hub プル制限（匿名 100 pulls/6h）を回避するためプライベートミラー（Harbor）を使用
- GitHub Actions では GHCR をプル元にして制限回避
- `podman pull --quiet` で帯域節約、`--retry 5` で一時失敗リカバー

## 関連ドキュメント

- [container_manager.md](container_manager.md) — container-manager 全体
- [security_hardening.md](security_hardening.md) — 署名・SBOM