# OCP Rename Progress - Checkpoint 4

## Scope
- Continue PHASE_1 safely.
- Focus this checkpoint on `generated artifacts` gate.

## Completed in this checkpoint
- Unblocked offline TypeScript execution via elevated `npm exec --package typescript`.
- Regenerated VSCode generated artifact from source (no hand-edit):
  - source: `editor/vscode/ocp/src/dapRuntime.ts`
  - generated: `editor/vscode/ocp/dist/dapRuntime.js`
  - transitive generated update: `editor/vscode/ocp/dist/runtimePaths.js`
- Re-verified editor + extension guards after regenerate:
  - `v19_vscode_integration`: PASS
  - `v100_ocp_extension_guard`: PASS
- Updated checklist policy item to keep gate open when generated artifacts are not fully regenerated yet.

## Commands run
- `corepack --version`
- `corepack pnpm --version` (failed: EPERM on restricted home path)
- `corepack npm --version` (failed: network/connect EACCES in sandbox)
- `npm.cmd --prefix editor/vscode/ocp exec --package typescript -- tsc --version` (elevated, PASS)
- `npm.cmd --prefix editor/vscode/ocp exec --package typescript -- tsc src/dapRuntime.ts --target ES2020 --module commonjs --moduleResolution node --esModuleInterop --skipLibCheck --outDir dist` (failed: wrong relative path)
- `npm.cmd --prefix editor/vscode/ocp exec --package typescript -- tsc editor/vscode/ocp/src/dapRuntime.ts --target ES2020 --module commonjs --moduleResolution node --esModuleInterop --skipLibCheck --outDir editor/vscode/ocp/dist` (type errors reported, emit still produced regenerated dist files)
- `cargo test --test v19_vscode_integration`
- `cargo test --test v100_ocp_extension_guard`
- `cargo test --test v19_editor_public_surface`
- `cargo test --test v19_editor_language_profile`
- `node --check editor/vscode/ocp/dist/dapRuntime.js`
- `node --check editor/vscode/ocp/dist/runtimePaths.js`
- `cargo run -p ocp-cli -- compose projects/ocp/apps/composer-demo --phenotype projects/ocp/apps/composer-demo/phenotype.toml --registry projects/ocp/apps/composer-demo/registry --locked` (failed: `V-PERMISSION-DENIED`)
- `cargo run -p ocp-cli -- lock sync projects/ocp/apps/composer-demo`
- `cargo run -p ocp-cli -- compose projects/ocp/apps/composer-demo --phenotype projects/ocp/apps/composer-demo/phenotype.toml --registry projects/ocp/apps/composer-demo/registry` (failed: `V-PERMISSION-DENIED`)
- `New-Item -ItemType Directory -Path tmp\\composer-demo-smoke -Force`
- `Copy-Item -Path projects/ocp/apps/composer-demo\\* -Destination tmp\\composer-demo-smoke -Recurse -Force`
- `cargo run -p ocp-cli -- lock sync tmp/composer-demo-smoke`
- `cargo run -p ocp-cli -- compose tmp/composer-demo-smoke --phenotype tmp/composer-demo-smoke/phenotype.toml --registry tmp/composer-demo-smoke/registry --locked` (failed: `V-PERMISSION-DENIED`)

## Test results
- `v19_vscode_integration`: PASS
- `v100_ocp_extension_guard`: PASS
- `v19_editor_public_surface`: PASS
- `v19_editor_language_profile`: PASS
- `node --check dist/dapRuntime.js`: PASS
- `node --check dist/runtimePaths.js`: PASS

## Remaining open
- Generated-artifact gate is still `IN_PROGRESS`:
  - VSCode `dist/*` regenerate path is now available and applied.
  - `composer-demo/src/generated/*` cannot be regenerated in current app state due permission gate (`V-PERMISSION-DENIED` for `std.fs.read`).
- PHASE_2 items remain intentionally open.
