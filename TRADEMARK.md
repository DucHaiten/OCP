# OCP Trademark Policy

## 1) Mục đích

File này giải thích cách tên và thương hiệu OCP được phép sử dụng trong bối cảnh public release, fork, community build, và tài liệu giới thiệu.

Mục tiêu của file này là:
- tránh mạo danh bản phát hành chính thức;
- tránh gây nhầm lẫn giữa official build và fork/community build;
- giữ cho người dùng có thể phân biệt rõ đâu là bản OCP chính thức, đâu là bản dẫn xuất.

## 2) Phạm vi

Policy này áp dụng cho:
- tên dự án `OCP`;
- cách mô tả `Official OCP Build` hoặc các cụm gần nghĩa;
- branding của extension VSCode chính thức;
- cách fork/community build được phép mô tả nguồn gốc của mình.

Policy này không thay thế license.  
Việc được phép dùng mã nguồn theo license không tự động đồng nghĩa với được phép trình bày sản phẩm của bạn như một bản chính thức của OCP.

## 3) Official OCP Build là gì?

Một bản build chỉ nên được gọi là `Official OCP Build` khi đồng thời thỏa các điều kiện sau:

1. Nó thuộc chuỗi phát hành chính thức của dự án.
2. Nó nằm trong signed artifact scope của release tương ứng.
3. Nó xuất hiện trong release manifest chính thức.
4. Nó thuộc trust chain/signature chain mà dự án công bố cho public release.

Nói ngắn gọn:
- bản build chỉ là official khi nó vừa đúng nguồn phát hành, vừa đúng trust chain.

Nếu thiếu một trong các điều kiện trên, không nên gọi bản đó là official.

## 4) Fork và community build được phép làm gì?

Fork hoặc community build được phép:
- nói rõ rằng mình là bản fork;
- dùng mô tả kiểu:
  - `based on OCP`
  - `fork of OCP`
  - `community build derived from OCP`
- mô tả trung thực phần nào của OCP mình đang kế thừa.

Fork hoặc community build không được:
- tự trình bày như thể đó là bản phát hành chính thức của OCP;
- dùng wording gây hiểu nhầm như:
  - `Official OCP`
  - `Official OCP`
  - `OCP Certified`
  - hoặc các cách trình bày khiến người dùng dễ nhầm đó là build chính thức;
- dùng logo/tên theo cách làm mờ ranh giới giữa fork và official build.

## 5) Quy tắc dùng tên “OCP”

Được phép:
- nhắc tới OCP để chỉ nguồn gốc dự án;
- mô tả tương thích hoặc nguồn gốc một cách trung thực;
- dùng câu như:
  - `This project is based on OCP`
  - `This tool is a fork of OCP`

Không được phép:
- đặt tên hoặc trình bày sản phẩm sao cho người dùng bình thường có thể hiểu nhầm đó là bản chính thức nếu thực tế không phải;
- dùng tên `OCP` như thương hiệu chính của một fork mà không làm rõ đó là fork/community build.

## 6) Quy tắc với logo/branding

Nếu chưa có văn bản riêng cấp phép branding khác, cách hiểu an toàn là:
- không dùng logo hoặc bộ nhận diện của OCP để làm cho fork/community build trông như official;
- không tái đóng gói lại build khác rồi trình bày bằng branding chính thức của OCP.

Nếu cần trình bày nguồn gốc dự án, hãy dùng mô tả bằng chữ rõ ràng thay vì bắt chước branding chính thức.

## 7) Quy tắc cho extension VSCode chính thức

Extension VSCode chỉ nên được coi là official khi:
- nó thuộc release chính thức của dự án;
- nó nằm trong signed release scope;
- publisher identity khớp với chuỗi phát hành chính thức của OCP.

Ở trạng thái v1.0 hiện tại, identity public-facing cần coi là official cho extension là:
- publisher: `ocp`

Điều đó có nghĩa:
- extension do publisher khác phát hành không nên được trình bày như extension OCP chính thức;
- fork/community extension phải làm rõ nó là bản dẫn xuất, không phải official extension của dự án.

## 8) Quan hệ giữa trademark policy và trust chain

Trademark policy này gắn trực tiếp với release trust chain:
- không chỉ nhìn tên file hoặc giao diện giống nhau là đủ;
- phải nhìn xem artifact có nằm trong manifest chính thức và chữ ký trust chain hay không.

Nếu một bản build không nằm trong trust chain chính thức, nó không nên được gắn nhãn official chỉ vì:
- tên giống;
- icon giống;
- code gần giống;
- hoặc được build từ source tương tự.

## 9) Điều policy này không làm

File này không:
- cấm fork dự án;
- cấm community build tồn tại;
- cấm mọi hình thức nhắc tới tên OCP;
- thay thế cho license mã nguồn mở.

Nó chỉ đặt ra ranh giới rõ ràng giữa:
- quyền dùng source theo license;
- và quyền trình bày sản phẩm như một bản chính thức của OCP.

## 10) Cách hiểu an toàn cho người dùng

Nếu bạn là người dùng cuối, cách kiểm đơn giản nhất là:
- chỉ tin một build là official khi nó thuộc release chính thức của dự án;
- và khi nó nằm trong manifest/signature chain chính thức.

Nếu bạn thấy một build nói là “OCP” nhưng không chứng minh được điều đó bằng chuỗi phát hành chính thức, hãy coi nó là:
- fork,
- community build,
- hoặc build không đủ bằng chứng official.

## 11) Kết luận

Trademark policy của OCP theo hướng rất đơn giản:
- fork được phép tồn tại và được phép nói rõ nguồn gốc của mình;
- nhưng không được mạo danh bản chính thức;
- chỉ build nằm trong official release + trust chain mới nên được gọi là `Official OCP Build`.
