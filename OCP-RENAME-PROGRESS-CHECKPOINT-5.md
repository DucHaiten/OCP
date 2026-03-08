# OCP Rename Progress - Checkpoint 5

## Scope
- Continue PHASE_1 safely.
- Close remaining non-PHASE_2 checklist items (generated-artifact gate).

## Completed in this checkpoint
- Fixed legacy FS permission key mapping in SDK:
  - `std.fs.read`, `std.fs.list`, `std.fs.write` now map to read/list/write actions (compat with legacy keys).
  - File: `projects/ocp/crates/ocp-sdk/src/lib.rs`
- Added runtime permission section for composer demo so compose can run deterministically in PHASE_1:
  - File: `projects/ocp/apps/composer-demo/Ocp.toml`
- Regenerated composer generated sources using real compose flow:
  - `projects/ocp/apps/composer-demo/src/generated/*`
  - `projects/ocp/apps/composer-demo/assembly_proof.toml`
  - `projects/ocp/apps/composer-demo/target/ocp/composer/cache.v1`
- Fixed m4 composer hash/verify cache logic to include `.ocp` generated files (while keeping `.ocp` legacy compat):
  - File: `projects/ocp/crates/ocp-sdk/src/m4.rs`
- Added targeted regression test for legacy FS key compat:
  - File: `tests/compat_std_fs_legacy_keys.rs`
- Checklist update:
  - Generated-artifact gate items are now checked.
  - Remaining unchecked items are PHASE_2-only.

## Commands run
- `cargo run -p ocp-cli -- compose projects/ocp/apps/composer-demo --phenotype projects/ocp/apps/composer-demo/phenotype.toml --registry projects/ocp/apps/composer-demo/registry --locked`
- `cargo run -p ocp-cli -- verify projects/ocp/apps/composer-demo --phenotype projects/ocp/apps/composer-demo/phenotype.toml --registry projects/ocp/apps/composer-demo/registry --locked`
- `cargo test --test compat_std_fs_legacy_keys`
- `cargo test -p ocp-sdk --test m4_composer`
- `cargo test --test v100_ocp_extension_guard`
- `cargo test --test v19_vscode_integration --test compat_std_fs_legacy_keys`

## Test results
- `compat_std_fs_legacy_keys`: PASS
- `m4_composer`: PASS
- `v100_ocp_extension_guard`: PASS
- `v19_vscode_integration`: PASS
- `compose + verify` on `composer-demo`: PASS

## Remaining open
- Only PHASE_2 checklist items remain open by design:
  - CLI rename `ocp -> ocp`
  - `Ocp.toml -> Ocp.toml`
  - `.ocp_* -> .ocp_*` migration + alias plan
