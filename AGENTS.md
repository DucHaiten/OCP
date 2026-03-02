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
- Tài liệu phải dùng tiếng Việt có dấu, UTF-8 chuẩn; cấm lỗi mã hóa kiểu `NgÃ y`, `Ä‘`, `â€”`.
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
