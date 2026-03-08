# Commercial Terms OCP v1.0

Tài liệu này giải thích nhánh thương mại của OCP theo cách người dùng và tổ chức có thể đọc để ra quyết định ban đầu.

Mục tiêu của file này không phải là thay thế hợp đồng thương mại cụ thể.  
Mục tiêu của nó là làm rõ:
- khi nào bạn nên trao đổi theo nhánh thương mại;
- nhánh thương mại này có vai trò gì;
- điều gì chưa được phép tự suy diễn nếu chưa có văn bản riêng.

## Tóm tắt ngắn

Nếu bạn chỉ cần kết luận nhanh, đây là 4 điểm quan trọng nhất:

1. OCP có mô hình `dual-license`.
2. Nhánh OSS vẫn là `AGPL-3.0-only`.
3. Nhánh thương mại không tự phát sinh chỉ vì bạn là công ty hay dùng trong bối cảnh thương mại.
4. Nếu bạn cần điều khoản khác với nhánh OSS, bạn phải đi qua một thỏa thuận thương mại riêng.

## 1) Nhánh thương mại của OCP là gì?

Nhánh thương mại là con đường cấp quyền riêng cho các trường hợp không muốn hoặc không thể dùng OCP chỉ theo nhánh OSS.

Nói đơn giản:
- nếu bạn dùng theo nhánh OSS, bạn đọc và tuân theo `LICENSE`;
- nếu bạn cần điều khoản khác, bạn không được tự suy diễn rằng dự án đã cho phép sẵn;
- bạn phải có một thỏa thuận thương mại riêng thì mới được coi là đang ở nhánh thương mại.

Điểm phải hiểu rõ:
- file này không tự nó cấp cho bạn commercial license;
- file này chỉ mô tả khi nào commercial path là phù hợp và cách hiểu an toàn về phạm vi của nó.

## 2) Khi nào nên trao đổi theo nhánh thương mại?

Bạn nên đi theo nhánh thương mại nếu rơi vào một hoặc nhiều trường hợp sau:

- bạn muốn phân phối, embed, hoặc tích hợp OCP vào một sản phẩm proprietary;
- bạn cần điều khoản pháp lý riêng cho tổ chức, doanh nghiệp, hoặc procurement;
- bạn cần support cam kết bằng văn bản;
- bạn cần SLA, warranty, indemnity, hoặc các điều khoản enterprise tương tự;
- bạn cần một vị thế licensing rõ ràng cho pháp chế, kiểm toán, hoặc mua sắm nội bộ;
- bạn cần một ngoại lệ mà nhánh OSS không cung cấp.

Nói ngắn gọn:
- nếu bạn đọc `LICENSE` và thấy nhánh OSS không phù hợp với mô hình sử dụng của mình, đó là dấu hiệu rõ ràng để chuyển sang trao đổi theo nhánh thương mại.

## 3) Những trường hợp nào thường phù hợp với nhánh OSS hơn?

Không phải cứ là tổ chức hoặc có yếu tố thương mại thì mặc định phải mua commercial license.

Về mặt public docs hiện tại, các tình huống sau thường bắt đầu bằng nhánh OSS:
- học tập, nghiên cứu, đánh giá kỹ thuật;
- thử nghiệm nội bộ;
- dự án mã nguồn mở phù hợp với license;
- các use-case không đòi hỏi điều khoản pháp lý hoặc support riêng.

Điều quan trọng là:
- bạn không được tự suy ra “công ty dùng nội bộ thì luôn phải mua”;
- nhưng cũng không được tự suy ra “dùng nội bộ thì chắc chắn không cần quan tâm license”.

Mỗi trường hợp vẫn phải được đánh giá dựa trên nhu cầu thật và khả năng tuân thủ nhánh OSS.

## 4) Nhánh thương mại hiện hứa những gì, và không hứa những gì?

### 4.1) Điều file này có thể nói rõ

Nhánh thương mại tồn tại để hỗ trợ các trường hợp cần:
- điều khoản cấp quyền khác với OSS side;
- điều khoản pháp lý phù hợp tổ chức/doanh nghiệp;
- gói support hoặc cam kết vận hành rõ ràng hơn;
- một đường làm việc sạch cho procurement/compliance.

### 4.2) Điều file này không tự động hứa

File này không có nghĩa là:
- mọi tổ chức dùng OCP đều bắt buộc phải mua;
- mọi use-case enterprise đều tự động được cấp ngoại lệ;
- có sẵn bảng giá, SLA, hoặc warranty mặc định cho mọi người;
- chỉ cần đọc file này là bạn đã có commercial license hợp lệ.

Muốn có quyền và nghĩa vụ theo nhánh thương mại, phải có thỏa thuận riêng được chấp thuận rõ ràng.

## 5) Nếu chưa có thỏa thuận thương mại riêng thì áp dụng gì?

Nếu chưa có một thỏa thuận thương mại riêng được xác nhận, cách hiểu an toàn là:
- bạn chưa ở nhánh thương mại;
- việc sử dụng vẫn phải được đánh giá theo nhánh OSS và file `LICENSE`.

