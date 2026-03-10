# Migration Note: `.oc` -> `.ocp` (PHASE_1)

## Summary
- Language-facing extension is `.ocp` (canonical in `v1.0.x`).
- Existing `.oc` projects still run in legacy-compat mode during PHASE_1.
- CLI binary/config names are intentionally unchanged in PHASE_1:
  - `ocp` command
  - `Ocp.toml`
  - `.ocp_artifacts`, `.ocp_cache`
  - `.ocpp`, `.ocppkg`, `.ocpbundle`

## What changed now
- Source/test/conformance fixtures moved from `.oc` to `.ocp` in source-of-truth paths.
- Editor association is aligned to `.ocp`.
- CI guard blocks new `.oc` usage outside approved legacy allowlist.

## What did not change yet
- CLI rename (`ocp -> ocp`) is PHASE_2.
- Manifest rename (`Ocp.toml -> Ocp.toml`) is PHASE_2.
- Runtime/cache directory rename (`.ocp_* -> .ocp_*`) is PHASE_2.

## Recommended actions for users
1. Rename entry/source files from `.oc` to `.ocp`.
2. Keep `Ocp.toml` unchanged for now.
3. Update any local scripts that hardcode `src/main.oc` paths.
4. If needed, keep legacy `.oc` files temporarily; compat mode is still accepted in PHASE_1.

## Legacy support timeline (planned)
1. PHASE_1 (current): `.ocp` is primary, `.oc` is accepted with `W-LEGACY-OCP-EXTENSION` warning.
2. PHASE_2 (planned): remove legacy `.oc` parsing path, rename CLI/config/artifacts in one controlled breaking migration.
3. Pre-PHASE_2 gate: publish migration guide + alias/deprecation window for `ocp -> ocp` CLI rename.

See also: `docs/en/migration/ocp-to-ocp-phase2-plan.md`.
