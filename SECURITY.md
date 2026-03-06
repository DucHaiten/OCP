# Security Policy

## Reporting
- Report vulnerabilities through private channels.
- Include reproduction steps, affected versions, and expected impact.

## Editor Security Baseline (v0.19)
- SoT contracts and release manifests are signed and verified.
- Workspace trust policy is deny-by-default for runtime features in untrusted workspaces.
- Bundled binaries (`ocl-cli`, `ocl-lsp`, `ocl-dap`) must pass hash and signature checks.

## Scope
- This policy covers core runtime, CLI, and editor distribution artifacts.
