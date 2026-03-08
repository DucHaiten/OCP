# Rules - Policy As Code

Mục tiêu: biến các nguyên tắc trong `AGENTS.md` thành kiểm tra fail-hard, để sai là bị chặn ngay.

## Cấu trúc
- `Rules/AGENTS.md`: bản mirror để quản lý tập trung trong thư mục Rules.
- `Rules/COVERAGE.md`: bản đồ coverage giữa AGENTS và các check tự động.
- `Rules/guard/guard_repo.py`: guard chính (UTF-8, mojibake, cấu trúc plan, DONE-closeout, anti-bulk-edit).
- `Rules/hooks/pre-commit`: chạy guard theo staged changes.
- `Rules/hooks/pre-push`: chạy guard full-scan trước khi push.
- `Rules/install_hooks.ps1`: script cài hooks path về `Rules/hooks`.

## Cài hooks
```powershell
powershell -ExecutionPolicy Bypass -File Rules/install_hooks.ps1
```

## Chạy guard thủ công
```powershell
python Rules/guard/guard_repo.py --mode changed --staged
python Rules/guard/guard_repo.py --mode all
```

## Chính sách fail-hard
- Sai UTF-8 hoặc mojibake trong `AGENTS.md`/`OCP-MVP-PLAN-v*.md` -> FAIL.
- Plan thiếu cấu trúc bắt buộc (`Template`, `Status`, `Operational commands`, `Planning Freeze`, `Implementation Closeout`) -> FAIL.
- Gate `DONE` nhưng thiếu log `Planning Freeze` hoặc `Implementation Closeout` -> FAIL.
- Block `Implementation Closeout` thiếu field bắt buộc -> FAIL.
- Sửa plan quá lớn trong 1 commit (nghi ngờ rewrite/snapshot) -> FAIL.
  - Override tạm thời: đặt env `OCP_RULES_ALLOW_LARGE_DOC=1`.

## Ghi chú
- Guard này chặn tối đa phần có thể chặn bằng code.
- Các mục thuộc hành vi hội thoại (ví dụ "giải thích rõ ràng") không thể khóa tuyệt đối bằng hook; được theo dõi trong `Rules/COVERAGE.md`.
