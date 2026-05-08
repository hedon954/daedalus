# CLAUDE.md

This crate implements the deterministic Rust tooling for daedalus. Its first responsibility is to keep filesystem learning state reliable for AI Agents and human operators.

## Development Rules

- Use Rust edition 2024 through the workspace configuration in `crates/Cargo.toml`.
- Keep business rules in `src/domain` and `src/application`; UI and command handling must stay in `src/interfaces`.
- Do not let CLI or TUI code mutate `.daedalus/state.toml` directly. Route state changes through application use cases.
- Preserve comments and human edits in `.daedalus/state.toml`; use `toml_edit` for targeted updates instead of full deserialize/serialize rewrites.
- Regenerate `.daedalus/state.md` after every state-changing operation.
- Keep `daedalus` Agent-friendly: stable text output, actionable errors, clear exit codes, and JSON output where supported.
- Keep `daedalus-tui` human-friendly but thin. It may read shared application results, but it must not introduce a second copy of business logic.
- Treat `--force`, `--skip`, and similar bypass options as controlled escape hatches. They require a specific reason and documented approval source.

## Module Boundaries

- `domain`: learning task, stage, transition, artifact, and domain errors. No filesystem, terminal, or CLI dependencies.
- `application`: use cases such as task initialization, stage transition, validation, and rendering.
- `infrastructure`: filesystem access, template copying, state TOML editing, workspace discovery, and clock adapters.
- `interfaces/agent_cli`: clap arguments, presenters, text/JSON output, and process-facing behavior.
- `interfaces/tui`: ratatui screens, read-only overview, and terminal presentation.

## Quality Checks

Run these from the repository root before considering crate work complete:

```sh
make ci
```

This covers:

- `cargo fmt --check`
- `cargo check --workspace --all-targets --all-features`
- `cargo clippy --workspace --all-targets --all-features --tests --benches -- -D warnings`
- `cargo test --workspace --all-features`

## Testing Expectations

- Add tests when changing state transitions, rendering, validation, template initialization, or presenter output.
- Prefer integration tests for CLI behavior that Agents depend on.
- Cover comment preservation when editing `state.toml`.
- Cover failure paths for missing artifacts, invalid stages, WIP violations, and force approval rules.

## Adding Crates Or Binaries

When adding or moving a crate or binary under `crates/`, update all relevant project-level checks:

- `crates/Cargo.toml`
- repository `Makefile`
- repository `.pre-commit-config.yaml`
- repository `.github/workflows/ci.yml`
- root `CLAUDE.md` if the workflow guidance changes

Do not leave new Rust code outside the workspace or outside `make ci` coverage.
