# Contributing to OCP

## Required Workflow
- Follow the active plan gate in `OCP-MVP-PLAN-v*.md`.
- Do not mark a gate `DONE` without targeted tests passing.
- Keep changes contract-first and evidence-backed.

## CLA and Contribution Licensing
- External contributions for code, docs, and contracts require a CLA under current project policy.
- The locked CLA model is `license_grant`.
- Inbound contribution path: `cla`.
- Outbound project model: `AGPL-3.0-only+commercial`.
- Contributions that do not go through the required CLA flow are not accepted into the official project line.
- For the public-facing explanation, read:
  - `docs/vi/legal/licensing.md`
  - `docs/en/legal/licensing.md`

## Commercial and OSS Path
- The OSS side of OCP is `AGPL-3.0-only`.
- The project keeps a `dual_license` model.
- If your intended use needs terms beyond the OSS side, read:
  - `docs/vi/legal/commercial.md`
  - `docs/en/legal/commercial.md`

## Editor Stack (v0.19)
- Keep command IDs and settings keys aligned with `contracts/editor/editor_public_surface.v1.json`.
- Keep VSCode contributions aligned with signed SoT files in `contracts/editor/`.
- Add or update tests under `tests/v19_*.rs` for any editor behavior change.

## Validation
- Run gate-targeted tests first.
- Run `cargo xtask editor-ci` before release signoff gate.
