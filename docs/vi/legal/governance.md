# Governance OCP-OCL v1.0

Tài liệu này giải thích cách OCP-OCL được quản trị ở mức public-facing:
- ai là người có quyền quyết định;
- thay đổi nào cần được kiểm soát chặt;
- người dùng và contributor nên hiểu quyền hạn của dự án theo cách nào.

File này được viết để người đọc hiểu mô hình governance của dự án.  
Nó không thay thế các file gốc ở root như `GOVERNANCE.md`, `CODEOWNERS`, hoặc các rule đã khóa trong plan và contracts.

## Tóm tắt ngắn

Nếu bạn chỉ cần hiểu rất nhanh:

1. OCP-OCL hiện là dự án do một owner chính điều phối.
2. Không phải mọi contributor đều có quyền quyết định policy, release, hoặc licensing direction.
3. Các thay đổi lớn phải đi qua plan gate, test, và evidence; không được chốt bằng cảm tính.
4. “Official build” và quyền phát hành chính thức phải gắn với trust chain, không phải ai fork cũng tự nhận được.

## 1) Governance trong OCP-OCL dùng để làm gì?

Governance ở đây không phải là lý thuyết trừu tượng.

Nó dùng để trả lời các câu hỏi rất thực tế:
- ai có quyền chốt release;
- ai có quyền đổi policy;
- ai có quyền đổi hướng licensing/commercial;
- contributor gửi thay đổi thì được xử lý theo quy trình nào;
- thế nào mới được gọi là bản phát hành chính thức.

Nếu không có governance rõ, dự án rất dễ gặp các vấn đề:
- tranh cãi ai có quyền quyết định;
- mơ hồ giữa contribution và quyền sở hữu quyết định cuối;
- mơ hồ giữa fork/community build và official build;
- khó giữ nhất quán giữa docs, legal, release, và trust chain.

## 2) Mô hình hiện tại của OCP-OCL nên hiểu thế nào?

Theo trạng thái repo và plan hiện hành, OCP-OCL đang ở mô hình:
- owner-led project;
- release/policy direction được giữ chặt;
- contribution được chấp nhận theo workflow rõ, không mặc định kéo theo quyền quyết định chiến lược.

Nói dễ hiểu:
- dự án có thể nhận contribution,
- nhưng quyền quyết định cuối cùng về release, legal direction, governance, và commercial direction không tự động chia đều cho mọi người gửi PR.

## 3) Ai có quyền quyết định những việc lớn?

Ở mức public-facing, người đọc nên hiểu:
- owner/core release authority của dự án là bên có quyền chốt:
  - release chính thức,
  - policy licensing,
  - commercial direction,
  - governance changes,
  - trust-root và official build direction.

Điều này không có nghĩa là contributor không quan trọng.

Nó chỉ có nghĩa là:
- đóng góp kỹ thuật và phản hồi cộng đồng là đầu vào;
- còn quyết định cuối cùng với các vấn đề chiến lược và pháp lý phải có owner authority rõ ràng.

## 4) Contributor có quyền gì, và không có quyền gì?

### 4.1) Contributor có thể làm gì

Contributor có thể:
- đề xuất thay đổi code, docs, contracts;
- báo lỗi, đề xuất cải tiến;
- thảo luận về thiết kế, UX, docs, hoặc workflow;
- giúp cải thiện chất lượng dự án.

### 4.2) Contributor không tự động có quyền gì

Contributor không tự động có quyền:
- tự chốt release chính thức;
- tự đổi licensing model;
- tự đổi commercial policy;
- tự gắn nhãn một bản build là official;
- tự sửa governance direction mà không đi qua quy trình owner-controlled.

Nói ngắn gọn:
- gửi contribution không đồng nghĩa với có quyền điều hành dự án.

## 5) Những thay đổi nào phải được kiểm soát chặt?

Các thay đổi sau phải được coi là nhạy cảm và không nên xử lý như thay đổi nhỏ thông thường:

