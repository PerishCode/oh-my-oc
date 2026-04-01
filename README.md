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
OH_MY_OC_GITHUB_TOKEN=... curl -fsSL https://raw.githubusercontent.com/PerishCode/oh-my-oc/main/scripts/manage/install.sh | sh -s -- --version v0.1.0
```

The installer uses `OH_MY_OC_REPO`, `OH_MY_OC_BASE_URL`, `OH_MY_OC_GITHUB_TOKEN`, `OH_MY_OC_INSTALL_ROOT`, and `OH_MY_OC_LOCAL_BIN_DIR` when you need to override defaults.

Release assets are produced for Linux x86_64 and macOS x86_64/aarch64 only.

## CLI

Apply the bundled Opencode patch into the default config directory:

```sh
oh-my-oc patch
```

Patch behavior:

- Default target: `~/.config/opencode`
- Override target path with `--path <value>` or `OH_MY_OC_PATCH_PATH`
- Override version selection with `--version <value>` or `OH_MY_OC_PATCH_VERSION`
- CLI flags win over env vars, and env vars win over defaults
- Use `--force` to overwrite managed files if they already exist

Only the current binary-bound version is available in this build. If another version is requested, the CLI fails clearly instead of fetching anything.
