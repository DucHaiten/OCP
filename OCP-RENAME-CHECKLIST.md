# Checklist đổi tên OCP -> OCP (an toàn, không bỏ sót)

## 1) Khóa scope theo phase (bắt buộc trước khi làm)
- [x] Chốt mapping theo phase:
- [x] `PHASE_1` (khuyến nghị): `OCP` (tên ngôn ngữ) -> `OCP`, `.ocp -> .ocp`.
- [x] `PHASE_2` (nếu bật): `ocp` (CLI) -> `ocp`, `Ocp.toml -> Ocp.toml`, `.ocp_* -> .ocp_*`. (đã khóa mapping + bật CLI alias/warning; migration path `.ocp_*` theo kế hoạch tương thích chuyển tiếp)
- [x] Chốt ngoại lệ giữ nguyên trong `PHASE_1`: `ocp` CLI/binary, `Ocp.toml`, `.ocp_artifacts`, `.ocp_cache` (và path legacy tương đương).
- [x] Khóa cứng trong `PHASE_1` không chạm: `.ocpp`, `.ocppkg`, `.ocpbundle`, file đóng gói nhị phân, và output generated (ví dụ `dist/*`) trừ khi có gate riêng.
- [x] Nếu bật `PHASE_2` (breaking rename CLI/config/artifacts), bắt buộc có kế hoạch alias + migration.
- [x] Chốt allowlist cho các tham chiếu OCP bên thứ ba (ví dụ: Object Constraint Language) nếu cần giữ nguyên.

## 2) Audit toàn bộ (chỉ đọc, chưa sửa)
- [x] Quét tên file/path (tracked) còn `ocp`, `OCP`, `.ocp`.
- [x] Quét tên file/path (untracked) còn `ocp`, `OCP`, `.ocp`.
- [x] Quét nội dung (content) chứa `ocp`, `OCP`, `.ocp` trong code, config, script, docs, CI, test.
- [x] Phân loại từng kết quả: `BẮT_BUỘC_ĐỔI` / `CẦN_XEM_NGỮ_CẢNH` / `GIỮ_NGUYÊN`.

## 3) Quy tắc thay thế chống đổi nhầm
- [x] Thay theo token/biên từ; cấm replace mù toàn chuỗi con.
- [x] Tách case rõ ràng: lowercase/UPPER/CamelCase/PascalCase/snake_case.
- [x] Chỉ đổi extension thật ở đuôi file (`*.ocp -> *.ocp`).
- [x] Không đổi giá trị hash, binary blob, checksum, hoặc snapshot không liên quan.

## 4) Đổi tên đường dẫn và file
- [x] Rename file extension `.ocp -> .ocp`.
- [x] Chỉ rename path thuộc vùng `BẮT_BUỘC_ĐỔI` và không nằm trong ngoại lệ `PHASE_1`.
- [x] Cập nhật import/include/path/reference bị ảnh hưởng.

## 4.1) Chính sách Source vs Generated (bắt buộc)
- [x] Lập danh sách `SOURCE_OF_TRUTH` và `GENERATED_ARTIFACTS` trước khi rename.
- [x] Không sửa tay file generated; chỉ sửa source rồi regenerate.
- [x] Nếu repo đang track generated artifacts, bắt buộc regenerate + đối chiếu diff trước/sau.
- [x] Nếu chưa regenerate/verify được generated artifacts, không đóng `DONE` cho gate rename tương ứng. (policy đã được áp dụng trong checkpoint trước khi gỡ blocker)
- [x] Tiến độ một phần: đã regenerate `editor/vscode/ocp/dist/dapRuntime.js` (kèm `dist/runtimePaths.js`) từ source bằng `tsc` (xem Checkpoint 4).
- [x] Đã gỡ chặn regenerate `projects/ocp/apps/composer-demo/src/generated/*` bằng cách vá permission mapping `std.fs.read/list/write` + cập nhật `[permissions.std_fs]`, sau đó chạy lại `ocp compose`.

## 5) Template Gate (bắt buộc để tránh sót sâu)
- [x] Chạy `init` cho toàn bộ template/preset đang ship (ví dụ: `tool-cli`, `tool-http`, `tool-proc`, `tool-wallclock`, `mini-game`, `shadow-preview`, `dep-permission`, `preset-*`).
- [x] Verify project mới tạo ra dùng `.ocp` và entrypoint trỏ đúng `.ocp`.
- [x] Chạy `check/run/replay` trên project mới tạo từ từng template quan trọng.

## 6) Editor/VSIX Gate
- [x] Cập nhật file association cho `.ocp`.
- [x] Cập nhật grammar/snippets/icon/language metadata liên quan.
- [x] Cập nhật `activationEvents`/command bridge nếu có.
- [x] Mở file `.ocp` phải có highlight + diagnostics đúng.

## 7) Đổi nội dung mã nguồn theo cụm nhỏ
- [x] Parser/lexer/grammar nhận diện `.ocp`.
- [x] CLI/help/error text đổi sang `ocp` theo scope đã chốt.
- [x] Config/build/CI/script đổi key/path liên quan theo scope.
- [x] Public API/SDK constants (nếu có) được cập nhật nhất quán.

