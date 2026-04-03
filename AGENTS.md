# AGENTS.md

This repository is for personal use. Keep it simple and clean first; avoid over-designing for collaboration, process, resilience, or generality.

`oh-my-oc` is a patch-first tool for the Opencode config directory, not a broader helper/runtime manager. Keep the root lightweight while that boundary stays documented.

## Active agent layout

- `commander`: default agent in `opencode.json`; owns planning, routing, and final coordination.
- `explorer`: read-only investigator for narrow fact gathering.
- `coder`: implementation worker for the smallest correct change.
- `advisor`: targeted reviewer that flags weak plans, gaps, and unnecessary complexity.

## Relevant files and directories

- `app/`: minimal Rust CLI surface for `oh-my-oc`; keep it small and avoid overbuilding it.
- `opencode.json`: root config; sets the schema, `openai/gpt-5.4`, full permission allow, and `default_agent: commander`.
- `.gitignore`: ignores `.tmp/`.

## Managed files policy

- Default patch target is `~/.config/opencode`; CLI args control path/version first, then env vars, then defaults.
- The managed file boundary is intentionally narrow: `opencode.json` and `agent/*.md`.
- Only those managed files should be written or overwritten by the patch flow.
- Runtime agent visibility should be assumed only for content inside that managed boundary after patching.
- Do not rely on external template paths, sidecar files, or soft references unless the patch/install chain gives them a stable runtime location and Opencode has a native way to load them.
- Quick-fail behavior is intentional when the target or files do not match the managed boundary.
- Version overrides are allowed; patch content is fetched only from the official `PerishCode/resources` release archive: `.tar.gz` on Unix and `.zip` on Windows.

## Maintenance guidance

- Keep changes minimal, simple, and clean.
- Prefer the smallest working config over broader patterns or future-proofing.
- Treat `app/` as a small distributable CLI surface, not a place for heavy architecture.
- Keep patch behavior constrained to the managed files policy above.
- If a behavior must be reliably present for commander at runtime, prefer encoding it directly in `agent/commander.md` or another managed file instead of introducing extra runtime dependencies.
- The current install flow is intentionally tiny: `scripts/manage/omo.sh` for Unix and `scripts/manage/omo.ps1` for Windows fetch GitHub release artifacts; local packaging helpers should stay equally minimal.
- Release-affecting changes should go through a beta prerelease first, then stable once the same line is verified.
- Keep release packaging logic sourced from `scripts/release/package.sh` and `scripts/release/package.ps1`; do not duplicate it elsewhere.
- Use the release verify/smoke checks to catch asset or install drift before promoting.
- Skill assets are split by platform when relevant: `skill.tar.gz` for Unix and `skill.zip` for Windows.
- Preserve role boundaries: commander orchestrates, explorer gathers facts, coder implements, advisor reviews.
- Do not reintroduce deleted legacy files, extra layers, or unused clutter.
- If agent behavior or repository layout changes, update this document at the same time so it stays accurate and compact.

## Contribution rules

- Make the smallest change that fits the current template.
- Keep wording and structure practical, direct, and personal-use oriented.
- Avoid creating new workflow conventions, collaboration mechanics, or general-purpose abstractions unless the repo actually uses them.
