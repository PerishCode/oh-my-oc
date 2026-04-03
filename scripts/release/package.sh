#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)
APP_DIR="$ROOT/app"
NAME=oh-my-oc
VERSION=$(sed -n 's/^version = "\(.*\)"$/\1/p' "$APP_DIR/Cargo.toml" | head -n 1)
RELEASE_VERSION=${1:-${RELEASE_VERSION:-$VERSION}}
TARGET=${TARGET:-$(rustc -Vv | sed -n 's/^host: //p')}
DIST_DIR=${DIST_DIR:-"$ROOT/dist"}
ARTIFACT_DIR="$DIST_DIR/$RELEASE_VERSION"
BIN="$APP_DIR/target/release/$NAME"

mkdir -p "$ARTIFACT_DIR"
cargo build --release --manifest-path "$APP_DIR/Cargo.toml"
TARBALL="$NAME-$TARGET.tar.gz"
SKILL_ZIP="skill.zip"

tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT INT TERM
cp "$BIN" "$tmpdir/$NAME"
tar -C "$tmpdir" -czf "$ARTIFACT_DIR/$TARBALL" "$NAME"
mkdir -p "$tmpdir/oh-my-oc"
cp "$ROOT/artifacts/skill/oh-my-oc/SKILL.md" "$tmpdir/oh-my-oc/SKILL.md"
(cd "$tmpdir" && zip -qr "$ARTIFACT_DIR/$SKILL_ZIP" oh-my-oc)
(
  cd "$ARTIFACT_DIR"
  printf 'VERSION: %s\n' "$RELEASE_VERSION" > checksums.txt
  shasum -a 256 "$TARBALL" >> checksums.txt
  shasum -a 256 "$SKILL_ZIP" >> checksums.txt
)

printf '%s\n' "$ARTIFACT_DIR/$TARBALL"
printf '%s\n' "$ARTIFACT_DIR/$SKILL_ZIP"
printf '%s\n' "$ARTIFACT_DIR/checksums.txt"
