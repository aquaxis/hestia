#!/usr/bin/env bash
# Phase 127 — hestia リリーススクリプト
#
# Usage:
#   scripts/release.sh <new_version>     # 例: scripts/release.sh 0.1.6
#
# 動作:
#   1. .hestia/tools/Cargo.toml の [workspace.package] version を <new> に更新
#   2. cargo build --release で Cargo.lock を再生成
#   3. git add → commit "Release v<new>" → tag v<new>
#   4. push はユーザに案内（自動 push はしない、git 安全プロトコル準拠）

set -euo pipefail

if [ "$#" -ne 1 ]; then
    echo "Usage: $0 <new_version>  (e.g. 0.1.6)" >&2
    exit 1
fi

NEW="$1"

# semver 形式チェック (X.Y.Z)
if ! [[ "${NEW}" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]]; then
    echo "ERROR: <new_version> must be semver (X.Y.Z), got: ${NEW}" >&2
    exit 1
fi

ROOT="$(git rev-parse --show-toplevel)"
TOOLS="${ROOT}/.hestia/tools"
CARGO_TOML="${TOOLS}/Cargo.toml"
TAG="v${NEW}"

if [ ! -f "${CARGO_TOML}" ]; then
    echo "ERROR: ${CARGO_TOML} not found" >&2
    exit 1
fi

# tag 重複チェック
if git -C "${ROOT}" rev-parse -q --verify "refs/tags/${TAG}" >/dev/null 2>&1; then
    echo "ERROR: tag ${TAG} already exists" >&2
    exit 1
fi

# 1. workspace version 書換 (Cargo.toml 内 [workspace.package] 直下の最初の
#    `version = "X.Y.Z"` 行のみ置換)
echo "==> Updating ${CARGO_TOML} workspace version → ${NEW}"
awk -v new="${NEW}" '
    /^\[workspace\.package\]/ { in_section=1; print; next }
    /^\[/ && !/^\[workspace\.package\]/ { in_section=0 }
    in_section && /^version = "[0-9]+\.[0-9]+\.[0-9]+"/ {
        print "version = \"" new "\""
        next
    }
    { print }
' "${CARGO_TOML}" > "${CARGO_TOML}.new"
mv "${CARGO_TOML}.new" "${CARGO_TOML}"

# 2. Cargo.lock 再生成
echo "==> Rebuilding to refresh Cargo.lock"
( cd "${TOOLS}" && cargo build --release --quiet )

# 3. commit + tag
cd "${ROOT}"
echo "==> git commit + tag"
git add .hestia/tools/Cargo.toml .hestia/tools/Cargo.lock
git commit -m "Release ${TAG}"
git tag "${TAG}"

# 4. ユーザ案内
echo
echo "✅ Release ${TAG} prepared locally."
echo "Next: git push origin main --follow-tags"
