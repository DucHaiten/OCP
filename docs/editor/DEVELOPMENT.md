# OCP Editor Development

## Scope
This document is the contributor setup guide for the VSCode editor stack in v0.19.

## Workspace Layout
- `editor/vscode/ocp/`: VSCode extension shell and client wiring.
- `projects/ocp/crates/ocp-lsp/`: language server (added in later gates).
- `projects/ocp/crates/ocp-dap/`: debug adapter (added in later gates).
- `contracts/editor/`: editor SoT contracts and signatures.

## Baseline Commands
- `cargo test --test v19_editor_language_profile`
- `cargo test --test v19_textmate_highlight_golden`
- `cargo test --test v19_language_configuration`
- `cargo xtask editor-ci`

## Guardrails
- Do not bypass contracts in `contracts/editor`.
- Do not add editor commands without updating `editor_public_surface.v1.json`.
- Do not change settings keys without updating `ocp_settings_schema.v1.json`.
