# Ghi chú migration: `.ocp` -> `.ocp` (PHASE_1)

## Tóm tắt
- Đuôi file theo ngôn ngữ chính thức là `.ocp`.
- Dự án `.ocp` cũ vẫn chạy ở chế độ tương thích trong PHASE_1.
- Tên CLI/config vẫn giữ nguyên trong PHASE_1:
  - lệnh `ocp`
  - `Ocp.toml`
  - `.ocp_artifacts`, `.ocp_cache`
  - `.ocpp`, `.ocppkg`, `.ocpbundle`

## Những gì đã đổi ở vòng này
- Nguồn chính (source-of-truth) đã đổi `.ocp` sang `.ocp` ở code/test/conformance.
- Editor association đã chuyển theo `.ocp`.
- CI guard đã khóa không cho phát sinh `.ocp` mới ngoài allowlist legacy.

## Những gì chưa đổi
- Đổi CLI (`ocp -> ocp`) là PHASE_2.
- Đổi manifest (`Ocp.toml -> Ocp.toml`) là PHASE_2.
- Đổi thư mục runtime/cache (`.ocp_* -> .ocp_*`) là PHASE_2.

## Khuyến nghị cho người dùng
1. Đổi source/entry từ `.ocp` sang `.ocp`.
2. Giữ nguyên `Ocp.toml` ở PHASE_1.
3. Cập nhật script nội bộ nếu đang hardcode `src/main.ocp`.
4. Nếu cần, tạm giữ `.ocp` trong giai đoạn chuyển tiếp vì vẫn có compat mode.

## Timeline bỏ hỗ trợ legacy (dự kiến)
1. PHASE_1 (hiện tại): `.ocp` là chuẩn chính, `.ocp` vẫn chạy được nhưng có cảnh báo `W-LEGACY-OCP-EXTENSION`.
2. PHASE_2 (dự kiến): bỏ đường parse `.ocp` legacy, đổi tên CLI/config/artifacts theo gói breaking migration có kiểm soát.
3. Trước PHASE_2: phát hành migration guide + cửa sổ alias/deprecate cho đổi lệnh `ocp -> ocp`.

Xem thêm: `docs/vi/migration/ocp-to-ocp-phase2-plan.md`.
