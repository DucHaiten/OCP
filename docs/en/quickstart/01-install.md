# Quickstart 01 - Install

## Windows
- Install by `ocp-v1.0.0-setup-win-x64.exe` or portable zip.
- The installer uses the OCP icon, lets the user choose an install directory, and explicitly asks whether `ocp` should be added to the system `PATH`.
- The installer ships `ocp.exe`, `ocp-lsp.exe`, `ocp-dap.exe`, and bundles `ocp-vscode-v1.0.0.vsix`.
- If VSCode is installed in standard locations or `code` CLI is available, the installer can auto-install the VSCode extension.
- If auto-install does not run, use the bundled VSIX for manual installation.

## Linux/macOS
- Use the platform tarball and place `ocp` in `PATH`.

## Verify before execution
- Read `docs/en/security/verify-download.md`.
