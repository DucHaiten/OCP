# AGENTS.md - Quy Trình Làm Việc An Toàn (Nghiêm Ngặt)

Mục tiêu: giảm tối đa rủi ro mất dữ liệu khi trợ lý làm việc trong repo này.

## 0) Chế độ mặc định
- Luôn bật `SAFE_MODE_STRICT`.
- Không ưu tiên tốc độ. Ưu tiên an toàn dữ liệu.

## 1) Cấm tuyệt đối
- Không chạy bất kỳ lệnh xóa hàng loạt nào (`del`, `rm`, `rmdir`, wildcard xóa).
- Không chạy lệnh quét toàn ổ đĩa (`C:\\`, `D:\\`, `E:\\`, ...).
- Không chạy lệnh nhiều segment có `|`, `&&`, `;` khi chưa có xác nhận rõ ràng.
- Không chạy tiến trình nền.
- Không dùng `git reset --hard`, `git clean -fd`, `checkout --`, hay thao tác phá hủy.

## 2) Nguyên tắc sửa file
- Ưu tiên `apply_patch` cho từng thay đổi nhỏ.
- Không thay đổi nhiều file cùng lúc nếu không cần thiết.
- Mỗi lần sửa: thông báo ngắn trước khi sửa, nếu có rủi ro thì dừng lại và hỏi.
- Không ghi đè nội dung lớn bằng script tự động nếu có thể viết tay.

## 3) Nguyên tắc dùng shell
- Mặc định: KHÔNG dùng shell, trừ chỉ là test không động tới cập nhật sửa dữ liệu hay xóa gì đó.
- Chỉ dùng shell khi người dùng yêu cầu rõ ràng bằng câu lệnh cụ thể.
- Trước mỗi lệnh shell: nêu 1 dòng mục đích + phạm vi tác động.
- Nếu lệnh có khả năng ảnh hưởng ngoài workspace: dừng lại và xin phép.

## 4) Bảo vệ dữ liệu
- Không được xóa/di chuyển file của người dùng nếu chưa có yêu cầu trực tiếp.
- Nếu phát hiện bất thường (file biến mất, thay đổi lớn bất ngờ): DỪNG NGAY, báo người dùng.
- Mỗi bước lớn phải có checkpoint (đã làm gì, file nào bị ảnh hưởng).

## 5) Quy trình trả lời
- Ngắn gọn, rõ ràng, không che giấu rủi ro.
- Nếu chưa chắc: nói rõ giả định, không đoán bừa.
- Ưu tiên phương án ít rủi ro trước.

## 6) Ưu tiên của người dùng trong repo này
- "An toàn dữ liệu" > "Tốc độ" > "Tiện lợi".
- Có thể làm chậm, nhưng không được làm liều.

## 7) Chuẩn viết tài liệu triển khai (bắt buộc)
- Tất cả file `OCP-OCL-MVP-PLAN-v*.md` phải đồng bộ cùng một cấu trúc chuẩn:
  - Header + mục tiêu.
  - Quy ước cập nhật + templates.
  - Trạng thái gate.
  - Scope khóa (in-scope/out-of-scope).
  - Operational commands.
  - Nhật ký triển khai theo cặp:
    - planning freeze
    - implementation closeout
  - Handoff sang bản kế tiếp.
- Mỗi gate bắt buộc có đủ 2 entry:
  - entry kế hoạch trước khi code.
  - entry đóng triển khai sau khi test.
- Mỗi entry triển khai bắt buộc có đủ:
  - `Date`, `Gate/Step`, `Implemented`, `Files changed`, `Commands run`, `Test results`, `Notes/risks`.
- Chỉ ghi lệnh đã chạy thật; không ghi lệnh “dự kiến” vào mục triển khai.
- Test kết quả phải ghi rõ trạng thái `PASS/FAIL` và phạm vi (targeted/full/lane).
- Tài liệu phải dùng tiếng Việt có dấu, UTF-8 chuẩn; cấm mọi dạng lỗi mã hóa/vỡ dấu.
- Khi dọn tài liệu:
  - ưu tiên bỏ phần trùng/lỗi/legacy.
  - không được xóa mất bằng chứng kỹ thuật quan trọng (files changed, commands run, test results).
- Không copy snapshot thô vào tài liệu chép tay.
  - Phải cập nhật thủ công từng mục bằng `apply_patch`, theo thay đổi nhỏ.
  - Không ghi đè cả file bằng script tự động nếu không thật sự cần.

## 8) Checklist trước khi kết thúc mỗi gate
- [ ] Gate status đã cập nhật đúng (`TODO/IN_PROGRESS/DONE`).
- [ ] Có đủ planning freeze + implementation closeout.
- [ ] `Files changed` khớp với phần code thực tế.
- [ ] `Commands run` là lệnh đã chạy thật.
- [ ] `Test results` đã ghi rõ PASS/FAIL.
- [ ] Không còn lỗi tiếng Việt/mã hóa.

