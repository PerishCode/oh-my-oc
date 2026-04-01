# oh-my-oc

Personal Opencode configuration and small Rust CLI workspace.

## Layout

- `resources/` - lightweight reference material and supporting content
- `app/` - minimal distributable Rust CLI surface for `oh-my-oc`
- `scripts/release/package.sh` - local packaging helper for release artifacts
- `scripts/manage/install.sh` - install script for tarball-based installs
- root config - Opencode setup and agent definitions

## Local install loop

The release path is intentionally simple:

1. Run `scripts/release/package.sh <tag>` from the repo root to build the CLI and create a release tarball plus `checksums.txt` under `dist/<tag>/`.
2. Publish those files as GitHub release assets.
3. Install with `curl -fsSL https://raw.githubusercontent.com/PerishCode/oh-my-oc/main/scripts/manage/install.sh | sh`, or pass `--version <tag>` / `OH_MY_OC_VERSION=<tag>` to pin a release.

For a private repo, fetch the installer with GitHub auth and set `OH_MY_OC_GITHUB_TOKEN` so it can resolve and download private release assets:

```sh
OH_MY_OC_GITHUB_TOKEN=... curl -fsSL https://raw.githubusercontent.com/PerishCode/oh-my-oc/main/scripts/manage/install.sh | sh -s -- --version v0.2.1
```

The installer uses `OH_MY_OC_REPO`, `OH_MY_OC_BASE_URL`, `OH_MY_OC_GITHUB_TOKEN`, `OH_MY_OC_INSTALL_ROOT`, and `OH_MY_OC_LOCAL_BIN_DIR` when you need to override defaults.

Release assets are produced for Linux x86_64 and macOS x86_64/aarch64 only.

## `oh-my-oc patch`

Install the current release, then apply the bundled patch into your Opencode config:

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
- `--version` selects a bundled patch resource when available in this build
- Override version with `--version` or `OH_MY_OC_PATCH_VERSION`
- Set `OH_MY_OC_PATCH_RESOURCE_URL_TEMPLATE` to fetch managed files from a URL template using `{version}` and `{path}` placeholders
- Public resources can be sourced from `PerishCode/resources`, for example:

```sh
OH_MY_OC_PATCH_RESOURCE_URL_TEMPLATE='https://raw.githubusercontent.com/PerishCode/resources/{version}/sources/oh-my-oc/opencode/{path}' \
oh-my-oc patch --version v0.1.0 --force
```

- This build only guarantees the patch resources shipped with the binary; version overrides need the URL template or may fail if that version is not included
- `patch` only writes or overwrites managed files in `opencode.json` and `agent/*.md`

Flags win over env vars, and env vars win over defaults.
