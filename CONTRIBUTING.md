# Contributing to OCP-OCL

## Required Workflow
- Follow the active plan gate in `OCP-OCL-MVP-PLAN-v*.md`.
- Do not mark a gate `DONE` without targeted tests passing.
- Keep changes contract-first and evidence-backed.

## Editor Stack (v0.19)
- Keep command IDs and settings keys aligned with `contracts/editor/editor_public_surface.v1.json`.
- Keep VSCode contributions aligned with signed SoT files in `contracts/editor/`.
- Add or update tests under `tests/v19_*.rs` for any editor behavior change.

## Validation
- Run gate-targeted tests first.
- Run `cargo xtask editor-ci` before release signoff gate.
