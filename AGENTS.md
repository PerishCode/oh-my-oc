# AGENTS.md

This repository is for personal use. Keep it simple and clean first; avoid over-designing for collaboration, process, resilience, or generality.

`oh-my-oc` is a patch-first tool for the Opencode config directory, not a broader helper/runtime manager. Keep the root lightweight while that boundary stays documented.

## Active agent layout

- `commander`: default agent in `opencode.json`; owns planning, routing, and final coordination.
- `explorer`: read-only investigator for narrow fact gathering.
- `coder`: implementation worker for the smallest correct change.
- `advisor`: targeted reviewer that flags weak plans, gaps, and unnecessary complexity.

## Relevant files and directories

- `resources/`: reference material and patch resources; keep it organized and low-maintenance.
- `app/`: minimal Rust CLI surface for `oh-my-oc`; keep it small and avoid overbuilding it.
- `opencode.json`: root config; sets the schema, `openai/gpt-5.4`, full permission allow, and `default_agent: commander`.
- `.opencode/package.json`: installs `@opencode-ai/plugin` for the local setup.
- `.opencode/agent/`: contains the active agent role definitions.
- `.gitignore`: ignores `.tmp/`.
- `.opencode/.gitignore`: ignores local dependency files in `.opencode/`.

## Managed files policy

- Default patch target is `~/.config/opencode`; CLI args control path/version first, then env vars, then defaults.
- The managed file boundary is intentionally narrow: `opencode.json` and `agent/*.md`.
- Only those managed files should be written or overwritten by the patch flow.
- Quick-fail behavior is intentional when the target or files do not match the managed boundary.
- Version overrides are allowed; bundled/current build resources are guaranteed, and remote version overrides may use `OH_MY_OC_PATCH_RESOURCE_URL_TEMPLATE`.

## Maintenance guidance

- Keep changes minimal, simple, and clean.
- Prefer the smallest working config over broader patterns or future-proofing.
- Treat `resources/` as supporting material, not an execution layer.
- Treat `app/` as a small distributable CLI surface, not a place for heavy architecture.
- Keep patch behavior constrained to the managed files policy above.
- The current install flow is a tiny `install.sh` that fetches GitHub release artifacts; keep it minimal and do not add CI or broader release automation.
- Preserve role boundaries: commander orchestrates, explorer gathers facts, coder implements, advisor reviews.
- Prefer updating the existing patch resources or agent file over adding parallel copies.
- Do not reintroduce deleted legacy files, extra layers, or unused clutter.
- If agent behavior or repository layout changes, update this document at the same time so it stays accurate and compact.

## Contribution rules

- Make the smallest change that fits the current template.
- Keep wording and structure practical, direct, and personal-use oriented.
- Avoid creating new workflow conventions, collaboration mechanics, or general-purpose abstractions unless the repo actually uses them.