## 8) Cập nhật test + dữ liệu kiểm thử
- [x] Sửa fixtures/snapshots/golden có `ocp/.ocp` trong vùng bắt buộc đổi.
- [x] Bổ sung targeted tests cho hành vi `.ocp`.
- [x] Nếu cần tương thích ngược: test `.ocp` + cảnh báo deprecate.

## 9) Tương thích ngược (khuyến nghị giai đoạn chuyển tiếp)
- [x] Chốt `compat_mode` trước khi làm: `parse_legacy_extension` hoặc `migrate_tool_only`.
- [x] Cho phép đọc `.ocp` trong thời gian chuyển tiếp.
- [x] In cảnh báo deprecate rõ ràng khi dùng `.ocp`.
- [x] Nếu đổi CLI ở `PHASE_2`: hỗ trợ alias `ocp -> ocp` có warning.
- [x] Tài liệu migration mô tả timeline bỏ hỗ trợ legacy.

## 9.1) State/Cache/Replay Migration Gate
- [x] Kiểm kê toàn bộ path/state legacy: `.ocp_state`, `.ocp_artifacts`, cache/replay liên quan.
- [x] Chốt chính sách theo phase: giữ nguyên (`PHASE_1`) hoặc migrate (`PHASE_2`) có fallback đọc dữ liệu cũ.
- [x] Có targeted tests cho migration/compat state-cache-replay (nêu rõ dữ liệu trước/sau).
- [x] Nếu giữ nguyên ở `PHASE_1`, phải ghi allowlist CI tương ứng và migration note cho phase sau.

## 10) CI Guard chống tái sót
- [x] Chốt rõ phạm vi scan CI: `include_roots` và `exclude_roots` (ví dụ loại `target/`, `dist/`, `node_modules/`, `.git/`, `.venv/`).
- [x] `Tracked-only guard`: CI fail nếu tracked files còn `*.ocp` trong vùng `BẮT_BUỘC_ĐỔI`.
- [x] `Content guard`: CI fail nếu còn pattern `\\.ocp\\b`, `\\bOCP\\b`, `\\bocp\\b` trong tracked files thuộc `include_roots`.
- [x] `Untracked guard`: chỉ scan untracked trong `include_roots` đã chốt; không scan ngoài scope.
- [x] CI fail nếu phát hiện đổi nhầm `.ocpp`, `.ocppkg`, `.ocpbundle` khi chưa bật gate tương ứng.
- [x] Dùng allowlist tường minh cho vùng `GIỮ_NGUYÊN` (kèm lý do).
- [x] Chạy guard ở cả pre-merge CI và lane regression.

## 11) Xác minh sau đổi
- [x] Quét lại tên file/path + nội dung, bảo đảm không còn `ocp/.ocp` trong vùng `BẮT_BUỘC_ĐỔI`.
- [x] Rà soát lại vùng `GIỮ_NGUYÊN` để xác nhận giữ có chủ đích.
- [x] So sánh trước/sau cho các path trọng yếu.

## 11.1) Contract Signature Gate (bắt buộc nếu chạm file ký số)
- [x] Mọi contract/report signed đổi nội dung phải regenerate `.sig` tương ứng.
- [x] Chạy verify signature targeted cho toàn bộ cặp `file + .sig` bị ảnh hưởng.
- [x] Đồng bộ lại hash/inventory/manifest liên quan sau rename.
- [x] Nếu file signed bị đổi tên/path, bắt buộc cập nhật `pointer layer` (manifest index/reference theo path). (`N/A` cho PHASE_1 hiện tại: chưa có rename path signed file)
- [x] Ghi đầy đủ evidence: files changed, commands run, test/verify results.

## 12) Test theo thay đổi
- [x] Targeted tests cho rename + extension handling: PASS.
- [x] Regression tests hỗ trợ: PASS.
- [x] Không đóng `DONE` nếu targeted tests chưa pass. (các gate đã đóng đều có targeted tests PASS)

## 13) Tài liệu và bàn giao
- [x] Cập nhật README/docs/examples theo tên mới.
- [x] Viết migration note: thay gì, vì sao, ảnh hưởng gì.
- [x] Chốt checklist với bằng chứng: files changed, commands run, test results.

## Gate thực thi đề xuất
- Gate A: Audit + phân loại + khóa scope phase.
- Gate B: Chốt `SOURCE_OF_TRUTH` vs `GENERATED_ARTIFACTS`.
- Gate C: Rename path/file + fix references.
- Gate D: Runtime/parser/CLI + targeted tests.
- Gate E: Template Gate + Editor/VSIX Gate.
- Gate F: State/Cache/Replay policy + compat tests.
- Gate G: CI Guard + Contract Signature Gate + quét sạch sau đổi.
- Gate H: Docs/migration + regression + closeout.

## Closeout status (2026-03-08)
- [x] Toàn bộ checkbox trong checklist đã được tick.
- [x] Đã có checkpoint bằng chứng cho các bước triển khai và test (`CHECKPOINT-2..7`).
- [x] Đã re-run targeted test sau lần vá cuối để xác nhận không có regression.
