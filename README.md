# oh-my-oc

Personal Opencode configuration and small Rust CLI workspace.

## Layout

- `app/` - minimal distributable Rust CLI surface for `oh-my-oc`
- `scripts/release/package.sh` / `scripts/release/package.ps1` - packaging source of truth for release artifacts
- `scripts/release/verify.sh` / `scripts/release/verify.ps1` - minimal release asset checks for accept/verify
- `scripts/manage/install.sh` / `scripts/manage/install.ps1` - install scripts for Unix and Windows
- root config - Opencode setup and agent definitions

## Local install loop

The release path is intentionally simple:

1. Run `scripts/release/package.sh <tag>` on Unix or `scripts/release/package.ps1 <tag>` on Windows to build the CLI and create release artifacts plus `checksums.txt` under `dist/<tag>/`.
2. Publish those files as GitHub release assets.
3. Install with `curl -fsSL https://raw.githubusercontent.com/PerishCode/oh-my-oc/main/scripts/manage/install.sh | sh`, or pass `--version <tag>` / `OH_MY_OC_VERSION=<tag>` to pin a release.

## Release flow

- Stable release: push a `vX.Y.Z` tag and let `.github/workflows/release.yml` publish it.
- Beta release: run `.github/workflows/release-beta.yml` manually with a `vX.Y.Z-beta.N` version that matches `app/Cargo.toml`.
- Beta publishes a GitHub prerelease and runs a small installer + skill smoke check on Unix and Windows before you promote the same line to a stable tag.

The installer uses `OH_MY_OC_REPO`, `OH_MY_OC_BASE_URL`, `OH_MY_OC_INSTALL_ROOT`, and `OH_MY_OC_LOCAL_BIN_DIR` when you need to override defaults.

Latest mode fetches release assets from the GitHub Releases `latest/download/` path.

Release assets are produced for Linux x86_64, macOS x86_64/aarch64, and Windows x86_64. Each release also includes `skill.zip` for Windows and `skill.tar.gz` for Unix, both unpacking to `oh-my-oc/SKILL.md` for optional agent guidance.

## Windows install

Install from PowerShell:

```powershell
irm https://raw.githubusercontent.com/PerishCode/oh-my-oc/main/scripts/manage/install.ps1 | iex
```

Pin a release:

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/PerishCode/oh-my-oc/main/scripts/manage/install.ps1))) --version v0.2.10
```

The PowerShell installer uses `OH_MY_OC_REPO`, `OH_MY_OC_BASE_URL`, `OH_MY_OC_INSTALL_ROOT`, and `OH_MY_OC_LOCAL_BIN_DIR` when you need to override defaults.

By default it installs versioned binaries under `%LOCALAPPDATA%\oh-my-oc\<version>` and puts `oh-my-oc.exe` in `%USERPROFILE%\.local\bin`. The installer prints that `bin` directory so you can wire it into your PATH with your own shell or environment manager.

## `oh-my-oc patch`

Install the current release, then apply the patch into your Opencode config:

```sh
curl -fsSL https://raw.githubusercontent.com/PerishCode/oh-my-oc/main/scripts/manage/install.sh | sh
oh-my-oc patch
```

On Windows:

```powershell
oh-my-oc patch
```

Examples:

```sh
oh-my-oc patch --path ~/.config/opencode
oh-my-oc patch --force
```

Notes:

- Default target: `~/.config/opencode` on Unix and Windows
- Override target path with `--path` or `OH_MY_OC_PATCH_PATH`
- By default `patch` fetches the latest `PerishCode/resources` release archive
- `--version` selects a specific resource release archive to fetch
- Override version with `--version` or `OH_MY_OC_PATCH_VERSION`
- The patch flow downloads the official `PerishCode/resources` release archive: `oh-my-oc-<version>.tar.gz` on Unix, `oh-my-oc-<version>.zip` on Windows
- The archive is expected to unpack with a top-level `oh-my-oc/opencode/` directory
- `patch` only writes or overwrites managed files in `opencode.json` and `agent/*.md`

Flags win over env vars, and env vars win over defaults.