## 9) Tiêu chí "Nhìn Là Hiểu" cho tài liệu triển khai
- Mỗi file plan `OCP-OCL-MVP-PLAN-v*.md` phải có một khối tóm tắt ở đầu file, đủ để đọc trong 30-60 giây:
  - Mục tiêu phiên bản.
  - Trạng thái tổng quan.
  - Gate đang làm/đã xong/chưa làm.
  - Bước kế tiếp ngay lập tức.
  - Lệnh kiểm chứng chuẩn.
  - Danh sách file code trọng yếu đã thay đổi.
- Người đọc không cần mở code vẫn phải nắm được:
  - quy trình làm việc trước/sau,
  - đã thêm gì, sửa gì, vì sao,
  - test nào đã chạy và kết quả.
- Nhật ký chi tiết vẫn giữ bên dưới, nhưng phần đầu phải là bản đồ điều hướng nhanh.

## 10) Mặc định khi viết/cập nhật tài liệu plan
- Mặc định `FULL_LOG_MODE` cho mọi file `OCP-OCL-MVP-PLAN-v*.md`.
- `FULL_LOG_MODE` nghĩa là:
  - Không tự ý tóm gọn so với các bản trước.
  - Giữ mức chi tiết tương đương giữa các phiên bản (v0.1, v0.2, v0.3, ...).
  - Mỗi gate phải có đủ cặp:
    - planning freeze
    - implementation closeout
  - Mỗi closeout phải đủ: `Files changed`, `Commands run`, `Test results`, `Notes/risks`.
- Nếu muốn chuyển sang bản rút gọn:
  - Phải nêu rõ trước khi sửa: "Bản này sẽ rút gọn so với full log".
  - Chỉ được rút gọn khi người dùng đồng ý rõ ràng.
- Nếu vô tình làm ngắn hơn chuẩn full log:
  - Phải khôi phục ngay về chuẩn full log trong cùng phiên làm việc, không chờ nhắc lần sau.

## 11) Quy tắc bắt buộc về kiểm tra mojibake/encoding
- Không được kết luận file bị lỗi mã hóa chỉ dựa trên output shell/tool.
- Nguồn sự thật ưu tiên cao nhất là nội dung hiển thị trong IDE của người dùng.
- Nếu tool hiển thị mojibake nhưng IDE của người dùng hiển thị đúng:
  - Kết luận đây là lỗi decode/view của tool.
  - Không được tự ý sửa file.
- Chỉ được sửa nội dung vì lý do encoding khi người dùng xác nhận rõ ràng dòng/đoạn bị lỗi trong IDE.
- Khi báo cáo tình trạng encoding phải ghi rõ:
  - `Tool-view issue` (lỗi hiển thị từ phía tool), hoặc
  - `File-content issue` (lỗi thật trong file đã được người dùng xác nhận).

## 11) Ưu tiên chuẩn thiết kế 100% (bắt buộc)
- Mặc định mọi triển khai phải theo `DESIGN_100_MODE`:
  - Bám đúng thiết kế đã khóa trong plan hiện hành.
  - Không tự hạ chuẩn, không tự đổi scope, không tự thay bằng bản đơn giản hơn.
- Chỉ được báo `DONE` khi đạt đủ 100% tiêu chí thiết kế của gate:
  - Đủ code delta theo scope.
  - Đủ test đúng phạm vi đã làm.
  - Đủ evidence trong log (files changed, commands run, test results).
- Nếu chưa đạt 100%:
  - Bắt buộc ghi trạng thái `IN_PROGRESS` hoặc `PARTIAL` (không được ghi `DONE`).
  - Bắt buộc nêu rõ phần thiếu, lý do thiếu, và bước tiếp theo để đạt 100%.
- Nếu gặp bất khả thi kỹ thuật:
  - Phải báo ngay, kèm bằng chứng cụ thể (lỗi build/test/lane, giới hạn môi trường).
  - Chỉ đề xuất phương án thay thế sau khi đã nêu rõ đây là phương án tạm, chưa đạt chuẩn thiết kế 100%.
- Cấm lặp lại tình trạng “pass test không liên quan nhưng báo done”.

## 11) Bám thiết kế mặc định (không chờ nhắc)
- Mặc định phải triển khai đúng thiết kế đã ghi trong file plan tương ứng.
- Không được tự ý đổi hướng triển khai khi chưa có xác nhận.
- Chỉ được lệch thiết kế khi có lý do bất khả thi (kỹ thuật, môi trường, phụ thuộc).
- Khi gặp bất khả thi phải dừng đúng điểm, báo ngay:
  - phần nào bất khả thi,
  - vì sao bất khả thi,
  - ảnh hưởng cụ thể,
  - phương án thay thế tối thiểu.
- Cấm “làm tạm rồi mới báo sau”; phải báo trước khi chốt code/đánh dấu `DONE`.
- Mọi implementation closeout phải có dòng xác nhận:
  - `Design alignment: FULL` hoặc
  - `Design alignment: PARTIAL (đã nêu rõ bất khả thi và phương án thay thế)`.

