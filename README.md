# oh-my-oc

Personal Opencode configuration and small Rust CLI workspace.

## Layout

- `app/` - minimal distributable Rust CLI surface for `oh-my-oc`
- `scripts/release/package.sh` - local packaging helper for release artifacts
- `scripts/manage/install.sh` - install script for tarball-based installs
- root config - Opencode setup and agent definitions

## Local install loop

The release path is intentionally simple:

1. Run `scripts/release/package.sh <tag>` from the repo root to build the CLI and create a release tarball plus `checksums.txt` under `dist/<tag>/`.
2. Publish those files as GitHub release assets.
3. Install with `curl -fsSL https://raw.githubusercontent.com/PerishCode/oh-my-oc/main/scripts/manage/install.sh | sh`, or pass `--version <tag>` / `OH_MY_OC_VERSION=<tag>` to pin a release.

The installer uses `OH_MY_OC_REPO`, `OH_MY_OC_BASE_URL`, `OH_MY_OC_INSTALL_ROOT`, and `OH_MY_OC_LOCAL_BIN_DIR` when you need to override defaults.

Latest mode fetches release assets from the GitHub Releases `latest/download/` path.

Release assets are produced for Linux x86_64 and macOS x86_64/aarch64 only.

## `oh-my-oc patch`

Install the current release, then apply the patch into your Opencode config:

```sh
curl -fsSL https://raw.githubusercontent.com/PerishCode/oh-my-oc/main/scripts/manage/install.sh | sh
oh-my-oc patch
```

Examples:

```sh
oh-my-oc patch --path ~/.config/opencode
oh-my-oc patch --force
```

Notes:

- Default target: `~/.config/opencode`
- Override target path with `--path` or `OH_MY_OC_PATCH_PATH`
- `--version` selects the resource release tarball to fetch
- Override version with `--version` or `OH_MY_OC_PATCH_VERSION`
- The patch flow downloads the official `PerishCode/resources` release tarball `oh-my-oc-<version>.tar.gz`
- The archive is expected to unpack with a top-level `oh-my-oc/opencode/` directory
- `patch` only writes or overwrites managed files in `opencode.json` and `agent/*.md`

Flags win over env vars, and env vars win over defaults.
