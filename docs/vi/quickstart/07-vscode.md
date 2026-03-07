# Quickstart 07 - VSCode

- Nếu installer Windows đã tự cài extension thì chỉ cần mở VSCode lại.
- Nếu chưa có extension, cài VSIX `ocp-ocl-vscode-v1.0.0.vsix`.
- Mở file `.ocl` để nhận syntax highlight + snippets; trong workspace trusted, extension sẽ bật LSP/DAP từ bundled runtimes.
- Khi workspace được trust, dùng command bridge trong extension để chạy `fmt`, `check`, `doctor`, `fix --plan`, mở trace artifact, và dùng hover/completion/rename/semantic tokens/debug adapter.
