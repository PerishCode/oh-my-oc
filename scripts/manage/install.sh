#!/bin/sh
set -eu

NAME=${OH_MY_OC_NAME:-oh-my-oc}
REPO=${OH_MY_OC_REPO:-PerishCode/oh-my-oc}
BASE_URL=${OH_MY_OC_BASE_URL:-https://github.com/$REPO/releases/download}
GITHUB_TOKEN=${OH_MY_OC_GITHUB_TOKEN:-}
INSTALL_ROOT=${OH_MY_OC_INSTALL_ROOT:-"$HOME/.local/share/$NAME"}
LOCAL_BIN_DIR=${OH_MY_OC_LOCAL_BIN_DIR:-"$HOME/.local/bin"}
VERSION=${OH_MY_OC_VERSION:-}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --version)
      shift
      VERSION=${1:-}
      [ -n "$VERSION" ] || { printf '%s\n' "missing value for --version" >&2; exit 1; }
      ;;
    *)
      printf '%s\n' "unknown argument: $1" >&2
      exit 1
      ;;
  esac
  shift
done

if [ -z "$VERSION" ]; then
  VERSION=$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" | sed -n 's/.*"tag_name": *"\([^"]*\)".*/\1/p' | head -n 1)
fi

[ -n "$VERSION" ] || { printf '%s\n' "could not resolve release version" >&2; exit 1; }

case "$(uname -s)-$(uname -m)" in
  Darwin-arm64)
    TARGET=aarch64-apple-darwin
    ;;
  Darwin-x86_64)
    TARGET=x86_64-apple-darwin
    ;;
  Linux-x86_64)
    TARGET=x86_64-unknown-linux-gnu
    ;;
  *)
    printf '%s\n' "unsupported host target: $(uname -s)-$(uname -m)" >&2
    exit 1
    ;;
esac

INSTALL_DIR="$INSTALL_ROOT/$VERSION"
mkdir -p "$INSTALL_DIR" "$LOCAL_BIN_DIR"

tarball="$NAME-$VERSION-$TARGET.tar.gz"
checksums_url="$BASE_URL/$VERSION/checksums.txt"
tarball_url="$BASE_URL/$VERSION/$tarball"

if [ -n "$GITHUB_TOKEN" ] && [ "${OH_MY_OC_BASE_URL+x}" != x ]; then
  api_headers="-H Authorization: Bearer $GITHUB_TOKEN -H Accept: application/vnd.github+json -H X-GitHub-Api-Version: 2022-11-28"
  release_json=$(curl -fsSL $api_headers "https://api.github.com/repos/$REPO/releases/tags/$VERSION")
  checksums_url=$(printf '%s\n' "$release_json" | sed -n "s/.*\"name\": *\"checksums.txt\".*\"browser_download_url\": *\"\([^\"]*\)\".*/\1/p" | head -n 1)
  tarball_url=$(printf '%s\n' "$release_json" | sed -n "s/.*\"name\": *\"$tarball\".*\"browser_download_url\": *\"\([^\"]*\)\".*/\1/p" | head -n 1)
fi

tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT INT TERM

curl -fsSL "$checksums_url" -o "$tmpdir/checksums.txt"
curl -fsSL "$tarball_url" -o "$tmpdir/$tarball"

expected=$(awk -v file="$tarball" '$2 == file { print $1 }' "$tmpdir/checksums.txt")
if command -v sha256sum >/dev/null 2>&1; then
  actual=$(sha256sum "$tmpdir/$tarball" | awk '{ print $1 }')
elif command -v shasum >/dev/null 2>&1; then
  actual=$(shasum -a 256 "$tmpdir/$tarball" | awk '{ print $1 }')
else
  printf '%s\n' "missing checksum tool: sha256sum or shasum" >&2
  exit 1
fi

if [ -z "$expected" ]; then
  printf '%s\n' "artifact unavailable: $tarball" >&2
  exit 1
fi

if [ "$expected" != "$actual" ]; then
  printf '%s\n' "checksum verification failed" >&2
  exit 1
fi

tar -xzf "$tmpdir/$tarball" -C "$tmpdir"
install -m 755 "$tmpdir/$NAME" "$INSTALL_DIR/$NAME"
ln -sf "$INSTALL_DIR/$NAME" "$LOCAL_BIN_DIR/$NAME"

printf '%s\n' "$LOCAL_BIN_DIR/$NAME"
