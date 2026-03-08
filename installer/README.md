# Installer Layout

Thư mục `installer/` là gốc source cho hệ đóng gói/cài đặt của OCP.

Quy ước hiện hành:
- Source installer Windows nằm ở `installer/windows/`.
- Script build/release hỗ trợ nằm ở `tools/release/`.
- Build artifacts không được lưu trong repo source này.

Artifacts chuẩn sau khi build:
- Windows installer: `target/ocp/w100/release/ocp-v1.0.0-setup-win-x64.exe`
- VSIX: `target/ocp/w100/release/ocp-vscode-v1.0.0.vsix`

Các file nguồn chính:
- `installer/windows/ocp-win-x64.iss`
- `installer/windows/ocp-installer.ico`
- `tools/release/build_win_installer.ps1`
- `tools/release/build_vsix.ps1`

Trạng thái thực tế hiện tại:
- Installer Windows đã được kiểm chứng cài thật ngoài repo.
- Payload cài đặt chuẩn hiện gồm:
  - `ocp.exe`
  - `ocp-lsp.exe`
  - `ocp-dap.exe`
  - `ocp-vscode-v1.0.0.vsix`
- Installer có:
  - icon OCP,
  - chọn thư mục cài,
  - opt-in `PATH`,
  - auto-install VSCode extension khi phát hiện VSCode.

Lưu ý:
- Không copy `.exe` hay file build vào `installer/`.
- `installer/` chỉ chứa source đóng gói, không chứa output phát hành.
