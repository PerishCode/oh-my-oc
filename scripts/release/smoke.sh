#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname "$0")/../.." && pwd)
VERSION=${1:-}
REPO=${GITHUB_REPOSITORY:-PerishCode/oh-my-oc}

[ -n "$VERSION" ] || { printf '%s\n' 'missing release version' >&2; exit 1; }

tmpdir=$(mktemp -d)
trap 'rm -rf "$tmpdir"' EXIT INT TERM

export HOME="$tmpdir/home"
export OH_MY_OC_INSTALL_ROOT="$tmpdir/install"
export OH_MY_OC_LOCAL_BIN_DIR="$tmpdir/bin"
mkdir -p "$HOME" "$OH_MY_OC_INSTALL_ROOT" "$OH_MY_OC_LOCAL_BIN_DIR"

sh "$ROOT/scripts/manage/omo.sh" install --version "$VERSION"

test -x "$OH_MY_OC_INSTALL_ROOT/$VERSION/oh-my-oc"
test -x "$OH_MY_OC_LOCAL_BIN_DIR/oh-my-oc"

skills_dir="$HOME/.agents/skills"
mkdir -p "$skills_dir"
curl -fsSL "https://github.com/$REPO/releases/download/$VERSION/skill.tar.gz" -o "$tmpdir/skill.tar.gz"
tar -xzf "$tmpdir/skill.tar.gz" -C "$skills_dir"
test -f "$skills_dir/oh-my-oc/SKILL.md"

mkdir -p "$tmpdir/target"
"$OH_MY_OC_LOCAL_BIN_DIR/oh-my-oc" patch --path "$tmpdir/target"

test -f "$tmpdir/target/opencode.json"
test -f "$tmpdir/target/agent/commander.md"
