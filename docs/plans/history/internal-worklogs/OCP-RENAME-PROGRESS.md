# OCP Rename Progress (PHASE_1)

## Scope đang áp dụng
- PHASE_1: đổi `.ocp -> .ocp` cho source-of-truth.
- Giữ nguyên theo ngoại lệ: `ocp` CLI/binary, `Ocp.toml`, `.ocp_artifacts`, `.ocp_cache`, `.ocpp`, `.ocppkg`, `.ocpbundle`.

## Gate A - Audit (DONE)
- Tracked `.ocp` ban đầu: `86` file.
- `.ocp` ngoài `.ocpbundle` (bắt buộc đổi): `61` file.
- `.ocp` trong `.ocpbundle` (giữ nguyên PHASE_1): `25` file.
- Untracked chứa `ocp` (theo `git ls-files --others --exclude-standard "*ocp*"`): `0`.

## Gate B/C - Rename + Runtime/CLI/SDK (IN_PROGRESS, core completed)
- Đã rename toàn bộ `61` file `.ocp` (ngoài `.ocpbundle`) sang `.ocp` bằng `git mv`.
- Đã cập nhật runtime/SDK/CLI để:
  - ưu tiên `.ocp`,
  - vẫn fallback `.ocp` (compat).
- Đã đồng bộ references `.ocp -> .ocp` trong vùng code/test/editor/contracts/tools (không đụng docs/history).
- Đã re-sign các contract bị đổi nội dung:
  - `contracts/editor/editor_public_surface.v1.json(.sig)`
  - `contracts/editor/golden_vectors.v1.json(.sig)`
  - `contracts/editor/ocp_language_profile.v1.json(.sig)`

## Lệnh đã chạy (thực tế)
- `git ls-files "*ocp*"`
- `git ls-files "*.ocp" ":!:*.ocpbundle/*"`
- `rg -n ...`/`rg -l ...` cho audit điểm nóng.
- `git mv` batch `.ocp -> .ocp` (ngoài `.ocpbundle`).
- `cargo run -p ocp-cli -- verify --contract-sign contracts/editor/editor_public_surface.v1.json`
- `cargo run -p ocp-cli -- verify --contract-sign contracts/editor/golden_vectors.v1.json`
- `cargo run -p ocp-cli -- verify --contract-sign contracts/editor/ocp_language_profile.v1.json`
- `cargo test --test ocp_fixture_runner --test cli_tool_cli_e2e --test cli_tool_http_e2e --test cli_tool_proc_e2e --test cli_tool_wallclock_e2e --test cli_mini_game_e2e --test cli_shadow_preview_e2e --test cli_shadow_preview_v2_e2e --test cli_trace_view --test v19_lsp_diagnostics --test v19_lsp_symbols --test v19_dap_launch --test v19_dap_breakpoints --test v19_dap_step_variables --test v19_lsp_definition_references --test v19_lsp_rename --test v19_semantic_tokens_golden --test v19_multiroot_workspace`
- `cargo test --no-run`
- `cargo test --test v19_editor_public_surface --test v19_editor_language_profile --test v19_editor_sot_signature_verify`
- `cargo test --test v19_editor_sot_signature_verify`

## Kết quả test
- Targeted tests: PASS.
- Regression compile lane (`cargo test --no-run`): PASS.
- Editor contract/signature targeted:
  - `v19_editor_public_surface`: PASS
  - `v19_editor_language_profile`: PASS
  - `v19_editor_sot_signature_verify`: PASS (sau rerun theo yêu cầu test)

## Còn mở (chưa đóng Gate toàn bộ)
- Chưa migrate docs/history/plan references `.ocp` sang `.ocp` (ngoài scope code runtime hiện tại).
- Chưa hoàn tất CI guard rules theo checklist mới.
- Chưa đóng full closeout cho Gate D/E/F/G/H trong checklist tổng.

## Update 2026-03-08 (Checkpoint 6-7)
- Checklist rename đã được hoàn tất theo scope đã khóa (bao gồm PHASE_2 alias/migration planning).
- CI guard + docs migration + signature evidence đã có checkpoint đi kèm.
- Đã vá nốt rủi ro kỹ thuật ở `ocp-cli/src/main.rs` (BOM + chuỗi ký tự lỗi) và re-run targeted tests: PASS.
