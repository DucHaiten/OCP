# Trademark OCP-OCL v1.0

Tài liệu này giải thích cách hiểu chính sách trademark/branding của OCP-OCL theo ngôn ngữ dễ đọc cho người dùng, contributor, và bên muốn phân phối build dẫn xuất.

Nó không thay thế file gốc ở root:
- `TRADEMARK.md`

File này chủ yếu giúp trả lời các câu hỏi thực tế:
- Khi nào một build được coi là bản chính thức của OCP-OCL?
- Fork có được nói là dựa trên OCP-OCL không?
- Có được dùng tên hoặc logo của OCP-OCL để phát hành bản riêng không?

## Tóm tắt ngắn

Nếu chỉ cần hiểu rất nhanh:

1. Không phải build nào dùng source OCP-OCL cũng là bản chính thức.
2. Chỉ build nằm trong release chính thức và trust chain chính thức mới nên được gọi là `Official OCL Build`.
3. Fork được phép nói rõ mình dựa trên OCP-OCL.
4. Fork không được trình bày như thể đó là bản chính thức của OCP-OCL.

## 1) Trademark policy này dùng để làm gì?

Trademark policy không quyết định bạn có được dùng source code hay không.  
Việc đó thuộc về license.

Trademark policy dùng để kiểm soát một chuyện khác:
- bạn được phép trình bày sản phẩm của mình như thế nào trước người dùng.

Nói đơn giản:
- license trả lời câu hỏi “có được dùng code không?”;
- trademark policy trả lời câu hỏi “có được gọi build đó là OCP-OCL chính thức không?”.

## 2) Khi nào một build được coi là bản chính thức?

Một build chỉ nên được coi là `Official OCL Build` khi:
- nó thuộc release chính thức của dự án;
- nó nằm trong signed artifact scope của release đó;
- nó xuất hiện trong release manifest chính thức;
- nó thuộc trust chain/signature chain mà dự án công bố.

Nếu thiếu các điều kiện này, cách hiểu an toàn là:
- đó không phải official build.

Điểm rất quan trọng:
- chỉ vì build đó chạy được,
- hoặc build đó dùng cùng source,
- hoặc giao diện trông giống hệt,

không có nghĩa là nó được quyền tự gọi là bản chính thức.

## 3) Fork có được tồn tại và phát hành không?

Có.

Fork hoặc community build được phép:
- tồn tại;
- phát hành bản riêng;
- nói rõ rằng mình dựa trên OCP-OCL;
- mô tả trung thực nguồn gốc của mình.

Ví dụ các cách nói chấp nhận được:
- `based on OCP-OCL`
- `fork of OCP-OCL`
- `community build derived from OCP-OCL`

## 4) Fork không được làm gì?

Fork hoặc build dẫn xuất không nên:
- tự gọi mình là `Official OCL Build`;
- tự gọi mình là `Official OCP-OCL`;
- trình bày như thể đó là bản phát hành chính thức của dự án;
- dùng branding theo cách khiến người dùng bình thường khó phân biệt giữa fork và official build.

Nói ngắn gọn:
- được phép nói rõ nguồn gốc;
- không được phép mạo danh.

## 5) Có được dùng tên “OCP-OCL” không?

Có, nhưng phải dùng đúng cách.

Được phép:
- nhắc tới OCP-OCL để nói về nguồn gốc dự án;
- mô tả rằng sản phẩm của bạn dựa trên OCP-OCL;
- nói rõ mối quan hệ giữa bản của bạn và dự án gốc.

Không nên:
- đặt tên hoặc quảng bá theo cách làm người dùng hiểu nhầm đó là bản chính thức;
- dùng tên `OCP-OCL` như thương hiệu chính của một fork mà không làm rõ đó là fork/community build.

## 6) Có được dùng logo hoặc branding của OCP-OCL không?

Cách hiểu an toàn hiện tại là:
- không dùng logo/branding của OCP-OCL để làm cho fork hoặc build riêng trông như official;
- không tái đóng gói build khác rồi trình bày bằng bộ nhận diện khiến người dùng tưởng đó là bản chính thức.

Nếu bạn chỉ cần nói về nguồn gốc, cách an toàn nhất là:
- dùng mô tả bằng chữ rõ ràng;
- tránh bắt chước branding chính thức.

## 7) Extension VSCode chính thức được xác định thế nào?

Ở trạng thái v1.0 hiện tại, extension VSCode chỉ nên được coi là official khi:
- nó thuộc release chính thức của dự án;
- nó nằm trong signed release scope;
- publisher identity khớp với chuỗi phát hành chính thức.

Trong public-facing docs hiện tại, identity chính thức được coi là:
- publisher: `ocp-ocl`

Điều đó có nghĩa:
- extension do publisher khác phát hành không nên được trình bày như extension OCP-OCL chính thức, trừ khi thật sự thuộc chuỗi phát hành chính thức của dự án.

## 8) Trademark policy liên quan gì tới trust chain?

Trong OCP-OCL, trademark không đứng riêng khỏi release integrity.

Một build được coi là official không chỉ vì:
- tên file giống,
- icon giống,
- hoặc nội dung gần giống.

Nó còn phải:
- nằm trong release manifest chính thức;
- có chữ ký phù hợp trust chain chính thức.

Đây là lý do vì sao người dùng nên nhìn vào release evidence, không chỉ nhìn tên gọi.

## 9) Người dùng cuối nên kiểm thế nào cho nhanh?

Nếu bạn chỉ muốn kiểm rất nhanh một build có phải official hay không, hãy hỏi:

1. Build này có nằm trong release chính thức của dự án không?
2. Nó có nằm trong manifest/signature chain chính thức không?
3. Nó có đang tự nhận là official mà không chứng minh được không?

Nếu không trả lời chắc được ba câu đó, cách hiểu an toàn là:
- đây chưa phải bản official đã được chứng minh.

## 10) Điều file này không làm

File này không:
- cấm fork;
- cấm community build;
- thay thế cho license;
- thay thế cho release verification guide;
- cấp cho ai quyền dùng thương hiệu như official build.

Nó chỉ làm một việc:
- giúp ranh giới giữa official build và build dẫn xuất trở nên rõ ràng.

## 11) Kết luận

Hiểu theo cách đơn giản nhất:
- bạn có thể fork và phát triển từ OCP-OCL;
- bạn có thể nói rõ bản của mình dựa trên OCP-OCL;
- nhưng chỉ build thuộc release chính thức và trust chain chính thức mới nên được gọi là `Official OCL Build`.
