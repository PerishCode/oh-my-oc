#!/bin/sh
set -eu

MODE=${1:-}
RELEASE_VERSION=${2:-}
ARTIFACT_DIR=${3:-}

[ -n "$MODE" ] || { printf '%s\n' 'missing mode' >&2; exit 1; }
[ -n "$RELEASE_VERSION" ] || { printf '%s\n' 'missing release version' >&2; exit 1; }
[ -n "$ARTIFACT_DIR" ] || { printf '%s\n' 'missing artifact dir' >&2; exit 1; }
[ -d "$ARTIFACT_DIR" ] || { printf 'artifact dir missing: %s\n' "$ARTIFACT_DIR" >&2; exit 1; }

require_file() {
  [ -f "$ARTIFACT_DIR/$1" ] || { printf 'missing artifact: %s\n' "$1" >&2; exit 1; }
}

require_checksum_entry() {
  python3 - "$ARTIFACT_DIR/checksums.txt" "$1" <<'PY'
import pathlib
import sys

checksums = pathlib.Path(sys.argv[1]).read_text(encoding='utf-8').splitlines()
name = sys.argv[2]
for line in checksums[1:]:
    parts = line.split()
    if len(parts) >= 2 and parts[-1] == name:
        sys.exit(0)
sys.exit(1)
PY
}

check_archive_members() {
  python3 - "$ARTIFACT_DIR" <<'PY'
import pathlib
import sys
import tarfile
import zipfile

root = pathlib.Path(sys.argv[1])

def ensure_tar_contains(name, member):
    path = root / name
    with tarfile.open(path, 'r:gz') as tar:
        names = tar.getnames()
    if member not in names:
        raise SystemExit(f"missing {member} in {name}")

def ensure_zip_contains(name, member):
    path = root / name
    with zipfile.ZipFile(path) as zf:
        names = zf.namelist()
    if member not in names:
        raise SystemExit(f"missing {member} in {name}")

ensure_tar_contains('oh-my-oc-x86_64-unknown-linux-gnu.tar.gz', 'oh-my-oc')
ensure_tar_contains('oh-my-oc-aarch64-apple-darwin.tar.gz', 'oh-my-oc')
ensure_tar_contains('oh-my-oc-x86_64-apple-darwin.tar.gz', 'oh-my-oc')
ensure_zip_contains('oh-my-oc-x86_64-pc-windows-msvc.zip', 'oh-my-oc.exe')
ensure_tar_contains('skill.tar.gz', 'oh-my-oc/SKILL.md')
ensure_zip_contains('skill.zip', 'oh-my-oc/SKILL.md')
PY
}

case "$MODE" in
  accept)
    require_file checksums.txt
    require_file oh-my-oc-x86_64-unknown-linux-gnu.tar.gz
    require_file oh-my-oc-aarch64-apple-darwin.tar.gz
    require_file oh-my-oc-x86_64-apple-darwin.tar.gz
    require_file oh-my-oc-x86_64-pc-windows-msvc.zip
    require_file skill.zip
    require_file skill.tar.gz
    version_line=$(sed -n 's/^VERSION: *//p' "$ARTIFACT_DIR/checksums.txt" | head -n 1)
    [ "$version_line" = "$RELEASE_VERSION" ] || {
      printf 'version mismatch: expected %s got %s\n' "$RELEASE_VERSION" "$version_line" >&2
      exit 1
    }
    for asset in \
      oh-my-oc-x86_64-unknown-linux-gnu.tar.gz \
      oh-my-oc-aarch64-apple-darwin.tar.gz \
      oh-my-oc-x86_64-apple-darwin.tar.gz \
      oh-my-oc-x86_64-pc-windows-msvc.zip \
      skill.zip \
      skill.tar.gz
    do
      require_checksum_entry "$asset" || {
        printf 'missing checksum entry: %s\n' "$asset" >&2
        exit 1
      }
    done
    ;;
  verify)
    check_archive_members
    ;;
  *)
    printf 'unknown mode: %s\n' "$MODE" >&2
    exit 1
    ;;
esac