- licensing model;
- commercial policy;
- CLA/contribution policy;
- trust-root và release signing;
- định nghĩa official build;
- branding/trademark rules;
- release gate status;
- governance/ownership files ở root.

Với OCP-OCL, các thay đổi như vậy phải đi theo hướng:
- có plan rõ,
- có evidence,
- có test phù hợp nếu phần đó machine-checkable,
- có closeout đúng chuẩn.

## 6) Official build nghĩa là gì trong logic governance?

Ở mức governance, “official build” không nên được hiểu là:
- bản nào nhìn giống OCL thì là official;
- bản nào do người khác build lại thì tự động thành official;
- fork nào giữ tên gần giống thì có thể tự nhận là bản chính thức.

Cách hiểu an toàn là:
- official build phải gắn với release authority của dự án;
- official build phải phù hợp trust chain và release policy của dự án;
- fork/community build có thể tồn tại, nhưng không nên được trình bày như official nếu không thuộc chuỗi phát hành chính thức.

Phần chi tiết hơn về tên/logo/branding sẽ thuộc:
- `TRADEMARK.md`
- `docs/vi/legal/trademark.md`

## 7) Governance liên quan gì tới plan và gate?

Trong OCP-OCL, plan không chỉ là ghi chú công việc.

Plan là một phần của governance thực tế, vì nó khóa:
- scope,
- gate,
- tiêu chí `DONE`,
- evidence,
- tests liên quan.

Điều này có ý nghĩa:
- không ai nên tự tuyên bố một gate đã xong nếu chưa đủ bằng chứng;
- không ai nên tự đổi design hoặc hạ chuẩn mà không ghi rõ lý do;
- governance của dự án gắn chặt với rule “contract-first, evidence-backed”.

Nói dễ hiểu:
- quyết định kỹ thuật lớn phải để lại dấu vết;
- release chính thức không được chốt bằng lời nói miệng.

## 8) Nếu có bất đồng thì xử lý theo nguyên tắc nào?

Ở mức public docs hiện tại, cách hiểu an toàn là:

1. Ưu tiên nguồn sự thật đã khóa:
   - contracts,
   - plan hiện hành,
   - root legal/governance files,
   - release evidence.
2. Nếu nội dung chỉ là đề xuất nhưng chưa được chốt trong các nguồn trên, chưa coi là policy chính thức.
3. Nếu có mâu thuẫn giữa lời giải thích và nguồn gốc đã khóa, ưu tiên nguồn gốc đã khóa.

Điều này giúp tránh tình trạng:
- người này nói một kiểu,
- PR kia ghi một kiểu,
- README ghi kiểu khác,
- cuối cùng không ai biết đâu là chính thức.

## 9) Người dùng cuối cần hiểu governance tới mức nào?

Phần lớn người dùng không cần đọc governance quá sâu.

Nhưng nếu bạn là:
- contributor dài hạn,
- người đánh giá adoption trong tổ chức,
- người quan tâm legal/release legitimacy,
- hoặc người muốn biết ai thật sự có quyền chốt đường đi của dự án,

thì file này giúp bạn hiểu:
- dự án không phải mô hình “ai sửa cũng có quyền quyết định ngang nhau”;
- release và policy direction được kiểm soát rõ;
- official status phải gắn với authority và trust chain.

## 10) File này không thay thế những gì?

`docs/vi/legal/governance.md` không thay thế cho:
- `GOVERNANCE.md` ở root;
- `CODEOWNERS`;
- `TRADEMARK.md`;
- plan và contracts đang khóa decision;
- các evidence release/signoff.

Nó chỉ là bản giải thích để người đọc hiểu cách dự án vận hành ở mức governance.

## 11) Kết luận

Hiểu theo cách ngắn gọn nhất:
- OCP-OCL hiện là dự án có owner authority rõ;
- contribution không đồng nghĩa với quyền điều hành;
- release, licensing, commercial direction, và official build phải đi qua governance rõ ràng;
- nếu chưa được chốt trong source of truth và evidence, thì chưa nên coi là quyết định chính thức.
