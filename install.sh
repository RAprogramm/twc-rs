#!/bin/sh
# twc-rs installer
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/RAprogramm/twc-rs/main/install.sh | sh
#
# Downloads the latest twc-rs release binary that matches your OS/arch and
# installs it to ~/.local/bin (or /usr/local/bin when that is writable).

set -eu

REPO="RAprogramm/twc-rs"
BIN="twc-rs"

err() {
    printf 'error: %s\n' "$1" >&2
    exit 1
}

info() {
    printf '%s\n' "$1" >&2
}

need() {
    command -v "$1" >/dev/null 2>&1 || err "required command not found: $1"
}

# --- Pick a downloader -------------------------------------------------------
if command -v curl >/dev/null 2>&1; then
    DL="curl -fsSL"
    DL_OUT="curl -fsSL -o"
elif command -v wget >/dev/null 2>&1; then
    DL="wget -qO-"
    DL_OUT="wget -qO"
else
    err "neither curl nor wget is available"
fi

need tar
need mktemp

# --- Detect platform ---------------------------------------------------------
os="$(uname -s)"
arch="$(uname -m)"

case "$os" in
    Linux) os_part="unknown-linux-gnu" ;;
    Darwin) os_part="apple-darwin" ;;
    *) err "unsupported OS: $os (supported: Linux, macOS)" ;;
esac

case "$arch" in
    x86_64 | amd64) arch_part="x86_64" ;;
    aarch64 | arm64) arch_part="aarch64" ;;
    *) err "unsupported architecture: $arch (supported: x86_64, aarch64/arm64)" ;;
esac

TARGET="${arch_part}-${os_part}"

# --- Resolve latest release tag ----------------------------------------------
info "Resolving latest release of ${REPO}..."
api_url="https://api.github.com/repos/${REPO}/releases/latest"
tag="$(
    $DL "$api_url" 2>/dev/null \
        | grep '"tag_name"' \
        | head -n1 \
        | sed -e 's/.*"tag_name"[[:space:]]*:[[:space:]]*"//' -e 's/".*//'
)"
[ -n "$tag" ] || err "could not determine the latest release tag from $api_url"

asset="${BIN}-${tag}-${TARGET}.tar.gz"
url="https://github.com/${REPO}/releases/download/${tag}/${asset}"

info "Latest release: ${tag}"
info "Target:         ${TARGET}"
info "Downloading:    ${url}"

# --- Download and extract ----------------------------------------------------
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT INT TERM

$DL_OUT "$tmp/$asset" "$url" || err "download failed: $url"
tar -xzf "$tmp/$asset" -C "$tmp" || err "failed to extract $asset"

if [ ! -f "$tmp/$BIN" ]; then
    found="$(find "$tmp" -type f -name "$BIN" | head -n1 || true)"
    [ -n "$found" ] || err "binary '$BIN' not found in archive"
    mv "$found" "$tmp/$BIN"
fi
chmod +x "$tmp/$BIN"

# --- Choose install dir ------------------------------------------------------
if [ -d "/usr/local/bin" ] && [ -w "/usr/local/bin" ]; then
    dest="/usr/local/bin"
else
    dest="${HOME}/.local/bin"
    mkdir -p "$dest"
fi

mv "$tmp/$BIN" "$dest/$BIN" || err "failed to install to $dest"
chmod +x "$dest/$BIN"

info ""
info "Installed ${BIN} ${tag} to ${dest}/${BIN}"

# --- Next steps --------------------------------------------------------------
case ":${PATH}:" in
    *":${dest}:"*) ;;
    *)
        info ""
        info "NOTE: ${dest} is not on your PATH. Add it, e.g.:"
        info "  export PATH=\"${dest}:\$PATH\""
        ;;
esac

info ""
info "Get started:"
info "  ${BIN} --version"
info "  ${BIN} auth flow        # authenticate"
info "  ${BIN} dashboard        # interactive TUI"
