# OCP Rename Progress - Checkpoint 3

## Scope
- Continue PHASE_1 safely.
- Keep legacy exceptions unchanged: `ocp` CLI/binary, `Ocp.toml`, `.ocp_artifacts`, `.ocp_cache`, `.ocpp`, `.ocppkg`, `.ocpbundle`.

## Completed in this checkpoint
- Added PHASE_1 legacy entry fallback in SDK:
  - `project_layout` now resolves `src/main.ocp` first, then `src/main.ocp` if legacy project is detected.
- Added deprecation warning for legacy `.ocp` source usage:
  - Emits `W-LEGACY-OCP-EXTENSION` when project still contains `.ocp` source files under `src/` or `tests/`.
- Fixed module-id normalization bug in two places:
  - `src/ocp/runner.rs` now strips `.ocp` correctly.
  - `projects/ocp/crates/ocp-sdk/src/lib.rs` now strips `.ocp` correctly.
- Added targeted compat test for state/cache/replay in legacy mode:
  - `tests/cli_cache.rs`:
    - rename `src/main.ocp -> src/main.ocp`,
    - run/replay/cache-stats/cache-clean still pass,
    - deprecate warning is asserted.
- Updated CI extension guard allowlist for intentional PHASE_1 legacy tokens:
  - `tests/v100_ocp_extension_guard.rs` allowlist now includes:
    - `tests/cli_cache.rs`
    - `src/ocp/runner.rs`
    - `projects/ocp/crates/ocp-sdk/src/lib.rs`
- Updated checklist statuses for newly closed items:
  - `OCP-RENAME-CHECKLIST.md`
- Added explicit legacy-removal timeline to migration notes:
  - `docs/en/migration/ocp-to-ocp-phase1.md`
  - `docs/vi/migration/ocp-to-ocp-phase1.md`

## Commands run
- `cargo test --test cli_cache`
- `cargo test --test v100_ocp_extension_guard`
- `cargo test --test ocp_fixture_runner`
- `cargo test --test v100_docs_required_sections`
- `pnpm --dir editor/vscode/ocp exec tsc --version` (failed: `pnpm` not available)
- `npm.cmd --version`
- `npm.cmd --prefix editor/vscode/ocp run`
- `npx.cmd tsc --version` (failed: cannot fetch toolchain in current environment)

## Test results
- `cli_cache`: PASS (includes new `cli_cache_replay_supports_legacy_main_ocp_with_warning`)
- `v100_ocp_extension_guard`: PASS
- `ocp_fixture_runner`: PASS
- `v100_docs_required_sections`: PASS

## Remaining open
- Generated-artifact policy items are still open (`source -> regenerate` evidence not fully closed):
  - current environment has no working local TS build toolchain for `editor/vscode/ocp/dist/*` regeneration.
- PHASE_2 items remain intentionally open.
- Signed-path pointer-layer item is marked `N/A` in PHASE_1 (no signed path rename in this wave).
