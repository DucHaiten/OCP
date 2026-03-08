# Kế hoạch migration: PHASE_2 (`ocp` -> `ocp`)

## Mục tiêu
- Nâng `ocp` thành lệnh CLI chính.
- Giữ `ocp` làm alias tương thích trong cửa sổ chuyển tiếp.
- Chuyển config/runtime path từ họ `ocp` sang `ocp` có fallback đọc dữ liệu cũ.

## Chính sách alias CLI
1. Lệnh chính: `ocp`.
2. Alias legacy: `ocp` (vẫn chạy được).
3. Khi chạy qua alias `ocp`, CLI phát cảnh báo:
   - `W-CLI-ALIAS-DEPRECATED`
4. Điều kiện gỡ alias:
   - chỉ gỡ sau khi hết cửa sổ migration và có release note rõ ràng.

## Chính sách config và runtime path
1. Config:
   - ưu tiên `Ocp.toml` / `ocp.toml`.
   - fallback `Ocp.toml` / `ocp.toml`.
2. Runtime/cache/artifacts:
   - tên đích: `.ocp_state`, `.ocp_cache`, `.ocp_artifacts`.
   - fallback đọc legacy `.ocp_*` trong giai đoạn chuyển tiếp.
3. Tương thích ngược:
   - project cũ vẫn phải chạy được ở các bản transition.

## Gate kiểm chứng
1. CLI version/help hiển thị `ocp`.
2. Alias `ocp` chạy được và có cảnh báo deprecate.
3. Legacy path/project cũ vẫn load được qua fallback.
4. CI guard vẫn chặn regress extension/package ngoài scope.
