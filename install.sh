#!/usr/bin/env bash
set -euo pipefail

# Hestia — Hardware Engineering Stack for Tool Integration and Automation
# One-line installer: curl -fsSL https://raw.githubusercontent.com/AQUAXIS/hestia/main/install.sh | sh

REPO_URL="https://github.com/AQUAXIS/hestia.git"
REPO_DIR="${HESTIA_REPO_DIR:-${HOME}/.hestia/src/hestia}"
PREFIX="${HESTIA_PREFIX:-/usr/local/bin}"
BRANCH="${HESTIA_BRANCH:-main}"
MSRV="1.75.0"

BOLD='\033[1m'
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m'

info()  { printf "${GREEN}[hestia]${NC} %s\n" "$1"; }
warn()  { printf "${YELLOW}[hestia]${NC} %s\n" "$1"; }
error() { printf "${RED}[hestia]${NC} %s\n" "$1" >&2; exit 1; }

check_rust() {
    if command -v rustc &>/dev/null; then
        local version
        version=$(rustc --version | awk '{print $2}' | tr -d 'v')
        local major minor patch
        IFS='.' read -r major minor patch <<< "$version"
        local min_major min_minor min_patch
        IFS='.' read -r min_major min_minor min_patch <<< "$MSRV"
        if (( major > min_major )) || \
           (( major == min_major && minor > min_minor )) || \
           (( major == min_major && minor == min_minor && patch >= min_patch )); then
            info "Rust $version detected (MSRV: $MSRV)"
            return 0
        fi
        warn "Rust $version found, but MSRV is $MSRV. Upgrading..."
    fi

    info "Installing Rust toolchain via rustup..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable
    # shellcheck source=/dev/null
    source "${HOME}/.cargo/env" 2>/dev/null || true

    if ! command -v rustc &>/dev/null; then
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
    local tools_dir="${REPO_DIR}/.hestia/tools"
    if [ ! -f "${tools_dir}/Cargo.toml" ]; then
        error "Cargo.toml not found at ${tools_dir}. Repository may be corrupted."
    fi

    info "Building Hestia (release mode)..."
    info "This may take a few minutes on first build..."
    cargo build --release --manifest-path "${tools_dir}/Cargo.toml"
}

install_binaries() {
    local src="${REPO_DIR}/.hestia/tools/target/release"
    if [ ! -d "${src}" ]; then
        error "Release binaries not found at ${src}. Did the build succeed?"
    fi

    # Ensure prefix directory exists
    if [ ! -d "${PREFIX}" ]; then
        info "Creating install directory ${PREFIX}..."
        sudo mkdir -p "${PREFIX}" 2>/dev/null || mkdir -p "${PREFIX}"
    fi

    local binaries=()
    for bin in "${src}"/hestia*; do
        # Skip .d dependency files
        [ -f "${bin}" ] || continue
        [[ "${bin}" == *.d ]] && continue
        # Only install actual executables (skip libraries)
        [ -x "${bin}" ] || continue
        binaries+=("$(basename "${bin}")")
    done

    info "Installing ${#binaries[@]} binaries to ${PREFIX}..."
    for name in "${binaries[@]}"; do
        if [ -w "${PREFIX}" ]; then
            cp "${src}/${name}" "${PREFIX}/${name}"
            chmod 755 "${PREFIX}/${name}"
        else
            sudo cp "${src}/${name}" "${PREFIX}/${name}"
            sudo chmod 755 "${PREFIX}/${name}"
        fi
    done

    info "Installed ${#binaries[@]} binaries:"
    for name in "${binaries[@]}"; do
        printf "  - %s\n" "${name}"
    done
}

check_path() {
    local prefix_dir
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
    echo ""
    printf "${BOLD}${GREEN}"
    echo "  _   _           _       "
    echo " | |_| |_  ___ __| | _____"
    echo " |  _| ' \\/ -_) _' |/ / -_)"
    echo "  \\__|_||_\\___\\__,_|_\\_\\___|"
    echo "  Hardware Engineering Stack"
    printf "${NC}"
    echo ""
}

usage() {
    echo "Usage: install.sh [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --prefix DIR    Install binaries to DIR (default: /usr/local/bin)"
    echo "  --branch BRANCH  Git branch to checkout (default: main)"
    echo "  --repo-dir DIR  Directory to clone the repository (default: ~/.hestia/src/hestia)"
    echo "  --skip-build    Skip the build step (use existing binaries)"
    echo "  --skip-clone    Skip the git clone step (use existing repo)"
    echo "  -h, --help      Show this help message"
    echo ""
    echo "Environment variables:"
    echo "  HESTIA_PREFIX     Install prefix (default: /usr/local/bin)"
    echo "  HESTIA_BRANCH     Git branch (default: main)"
    echo "  HESTIA_REPO_DIR   Repository directory (default: ~/.hestia/src/hestia)"
    echo ""
    echo "One-liner:"
    echo "  curl -fsSL https://raw.githubusercontent.com/AQUAXIS/hestia/main/install.sh | sh"
    echo "  curl -fsSL https://raw.githubusercontent.com/AQUAXIS/hestia/main/install.sh | sh -s -- --prefix ~/.local/bin"
}

SKIP_BUILD=false
SKIP_CLONE=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --prefix)   PREFIX="$2"; shift 2 ;;
        --branch)   BRANCH="$2"; shift 2 ;;
        --repo-dir)  REPO_DIR="$2"; shift 2 ;;
        --skip-build) SKIP_BUILD=true; shift ;;
        --skip-clone) SKIP_CLONE=true; shift ;;
        -h|--help)   usage; exit 0 ;;
        *) error "Unknown option: $1. Use --help for usage." ;;
    esac
done

main() {
    print_banner
    check_rust

    if [ "$SKIP_CLONE" = false ]; then
        clone_repo
    fi

    if [ "$SKIP_BUILD" = false ]; then
        build_release
    fi

    install_binaries
    check_path

    echo ""
    info "${BOLD}Hestia installed successfully!${NC}"
    echo "  Run 'hestia init' to initialize a project."
    echo "  Run 'hestia --help' for available commands."
    echo ""
}

main "$@"