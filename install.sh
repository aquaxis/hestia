#!/bin/sh
set -eu

# Hestia — Hardware Engineering Stack for Tool Integration and Automation
# One-line installer: curl -fsSL https://raw.githubusercontent.com/AQUAXIS/hestia/main/install.sh | sh

REPO_URL="https://github.com/AQUAXIS/hestia.git"
REPO_DIR="${HESTIA_REPO_DIR:-${HOME}/.hestia/src/hestia}"
PREFIX="${HESTIA_PREFIX:-${HOME}/.local/bin}"
SHARE_DIR="${HESTIA_SHARE_DIR:-${HOME}/.hestia/share}"
BRANCH="${HESTIA_BRANCH:-main}"
MSRV="1.75.0"

info()  { printf "\033[0;32m[hestia]\033[0m %s\n" "$1"; }
warn()  { printf "\033[0;33m[hestia]\033[0m %s\n" "$1"; }
error() { printf "\033[0;31m[hestia]\033[0m %s\n" "$1" >&2; exit 1; }

version_gte() {
    # $1 = installed, $2 = minimum (x.y.z format)
    _v1_major="${1%%.*}"; _rest="${1#*.}"; _v1_minor="${_rest%%.*}"; _v1_patch="${_rest#*.}"
    _v2_major="${2%%.*}"; _rest="${2#*.}"; _v2_minor="${_rest%%.*}"; _v2_patch="${_rest#*.}"
    [ "$_v1_major" -gt "$_v2_major" ] && return 0
    [ "$_v1_major" -lt "$_v2_major" ] && return 1
    [ "$_v1_minor" -gt "$_v2_minor" ] && return 0
    [ "$_v1_minor" -lt "$_v2_minor" ] && return 1
    [ "$_v1_patch" -ge "$_v2_patch" ] && return 0
    return 1
}

check_rust() {
    if command -v rustc >/dev/null 2>&1; then
        version=$(rustc --version | awk '{print $2}' | tr -d 'v')
        if version_gte "$version" "$MSRV"; then
            info "Rust $version detected (MSRV: $MSRV)"
            return 0
        fi
        warn "Rust $version found, but MSRV is $MSRV. Upgrading..."
    fi

    info "Installing Rust toolchain via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    . "${HOME}/.cargo/env" 2>/dev/null || true

    if ! command -v rustc >/dev/null 2>&1; then
        error "rustc not found after rustup install. Please add ~/.cargo/bin to your PATH and retry."
    fi
    info "Rust $(rustc --version | awk '{print $2}') installed."
}

clone_repo() {
    if [ -d "${REPO_DIR}/.git" ]; then
        info "Repository already exists at ${REPO_DIR}. Pulling latest..."
        git -C "${REPO_DIR}" pull --ff-only || warn "git pull failed, using existing checkout"
    else
        info "Cloning Hestia repository to ${REPO_DIR}..."
        git clone --depth 1 --branch "${BRANCH}" "${REPO_URL}" "${REPO_DIR}"
    fi
}

build_release() {
    tools_dir="${REPO_DIR}/.hestia/tools"
    if [ ! -f "${tools_dir}/Cargo.toml" ]; then
        error "Cargo.toml not found at ${tools_dir}. Repository may be corrupted."
    fi

    info "Building Hestia (release mode)..."
    info "This may take a few minutes on first build..."
    cargo build --release --manifest-path "${tools_dir}/Cargo.toml"
}

install_binaries() {
    src="${REPO_DIR}/.hestia/tools/target/release"
    if [ ! -d "${src}" ]; then
        error "Release binaries not found at ${src}. Did the build succeed?"
    fi

    if [ ! -d "${PREFIX}" ]; then
        info "Creating install directory ${PREFIX}..."
        mkdir -p "${PREFIX}" 2>/dev/null || sudo mkdir -p "${PREFIX}"
    fi

    count=0
    for bin in "${src}"/hestia*; do
        [ -f "${bin}" ] || continue
        case "${bin}" in *.d) continue ;; esac
        [ -x "${bin}" ] || continue

        name=$(basename "${bin}")
        if [ -w "${PREFIX}" ]; then
            cp "${src}/${name}" "${PREFIX}/${name}"
            chmod 755 "${PREFIX}/${name}"
        else
            sudo cp "${src}/${name}" "${PREFIX}/${name}"
            sudo chmod 755 "${PREFIX}/${name}"
        fi
        printf "  - %s\n" "${name}"
        count=$((count + 1))
    done

    info "Installed ${count} binaries to ${PREFIX}"
}

