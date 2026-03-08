# OCP Rename Progress - Checkpoint 2

## Scope
- Continue PHASE_1 safely.
- Keep legacy exceptions unchanged: `ocp` CLI/binary, `Ocp.toml`, `.ocp_artifacts`, `.ocp_cache`, `.ocpp`, `.ocppkg`, `.ocpbundle`.

## Completed in this checkpoint
- Updated current docs (non-history) from `.ocp` to `.ocp` in:
  - `docs/en/USER_GUIDE.md`
  - `docs/vi/USER_GUIDE.md`
  - `docs/en/troubleshooting/editor.md`
  - `docs/vi/troubleshooting/editor.md`
  - `docs/en/quickstart/07-vscode.md`
  - `docs/vi/quickstart/07-vscode.md`
  - `docs/PROJECT-BOUNDARY.md`
  - `docs/vi/MAINTAINER_GUIDE.md`
  - `docs/OCP-DEBUG-REPLAY-WORKFLOW-v0.11.md`
- Added CI guard test:
  - `tests/v100_ocp_extension_guard.rs`
  - Guard covers tracked/untracked `.ocp` and `.ocp` content token in include roots, with explicit PHASE_1 allowlist.
- Updated user-journey contract and signature:
  - `contracts/v20/user_journey_matrix.v1.json`
  - `contracts/v20/user_journey_matrix.v1.json.sig`

## Commands run
- `cargo run -p ocp-cli -- verify --contract-sign contracts/v20/user_journey_matrix.v1.json`
- `cargo test --test v100_ocp_extension_guard`
- `cargo test --test v100_docs_quickstart_flow --test v100_docs_troubleshooting_coverage --test v100_docs_required_sections --test v19_editor_language_profile --test v19_editor_public_surface --test v19_editor_sot_signature_verify --test v20_user_journeys_editor --test v100_gate_c_common`

## Test results
- `v100_ocp_extension_guard`: PASS
- `v100_docs_quickstart_flow`: PASS
- `v100_docs_troubleshooting_coverage`: PASS
- `v100_docs_required_sections`: PASS
- `v19_editor_language_profile`: PASS
- `v19_editor_public_surface`: PASS
- `v19_editor_sot_signature_verify`: PASS
- `v20_user_journeys_editor`: PASS

## Remaining open
- `OCP-MVP-PLAN-v1.0.md` and `docs/plans/history/*` still contain `.ocp` references (kept unchanged in this checkpoint).
- Full plan closeout entries are still pending.
