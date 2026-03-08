# HAR (Human-like AI Runtime)

Thu muc nay la workspace rieng cho du an HAR, tach biet voi core OCP.

## Muc tieu
- Phat trien HAR song song tren nen OCP.
- Giu OCP v1.0 o che do on dinh: khong mo rong tinh nang, chi sua khi co loi.

## Nguyen tac tach biet
- Code HAR dat trong `HAR/` (hoac cac thu muc con cua no).
- Khong sua code core OCP khi chua co bang chung loi nam o OCP.
- Moi thay doi core OCP phai la bugfix co test tai hien ro rang.

## Phan loai loi (HAR vs OCP)
- Loi HAR:
  - Tai hien trong `HAR/` va khong xuat hien voi sample/template OCP chuan.
  - Thuong nam o business logic, workflow, hoac du lieu cua HAR.
- Loi OCP:
  - Tai hien voi project OCP toi thieu (khong phu thuoc code HAR).
  - Nam o runtime/CLI/compiler/permission/replay/contracts cua OCP.

## Quy trinh goi y khi gap loi
1. Tai hien loi bang testcase nho nhat trong `HAR/`.
2. Thu tai hien lai bang sample OCP toi thieu ben ngoai HAR.
3. Neu loi chi o HAR: sua trong `HAR/`.
4. Neu loi tai hien duoc tren sample OCP toi thieu: mo bugfix cho OCP core.