## 12) Chống "DONE giả" (bắt buộc)
- `PASS` của test baseline không đồng nghĩa gate đã hoàn tất.
- Tuyệt đối không chuyển gate sang `DONE` nếu chưa có **code delta thật** cho chính gate đó.
- `verification-only` (chỉ chạy lại test/lane, không có code delta) phải ghi rõ là:
  - `Verification Snapshot`
  - Gate status giữ `TODO` hoặc `IN_PROGRESS` (không được `DONE`).
- Mỗi gate chỉ được `DONE` khi đồng thời có đủ:
  - thay đổi code/runtime/CLI/SDK đúng phạm vi gate,
  - test mới hoặc mở rộng test chứng minh thay đổi của gate,
  - `Commands run` + `Test results` phản ánh đúng thay đổi vừa làm.
- Không dùng lại cùng một bộ lệnh baseline cho nhiều gate khác nhau để đóng trạng thái.
- Nếu phát hiện đã lỡ đánh dấu `DONE` sai:
  - phải hạ trạng thái về `TODO` ngay,
  - thêm `Correction Note` nêu rõ nguyên nhân và phạm vi ảnh hưởng,
  - cập nhật lại quy trình trước khi làm tiếp.

## 13) Checklist đóng gate thật
- [ ] Gate có code delta thật (không chỉ sửa file plan).
- [ ] Có test liên quan trực tiếp đến delta của gate.
- [ ] Có bằng chứng "trước/sau" đủ để chứng minh thay đổi.
- [ ] Nếu chỉ verification-only: đã ghi đúng nhãn và chưa chuyển `DONE`.

## 14) Quy tắc test theo thay đổi (bắt buộc)
- Chỉ được gọi là `PASS` cho gate khi **test đúng phần vừa làm** đã pass.
- Cấm dùng test không liên quan để kết luận gate pass.
- Mỗi gate phải có:
  - test targeted cho code delta của gate (ưu tiên fail-before/pass-after),
  - test regression lane tổng chỉ để phụ trợ, không thay thế targeted test.
- Nếu targeted test chưa có hoặc chưa pass:
  - không được ghi `DONE`,
  - chỉ được giữ `TODO/IN_PROGRESS` và ghi rõ phần còn thiếu.

## 15) Mẫu ghi test bắt buộc trong closeout
- `Targeted tests (must-pass for gate):`
- `Regression tests (supporting only):`
- `Kết luận gate:`
  - Chỉ ghi `DONE` khi nhóm targeted pass.

## 16) 🚨 CẤM TUYỆT ĐỐI CÁCH SỬA DỄ LÀM HỎNG TEXT
### KHÓA CỨNG (không được phép bỏ qua)
- TUYỆT ĐỐI CẤM mọi cách “chuyển mã hàng loạt” hoặc “sửa encoding toàn file”.
- TUYỆT ĐỐI CẤM mọi cách “đọc toàn file -> biến đổi tự động -> ghi đè lại toàn file”.
- TUYỆT ĐỐI CẤM dùng script/lệnh tắt để thay ký tự hàng loạt khi xử lý lỗi tiếng Việt.
- TUYỆT ĐỐI CẤM dùng các kiểu thao tác tương tự `Set-Content`/`Out-File`/re-encode để ghi lại nguyên file plan.
- TUYỆT ĐỐI CẤM ưu tiên tốc độ khi sửa lỗi text.

### Chỉ được làm theo cách an toàn
- Chỉ sửa bằng `apply_patch` theo từng hunk nhỏ, có kiểm soát.
- Mỗi lần chỉ sửa một cụm lỗi ngắn, rồi dừng để kiểm tra.
- Không sửa cả file trong một lần nếu mục tiêu chỉ là vá lỗi tiếng Việt/mã hóa.
- Với file plan lớn (`OCP-OCL-MVP-PLAN-v*.md`), phải ưu tiên vá từng đoạn thủ công, không dùng biến đổi tự động.

### Checkpoint bắt buộc khi sửa text
- Sau mỗi cụm sửa, phải báo checkpoint:
  - đã sửa đoạn nào,
  - còn bao nhiêu đoạn lỗi,
  - có phát sinh rủi ro mới hay không.
- Nếu phát hiện dấu hiệu vỡ chữ/mất dấu tăng thêm:
  - DỪNG NGAY,
  - không sửa tiếp bằng lệnh tắt,
  - báo người dùng trước khi làm bước tiếp theo.

## 17) Lệnh commit mặc định (để không quên)
- Luôn dùng đúng 1 lệnh này khi commit:
  - `powershell -ExecutionPolicy Bypass -File tools/commit_safe.ps1 "Noi-dung-commit"`
- Lệnh trên tự:
  - bỏ stage các build artifacts thường gặp (`target`, `.../ocl-cli/target`, `Rules/guard/__pycache__`),
  - chạy guard staged,
  - commit khi guard pass.
- Không dùng commit trực tiếp nếu đang có nhiều file build/generated trong danh sách thay đổi.
