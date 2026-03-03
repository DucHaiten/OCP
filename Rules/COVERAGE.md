# Coverage AGENTS -> Rule-Code

## Mức coverage
- `AUTO_HARD`: chặn tự động, fail-hard.
- `AUTO_SOFT`: có check tự động nhưng chỉ cảnh báo.
- `MANUAL`: không thể khóa hoàn toàn bằng hook, cần kỷ luật vận hành.

## Mapping
1. `AGENTS §7, §8, §9, §10, §12, §14, §15, §16`
- Status: `AUTO_HARD`
- Enforce bởi:
  - `Rules/guard/guard_repo.py`
  - `Rules/hooks/pre-commit`
  - `Rules/hooks/pre-push`
- Check:
  - UTF-8 strict
  - phát hiện mojibake
  - cấu trúc plan bắt buộc
  - gate `DONE` bắt buộc có planning + closeout
  - closeout bắt buộc đủ field
  - chống bulk rewrite plan trong 1 commit

2. `AGENTS §1` (cấm lệnh phá hủy), `§3` (shell strict)
- Status: `AUTO_SOFT + MANUAL`
- Enforce:
  - Hook không thấy được mọi lệnh shell realtime.
  - Khuyến nghị chạy qua wrapper script nội bộ nếu muốn nâng lên hard-enforce.

3. `AGENTS §2`, `§4`, `§5`, `§6`, `§11`, `Design alignment`
- Status: `MANUAL + một phần AUTO_HARD`
- Enforce một phần:
  - `DONE` phải có bằng chứng log + closeout.
  - Gate docs thiếu bằng chứng sẽ bị chặn.
- Phần còn lại:
  - chất lượng giải thích, thái độ hợp tác, diễn giải rủi ro là hành vi hội thoại, không thể khóa tuyệt đối bằng hook.

## Nguyên tắc vận hành
- Không có dấu `PASS` guard => không được coi gate là `DONE`.
- Nếu hook fail:
  - sửa đúng lỗi được chỉ ra,
  - chạy lại guard,
  - mới tiếp tục.
