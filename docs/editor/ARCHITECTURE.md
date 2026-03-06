# OCL Editor Architecture

## Layers
1. VSCode extension shell (`editor/vscode/ocp-ocl`).
2. LSP process (`ocl-lsp`) for realtime language features.
3. DAP process (`ocl-dap`) for debug flows.
4. Shared contracts and taxonomy in `ocl-sdk`.

## Contract-First Rules
- Public editor surface is pinned in `contracts/editor/editor_public_surface.v1.json`.
- Command and settings schemas are pinned and signed.
- Gate reports are written to `target/ocl/w19/*` and linked to run manifest.

## Data Flow (v0.19 line)
- Extension loads language profile and grammar.
- Extension registers command IDs from public surface contract.
- LSP/DAP wiring expands by later gates, but contract IDs are fixed from gate 19-A.
