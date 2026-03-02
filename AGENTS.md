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
