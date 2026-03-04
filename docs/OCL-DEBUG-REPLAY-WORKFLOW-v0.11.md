# OCL v0.11 - Workflow Debug/Replay End-to-End

Ngày cập nhật: 2026-03-05

Mục tiêu: hướng dẫn luồng làm việc chuẩn khi debug bằng replay-native DX của v0.11, gồm:
- xem trace,
- chạy debugger time-travel,
- so sánh 2 lần chạy,
- rút gọn repro.

## 1) Chuẩn bị artifact chạy
1. Chạy project để tạo artifact:
   - `ocl run <project_dir> --locked --engine dual`
2. Xác định thư mục artifact:
   - `<project_dir>/.ocl_artifacts/<run_id>/`
3. Kiểm tra file tối thiểu:
   - `audit.jsonl`
   - `signature.txt`
   - `replay.toml`

Ghi chú:
- v0.11 dùng `audit.jsonl` làm source-of-truth cho trace/debug/diff.
- Legacy trace pipe chỉ dùng khi bật `--legacy-pipe`.

## 2) Xem trace nhanh
1. Summary JSON:
   - `ocl trace view <artifact_dir> --json`
2. Lọc theo loại sự kiện:
   - `ocl trace view <artifact_dir> --type Observe --json`
3. Lọc theo key/kind/reason:
   - `ocl trace view <artifact_dir> --key std.fs.read_text --kind INSUFFICIENT --reason RC-FS-NOT-FOUND --json`
4. Lọc theo module:
   - `ocl trace view <artifact_dir> --module src/main.ocl --json`

## 3) Debug time-travel
1. Mở debugger:
   - `ocl dbg <artifact_dir>`
2. Lệnh cơ bản:
   - `step`, `back`, `jump <i>`, `continue`
   - `where`, `locals`, `last`, `diffenv`
3. Breakpoint:
   - `break on type Error`
   - `break on key std.fs.read_text`
   - `break on kind INSUFFICIENT`
   - `break on reason RC-FS-NOT-FOUND`
   - `break on loc src/main.ocl:42`

## 4) So sánh hai lần chạy (trace diff)
1. Strict mode:
   - `ocl trace diff <artifactA> <artifactB> --mode strict --out <out_dir>`
2. Align mode:
   - `ocl trace diff <artifactA> <artifactB> --mode align --out <out_dir>`
3. Kết quả:
   - `<out_dir>/diff_report.txt`
   - `<out_dir>/diff_report.json`

## 5) Rút gọn repro (minimizer)
1. Theo error code:
   - `ocl minimize <artifact_dir> --goal error_code:X-READ --out <min_dir>`
2. Theo divergence:
   - `ocl minimize <artifact_dir> --goal divergence --against <artifactB> --out <min_dir>`
3. Theo kind/key:
   - `ocl minimize <artifact_dir> --goal kind:INSUFFICIENT --key std.fs.read_text --out <min_dir>`

Kết quả minimizer:
- `<min_dir>/audit.jsonl`
- `<min_dir>/trace_index.json`
- `<min_dir>/minimize_report.json`
- nếu có cassette: `<min_dir>/cassette/*` đã được prune theo `call_id`
- nếu có fixtures manifest: `<min_dir>/io/fixtures_manifest.json` đã được rút gọn theo trace references

## 6) Quy tắc an toàn khi điều tra lỗi
- Không sửa trực tiếp artifact gốc; luôn ghi output vào thư mục mới.
- Nếu debug lane quarantine, luôn giữ cassette đồng bộ với artifact.
- Dùng diff report để xác định divergence đầu tiên trước khi minimizer.
- Chỉ chốt bugfix sau khi replay lại trên minimized artifact vẫn tái hiện đúng lỗi.

## 7) Checklist vận hành
- [ ] Có artifact hợp lệ (`audit.jsonl`, `signature.txt`, `replay.toml`).
- [ ] `ocl trace view` đọc được artifact.
- [ ] `ocl dbg` điều hướng được tới lỗi.
- [ ] `ocl trace diff` xuất được report text + JSON.
- [ ] `ocl minimize` giảm được trace/repro và giữ mục tiêu lỗi.
