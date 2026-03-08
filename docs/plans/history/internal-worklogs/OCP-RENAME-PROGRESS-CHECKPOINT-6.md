# OCP Rename Progress - Checkpoint 6

## Scope
- Continue to the remaining checklist items (PHASE_2 planning + CLI alias gate).

## Completed in this checkpoint
- Added PHASE_2 CLI binary setup in `ocp-cli` crate:
  - primary binary: `ocp`
  - legacy alias binary: `ocp`
  - file: `projects/ocp/crates/ocp-cli/Cargo.toml`
- Added runtime alias warning when invoked via legacy `ocp` binary:
  - warning code: `W-CLI-ALIAS-DEPRECATED`
  - file: `projects/ocp/crates/ocp-cli/src/main.rs`
- Updated CLI visible brand/version output to `ocp`:
  - `print_version` now outputs `ocp v1.0.0`
  - help headline uses `ocp <command> [args]`
  - usage lines switched to `usage: ocp ...`
- Added manifest path compatibility resolver updates:
  - `Ocp.toml`/`ocp.toml` preferred when present
  - fallback to `Ocp.toml`/`ocp.toml`
  - files:
    - `projects/ocp/crates/ocp-cli/src/main.rs`
    - `projects/ocp/crates/ocp-sdk/src/lib.rs`
- Added targeted CLI alias test:
  - `tests/v100_cli_alias_warning.rs`
- Updated existing version test to `ocp`:
  - `tests/v100_cli_version_flag.rs`
- Added PHASE_2 migration plan docs:
  - `docs/en/migration/ocp-to-ocp-phase2-plan.md`
  - `docs/vi/migration/ocp-to-ocp-phase2-plan.md`
  - linked from PHASE_1 migration notes.
- Checklist updated: all items are now checked.

## Commands run
- `cargo test --test v100_cli_version_flag --test v100_cli_alias_warning`
- `cargo test --test cli_tool_cli_e2e`
- `cargo test --test v100_ocp_extension_guard`

## Test results
- `v100_cli_version_flag`: PASS
- `v100_cli_alias_warning`: PASS
- `cli_tool_cli_e2e`: PASS
- `v100_ocp_extension_guard`: PASS

## Notes
- Cargo warns that `src/main.rs` is referenced by both `ocp` and `ocp` bin targets; behavior is correct, warning is non-blocking.
