# OCP Governance

## 1) Mục đích

File này định nghĩa cách OCP được quản trị ở mức dự án:
- ai có quyền quyết định cuối cùng;
- thay đổi nào cần kiểm soát chặt;
- bản phát hành nào được coi là chính thức;
- contribution được tiếp nhận theo nguyên tắc nào.

Đây là file governance gốc ở root repo.  
Các tài liệu diễn giải thêm cho người đọc nằm dưới `docs/vi/legal/` và `docs/en/legal/`.

## 2) Mô hình governance hiện tại

OCP hiện vận hành theo mô hình:
- owner-led project;
- release authority tập trung;
- contribution mở nhưng không tự động kéo theo quyền điều hành dự án.

Nói ngắn gọn:
- dự án có thể nhận contribution từ bên ngoài;
- nhưng quyền quyết định cuối cùng về release, licensing, commercial direction, trust chain, governance, và branding không tự động chia đều cho mọi contributor.

## 3) Release authority

Release authority của OCP có quyền chốt:
- release chính thức;
- gate status cuối cùng trước phát hành;
- trust-root và signed artifact chain;
- nội dung legal/governance/commercial policy chính thức;
- định nghĩa và phạm vi của official build.

Không ai nên tự coi một thay đổi là policy chính thức chỉ vì:
- đã có PR,
- đã có nhánh riêng,
- đã có build nội bộ,
- hoặc đã có bản fork hoạt động được.

## 4) Quyền của contributor

Contributor có thể:
- đề xuất thay đổi code, docs, contracts;
- báo lỗi và đề xuất cải tiến;
- thảo luận về thiết kế và quy trình;
- giúp hoàn thiện test, docs, tooling, UX.

Contributor không tự động có quyền:
- chốt release chính thức;
- đổi licensing model;
- đổi commercial policy;
- đổi trust-root hoặc release authority;
- tự gắn nhãn official cho build hoặc extension;
- đổi governance direction của dự án.

## 5) Quy trình quyết định thay đổi lớn

Các thay đổi lớn phải đi theo nguyên tắc:
- contract-first;
- evidence-backed;
- có plan gate rõ;
- có targeted tests đúng phạm vi nếu phần đó machine-checkable;
- có implementation closeout đúng chuẩn trước khi được coi là hoàn tất.

Các thay đổi sau được coi là thay đổi lớn:
- licensing model;
- community/CLA policy;
- release trust chain;
- installer/release scope;
- official build definition;
- trademark/branding rules;
- root legal/governance files;
- gate status `DONE` trước public release.

## 6) Official build

Một bản build chỉ nên được coi là `Official OCP Build` khi:
- thuộc chuỗi phát hành chính thức của dự án;
- nằm trong signed artifact scope của release;
- phù hợp trust chain đã khóa trong contracts và release evidence.

Build từ fork, rebuild cộng đồng, hoặc bản nội bộ:
- có thể hợp lệ cho mục đích riêng;
- nhưng không nên tự nhận là official nếu không thuộc chuỗi phát hành chính thức.

Chi tiết branding/trademark sẽ được khóa thêm trong `TRADEMARK.md`.

## 7) Quan hệ với các file khác

### `LICENSE`
- quy định license OSS gốc.

### `CONTRIBUTING.md`
- giải thích quy trình đóng góp thường ngày.

### `CODEOWNERS`
- khi có, file này quy định ownership vận hành theo path.

### `TRADEMARK.md`
- khi có, file này quy định cách dùng tên/logo và định nghĩa branding chính thức.

### `docs/vi/legal/governance.md`
- là bản giải thích cho người đọc, không thay thế file root này.

## 8) Cách xử lý khi có mâu thuẫn

Nếu có mâu thuẫn giữa:
- phát biểu trong issue/PR/discussion,
- README/docs,
- build nội bộ,
- hoặc hiểu ngầm của contributor,

thì ưu tiên theo thứ tự:
1. contracts và policy đã khóa;
2. root legal/governance files;
3. plan hiện hành và gate evidence;
4. signed release artifacts.

Nếu một thay đổi chưa được phản ánh trong các nguồn trên, chưa nên coi đó là quyết định governance chính thức.

## 9) Thay đổi file này

Mọi thay đổi với `GOVERNANCE.md` phải được coi là thay đổi nhạy cảm.

Rule:
- không sửa bằng cảm tính;
- phải đi cùng plan gate phù hợp;
- phải nhất quán với legal docs, release policy, trust chain, và contribution policy.

## 10) Kết luận

Governance của OCP hiện dựa trên một nguyên tắc đơn giản:
- contribution là đầu vào mở;
- authority chốt release và policy phải rõ;
- official status phải gắn với trust chain và release evidence;
- thay đổi lớn phải có bằng chứng, không chỉ có ý định.
