# Licensing OCP-OCL v1.0

Tài liệu này dành cho người đang muốn trả lời nhanh các câu hỏi thực dụng:
- Tôi có thể dùng OCP-OCL theo hướng mã nguồn mở không?
- Khi nào tôi cần trao đổi theo hướng thương mại?
- Nếu tôi đóng góp vào dự án thì điều đó ảnh hưởng gì tới quyền sử dụng và phát hành sau này?

Đây là tài liệu giải thích cho người dùng đọc.  
Nó không thay thế file `LICENSE`, hợp đồng thương mại riêng, hoặc tư vấn pháp lý.

## Tóm tắt ngắn

Nếu bạn chỉ cần kết luận nhanh, đây là 3 điểm quan trọng nhất:

1. Nhánh mã nguồn mở của OCP-OCL dùng `AGPL-3.0-only`.
2. Dự án theo mô hình `dual-license`, tức là ngoài nhánh mã nguồn mở còn có thể có nhánh thương mại riêng.
3. Nếu bạn đóng góp vào dự án, bạn phải đi qua CLA theo chính sách hiện hành.

## 1) OCP-OCL đang dùng license gì?

License mã nguồn mở chính thức của OCP-OCL là:
- `AGPL-3.0-only`

Đây là điểm cần hiểu thật rõ:
- OCP-OCL không dùng một license tự chế thay cho AGPL.
- Nếu ở đâu đó có các cụm mô tả như `GGPL`, `governed GPL`, hoặc cách gọi gần nghĩa, hãy hiểu đó là cách nói về triết lý/quản trị dự án, không phải tên license pháp lý thay thế cho `AGPL-3.0-only`.

Nếu có khác biệt giữa lời giải thích và văn bản license gốc, file `LICENSE` ở root repo mới là nguồn pháp lý chính để đọc theo nhánh mã nguồn mở.

## 2) “Dual-license” ở đây nghĩa là gì?

`Dual-license` không có nghĩa là bạn được trộn hai license tùy ý.

Nó có nghĩa là dự án có hai con đường cấp quyền khác nhau:

### 2.1) Nhánh mã nguồn mở
Bạn dùng OCP-OCL theo `AGPL-3.0-only`.

### 2.2) Nhánh thương mại
Trong một số trường hợp, dự án có thể cung cấp license hoặc điều khoản thương mại riêng, tách khỏi nhánh mã nguồn mở.

Nói đơn giản:
- nếu bạn dùng theo nhánh OSS, bạn phải tuân theo `AGPL-3.0-only`;
- nếu bạn cần điều khoản khác, bạn không được tự suy ra, mà phải đi theo nhánh thương mại riêng.

## 3) Khi nào tôi có thể dùng theo nhánh mã nguồn mở?

Thông thường, bạn có thể xem xét dùng theo nhánh OSS nếu:
- bạn chấp nhận dùng OCP-OCL theo `AGPL-3.0-only`;
- bạn chấp nhận các nghĩa vụ tương ứng của license đó;
- bạn không cần ngoại lệ pháp lý riêng;
- bạn không cần support/SLA/warranty/indemnity được cam kết bằng văn bản riêng.

Các tình huống thường phù hợp với nhánh OSS:
- học tập, nghiên cứu, thử nghiệm kỹ thuật;
- đánh giá nội bộ trước khi quyết định dùng rộng hơn;
- dùng trong dự án mã nguồn mở phù hợp với license;
- đóng góp lại cho hệ sinh thái theo quy trình của dự án.

Điểm quan trọng:
- dự án không tự động cấp ngoại lệ chỉ vì bạn là cá nhân, nhóm nhỏ, startup, hoặc “mới dùng thử”.
- nếu bạn định dùng OCP-OCL theo nhánh OSS trong một bối cảnh quan trọng, bạn nên tự đọc kỹ file `LICENSE`.

## 4) Khi nào tôi nên đi theo nhánh thương mại?

Bạn nên trao đổi theo hướng thương mại nếu rơi vào một hoặc nhiều trường hợp sau:
- bạn muốn dùng hoặc phân phối OCP-OCL theo mô hình proprietary;
- bạn cần điều khoản pháp lý riêng cho tổ chức/doanh nghiệp;
- bạn cần support cam kết, SLA, warranty, indemnity, hoặc hồ sơ mua sắm;
- bạn cần một vị thế license rõ ràng cho pháp chế, kiểm toán, hoặc procurement nội bộ;
- bạn cần một ngoại lệ không có sẵn trong nhánh OSS.

