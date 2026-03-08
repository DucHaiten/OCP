# Migration Plan: PHASE_2 (`ocp` -> `ocp`)

## Goal
- Promote `ocp` as primary CLI command.
- Keep `ocp` as compatibility alias during migration window.
- Move config/runtime paths from legacy `ocp` names to `ocp` names with fallback reads.

## CLI alias policy
1. Primary command: `ocp`.
2. Legacy alias: `ocp` (still executable).
3. When running via alias `ocp`, CLI emits:
   - `W-CLI-ALIAS-DEPRECATED`
4. Removal condition:
   - remove alias only after migration window and release note.

## Config and runtime path policy
1. Config:
   - prefer `Ocp.toml` / `ocp.toml`.
   - fallback to `Ocp.toml` / `ocp.toml`.
2. Runtime/cache/artifacts:
   - target names: `.ocp_state`, `.ocp_cache`, `.ocp_artifacts`.
   - fallback reads from legacy `.ocp_*` during transition.
3. Backward compatibility:
   - existing projects must remain runnable in transition releases.

## Verification gates
1. CLI version/help indicates `ocp`.
2. Alias `ocp` works and emits deprecation warning.
3. Legacy project paths still load with fallback.
4. CI guard continues preventing accidental extension/package regressions.