install_data() {
    personas_src="${REPO_DIR}/.hestia/personas"
    if [ ! -d "${personas_src}" ]; then
        warn "Persona files not found at ${personas_src}. Skipping data installation."
        return
    fi

    info "Installing persona files to ${SHARE_DIR}/personas/..."
    mkdir -p "${SHARE_DIR}/personas"

    count=0
    for file in "${personas_src}"/*.md; do
        [ -f "${file}" ] || continue
        cp "${file}" "${SHARE_DIR}/personas/$(basename "${file}")"
        count=$((count + 1))
    done

    info "Installed ${count} persona files to ${SHARE_DIR}/personas/"
}

check_path() {
    prefix_dir=$(dirname "${PREFIX}/hestia")
    case ":${PATH}:" in
        *":${prefix_dir}:"*) return 0 ;;
        *)
            warn "${prefix_dir} is not in your PATH."
            echo ""
            echo "  Add it to your shell profile:"
            echo ""
            if [ -f "${HOME}/.zshrc" ]; then
                echo "    echo 'export PATH=\"${prefix_dir}:\$PATH\"' >> ~/.zshrc"
                echo "    source ~/.zshrc"
            else
                echo "    echo 'export PATH=\"${prefix_dir}:\$PATH\"' >> ~/.bashrc"
                echo "    source ~/.bashrc"
            fi
            echo ""
            return 1
            ;;
    esac
}

print_banner() {
    printf "\033[1m\033[0;32m[hestia]\033[0m Hardware Engineering Stack for Tool Integration and Automation\n"
}

usage() {
    echo "Usage: install.sh [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --prefix DIR     Install binaries to DIR (default: ~/.local/bin)"
    echo "  --branch BRANCH  Git branch to checkout (default: main)"
    echo "  --repo-dir DIR   Directory to clone the repository (default: ~/.hestia/src/hestia)"
    echo "  --skip-build     Skip the build step (use existing binaries)"
    echo "  --skip-clone     Skip the git clone step (use existing repo)"
    echo "  -h, --help       Show this help message"
    echo ""
    echo "Environment variables:"
    echo "  HESTIA_PREFIX     Install prefix (default: ~/.local/bin)"
    echo "  HESTIA_BRANCH     Git branch (default: main)"
    echo "  HESTIA_REPO_DIR   Repository directory (default: ~/.hestia/src/hestia)"
    echo "  HESTIA_SHARE_DIR  Data directory (default: ~/.hestia/share)"
    echo ""
    echo "One-liner:"
    echo "  curl -fsSL https://raw.githubusercontent.com/AQUAXIS/hestia/main/install.sh | sh"
    echo "  curl -fsSL https://raw.githubusercontent.com/AQUAXIS/hestia/main/install.sh | sh -s -- --prefix ~/.local/bin"
}

SKIP_BUILD=false
SKIP_CLONE=false

while [ $# -gt 0 ]; do
    case "$1" in
        --prefix)     PREFIX="$2"; shift 2 ;;
        --branch)     BRANCH="$2"; shift 2 ;;
        --repo-dir)   REPO_DIR="$2"; shift 2 ;;
        --skip-build) SKIP_BUILD=true; shift ;;
        --skip-clone) SKIP_CLONE=true; shift ;;
        -h|--help)     usage; exit 0 ;;
        *) error "Unknown option: $1. Use --help for usage." ;;
    esac
done

print_banner
check_rust

if [ "$SKIP_CLONE" = false ]; then
    clone_repo
fi

if [ "$SKIP_BUILD" = false ]; then
    build_release
fi

install_binaries
install_data
check_path

echo ""
info "Hestia installed successfully!"
echo "  Run 'hestia init' to initialize a project."
echo "  Run 'hestia --help' for available commands."
echo ""