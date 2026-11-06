#!/bin/sh
# typedef installer. No sudo, no shell-profile edits, no Rust toolchain required.
set -eu

main() {
    version=${TYPEDEF_VERSION:-v0.5.2}
    bin_dir=${TYPEDEF_INSTALL_DIR:-${HOME:?HOME is not set}/.local/bin}
    while [ "$#" -gt 0 ]; do
        case "$1" in
            --version|--bin-dir)
                [ "$#" -ge 2 ] || { printf '%s\n' "Missing value for $1" >&2; exit 1; }
                if [ "$1" = --version ]; then version=$2; else bin_dir=$2; fi
                shift 2 ;;
            --help|-h)
                printf '%s\n' 'Install typedef and tpdf from verified GitHub release binaries.' \
                    'Usage: sh install.sh [--version v0.5.2] [--bin-dir PATH]' \
                    'Default directory: ~/.local/bin. No sudo or shell configuration changes.'
                return ;;
            *) printf 'Unknown argument: %s\n' "$1" >&2; exit 1 ;;
        esac
    done
    printf '%s\n' "$version" | grep -Eq '^v[0-9]+\.[0-9]+\.[0-9]+$' || {
        printf '%s\n' 'Version must look like v0.5.2' >&2; exit 1;
    }
    case "$bin_dir" in /*) ;; *) printf '%s\n' 'Install directory must be an absolute path' >&2; exit 1 ;; esac
    for tool in curl tar mktemp uname; do
        command -v "$tool" >/dev/null 2>&1 || { printf 'Required command not found: %s\n' "$tool" >&2; exit 1; }
    done
    case "$(uname -s)" in
        Darwin) platform=apple-darwin ;;
        Linux) platform=unknown-linux-musl ;;
        *) printf '%s\n' 'This installer supports macOS and Linux. Windows ZIPs are on GitHub Releases.' >&2; exit 1 ;;
    esac
    case "$(uname -m)" in
        arm64|aarch64) arch=aarch64 ;;
        x86_64|amd64) arch=x86_64 ;;
        *) printf '%s\n' 'Unsupported CPU. Build from source with Cargo.' >&2; exit 1 ;;
    esac
    if command -v sha256sum >/dev/null 2>&1; then
        checksum=sha256sum
    elif command -v shasum >/dev/null 2>&1; then
        checksum=shasum
    else
        printf '%s\n' 'SHA-256 verification requires sha256sum or shasum.' >&2; exit 1
    fi
    tmp_dir=$(mktemp -d "${TMPDIR:-/tmp}/typedef-install.XXXXXXXX")
    trap 'rm -rf "$tmp_dir"' EXIT
    trap 'exit 130' INT
    trap 'exit 143' TERM
    archive="typedef-${version}-${arch}-${platform}.tar.gz"
    base="https://github.com/rawatpriyansh17/typedef/releases/download/${version}"
    printf 'Downloading typedef %s for %s…\n' "$version" "$arch-$platform"
    curl --proto '=https' --tlsv1.2 --fail --silent --show-error --location --retry 3 "$base/$archive" -o "$tmp_dir/$archive"
    curl --proto '=https' --tlsv1.2 --fail --silent --show-error --location --retry 3 "$base/$archive.sha256" -o "$tmp_dir/checksum"
    expected=$(awk 'NR == 1 { print $1 }' "$tmp_dir/checksum")
    case "$expected" in ''|*[!a-fA-F0-9]*) printf '%s\n' 'Invalid release checksum' >&2; exit 1 ;; esac
    [ "${#expected}" -eq 64 ] || { printf '%s\n' 'Invalid release checksum length' >&2; exit 1; }
    if [ "$checksum" = sha256sum ]; then
        actual=$(sha256sum "$tmp_dir/$archive" | awk '{print $1}')
    else
        actual=$(shasum -a 256 "$tmp_dir/$archive" | awk '{print $1}')
    fi
    [ "$actual" = "$expected" ] || { printf '%s\n' 'Checksum mismatch; nothing installed.' >&2; exit 1; }
    # Extract only the two known executable members after verifying the archive.
    tar -xzf "$tmp_dir/$archive" -C "$tmp_dir" typedef tpdf
    for binary in typedef tpdf; do
        [ -f "$tmp_dir/$binary" ] && [ ! -L "$tmp_dir/$binary" ] || {
            printf '%s\n' 'Invalid release archive; nothing installed.' >&2; exit 1;
        }
    done
    mkdir -p "$bin_dir"
    for binary in typedef tpdf; do
        [ ! -d "$bin_dir/$binary" ] || { printf 'Destination is a directory: %s\n' "$bin_dir/$binary" >&2; exit 1; }
    done
    # Stage inside the destination filesystem, then replace rather than follow symlinks.
    for binary in typedef tpdf; do
        staged=$(mktemp "$bin_dir/.${binary}.XXXXXXXX")
        if ! cp "$tmp_dir/$binary" "$staged" || ! chmod 755 "$staged" || ! mv -f "$staged" "$bin_dir/$binary"; then
            rm -f "$staged"; printf 'Could not install %s\n' "$binary" >&2; exit 1
        fi
    done
    printf 'Installed typedef and tpdf in %s\n' "$bin_dir"
    case ":${PATH:-}:" in
        *":$bin_dir:"*) printf '%s\n' 'Start practicing: tpdf' ;;
        *) printf 'Add this directory to your PATH: %s\n' "$bin_dir"
           printf 'Or run directly: "%s/tpdf"\n' "$bin_dir" ;;
    esac
}

main "$@"