Tài liệu chi tiết cho nhánh này sẽ nằm ở:
- `docs/vi/legal/commercial.md`

Nếu file đó chưa hoàn tất trong snapshot hiện tại, cách hiểu an toàn là:
- nhánh thương mại chưa được mô tả đầy đủ ở mức public docs;
- bạn không nên tự suy diễn thêm ngoài những gì đã được viết rõ.

## 5) Dự án hiện không tuyên bố những gì?

Để tránh hiểu sai, hiện tại bạn không nên tự diễn giải OCP-OCL theo các kiểu sau nếu chưa có tài liệu riêng nói rõ:
- “miễn phí cho nhỏ, bắt buộc trả tiền cho lớn”;
- “cứ dùng nội bộ là luôn không cần quan tâm tới license”;
- “mọi tổ chức đều bắt buộc phải mua commercial license”;
- “mọi use-case enterprise đều bị cấm nếu chưa trả tiền”.

Những câu kiểu trên chỉ có giá trị khi được viết rõ trong văn bản pháp lý hoặc chính sách thương mại riêng.

## 6) Nếu tôi muốn đóng góp cho dự án thì sao?

Theo chính sách hiện hành:
- CLA là bắt buộc;
- kiểu CLA đã khóa là `license_grant`;
- phạm vi CLA áp dụng cho:
  - code,
  - docs,
  - contracts.

Về thực chất, điều này được giữ để dự án có thể:
- tiếp tục phát hành nhánh mã nguồn mở một cách nhất quán;
- đồng thời vẫn có khả năng vận hành mô hình dual-license hợp pháp.

Nói ngắn gọn:
- nếu contribution không đi qua quy trình CLA đúng chuẩn, nó không phù hợp với mô hình licensing mà dự án đã khóa.

Nếu bạn muốn đóng góp, hãy đọc thêm:
- `CONTRIBUTING.md`

## 7) Tôi nên đọc file nào nếu cần chắc chắn hơn?

Mỗi file có một vai trò khác nhau:

### `LICENSE`
Đây là văn bản license OSS gốc.  
Nếu bạn cần biết nghĩa vụ pháp lý của nhánh mã nguồn mở, đây là file quan trọng nhất.

### `docs/vi/legal/licensing.md`
Đây là file hiện tại.  
Nó giúp bạn hiểu mô hình licensing tổng thể của dự án bằng ngôn ngữ dễ đọc hơn.

### `docs/vi/legal/commercial.md`
File này sẽ giải thích nhánh thương mại, các tình huống cần commercial terms, và cách trao đổi theo hướng đó.

### `docs/vi/legal/governance.md`
File này sẽ giải thích ai có quyền quyết định các thay đổi lớn về release, policy, governance, và định hướng licensing của dự án.

### `CONTRIBUTING.md`
File này giải thích cách đóng góp vào dự án.  
Nó không thay thế cho license hay CLA, nhưng cần đọc nếu bạn định gửi contribution.

## 8) Giới hạn của tài liệu này

Tài liệu này được viết để:
- giảm hiểu nhầm;
- giúp người dùng và tổ chức ra quyết định ban đầu nhanh hơn;
- tránh tranh cãi kiểu “tôi tưởng dự án dùng license này nhưng thực ra là license khác”.

Nhưng file này không phải là:
- hợp đồng thương mại;
- bản thay thế cho `LICENSE`;
- tư vấn pháp lý.

Nếu bạn đang ra quyết định có tác động pháp lý hoặc thương mại lớn, bạn nên:
1. đọc `LICENSE`,
2. đọc tài liệu thương mại khi file đó hoàn tất,
3. và nếu cần thì tham khảo bộ phận pháp lý của bạn.

## 9) Kết luận

Hiểu theo cách đơn giản nhất:
- OCP-OCL có nhánh mã nguồn mở theo `AGPL-3.0-only`;
- dự án giữ mô hình `dual-license`;
- contribution đi qua CLA là yêu cầu bắt buộc của policy hiện hành;
- nếu bạn cần điều khoản khác với nhánh OSS, bạn phải đi theo nhánh thương mại riêng, không tự suy diễn.