Đây là điểm rất quan trọng để tránh hiểu sai.

Không được tự suy diễn theo kiểu:
- “tôi định mua sau nên giờ được tính là commercial rồi”;
- “tôi đang nói chuyện với dự án nên tạm coi như đã có ngoại lệ”;
- “tổ chức của tôi chắc chắn sẽ trả tiền nên giai đoạn này không cần quan tâm AGPL”.

Nếu chưa có văn bản hoặc xác nhận thương mại riêng, bạn không nên giả định rằng các ngoại lệ thương mại đã tồn tại.

## 6) Quan hệ giữa `commercial.md`, `licensing.md`, và `LICENSE`

Ba file này có vai trò khác nhau:

### `LICENSE`
- là văn bản license OSS gốc;
- dùng để đọc nghĩa vụ pháp lý của nhánh mã nguồn mở.

### `docs/vi/legal/licensing.md`
- giải thích bức tranh tổng thể:
  - OCP dùng license gì;
  - dual-license nghĩa là gì;
  - khi nào nên đọc theo nhánh OSS, khi nào nên đọc theo nhánh thương mại.

### `docs/vi/legal/commercial.md`
- giải thích riêng nhánh thương mại:
  - trường hợp nào nên chuyển sang trao đổi thương mại;
  - điều gì có thể kỳ vọng;
  - điều gì không được tự suy diễn nếu chưa có văn bản riêng.

Nói ngắn gọn:
- `LICENSE` là nguồn pháp lý gốc cho OSS side;
- `licensing.md` là bản giải thích tổng quan;
- `commercial.md` là bản giải thích riêng cho commercial path.

## 7) Quan hệ giữa nhánh thương mại và CLA/contribution policy

Theo policy hiện hành:
- contribution ngoài core team đi qua CLA là bắt buộc;
- dự án khóa `cla_type = license_grant`;
- outbound model là `AGPL-3.0-only+commercial`.

Điều này có nghĩa về mặt vận hành:
- dự án giữ khả năng tiếp tục phát hành OSS side;
- đồng thời vẫn giữ được quyền để vận hành nhánh thương mại một cách hợp lệ.

Với người dùng thương mại, ý nghĩa thực dụng là:
- commercial path không phải lời hứa miệng tùy hứng;
- nó gắn với một contribution/licensing model đã được khóa rõ trong repo.

## 8) Điều dự án hiện không nên nói mập mờ

Đây là các cách diễn đạt không nên dùng nếu chưa có văn bản riêng nói rõ:

- “miễn phí cho nhỏ, thu phí bắt buộc cho lớn”;
- “mọi tổ chức đều phải mua commercial license”;
- “dùng nội bộ là luôn miễn nghĩa vụ license”;
- “trả tiền là được bỏ qua mọi điều kiện khác”.

Nếu sau này dự án có chính sách thương mại cụ thể hơn, nó phải được viết rõ trong legal docs và/hoặc thỏa thuận tương ứng, không nên để người đọc tự đoán.

## 9) Nếu tôi đang cân nhắc nhánh thương mại thì nên làm gì?

Theo cách hiểu an toàn, bạn nên làm theo thứ tự sau:

1. Đọc `LICENSE` để biết OSS side có phù hợp không.
2. Đọc `docs/vi/legal/licensing.md` để hiểu mô hình tổng thể.
3. Nếu thấy cần điều khoản khác, coi đó là tín hiệu để đi theo commercial path.
4. Chỉ coi commercial path là đã có hiệu lực khi có xác nhận hoặc thỏa thuận thương mại riêng.

Ở snapshot public docs hiện tại, file này chủ yếu giúp bạn xác định:
- có nên đi tiếp theo nhánh thương mại hay không;
- và không được hiểu nhánh thương mại theo cách quá rộng hoặc quá mơ hồ.

## 10) Giới hạn của tài liệu này

File này là tài liệu giải thích cho người dùng đọc.

Nó không phải là:
- hợp đồng thương mại hoàn chỉnh;
- báo giá;
- cam kết support/SLA mặc định;
- tư vấn pháp lý.

Nếu bạn đang đưa ra quyết định có ảnh hưởng pháp lý, tài chính, hoặc procurement thực sự, bạn nên:
- đọc `LICENSE`,
- đọc `licensing.md`,
- và chỉ coi nhánh thương mại là có hiệu lực khi có văn bản riêng được chấp thuận.

## 11) Kết luận

Hiểu theo cách thực dụng nhất:
- OCP có nhánh thương mại, nhưng nhánh này không tự động phát sinh;
- nhánh OSS vẫn là điểm mặc định nếu chưa có thỏa thuận riêng;
- commercial path tồn tại để xử lý các nhu cầu không phù hợp với OSS side;
- mọi ngoại lệ hoặc quyền thương mại riêng phải được xác nhận rõ, không tự suy diễn.
