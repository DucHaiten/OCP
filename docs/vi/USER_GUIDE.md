# Hướng Dẫn Sử Dụng OCP v1.0 (VI)

Tài liệu này là điểm vào chính cho người dùng v1.0.  
Mục tiêu: đọc một lần, làm được từ A đến Z mà không cần dò nhiều nơi khác.

Phạm vi tài liệu:
- Cài đặt và xác minh tính toàn vẹn bản phát hành.
- Học ngôn ngữ OCP từ vỡ lòng đến tự viết được chương trình cơ bản.
- Dùng CLI theo luồng chuẩn: `init -> lock -> perm -> build/verify attest -> run/replay -> debug`.
- Vận hành thường ngày: `doctor`, `fix`, `budget analyze`.
- Lỗi thường gặp và cách khắc phục thực tế.

Ghi chú:
- Tài liệu đã gộp cả phần học ngôn ngữ + tra cứu nhanh vào cùng một file.
- Nội dung nâng cao được đẩy xuống phía dưới, sau phần nhập môn.

Quy ước nhãn ví dụ trong file này:
- `Canonical pattern`: mẫu nên học và copy khi tự viết logic mới.
- `Template đang ship`: mẫu bám đúng scaffold hiện tại do CLI sinh ra, dùng để phản ánh trạng thái source thật.
- `Smoke scaffold`: mẫu tối giản để kiểm tra init/run/replay nhanh, không phải pattern đẹp nhất để mở rộng logic.

---

## Mục Lục
- [Tra cứu nhanh](#quick-reference)
- [Lộ trình đọc 1 file duy nhất (A->Z)](#learning-path)
- [Install](#install)
- [Quickstart](#quickstart)
- [Learn OCP](#learn-ocp)
- [5 mẫu code bạn sẽ copy nhiều nhất](#copy-patterns)
- [Manifest mẫu theo template](#template-manifests)
- [Capability starter pack](#capability-starter-pack)
- [Language reference ngắn (ví dụ chạy được)](#language-reference-short)
- [Project mẫu mini-game hoàn chỉnh](#mini-game-project)
- [Project mẫu tool-http hoàn chỉnh](#tool-http-project)
- [Language Quickstart (viết code OCP đầu tiên)](#language-quickstart)
- [Nền tảng cần biết để dùng OCP](#concepts)
- [Permissions](#permissions)
- [Lock Trust Attest](#lock-trust-attest)
- [Run Replay](#run-replay)
- [Debug](#debug)
- [Danh mục lệnh đầy đủ](#commands)
- [Public surface mở rộng quan trọng](#public-surface)
- [Compose / Phenotype](#compose-phenotype)
- [Organ / Kit / Preset](#organ-kit-preset)
- [Editor workflow (VSCode/LSP/DAP)](#editor-workflow)
- [Reactor workflow (run/trace/profile)](#reactor-workflow)
- [Luồng làm việc khuyến nghị](#recommended-workflow)
- [Troubleshooting](#troubleshooting)
- [Phụ lục nâng cao](#advanced-appendix)
- [Reference](#reference)
- [Maintainer Guide](#maintainer-guide)

---

<a id="quick-reference"></a>
## Tra cứu nhanh

Nếu bạn cần bắt đầu ngay, đi theo thứ tự này:
1. Cài đặt và verify bản tải (`Install`).
2. Viết và chạy chương trình đầu tiên (`Learn OCP` + `Language Quickstart`).
3. Chạy flow chuẩn (`Quickstart` -> `Permissions` -> `Lock Trust Attest` -> `Run Replay` -> `Debug`).
4. Nếu kẹt: xem `Troubleshooting`.

Chuỗi lệnh tối thiểu để chạy một project:

```bash
ocp init <project_dir> --template tool-cli
ocp lock sync <project_dir>
ocp lock sign <project_dir> --key <keyid>
ocp lock verify <project_dir> --lock deps.lock.v3
ocp perm snapshot <project_dir>
ocp build <project_dir> --attest
ocp run <project_dir>
ocp replay <artifact_dir>
```

<a id="learning-path"></a>
## Lộ trình đọc 1 file duy nhất (A->Z)

Để học đúng thứ tự, không bị ngợp:
1. `Install` để cài + verify bản tải.
2. `Quickstart` để chạy project đầu tiên.
3. `Learn OCP` + `Language Quickstart` để tự viết/sửa `src/main.ocp`.
4. `Nền tảng cần biết` (mục 1..6) để nắm chắc mô hình `observe -> match -> commit`.
5. `Permissions` -> `Lock Trust Attest` -> `Run Replay` -> `Debug`.
6. `Troubleshooting` khi gặp lỗi thực tế.
7. Chỉ sau đó mới đọc `Phụ lục nâng cao`.

---

<a id="install"></a>
## [install] Install

### 1) Chuẩn bị môi trường
- Hệ điều hành hỗ trợ: Windows, Linux, macOS (theo profile phát hành v1.0).
- Quyền ghi file trong thư mục dự án.
- Nếu dùng VSCode: cần VSCode để cài VSIX editor extension.

### 2) Cài đặt trên Windows (installer)
1. Tải trực tiếp file `ocp-v1.0.0-setup-win-x64.exe` tại:
   - `https://github.com/DucHaiten/OCP/blob/main/installer/releases/ocp-v1.0.0-setup-win-x64.exe?raw=1`
2. Chạy installer và chọn:
   - Thư mục cài đặt trên ổ bạn muốn dùng.
   - Tùy chọn thêm `ocp` vào `PATH` (installer hỏi rõ, khuyến nghị bật).
   - Tùy chọn cài VSCode extension ngay nếu installer phát hiện VSCode.
3. Mở terminal mới, chạy:

```bash
ocp --version
```

Nếu lệnh không nhận:
- Đóng/mở lại terminal.
- Kiểm tra lại `PATH`.

Ghi rõ phạm vi:
- Installer Windows này dùng icon OCP, cho phép chọn thư mục cài, có xác nhận riêng cho `PATH`, cài `ocp.exe` + `ocp-lsp.exe` + `ocp-dap.exe`, và bundle kèm `ocp-vscode-v1.0.0.vsix`.
- Nếu máy có VSCode ở vị trí chuẩn hoặc có `code` CLI, installer sẽ tự cài VSCode extension.
- Nếu auto-install không chạy, dùng file VSIX đã được cài kèm để cài thủ công ở bước `5) Cài VSCode extension`.

### 3) Cài đặt portable (Windows/Linux/macOS)
1. Tải gói portable đúng nền tảng.
2. Giải nén vào thư mục cố định.
3. Thêm thư mục chứa binary `ocp` vào `PATH`.
4. Kiểm tra:

```bash
ocp --version
```

### 4) Kiểm tra tính toàn vẹn bản tải (bắt buộc trước production)
Tối thiểu kiểm:
- `release_artifact_manifest.json`
- `release_artifact_manifest.sig`
- `SHA256SUMS`
- `SHA256SUMS.sig`

Quy trình nhanh ngay tại đây:
1. Tải đủ 4 file integrity ở trên cùng với asset cần cài.
2. So khớp SHA256 asset với `SHA256SUMS`.
3. Verify trust-root trước:

```bash
ocp verify --contract-signature contracts/security/v1.0/signing_trust_root.v1.json.sig
```

Kết quả mong đợi (ví dụ):
- `verify contract signature ok (...)`

4. Verify chữ ký file phát hành (`release_artifact_manifest.sig`, `SHA256SUMS.sig`):
   - kiểm tra `schema = ocp.release.file.sig.v1`,
   - `pubkey_id` và `trust_epoch` phải thuộc trust-root ở bước 3,
   - `file_hash_sha256` phải khớp hash thực tế của file tương ứng.
5. Chỉ cài đặt khi cả hash và signature đều hợp lệ.

Ví dụ tính hash file trên Windows:

```powershell
Get-FileHash release_artifact_manifest.json -Algorithm SHA256
Get-FileHash SHA256SUMS -Algorithm SHA256
```

Nếu có bất kỳ mismatch nào: dừng cài đặt ngay.

Chi tiết đầy đủ (bao gồm verification precedence): `docs/vi/security/verify-download.md`.

### 5) Cài VSCode extension
Nếu bạn dùng VSCode:
1. Nếu installer đã auto-install extension, chỉ cần mở lại VSCode.
2. Nếu chưa có extension, tải hoặc dùng file đã được installer cài kèm: `ocp-vscode-v1.0.0.vsix`.
3. Cài VSIX trong VSCode.
4. Mở file `.ocp` để kiểm syntax highlight; workflow editor nâng cao xem ở mục `Editor workflow (VSCode/LSP/DAP)`.

### 6) Tải source bằng git (build local)
Dùng cách này nếu bạn muốn cài từ source thay vì dùng binary đã build sẵn.

1. Clone repository:

```bash
git clone https://github.com/DucHaiten/OCP.git
cd OCP
```

2. Build và kiểm tra CLI:

```bash
cargo run --bin ocp-cli -- --version
```

3. Nâng cao (build release):

```bash
cargo build --release --bin ocp-cli
```

---

<a id="quickstart"></a>
## [quickstart] Quickstart

### 1) Khởi tạo dự án

```bash
ocp init hello --template tool-cli
cd hello
```

Mục đích:
- Tạo skeleton dự án OCP chuẩn.
- Thiết lập điểm vào để chạy/kiểm tra tiếp theo.

Ghi chú:
- Guide này pin `--template tool-cli` để Quickstart không drift theo template mặc định của từng version CLI.

### 2) Đồng bộ lock trước khi chạy

```bash
ocp lock sync <project_dir>
ocp lock sign <project_dir> --key <keyid>
ocp lock verify <project_dir> --lock deps.lock.v3
```

Mục đích:
- `lock sync`: cập nhật lockfile theo dependency hiện tại.
- `lock sign`: ký `deps.lock.v3` để `lock verify` có đủ chain trust trong strict flow.
- `lock verify`: xác thực lock/trust trước khi build/run.

### 3) Chụp và duyệt permission

```bash
ocp perm snapshot <project_dir> --out-dir <perm_dir>
ocp perm diff <old_snapshot> <new_snapshot> --out <diff_report.json> --approval <permissions.approval.toml>
ocp perm approve <diff_report.json> --approval <permissions.approval.toml> --by <id> --date <YYYY-MM-DD>
```

Mục đích:
- `perm snapshot`: chụp trạng thái quyền hiện tại.
- `perm diff`: so sánh thay đổi quyền.
- `perm approve`: duyệt thay đổi hợp lệ theo policy lane.

Chuỗi file khuyến nghị:
- `perm snapshot` tạo baseline trong `<perm_dir>`.
- `perm diff` đọc 2 snapshot (`<old_snapshot>`, `<new_snapshot>`) và tạo `<diff_report.json>`.
- `perm approve` đọc `<diff_report.json>` để ghi approval vào `<permissions.approval.toml>`.

Ví dụ chạy phát ăn ngay:

```bash
ocp perm snapshot . --out-dir ./.ocp_perm/baseline
ocp perm snapshot . --out-dir ./.ocp_perm/current
ocp perm diff ./.ocp_perm/baseline/permissions.snapshot.json ./.ocp_perm/current/permissions.snapshot.json --out ./.ocp_perm/permission_diff_report.json --approval permissions.approval.toml
ocp perm approve ./.ocp_perm/permission_diff_report.json --approval permissions.approval.toml --by dev.local --date 2026-03-07
```

Gợi ý dùng đúng nhịp:
1. Chụp `baseline` trước khi sửa code/dependency.
2. Sửa xong rồi chụp `current`.
3. So diff giữa 2 file `permissions.snapshot.json`.
4. Chỉ approve khi diff hợp lệ và đã review rõ lý do tăng quyền.

### 4) Build + verify attest

```bash
ocp build <project_dir> --attest
ocp verify --attest <artifact_dir>
```

Mục đích:
- Sinh bằng chứng build/attestation.
- Xác nhận chain trust trước khi chạy thực tế.

### 5) Chạy và replay

```bash
ocp run <project_dir>
ocp replay ./.ocp_artifacts/<run_id>/
```

Mục đích:
- `run`: chạy workflow bình thường.
- `replay`: tái lập theo đúng artifact run để kiểm determinism/signature.

Gợi ý:
- Sau `ocp run`, tìm thư mục run mới nhất trong `.ocp_artifacts/` rồi đưa path đó vào `ocp replay`.

### 6) Debug khi có sự cố

```bash
ocp dbg <artifact_dir>
```

Mục đích:
- Mở luồng debug dựa trên trace/replay.
- Khoanh vùng nguyên nhân fail-honest nhanh hơn.

### 7) Luồng nhanh cho tool có IO (v0.7.2)
Nếu bạn làm tool đọc/ghi file, state cục bộ, hoặc cần test replay ổn định:

```bash
ocp init my-tool --template tool-cli
cd my-tool
ocp test . --golden fixtures/expected --clean
ocp run .
ocp replay ./.ocp_artifacts/<run_id>/
```

Mục tiêu:
- Có scaffold `fixtures` sẵn.
- Có golden compare cho output.
- Có replay metadata đầy đủ cho IO flow.

### 8) Luồng nhanh cho mini-game/shadow-preview (v0.7.3)
Nếu bạn muốn thử app/game có UI hoặc shadow preview:

```bash
ocp init demo-game --template mini-game
ocp run demo-game
ocp replay ./.ocp_artifacts/<run_id>/
```

Hoặc:

```bash
ocp init demo-shadow --template shadow-preview
ocp run demo-shadow --shadow branch-a --shadow-policy forbid_commit
ocp replay ./.ocp_artifacts/<run_id>/
```

---

<a id="learn-ocp"></a>
## Learn OCP

Mục tiêu phần này: sau khi đọc xong, bạn phải tự sửa template và tự dựng được một project OCP nhỏ mà không cần mở thêm file hướng dẫn khác.

### OCP trong 90 giây
- OCP tách rõ 3 bước:
  1. `observe(...)`: lấy dữ liệu/effect handle từ capability đã cấp quyền.
  2. `match` Result4: quyết định nhánh xử lý.
  3. `commit(...)`: chỉ dùng khi muốn chấp nhận side-effect.
- OCP không giả định I/O luôn thành công.
- `replay` là bằng chứng tái lập để audit/debug.

### 5 quy tắc để viết được OCP
1. Muốn đọc dữ liệu ngoài => `observe`.
2. Sau `observe` luôn `match` theo 4-kind.
3. Chỉ `commit` ở nhánh policy/lane cho phép.
4. Gặp `INSUFFICIENT/DEFERRED` => fail-honest (không đi đường tắt).
5. Sau khi chạy, luôn kiểm lại bằng `ocp replay <artifact_dir>`.

### Baseline cú pháp v1.0 (nhập môn chắc chắn dùng được)
Đây là baseline nhập môn, không phải đặc tả đầy đủ toàn bộ ngôn ngữ.

Các cấu phần chắc chắn có trong public workflow:
- `module ...;`
- `let`
- record/list/string/number/bool literals
- `observe(...) -> result;`
- `match result { OK | DEGRADED | INSUFFICIENT | DEFERRED => ... }`
- `commit(result)`
- `condition(true|false)`

### Surface ngôn ngữ mở rộng (public v1.0, đọc sau baseline)
Ngoài baseline, line v1.0 còn có các nhóm construct thường gặp trong code thực chiến:
- `import` / `export`: tách module và tái sử dụng API nội bộ.
- `fn` / `return`: gom logic thuần, giảm lặp trong pipeline.
- bounded loop và thao tác trên list/map: `repeat`, `for ... cap N`, và `for-range` cũ vẫn còn để tương thích.
- `?`, `try/else`, `guard`, và field access trên `Result4` (`r.kind`, `r.reason_code`, `r.audit`).
- reactor tick integration (khi project dùng runtime/event loop tương ứng).

Khi nào nên dùng:
- Dự án nhỏ, mới bắt đầu: giữ baseline để dễ kiểm soát.
- Dự án đã có nhiều module hoặc xử lý batch: mở dần các construct trên.

Quy tắc an toàn:
1. Mỗi lần mở rộng cú pháp, chạy `ocp check <project_dir> --locked`.
2. Luôn chạy lại `ocp run` + `ocp replay` sau khi refactor logic ngôn ngữ.

### Cú pháp tối thiểu đầy đủ (đủ để tự bắt đầu viết độc lập)
Đây là phần “tối thiểu nhưng đủ dùng” để bạn không bị kẹt ở chỗ phải đoán cú pháp.
Trong mục này:
- `module`, `let`, record/list literals, `observe`, `match`, `commit`, `condition` là baseline nhập môn.
- `repeat` và `for ... cap N` là cú pháp mở rộng public, hữu ích khi bạn bắt đầu xử lý batch/bounded loop.

Mẫu cú pháp gọn nhất:

```ocp
// comment 1 dòng
module app.demo;

import toolkit.api.math;

fn dep_ready() {
  return true;
}

let ok = dep_ready();
let xs = [1, 2, 3];
let req = { path: "./out/out.json", text: "{\"status\":\"OK\"}", overwrite: true };

repeat 2 { let ping = true; }
for item in xs cap 2 { let seen = item; }

observe("std.fs.write_text", "tier2", req, budget(5)) -> wr;
match wr {
  OK => { commit(wr); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}
```

Ghi chú:
- Đây là mẫu cú pháp để nhìn nhanh toàn bộ construct; nếu editor báo `unused` ở `import toolkit.api.math`, `ping`, hoặc `seen` thì có thể xóa mà không làm đổi ý nghĩa bài học.

Những quy tắc cú pháp bạn phải nhớ:
- Comment 1 dòng dùng `//`.
- `module ...;`, `import ...;`, `let ...;`, `observe(...) -> r;`, `commit(r);`, `condition(...);` đều kết thúc bằng `;`.
- `fn { ... }`, `match { ... }`, `repeat { ... }`, `for ... cap N { ... }` là khối nên không thêm `;` sau dấu `}` ngoài cùng.
- `match` chuẩn bắt buộc đủ 4 arm: `OK`, `DEGRADED`, `INSUFFICIENT`, `DEFERRED`.
- `repeat N { ... }` lặp đúng `N` lần theo cách bounded.
- `for item in xs cap N { ... }` là vòng lặp bounded trên collection; `cap N` là giới hạn cứng số vòng để tránh chạy không kiểm soát.
- `guard r;` là sugar cho early-exit theo `guard_mode`.
- `let x = try r else { fallback };` là sugar lấy giá trị thành công hoặc fallback từ `Result4`.
- Field access đã có trong public surface, ví dụ: `r.kind`, `r.reason_code`, `r.audit`.

### Bảng `Result4` chuẩn

| Kind | Ý nghĩa thực tế | `reason_code` | Nhập môn có `commit` không? |
| --- | --- | --- | --- |
| `OK` | Thành công theo contract kỳ vọng | Thường rỗng/null hoặc không cần dùng | Có, nếu key đó là commit-able |
| `DEGRADED` | Có kết quả nhưng bị giảm cấp/chất lượng | Có thể có để giải thích lý do giảm cấp | Mặc định nhập môn: không; chỉ commit khi contract/lane cho phép rõ |
| `INSUFFICIENT` | Thiếu quyền, budget, quota, hoặc giới hạn runtime | Thường có và cần đọc | Không |
| `DEFERRED` | Runtime/policy yêu cầu hoãn | Thường có và cần đọc | Không |

Điểm cần nhớ:
- `r.kind` cho biết nhánh hiện tại là gì.
- `r.reason_code` là đầu mối chính để sửa lỗi theo kiểu fail-honest.
- `r.audit` là evidence handle/runtime metadata; hữu ích khi debug, không phải thứ đầu tiên người mới cần thao tác.
- Trong nhánh thành công, bạn có thể dùng `r?` hoặc `try r else { ... }` khi muốn viết gọn hơn `match`.

### Bảng capability phổ biến cho người mới

| Key | Dùng để | Request record typed tối thiểu | Có cần `commit`? |
| --- | --- | --- | --- |
| `std.fs.read_text` | Đọc file text | `{ path: "./fixtures/in/sample.json" }` | Không |
| `std.fs.mkdir` | Tạo thư mục | `{ path: "./out", recursive: true }` | Có |
| `std.fs.write_text` | Ghi file text | `{ path: "./out/out.json", text: "{\"status\":\"OK\"}", overwrite: true }` | Có |
| `std.kv.put` | Ghi state KV cục bộ | `{ key: "tool.last_run", value: "ok", overwrite: true }` | Có |
| `std.time.tick_info` | Lấy tick/time logic deterministic | `{ scope: "tool" }` | Không |

Nguyên tắc tra schema:
- Đây là 5 key phổ biến nhất để bắt đầu.
- Khi dùng key khác, chạy `ocp doc packs` hoặc `ocp doc packs --json` để xem catalog/schema packs trước khi đoán request.
- Nếu payload sai shape, runtime/typecheck sẽ báo các lỗi kiểu `RC-CTX-INVALID` hoặc lỗi contract liên quan.

### Chương trình OCP nhỏ nhất

```ocp
module app.hello;
condition(true);
```

### Ví dụ 1: 1 observe + 1 match (không commit)

```ocp
module app.read_only;

let rd_req = { path: "./fixtures/in/sample.json" };
observe("std.fs.read_text", "tier2", rd_req, budget(5)) -> rd;

match rd {
  OK => { condition(true); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}
```

Giải thích:
- `OK`: đọc được theo kỳ vọng.
- `DEGRADED`: có dữ liệu nhưng không đạt mức kỳ vọng.
- `INSUFFICIENT`: thiếu quyền hoặc thiếu budget.
- `DEFERRED`: cần hoãn theo policy/runtime.

### Ví dụ 2: 2 bước nối nhau (mkdir -> write_text)

```ocp
module app.pipeline;

let mk_req = { path: "./out", recursive: true };
observe("std.fs.mkdir", "tier2", mk_req, budget(5)) -> mk;
match mk {
  OK => { commit(mk); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}

let wr_req = { path: "./out/out.json", text: "{\"status\":\"OK\"}", overwrite: true };
observe("std.fs.write_text", "tier2", wr_req, budget(5)) -> wr;
match wr {
  OK => { commit(wr); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}

condition(true);
```

### Ví dụ 3: đọc input -> ghi report

```ocp
module app.report;

let rd_req = { path: "./fixtures/in/sample.txt" };
observe("std.fs.read_text", "tier2", rd_req, budget(5)) -> rd;

match rd {
  OK => {
    let wr_req = { path: "./out/report.txt", text: "report generated", overwrite: true };
    observe("std.fs.write_text", "tier2", wr_req, budget(5)) -> wr;

    match wr {
      OK => { commit(wr); condition(true); }
      DEGRADED => { condition(false); }
      INSUFFICIENT => { condition(false); }
      DEFERRED => { condition(false); }
    }
  }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}
```

Giải thích:
- Bước 1: đọc input bằng `std.fs.read_text`.
- Bước 2: nếu đọc thành công, tạo request ghi report.
- Bước 3: gọi `std.fs.write_text` và chỉ `commit(wr)` ở nhánh `OK`.

### Từ ví dụ sang tự viết (checklist)
1. Chọn capability key cần gọi (`std.fs.*`, `std.kv.*`, ...).
2. Viết request record đúng schema key đó.
3. Đặt budget hợp lý.
4. `observe`.
5. `match` 4-kind.
6. Quyết định nhánh nào được `commit`.
7. `ocp run <project_dir>`.
8. `ocp replay <artifact_dir>`.
9. Nếu fail: `ocp dbg <artifact_dir>`.

### 3 lỗi người mới gặp nhiều nhất
1. `RC-CTX-INVALID`
- Nguyên nhân: payload không đúng schema key.
- Cách xử lý: đổi sang record typed đúng field.

2. `RC-*-PERMISSION-DENIED`
- Nguyên nhân: thiếu permission trong `Ocp.toml`.
- Cách xử lý: snapshot/diff/approve permission đúng workflow.

3. `INSUFFICIENT`
- Nguyên nhân: thiếu budget hoặc thiếu quyền.
- Cách xử lý: tăng budget có kiểm soát hoặc giảm phạm vi I/O request.

### FAQ người mới
#### Vì sao không gọi write trực tiếp mà phải observe rồi commit?
Vì OCP tách “nhìn thấy effect” và “chấp nhận effect” để audit/policy kiểm soát rõ.

#### DEGRADED có được commit không?
Có thể, nhưng chỉ khi contract capability và policy/lane cho phép. Ở mức nhập môn, mặc định chỉ commit ở nhánh `OK`.

#### `condition(false)` khác crash thế nào?
`condition(false)` là tín hiệu fail-honest theo contract; không phải crash mù.

#### Khi nào dùng `ctx("k=v;...")`?
Chỉ khi đang tương thích code cũ. Code mới ưu tiên record typed.

### Result4 sugar và field access (nâng từ nhập môn lên thực chiến)
Khi code đã dài hơn baseline, bạn có thể dùng sugar để giảm boilerplate:
- `?` trên `Result4`
- `try ... else { ... }`
- `guard r;`
- Field access: `r.kind`, `r.reason_code`, `r.audit`

Ví dụ:

```ocp
observe("std.json.parse", "tier2", { text: "{\"ok\":true}" }, budget(5)) -> rs;
let parsed = try rs else { 0 };
let reason = try rs else { r.reason_code };

observe("std.fs.read_text", "tier2", { path: "./fixtures/in/sample.json" }, budget(5)) -> r;
guard r;
let k = r.kind;
let a = r.audit;
condition(true);
```

Nguyên tắc:
- Sugar phải cho ra semantics tương đương với `match` 4-kind canonical.
- `std.json.parse` trong ví dụ này chỉ dùng để minh họa sugar trên `Result4`; khi cần request/schema chính xác cho key này, hãy tra `ocp doc packs --json`.
- Trong `try r else { ... }`, biến ngầm `r` bên trong khối `else` trỏ tới chính `Result4` gốc; vì vậy `try rs else { r.reason_code }` là cú pháp hợp lệ, không phải typo.
- Luôn chạy lại `ocp check --locked` + `ocp replay` sau khi refactor sang sugar.

### Lộ trình thực hành 15 phút
1. `ocp init hello-ocp --template tool-cli`
2. Sửa `src/main.ocp` theo Ví dụ 2.
3. `ocp run .`
4. `ocp replay ./.ocp_artifacts/<run_id>/`
5. Thử cố ý giảm `budget(1)` để thấy `INSUFFICIENT`, rồi sửa lại.

### 3 bài tập tăng dần để tự tin tự viết

#### Bài 1: đọc một file JSON
Mục tiêu:
- Tự viết được 1 `observe` + `match` chuẩn.

Yêu cầu:
1. Tạo `fixtures/in/sample.json`.
2. Gọi `std.fs.read_text`.
3. Ở nhánh `OK`, chỉ cần `condition(true);`.

Đạt khi:
- `ocp run .` pass.
- `ocp replay <artifact_dir>` pass.

#### Bài 2: đọc input rồi ghi report
Mục tiêu:
- Nối được 2 bước I/O có kiểm soát.

Yêu cầu:
1. Đọc `./fixtures/in/sample.json`.
2. Tạo thư mục `./out`.
3. Ghi `./out/report.txt`.
4. Chỉ `commit` ở các nhánh `OK`.

Đạt khi:
- Sinh ra `./out/report.txt`.
- Artifact replay vẫn pass.

#### Bài 3: cố tình làm fail-honest rồi sửa lại
Mục tiêu:
- Biết đọc lỗi thay vì sửa mò.

Yêu cầu:
1. Giảm `budget(5)` xuống `budget(1)` ở một call.
2. Chạy `ocp run .` để quan sát `INSUFFICIENT` hoặc reason code liên quan.
3. Tăng budget lại hoặc giảm phạm vi request.
4. Chạy lại `ocp run .` và `ocp replay <artifact_dir>`.

Đạt khi:
- Bạn nhìn được `reason_code`.
- Bạn sửa đúng nguyên nhân và replay pass lại.

### Project mẫu hoàn chỉnh nhỏ (copy nguyên là chạy)
Nếu bạn không muốn bắt đầu từ template rồi sửa dần, hãy dựng nguyên cây thư mục sau:

Nhãn ví dụ:
- `Canonical pattern`: project mẫu này được viết để người mới copy và mở rộng logic theo style v1.0.

```text
hello-guide/
  Ocp.toml
  src/
    main.ocp
  fixtures/
    in/
      sample.json
    expected/
      out.json
```

`Ocp.toml` mẫu:

```toml
[package]
name = "hello_guide"
version = "0.1.0"

[project]
lane = "locked_v071"
entry = "src/main.ocp"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[language]
guard_mode = "return"

[permissions.package]
allow = ["std.fs.*", "std.kv.*", "std.time.*"]
deny = []

[permissions.std_fs]
read = ["./fixtures/in/**", "./out/**"]
write = ["./out/**"]
remove = ["./out/**"]
rename = ["./out/**"]
list = ["./fixtures/in/**", "./out/**"]
max_read_bytes = 1048576
max_write_bytes = 1048576
max_list_entries = 500

[permissions.std_kv]
enabled = true
max_keys = 512
max_value_bytes = 65536
key_prefix = "tool."

[permissions.std_time]
enabled = true
tick_mode = "logical"
dt_ms = 16
```

Ghi chú:
- Đây là manifest mẫu onboarding để bạn học nhanh một project đầu tay.
- Nó cố ý cấp đủ nhóm quyền cho ví dụ trong guide, chưa phải cấu hình least-privilege chặt nhất cho production.
- Khi bắt đầu làm project thật, hãy thu hẹp dần `permissions.*` theo đúng key bạn dùng.

`src/main.ocp` mẫu:

```ocp
module app.tool_cli;

let rd_req = { path: "./fixtures/in/sample.json" };
observe("std.fs.read_text", "tier2", rd_req, budget(5)) -> rd;
match rd {
  OK => { }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}

let mk_req = { path: "./out", recursive: true };
observe("std.fs.mkdir", "tier2", mk_req, budget(5)) -> mk;
match mk {
  OK => { commit(mk); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}

let wr_req = { path: "./out/out.json", text: "{\"status\":\"OK\"}", overwrite: true };
observe("std.fs.write_text", "tier2", wr_req, budget(5)) -> wr;
match wr {
  OK => { commit(wr); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}

let kv_req = { key: "tool.last_run", value: "ok", overwrite: true };
observe("std.kv.put", "tier2", kv_req, budget(5)) -> kvp;
match kvp {
  OK => { commit(kvp); condition(true); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}
```

`fixtures/in/sample.json` mẫu:

```json
{
  "input": "sample"
}
```

`fixtures/expected/out.json` tối thiểu:

```json
{"status":"OK"}
```

Cách chạy:
1. `ocp check . --locked`
2. `ocp test . --golden fixtures/expected --clean`
3. `ocp run .`
4. `ocp replay ./.ocp_artifacts/<run_id>/`

Kỳ vọng tối thiểu:
- Có file `./out/out.json` với nội dung JSON hợp lệ, ví dụ `{"status":"OK"}`.
- Có artifact `.ocp_artifacts/<run_id>/`.
- Có thể replay lại không lệch signature.

Ghi chú quan trọng:
- `ocp init --template tool-cli` hiện sinh `src/main.ocp` kiểu compat với `ctx("...")`.
- Trong guide này mình chuyển ngay sang record typed để dạy kiểu viết canonical hơn cho v1.0 user-facing.

<a id="copy-patterns"></a>
### 10) 5 mẫu code bạn sẽ copy nhiều nhất
Đây là 5 pattern thực dụng nhất. Nếu chưa nhớ hết ngôn ngữ, bạn vẫn có thể bắt đầu bằng cách sửa từ các mẫu này.

#### Mẫu 1: đọc file, không commit

```ocp
let rd_req = { path: "./fixtures/in/sample.json" };
observe("std.fs.read_text", "tier2", rd_req, budget(5)) -> rd;
match rd {
  OK => { condition(true); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}
```

#### Mẫu 2: tạo thư mục rồi ghi file

```ocp
let mk_req = { path: "./out", recursive: true };
observe("std.fs.mkdir", "tier2", mk_req, budget(5)) -> mk;
match mk {
  OK => { commit(mk); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}

let wr_req = { path: "./out/out.json", text: "{\"status\":\"OK\"}", overwrite: true };
observe("std.fs.write_text", "tier2", wr_req, budget(5)) -> wr;
match wr {
  OK => { commit(wr); condition(true); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}
```

#### Mẫu 3: đọc input rồi ghi report

```ocp
let rd_req = { path: "./fixtures/in/sample.txt" };
observe("std.fs.read_text", "tier2", rd_req, budget(5)) -> rd;
match rd {
  OK => {
    let wr_req = { path: "./out/report.txt", text: "report generated", overwrite: true };
    observe("std.fs.write_text", "tier2", wr_req, budget(5)) -> wr;
    match wr {
      OK => { commit(wr); condition(true); }
      DEGRADED => { condition(false); }
      INSUFFICIENT => { condition(false); }
      DEFERRED => { condition(false); }
    }
  }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}
```

#### Mẫu 4: ghi state cục bộ bằng KV

```ocp
let kv_req = { key: "tool.last_run", value: "ok", overwrite: true };
observe("std.kv.put", "tier2", kv_req, budget(5)) -> kvp;
match kvp {
  OK => { commit(kvp); condition(true); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}
```

#### Mẫu 5: `guard` và `try ... else`

```ocp
observe("std.fs.read_text", "tier2", { path: "./fixtures/in/sample.json" }, budget(5)) -> r;
guard r;

observe("std.json.parse", "tier2", { text: "{\"ok\":true}" }, budget(5)) -> rs;
let parsed = try rs else { 0 };
let reason = try rs else { r.reason_code };
condition(true);
```

Lưu ý:
- `r` trong `else` là binding ngầm cục bộ, không phải biến `r` ở scope ngoài.

Mẹo dùng nhanh:
- Khi chưa chắc, bắt đầu từ Mẫu 1 hoặc Mẫu 2.
- Khi muốn thêm state nội bộ, chèn Mẫu 4.
- Khi muốn giảm boilerplate sau khi đã chạy ổn bằng `match`, chuyển dần sang Mẫu 5.

<a id="template-manifests"></a>
### 11) Manifest mẫu theo template
Phần này dành cho lúc bạn không chỉ muốn viết `src/main.ocp`, mà còn muốn hiểu ngay template nào kéo theo manifest gì.

Nguyên tắc:
- Các manifest dưới đây bám theo template CLI hiện đang có trong repo.
- Đây là mẫu khởi động/onboarding, không phải profile least-privilege chặt nhất cho production.
- Nếu bạn chỉnh template hoặc runtime line sau này, luôn kiểm tra lại bằng `ocp init ... --template ...` và `ocp check --locked`.

#### `tool-cli` dùng khi nào?
- Tool file/config chạy trong lane locked.
- Cần `std.fs.*`, `std.kv.*`, `std.time.*`.
- Muốn test theo golden + replay ổn định.

Manifest: xem ngay ở mục `Project mẫu hoàn chỉnh nhỏ`; đó chính là manifest kiểu `tool-cli` được viết lại cho flow học tập canonical.

#### `tool-http` dùng khi nào?
- Tool cần HTTP thật theo lane `quarantine`.
- Cần cassette record/replay.
- Chấp nhận env gate `OCP_QUARANTINE=1`.

`Ocp.toml` mẫu `tool-http` theo template hiện tại:

```toml
[package]
name = "tool_http"
version = "0.1.0"

[project]
lane = "quarantine"
entry = "src/main.ocp"

[quarantine]
mode = "record"
max_cassette_bytes = 10485760
max_entries = 2000

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[language]
guard_mode = "return"

[compat]
ctx_string = "deny"
ctx_extra_fields = "warn"

[permissions.package]
allow = ["std.net.http.*", "std.fs.*"]
deny = []

[permissions.std_net_http]
enabled = true
allow_hosts = ["mock.local"]
allow_methods = ["GET"]
timeout_ms = 3000
max_body_bytes = 64

[permissions.std_fs]
read = ["./out/**"]
write = ["./out/**"]
remove = ["./out/**"]
rename = ["./out/**"]
list = ["./out/**"]
max_read_bytes = 1048576
max_write_bytes = 1048576
max_list_entries = 500
```

Luồng chạy tối thiểu:

```bash
ocp init my-http --template tool-http
cd my-http
export OCP_QUARANTINE=1
ocp run .
ocp replay ./.ocp_artifacts/<run_id>/
```

```powershell
ocp init my-http --template tool-http
cd my-http
$env:OCP_QUARANTINE="1"
ocp run .
ocp replay ./.ocp_artifacts/<run_id>/
```

Điểm cần nhớ:
- `tool-http` là record/replay cho IO nondeterministic, không phải locked lane.
- Template source hiện tại cho `tool-http` đã dùng record typed cho HTTP/fs request.
- Mẫu này chủ yếu minh họa `quarantine` + cassette; khi cần đọc payload HTTP thật (`status`, `body`, metadata response), hãy xem `ocp doc packs --json` và artifact replay của chính run đó.

#### `mini-game` dùng khi nào?
- Muốn thử capability app/game theo lane locked.
- Cần `engine.game.run`.
- Muốn chạy/replay một mini scenario mà không phải tự dựng runtime từ số 0.

`Ocp.toml` mẫu `mini-game` theo template hiện tại:

```toml
[package]
name = "mini_game"
version = "0.1.0"

[project]
lane = "locked_v071"
entry = "src/main.ocp"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[language]
guard_mode = "return"

[permissions.package]
allow = ["engine.game.*"]
deny = []

[permissions.std_game]
enabled = true
fixed_dt_ms = 16
rng_streams = ["main", "loot"]
rng_max_count = 32
state_delta_max_bytes = 65536
```

Luồng chạy tối thiểu:

```bash
ocp init demo-game --template mini-game
cd demo-game
ocp run .
ocp replay ./.ocp_artifacts/<run_id>/
```

Điểm cần nhớ:
- `mini-game` template hiện tại vẫn sinh source compat kiểu `ctx("...")` cho `engine.game.run`.
- Điều này phản ánh đúng template đang ship; nếu muốn style user-facing canonical hơn, bạn có thể giữ manifest nhưng viết lại `src/main.ocp` sau khi init.

<a id="capability-starter-pack"></a>
### 12) Capability starter pack
Mục này là “bản đồ khởi động” cho các capability bạn có khả năng dùng sớm nhất.

Nguyên tắc đọc mục này:
- Đây là starter pack để bắt đầu code, không phải catalog đầy đủ toàn bộ capability.
- Chỉ liệt kê key đã có ví dụ/template/test thật trong repo hiện tại.
- Nguồn sự thật chi tiết nhất về schema vẫn là:

```bash
ocp doc packs
ocp doc packs --json
```

#### Nhóm file system (`permissions.std_fs`)

`std.fs.read_text`
- Dùng khi nào: đọc file text/json đầu vào.
- Request tối thiểu:

```ocp
{ path: "./fixtures/in/sample.json" }
```

- Request mở rộng thường gặp:

```ocp
{ path: "./README.md", max_bytes: 128 }
```

- Có `commit` không: không.
- Ghi nhớ: đây là key nhập môn tốt nhất để bắt đầu.
- Result/payload starter:

```text
OK/DEGRADED => { text, truncated, bytes }
```

- Shape đã có test thật:
  - `text`: nội dung text đọc được
  - `truncated`: `true|false`
  - `bytes`: số byte thực trả về
- Nếu file quá lớn theo limit: thường ra `DEGRADED` với `truncated=true`.

`std.fs.mkdir`
- Dùng khi nào: tạo thư mục output/workdir.
- Request tối thiểu:

```ocp
{ path: "./out", recursive: true }
```

- Có `commit` không: có.
- Ghi nhớ: trong flow nhập môn, chỉ commit ở nhánh `OK`.

`std.fs.write_text`
- Dùng khi nào: ghi file text/json/report.
- Request tối thiểu:

```ocp
{ path: "./out/out.json", text: "{\"status\":\"OK\"}", overwrite: true }
```

- Có `commit` không: có.
- Ghi nhớ: nếu muốn giữ ý nghĩa file `.json`, hãy ghi JSON thật thay vì text rời.

#### Nhóm KV (`permissions.std_kv`)

`std.kv.get`
- Dùng khi nào: đọc state cục bộ deterministic.
- Request tối thiểu:

```ocp
{ key: "app.flag" }
```

- Có `commit` không: không.
- Ghi nhớ: key phải khớp prefix/giới hạn trong manifest.

`std.kv.put`
- Dùng khi nào: ghi checkpoint, marker, hoặc state nhỏ.
- Request tối thiểu:

```ocp
{ key: "tool.last_run", value: "ok", overwrite: true }
```

- Biến thể đã có trong test:

```ocp
{ key: "app.answer", value_json: 42, overwrite: true }
```

- Có `commit` không: có.
- Ghi nhớ: nếu ghi structured value, ưu tiên trường kiểu `value_json` khi contract key hỗ trợ.

#### Nhóm time (`permissions.std_time`)

`std.time.tick_info`
- Dùng khi nào: lấy tick/time logic deterministic trong lane locked.
- Request tối thiểu:

```ocp
{ scope: "tool" }
```

- Biến thể đã có trong test:

```ocp
{ tick: 7, dt_ms: 20 }
```

- Có `commit` không: không.
- Ghi nhớ: đây là time logic, khác với wallclock thật.
- Result/payload starter:

```text
OK => { tick, dt_ms }
```

- Trong runtime/test hiện tại, payload thường được encode thành cặp giá trị chuỗi như `"tick" = "7"`, `"dt_ms" = "20"`.

`std.time.wallclock.now`
- Dùng khi nào: cần wallclock thật trong lane `quarantine`.
- Request tối thiểu:

```ocp
{ scope: "tool" }
```

- Có `commit` không: không.
- Ghi nhớ: key này thuộc nhóm nondeterministic; dùng cùng `OCP_QUARANTINE=1`.

#### Nhóm network / process (`quarantine`)

`std.net.http.request`
- Dùng khi nào: gọi HTTP thật có cassette record/replay.
- Request tối thiểu:

```ocp
{ method: "GET", url: "http://mock.local/demo", timeout_ms: 3000, max_body_bytes: 16 }
```

- Có `commit` không: không.
- Ghi nhớ:
  - lane phải là `quarantine`,
  - host/method/body limits phải khớp permission,
  - replay không được fallback sang network thật.
- Result/cassette starter:
  - Nếu host không nằm trong allowlist: `INSUFFICIENT` với reason kiểu `NetHostDenied`.
  - Nếu method không nằm trong allowlist: `INSUFFICIENT` với reason kiểu `NetMethodDenied`.
  - Trong cassette/replay evidence, các field ổn định đã có test gồm:

```text
cap = "std.net.http.request"
call_id
status
truncated
req_hash
```

- Với HTTP, nếu bạn cần đọc full payload map của response để viết logic, hãy xem trực tiếp `ocp doc packs --json` và artifact replay của run thật trước khi hardcode.

`std.proc.exec`
- Dùng khi nào: chạy process thật trong lane `quarantine`.
- Request tối thiểu theo template:

```ocp
{ bin: "mock.proc", args: "--template", timeout_ms: 3000, max_stdout_bytes: 32 }
```

- Có `commit` không: không.
- Ghi nhớ: binary phải nằm trong allowlist `allow_bins`.

#### Nhóm app/game/shadow

`engine.game.run`
- Dùng khi nào: loop app/game deterministic theo runtime pack.
- Request starter hiện có trong template đang ship:

```ocp
ctx("entry_module=app.mini_game;phase=frame;tick=1;stream=main;count=4;input_cap=8;events=key:Space|text:start;draw_cap=8;draw_list=text:1,1,mini-game,12;state_json={\"score\":0}")
```

- Có `commit` không: không theo template starter.
- Ghi nhớ:
  - template hiện tại vẫn dùng compat `ctx("...")`,
  - nếu bạn định viết app/game thật, nên xem thêm `ocp doc packs` và template `mini-game`.

`std.shadow.search`
- Dùng khi nào: tìm/so nhánh shadow bounded.
- Request starter hiện có trong fixture/template:

```ocp
ctx("policy=round_robin;variants_json=[{\"x\":1},{\"x\":2},{\"x\":3}];max_branches=3;per_branch_step_cap=80;per_branch_budget_cap=2000;global_step_cap=240;global_budget_cap=12000;top_k=3;rounds=2")
```

- Có `commit` không: không trong flow shadow search chuẩn.
- Ghi nhớ:
  - đây là capability nâng cao,
  - nên bắt đầu bằng template `shadow-preview` thay vì viết tay từ số 0.

Checklist khi dùng capability mới:
1. Tìm key gần nhất trong starter pack này.
2. Chạy `ocp doc packs` để xem `permission_class`, `ctx_schema`, `payload_schema`, `example`.
3. Viết request record hoặc `ctx("...")` đúng theo key đó.
4. Chạy `ocp check . --locked` hoặc lane tương ứng.
5. Chạy `ocp run .` rồi `ocp replay <artifact_dir>`.

#### Cách đọc `result/payload` theo capability
Điều ổn định nhất ở mọi capability là lớp vỏ `Result4`:
- `r.kind`
- `r.reason_code`
- `r.audit`

Với project đầu tay, đây là cách đọc an toàn:
1. Dùng `match` hoặc `guard` trước.
2. Chỉ đọc sâu vào payload khi bạn đã biết schema của key đó.
3. Nếu chưa chắc payload shape, xem `ocp doc packs --json` trước khi hardcode logic.

Một số payload shape đã có test thật trong repo:

`std.kv.get`
- Thành công thường trả payload map kiểu:

```text
{ found: true|false, value: ... }
```

- Nghĩa là bạn thường quan tâm 2 việc:
  - key có tồn tại không (`found`)
  - giá trị trả về là gì (`value`)

`std.kv.keys`
- Khi bị truncate có thể trả `DEGRADED` với payload map kiểu:

```text
{ keys: [...], truncated: true }
```

`engine.game.run`
- Thành công có thể trả payload map chứa:

```text
{ entry_module, tick, stream, count, rng_values, ... }
```

- Nghĩa là đây không chỉ là “đã chạy game”, mà còn mang state/report đủ để replay/debug.

Với các key như `std.fs.read_text`, `std.time.tick_info`, `std.net.http.request`, `std.shadow.search`:
- Guide này chỉ pin request starter.
- Khi cần đọc payload thật để viết logic, hãy xem:
  1. `ocp doc packs`
  2. `ocp doc packs --json`
  3. artifact replay/audit của chính run đó

<a id="language-reference-short"></a>
### 13) Language reference ngắn (ví dụ chạy được)
Mục này không thay thế đặc tả ngôn ngữ đầy đủ. Nó chỉ gom các construct bạn có thể bắt gặp sớm nhất trong code v1.0.

Ghi chú:
- Một số import/binding trong các ví dụ reference này chỉ để minh họa cú pháp; nếu editor báo `unused` thì có thể xóa mà không làm đổi ý nghĩa bài học.

#### `module`, `import`, `fn`, `return`

```ocp
module app.demo;
import toolkit.api.math;

fn dep_ready() {
  return true;
}

let ok = dep_ready();
condition(ok);
```

Khi dùng:
- `module`: tên module hiện tại.
- `import`: gọi module đã export từ dependency/project khác.
- `fn`/`return`: gom logic thuần, giảm lặp.

#### `repeat` và `for ... cap`

```ocp
module app.loops;

let xs = [1, 2, 3];
repeat 2 { let ping = true; }
for item in xs cap 2 { let seen = item; }
condition(true);
```

Khi dùng:
- `repeat N`: làm đúng `N` lần.
- `for item in xs cap N`: duyệt collection nhưng vẫn bounded.

#### `guard` và `try ... else`

```ocp
module app.sugar;

observe("std.fs.read_text", "tier2", { path: "./fixtures/in/sample.json" }, budget(5)) -> r;
guard r;

observe("std.json.parse", "tier2", { text: "{\"ok\":true}" }, budget(5)) -> rs;
let parsed = try rs else { 0 };
let reason = try rs else { r.reason_code };
condition(true);
```

Khi dùng:
- `guard r;`: dừng sớm theo `guard_mode` nếu `r` không ở nhánh thành công.
- `try rs else { ... }`: lấy value khi thành công, hoặc fallback khi không thành công.
- Trong khối `else`, biến ngầm `r` trỏ tới chính `Result4` đang xử lý.
- `r` trong `else` là binding ngầm cục bộ, không phải biến `r` ở scope ngoài.

Nguyên tắc an toàn:
- Viết được bằng `match` trước, rồi mới refactor sang sugar.
- Sau mỗi refactor, chạy `ocp check . --locked` và `ocp replay <artifact_dir>`.

<a id="mini-game-project"></a>
### 14) Project mẫu mini-game hoàn chỉnh
Đây là project mẫu tối thiểu cho user muốn thử OCP ở hướng app/game chứ không phải tool file.

Nhãn ví dụ:
- `Canonical pattern` cho user mới: mẫu code dưới đây đã thêm `match frame` để bám đúng nguyên tắc nhập môn.
- `Template đang ship`: CLI hiện còn sinh scaffold ngắn hơn, xem ghi chú cuối mục.

Cây thư mục:

```text
demo-game/
  Ocp.toml
  src/
    main.ocp
```

`Ocp.toml`:

```toml
[package]
name = "mini_game"
version = "0.1.0"

[project]
lane = "locked_v071"
entry = "src/main.ocp"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[language]
guard_mode = "return"

[permissions.package]
allow = ["engine.game.*"]
deny = []

[permissions.std_game]
enabled = true
fixed_dt_ms = 16
rng_streams = ["main", "loot"]
rng_max_count = 32
state_delta_max_bytes = 65536
```

`src/main.ocp`:

```ocp
module app.mini_game;

observe("engine.game.run", "tier2", ctx("entry_module=app.mini_game;phase=frame;tick=1;stream=main;count=4;input_cap=8;events=key:Space|text:start;draw_cap=8;draw_list=text:1,1,mini-game,12;state_json={\"score\":0}"), budget(10)) -> frame;
match frame {
  OK => { condition(true); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}
```

Chạy:

```bash
ocp check . --locked
ocp run .
ocp replay ./.ocp_artifacts/<run_id>/
```

Kỳ vọng:
- Có `.ocp_artifacts/<run_id>/audit.jsonl`
- Có `.ocp_artifacts/<run_id>/signature.txt`
- Có `.ocp_artifacts/<run_id>/replay.toml`
- Replay pass ổn định

Ghi chú:
- Đây là mẫu `canonical pattern` cho user mới; nó cố ý xử lý `frame` qua `match` để không dạy sai thói quen.
- Template `mini-game` đang ship hiện tại ngắn hơn và vẫn dùng `ctx("...")` cho `engine.game.run`.
- Scaffold đang ship đó phù hợp cho smoke-test init/run/replay, nhưng không phải pattern đẹp nhất để mở rộng logic mới.
- Với app/game, cách bắt đầu an toàn nhất là init từ template rồi sửa dần, không nên tự bịa request string từ số 0.

<a id="tool-http-project"></a>
### 15) Project mẫu tool-http hoàn chỉnh
Đây là mẫu tối thiểu cho user muốn thử `quarantine` + cassette record/replay bằng HTTP.

Nhãn ví dụ:
- `Canonical pattern` cho user mới: mẫu code dưới đây xử lý `Result4` đầy đủ.
- `Template đang ship`: CLI hiện sinh scaffold ngắn hơn, xem ghi chú cuối mục.

Cây thư mục:

```text
my-http/
  Ocp.toml
  src/
    main.ocp
```

`Ocp.toml`:

```toml
[package]
name = "tool_http"
version = "0.1.0"

[project]
lane = "quarantine"
entry = "src/main.ocp"

[quarantine]
mode = "record"
max_cassette_bytes = 10485760
max_entries = 2000

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[language]
guard_mode = "return"

[compat]
ctx_string = "deny"
ctx_extra_fields = "warn"

[permissions.package]
allow = ["std.net.http.*", "std.fs.*"]
deny = []

[permissions.std_net_http]
enabled = true
allow_hosts = ["mock.local"]
allow_methods = ["GET"]
timeout_ms = 3000
max_body_bytes = 64

[permissions.std_fs]
read = ["./out/**"]
write = ["./out/**"]
remove = ["./out/**"]
rename = ["./out/**"]
list = ["./out/**"]
max_read_bytes = 1048576
max_write_bytes = 1048576
max_list_entries = 500
```

`src/main.ocp`:

```ocp
module app.tool_http;

observe("std.net.http.request", "tier2", { method: "GET", url: "http://mock.local/template-http", timeout_ms: 3000, max_body_bytes: 32 }, budget(8)) -> net;
match net {
  OK => { }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}

observe("std.fs.mkdir", "tier2", { path: "./out", recursive: true }, budget(5)) -> mk;
match mk {
  OK => { commit(mk); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}

observe("std.fs.write_text", "tier2", { path: "./out/http.txt", text: "HTTP_TOOL_OK", overwrite: true }, budget(5)) -> wr;
match wr {
  OK => { commit(wr); condition(true); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}
```

Chạy record:

```powershell
$env:OCP_QUARANTINE="1"
ocp check .
ocp run .
```

```bash
export OCP_QUARANTINE=1
ocp check .
ocp run .
```

Replay:

```powershell
$env:OCP_QUARANTINE="1"
ocp replay ./.ocp_artifacts/<run_id>/
```

```bash
export OCP_QUARANTINE=1
ocp replay ./.ocp_artifacts/<run_id>/
```

Kỳ vọng:
- Có file `./out/http.txt`
- Có `.ocp_artifacts/<run_id>/cassette/cassette.jsonl`
- Cassette có entry `cap = "std.net.http.request"`
- Replay pass khi env gate vẫn bật

Fail-honest cần nhớ:
- Nếu không bật `OCP_QUARANTINE="1"`, cả `run` lẫn `replay` của lane `quarantine` phải fail.
- Nếu host hoặc method không nằm trong allowlist, runtime phải trả `INSUFFICIENT`, không tự hạ chuẩn.

Ghi chú:
- Khác với `tool-cli`, template `tool-http` hiện tại đã dùng request record typed cho HTTP/fs.
- Mẫu ở đây là `canonical pattern` cho user mới; nó xử lý `net`, `mk`, `wr` theo `match` để bám đúng phần nhập môn.
- Template `tool-http` đang ship hiện ngắn hơn, phù hợp cho smoke scaffold nhưng không nên coi là pattern đẹp nhất để viết logic mới.
- Hãy ưu tiên init từ template thật rồi sửa dần về canonical pattern này.
- Mẫu này ưu tiên dạy `quarantine` + cassette + side-effect có kiểm soát; khi cần viết logic đọc `status`/`body` từ response HTTP, hãy tra `ocp doc packs --json` và xem payload trong artifact replay của run thật.

<a id="language-quickstart"></a>
### 16) Viết code OCP đầu tiên (Language Quickstart)
Mục tiêu phần này: bạn sửa `src/main.ocp` và chạy được ngay.

Nhãn ví dụ:
- `Canonical pattern`: phần này là luồng học chính cho user mới, ưu tiên style record typed + `match` đầy đủ.

Quy ước của guide v1.0:
- Code mới: ưu tiên payload record typed.
- `ctx("k=v;...")`: chỉ giữ để tương thích script/template cũ.

Bước 1 - tạo project từ template có IO thực tế:

```bash
ocp init hello-ocp --template tool-cli
cd hello-ocp
```

Bước 2 - cấu trúc tối thiểu cần biết:
- `Ocp.toml`: lane, permissions, entrypoint.
- `src/main.ocp`: code chạy chính.
- `fixtures/`: dữ liệu test mẫu của template.

Ghi chú:
- Template `tool-cli` hiện tại trong CLI vẫn sinh source compat dùng `ctx("k=v;...")`.
- Trong hướng dẫn này, bạn thay ngay sang record typed để học theo style canonical v1.0.

Bước 3 - thay `src/main.ocp` bằng mẫu chạy được:

```ocp
module app.tool_cli;

let mk_req = { path: "./out", recursive: true };
observe("std.fs.mkdir", "tier2", mk_req, budget(5)) -> mk;
match mk {
  OK => { commit(mk); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}

let wr_req = { path: "./out/out.json", text: "{\"status\":\"OK\"}", overwrite: true };
observe("std.fs.write_text", "tier2", wr_req, budget(5)) -> wr;
match wr {
  OK => { commit(wr); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}

let ti_req = { scope: "tool" };
observe("std.time.tick_info", "tier2", ti_req, budget(5)) -> ti;

let kv_req = { key: "tool.last_run", value: "ok", overwrite: true };
observe("std.kv.put", "tier2", kv_req, budget(5)) -> kvp;
match kvp {
  OK => { commit(kvp); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}

condition(true);
```

Bước 4 - chạy:

```bash
ocp run .
ocp replay ./.ocp_artifacts/<run_id>/
```

Bước 5 - đọc kết quả:
- File output ở `./out/out.json`.
- Artifact replay ở `.ocp_artifacts/<run_id>/`.

Cheat sheet cú pháp tối thiểu (để tự viết tiếp):
- Khai báo biến: `let x = 1;`
- Record typed: `let req = { path: "./out/a.txt", overwrite: true };`
- Danh sách: `let items = ["a", "b"];`
- Chuỗi lệnh observe an toàn:
  1. `observe(...) -> r;`
  2. `match r { OK => commit(r); ... }`
- Mẫu chuẩn Result4:

```ocp
match r {
  OK => { /* nhánh thành công */ }
  DEGRADED => { /* nhánh giảm cấp */ }
  INSUFFICIENT => { /* thiếu ngân sách/quyền */ }
  DEFERRED => { /* cần hoãn */ }
}
```

Mẫu xử lý đọc file theo kiểu fail-honest:

```ocp
let rd_req = { path: "./fixtures/in/sample.json" };
observe("std.fs.read_text", "tier2", rd_req, budget(5)) -> rd;
match rd {
  OK => { /* xử lý dữ liệu đọc được */ condition(true); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}
```

---

<a id="concepts"></a>
## [concepts] Nền tảng cần biết để dùng OCP

Phần này là kiến thức sử dụng thực tế.  
Lịch sử chi tiết theo phiên bản nằm ở `docs/plans/history/*` (không nằm trong user guide vận hành).

Cách đọc phần này:
- Mục `1` đến `6`: cốt lõi, nên đọc ngay.
- Mục `7` đến `15`: nâng cao, có thể đọc sau khi đã chạy ổn quickstart/language quickstart.

### 1) Mô hình chạy cốt lõi
1. Mọi đọc dữ liệu ngoài hệ thống đi qua `observe(...)`.
2. Kết quả xử lý luôn theo 4-kind: `OK/DEGRADED/INSUFFICIENT/DEFERRED`.
3. Mọi side-effect phải đi qua `commit(...)` và bị chặn/mở bởi policy.
4. Runtime là bounded: có budget/caps, không cho “chạy vô hạn”.

### 2) Locked vs unlocked (điểm cần nhớ nhất)
- `--locked`:
  - ép lockfiles/trust/signature theo chain đã pin,
  - fail-hard khi thiếu hoặc sai lock/trust.
- `--unlocked`:
  - tiện cho dev nội bộ, nhưng không thay thế release-grade flow.

Lanes quan trọng:
- `locked_v071`: lane mặc định hiện hành.
- `locked_v06`: lane tương thích để replay/đối chiếu baseline cũ.
- `quarantine`: lane real-IO có điều kiện, không thay deterministic SoT lane.

Khuyến nghị:
- Dev nhanh có thể bắt đầu unlocked.
- Trước merge/release luôn chạy locked flow đầy đủ.

### 2.1) Thuật ngữ dễ nhầm (pin nghĩa để dùng thống nhất)
- `tier2`:
  - Mức trust/runtime tier cho observe key trong template mặc định.
  - Khi chưa có policy nội bộ riêng, giữ nguyên theo template để tránh lệch lane.
  - Không đổi tier khi chưa nắm rõ policy lane/project.
- `budget(5)`:
  - Ngân sách work units cho lệnh observe.
  - Giá trị càng thấp càng dễ bị `INSUFFICIENT` nếu tác vụ lớn.
  - Khi gặp `INSUFFICIENT`: tăng budget hoặc giảm phạm vi I/O trong request.
- `guard_mode = "return"`:
  - Guard fail thì trả kết quả sớm theo contract thay vì cố chạy tiếp.
  - Đây là mode an toàn mặc định trong template locked.
- `locked_v071` / `locked_v06` / `quarantine`:
  - `locked_v071`: lane chuẩn để dev/release hiện tại.
  - `locked_v06`: lane tương thích khi cần đối chiếu baseline cũ.
  - `quarantine`: chỉ dùng khi cần capability nondeterministic (HTTP/proc/wallclock).
- `std.ui.*`:
  - Là capability theo mô hình draw-list có governance.
  - Không phải desktop widget framework kiểu Tkinter/Qt trong core runtime.

### 3) Reproducibility và replay
- `run` để chạy thật.
- `replay` để chứng minh tái lập theo dữ liệu đã ghi.
- `trace/profile/dbg` để điều tra nguyên nhân với bằng chứng có cấu trúc.

Nguyên tắc:
- Không “vá bằng cảm giác”; luôn đọc report/trace trước khi sửa policy.

### 4) Cosmology/hive/shadow dùng khi nào
- `--universe/--domain`: khi cần cô lập môi trường, policy, hoặc vùng trạng thái.
- `--shadow`: chạy so sánh mà không commit ra truth world.
- Hive/runtime nâng cao: dùng khi có bài toán nhiều tác vụ có điều phối.

Nếu chỉ làm app CRUD/automation cơ bản:
- Bắt đầu từ preset chuẩn, chưa cần mở tuning sâu.

### 5) Supply-chain và trust
Trước khi coi bản build là hợp lệ, tối thiểu phải qua:
1. lock sync/verify,
2. build + attest,
3. verify attest/supply.

Đây là lý do OCP phù hợp môi trường cần audit/release proof.

### 6) Manifest tối thiểu để chạy lane locked
`Ocp.toml` tối thiểu nên có các phần sau:

```toml
[project]
lane = "locked_v071"
entry = "src/main.ocp"

[language]
guard_mode = "return"

[permissions.package]
allow = ["std.log.info"]
deny = []
```

Ý nghĩa:
- `lane`: xác định policy/runtime lane.
- `guard_mode`: cách `guard` early-exit ở runtime.
- `[permissions.package]`: baseline permission bắt buộc trong locked flow.

<a id="permissions"></a>
## [permissions] Permissions

### 1) Vì sao permission là bắt buộc
OCP không cho side-effect “trôi tự do”.  
Mọi quyền I/O đều phải minh bạch, review được, và có dấu vết.

### 2) Ý nghĩa từng lệnh permission
- `ocp perm snapshot <project_dir> [--out-dir <dir>]`
  - Chụp baseline quyền.
  - Dùng khi bắt đầu thay đổi logic hoặc sau khi cập nhật dependency.
- `ocp perm diff <old> <new> [--out <report.json>] [--approval <permissions.approval.toml>]`
  - So sánh quyền mới với baseline.
  - Dùng để phát hiện tăng quyền bất thường.
- `ocp perm approve <diff_report.json> [--approval <permissions.approval.toml>] --by <id> --date <YYYY-MM-DD> [--note <text>]`
  - Chấp thuận diff hợp lệ theo lane policy.
  - Không dùng để “duyệt nhanh cho qua”.

### 3) Nguyên tắc an toàn khi approve
- Luôn đọc diff trước khi approve.
- Tránh wildcard trong strict lane.
- Nếu chưa rõ lý do tăng quyền: dừng và điều tra trước.

### 4) Mẫu quy trình chuẩn
1. Code thay đổi.
2. Chạy `ocp perm snapshot <project_dir> --out-dir ./.ocp_perm`.
3. Chạy `ocp perm diff <old_snapshot> <new_snapshot> --out permission_diff_report.json --approval permissions.approval.toml`.
4. Nếu diff hợp lệ và có lý do rõ ràng: `ocp perm approve permission_diff_report.json --approval permissions.approval.toml --by <id> --date <YYYY-MM-DD>`.
5. Lưu `permission_diff_report.json` + `permissions.approval.toml` làm bằng chứng review.

---

<a id="lock-trust-attest"></a>
## [lock-trust-attest] Lock Trust Attest

### 1) `ocp lock sync <project_dir>`
Tác dụng:
- Đồng bộ lockfile theo trạng thái dependency hiện tại.
- Chuẩn bị đầu vào ổn định cho verify/build.

### 2) `ocp lock sign <project_dir> --key <keyid> [--lock deps.lock.v3]`
Tác dụng:
- Ký `deps.lock.v3` sau khi sync để tạo `deps.lock.v3.sig`.
- Đây là bước cần có trước `lock verify` khi đi theo strict flow có chữ ký.

### 3) `ocp lock verify <project_dir> [--lock deps.lock.v3]`
Tác dụng:
- Kiểm lock/trust policy.
- Phát hiện mismatch sớm trước khi chạy pipeline dài.

### 4) `ocp build <project_dir> --attest`
Tác dụng:
- Build kèm bằng chứng attestation.
- Dùng cho môi trường cần chain-of-evidence.

### 5) `ocp verify --attest <artifact_dir>`
Tác dụng:
- Xác minh attestation vừa tạo.
- Chặn trường hợp artifact không đúng trust chain.

### 6) Khi nào cần chạy đủ bộ lock/trust/attest
- Trước merge nhánh chính.
- Trước cắt release.
- Trước khi triển khai production.

---

<a id="run-replay"></a>
## [run-replay] Run Replay

### 1) `ocp run <project_dir>`
Tác dụng:
- Chạy chương trình theo policy/lane hiện hành.
- Sinh trace/artifacts phục vụ audit và debug.

### 2) `ocp replay <artifact_dir>`
Tác dụng:
- Tái hiện hành vi từ dữ liệu đã ghi.
- Kiểm tra ổn định, tránh “máy này chạy khác máy kia”.

Bundle artifact tối thiểu cần có:
- `.ocp_artifacts/<run_id>/audit.jsonl`
- `.ocp_artifacts/<run_id>/signature.txt`
- `.ocp_artifacts/<run_id>/replay.toml`

Ví dụ:

```bash
ocp replay ./.ocp_artifacts/000123/
```

Khi pass, CLI sẽ báo kiểu `replay ok (signature match: ...)`.

### 3) Nguyên tắc fail-honest
Replay phải fail rõ ràng khi:
- Thiếu cassette/chunk cần thiết.
- Dữ liệu replay bị lệch so với expectation.
- Policy không cho phép tiếp tục.

Không được “âm thầm fallback” sang IO thật.

### 4) Checklist nhanh khi replay lỗi
1. Kiểm tra lock/trust đã verify.
2. Kiểm tra dữ liệu cassette/chunk còn đủ.
3. Chạy `ocp dbg` để xem trace chi tiết.

---

<a id="debug"></a>
## [debug] Debug

### 1) `ocp dbg <artifact_dir>` dùng khi nào
- Run thất bại nhưng chưa rõ lý do.
- Replay mismatch.
- Muốn xác định đúng điểm chuyển trạng thái gây lỗi.

### 2) Mục tiêu của debug trong OCP
- Không chỉ “thấy lỗi”.
- Phải truy được nguyên nhân theo trace có cấu trúc.

### 3) Quy trình debug ngắn
1. Chạy `ocp dbg <artifact_dir>`.
2. Xác định stage lỗi (observe/match/commit).
3. Đối chiếu permission/lock/attest nếu liên quan.
4. Sửa, rồi chạy lại quickstart flow tối thiểu.

---

<a id="commands"></a>
## Danh mục lệnh đầy đủ

### A) Bộ lệnh canonical (workflow chuẩn v1.0)

Đây là nhóm lệnh “bắt buộc nắm” để vận hành dự án theo luồng chính:
- `ocp init <project_dir> [--template tool-cli|tool-http|tool-proc|tool-wallclock|mini-game|shadow-preview|dep-permission] [--preset workflow_basic|agent_swarm_basic]`
- `ocp lock sync <project_dir>`
- `ocp lock sign <project_dir> --key <keyid> [--lock deps.lock.v3]`
- `ocp lock verify <project_dir> [--lock deps.lock.v3]`
- `ocp perm snapshot <project_dir> [--out-dir <dir>]`
- `ocp perm diff <old> <new> [--out <report.json>] [--approval <permissions.approval.toml>]`
- `ocp perm approve <diff_report.json> [--approval <permissions.approval.toml>] --by <id> --date <YYYY-MM-DD> [--note <text>]`
- `ocp build <project_dir> --attest`
- `ocp verify --attest <artifact_dir>`
- `ocp run <project_dir>`
- `ocp replay <artifact_dir>`
- `ocp dbg <artifact_dir> [--script <file>]`
- `ocp doctor <project_dir> [--out <report.json>]`
- `ocp fix --plan <project_dir> [--out <permission_fix_plan.json>]`
- `ocp fix --apply <project_dir> [--plan-file <permission_fix_plan.json>] [--patch <permission_fix.patch.toml>] [--out <permission_fix_safety_report.json>] [--approval <permissions.approval.toml>] --ack-risk --justification <text> --by <id> --date <YYYY-MM-DD>`
- `ocp budget analyze <artifact_dir|audit.jsonl> [--json]`

### B) Bộ lệnh nâng cao (có trong CLI, dùng theo nhu cầu)

#### 1) Thực thi, kiểm tra, và profiling nâng cao
- `ocp check <project_dir> [--json] [--locked] [--universe <id>]`
- `ocp run <project_dir> [--engine interpreter|bytecode|dual] [--reactor --ticks N --runtime deterministic|throughput --socket-listen <addr> --runtime-report <file> --replay-audit <file>] [--shadow <id> --shadow-policy forbid_commit|shadow_commit_log] [--locked] [--universe <id>] [--domain <id>] [--view <id>]`
- `ocp fmt <project_dir> [--check]`
- `ocp test <project_dir> [--locked] [--universe <id>] [--domain <id>] [...]`
- `ocp cache stats <project_dir> [--json]`
- `ocp cache clean <project_dir> --yes [--json]`
- `ocp cache bench <project_dir> [--engine ...] [--locked] [--json]`
- `ocp trace run <project_dir> [--out <file>] [--engine interpreter|bytecode|dual] [--reactor --ticks N --runtime deterministic|throughput] [--shadow <id> --shadow-policy forbid_commit|shadow_commit_log] [--locked] [--universe <id>] [--domain <id>] [--view <id>]`
- `ocp trace view <artifact_dir|audit.jsonl|legacy.trace> [...]`
- `ocp trace diff <artifactA|auditA> <artifactB|auditB> [...]`
- `ocp minimize <artifact_dir> --goal <...> [...]`
- `ocp profile run <project_dir> [--out <file>] [--engine interpreter|bytecode|dual] [--reactor --ticks N --runtime deterministic|throughput] [--shadow <id> --shadow-policy forbid_commit|shadow_commit_log] [--locked] [--universe <id>] [--domain <id>] [--view <id>]`
- `ocp profile view <profile_file> [--top N] [--json]`
- `ocp budget doctor <artifact_dir|audit.jsonl> [--json]`
- `ocp doc packs [--json]`

#### 2) Supply-chain, package, dependency
- `ocp publish <artifact.ocppkg> [--registry <dir>]`
- `ocp fetch <artifact|package> [--registry <dir>] [--out <dir>]`
- `ocp verify-supply <artifact.ocppkg>`
- `ocp deps resolve <project_dir> [--write-legacy-lock]`
- `ocp deps update <project_dir> [pkg] [--write-legacy-lock]`
- `ocp deps verify <project_dir> [--no-lane-policy]`
- `ocp pack build <project_dir> [--locked]`
- `ocp pack sign <artifact.ocppkg>`
- `ocp pack publish <artifact.ocppkg> [--registry <dir>]`
- `ocp pack verify <artifact.ocppkg>`

#### 3) Policy/cosmos/plugin/organ/kit
- `ocp init <project_dir> --preset workflow_basic|agent_swarm_basic`
- `ocp policy lock sync <project_dir>`
- `ocp cosmos init <project_dir> [--preset default|ci]`
- `ocp cosmos lock sync <project_dir> [--locked] [...]`
- `ocp plugin lock sync <project_dir>`
- `ocp plugin verify <project_dir>`
- `ocp organ lock sync <project_dir> [--registry <index.toml>] [--json]`
- `ocp organ verify <project_dir> [--locked|--unlocked] [--json]`
- `ocp organ install <name> <version> [--project <dir>] [...]`
- `ocp kit list [<project_dir>] [--json]`
- `ocp kit doctor <project_dir> [--locked|--json]`
- `ocp compose <project_dir> --phenotype <file> [...]`

Phân biệt nhanh:
- `ocp init --preset workflow_basic|agent_swarm_basic`: preset foundation cấp project (public onboarding).
- `ocp cosmos init --preset default|ci`: preset cosmos config chuyên sâu.

#### 4) Verify nâng cao
- `ocp verify --attest <artifact_dir>`
- `ocp verify --repro <artifact_dir>`
- `ocp verify <project_dir> --phenotype <file> [...]`

### C) Alias cần biết
- `ocp doctor ...` là alias của `ocp perm doctor ...`
- `ocp fix --plan|--apply ...` là alias của `ocp perm fix ...`
- `ocp perm review ...` là alias của `ocp perm diff ...`

### D) Khuyến nghị dùng lệnh an toàn
- Luôn chạy `ocp --help` hoặc `ocp <group> --help` trước khi dùng lệnh nâng cao.
- Với lệnh có khả năng thay đổi lock/permission/artifact, ưu tiên chạy trên nhánh riêng trước.

Gợi ý tra cứu nhanh (canonical workflow):
- `docs/vi/reference/cli.md`

<a id="public-surface"></a>
## Public surface mở rộng quan trọng

Ngoài core flow, v1.0 còn có các nhánh public thường bị bỏ sót nếu chỉ nhìn danh mục lệnh:
1. Language surface mở rộng (import/export/fn/return/bounded loop như `repeat`, `for ... cap N`, `for-range` cũ để tương thích/list-map + `?`/`try-else`/`guard` + reactor tick).
2. Compose / phenotype để sinh code có kiểm chứng.
3. Organ / kit / preset cho bootstrap hệ sinh thái.
4. Editor workflow (VSCode + LSP/DAP bridge command).

Mục tiêu phần này: biết rõ dùng khi nào, và workflow ngắn để chạy được.

<a id="compose-phenotype"></a>
## Compose / Phenotype

Dùng khi bạn muốn sinh module/cấu hình từ phenotype contract thay vì viết tay toàn bộ.

Workflow ngắn:
1. Chuẩn bị file phenotype.
2. Chạy compose:

```bash
ocp compose <project_dir> --phenotype <phenotype_file>
```

3. Verify phenotype/build consistency:

```bash
ocp verify <project_dir> --phenotype <phenotype_file>
```

Artifacts thường gặp sau compose:
- `src/generated/*.ocp`
- `src/generated/mod.ocp`
- `assembly_proof.toml`

Khi nào dùng:
- Cần reproducible generation.
- Cần chứng minh mã sinh khớp catalog/phenotype đã pin.

<a id="organ-kit-preset"></a>
## Organ / Kit / Preset

Dùng khi bạn bootstrap workspace/team flow thay vì dựng thủ công từng phần.

Workflow ngắn:
1. Init theo preset workflow public:

```bash
ocp init <project_dir> --preset workflow_basic
# hoặc
ocp init <project_dir> --preset agent_swarm_basic
```

2. Preset sẽ kéo chain lock cần thiết (deps -> policy -> cosmos -> organ/kit khi có yêu cầu).  
   Sau đó đồng bộ lock cho organ graph:

```bash
ocp organ lock sync <project_dir>
```

3. Verify organ trong lane mong muốn:

```bash
ocp organ verify <project_dir> --locked
```

4. Cài thêm organ + kiểm tra kit:

```bash
ocp organ install <name> <version> --project <project_dir>
ocp kit doctor <project_dir> --locked
```

Gợi ý chọn preset:
- `workflow_basic`: luồng ứng dụng chuẩn, nhẹ để bắt đầu nhanh.
- `agent_swarm_basic`: bài toán swarm/hive bounded có wiring sẵn.

Lưu ý lane locked:
- Khi chạy flow locked, cần signer/trust flags và approval records đúng contract hiện hành.

<a id="editor-workflow"></a>
## Editor workflow (VSCode/LSP/DAP)

`USER_GUIDE` không chỉ dừng ở “cài VSIX”; workflow khuyến nghị khi code trong editor:
1. Cài VSIX và mở file `.ocp`.
2. Mặc định có TextMate highlight + snippets; command bridge dùng bundled CLI khi workspace được trust.
3. Trust behavior (quan trọng):
   - Workspace untrusted: chỉ shell tĩnh (grammar/snippets), không chạy command bridge runtime.
   - Workspace trusted: bật command bridge `fmt/check/doctor/fix/debug trace` từ bundled CLI.
4. Editor bridge dùng bundled CLI đã verify từ extension package, không phụ thuộc `PATH` host.
5. Trạng thái runtime editor hiện tại trong repo:
   - command bridge dùng bundled CLI cho `fmt`, `check`, `doctor`, `fix --plan`, và mở trace từ artifact;
   - runtime editor thật đã được bundle thành `ocp-lsp` / `ocp-dap`, hỗ trợ diagnostics realtime, hover, completion, go to definition, references, rename, formatting, code actions, semantic tokens, và replay-backed debug adapter trong workspace trusted.
6. Trước commit: chạy bridge command chuẩn:

```bash
ocp fmt <project_dir> --check
ocp check <project_dir> --locked
ocp doctor <project_dir>
ocp fix --plan <project_dir>
```

7. Nếu có artifact lỗi/replay mismatch:

```bash
ocp dbg <artifact_dir>
```

Khi nào dùng DAP/trace debug:
- Lỗi logic khó tái hiện bằng nhìn code.
- Cần đối chiếu run hiện tại với replay evidence.

<a id="reactor-workflow"></a>
## Reactor workflow (run/trace/profile)

Dùng reactor khi bạn cần vòng lặp/tick bounded hoặc cần runtime report/audit tách file.

Run reactor cơ bản:

```bash
ocp run <project_dir> --reactor --ticks 50 --runtime deterministic --locked --universe dev --domain main --view default
```

Trace reactor:

```bash
ocp trace run <project_dir> --reactor --ticks 50 --runtime deterministic --locked --out target/ocp/trace/reactor.audit.jsonl
ocp trace view target/ocp/trace/reactor.audit.jsonl --tail 50
```

Profile reactor:

```bash
ocp profile run <project_dir> --reactor --ticks 50 --runtime deterministic --locked --out target/ocp/profile/reactor.profile.json
ocp profile view target/ocp/profile/reactor.profile.json --top 20
```

Lưu ý:
- `--socket-listen`, `--runtime-report`, `--replay-audit` chỉ hợp lệ khi đi cùng `--reactor`.
- Nếu đã pin `--universe/--domain/--view` trong team workflow, giữ cố định để giảm drift giữa các máy.

---

<a id="recommended-workflow"></a>
## Luồng làm việc khuyến nghị

Luồng mặc định cho nhóm phát triển:
1. `ocp init <project_dir>` (hoặc vào dự án có sẵn).
2. `ocp lock sync <project_dir>` + `ocp lock sign <project_dir> --key <keyid>` + `ocp lock verify <project_dir> --lock deps.lock.v3`.
3. `ocp perm snapshot <project_dir>` + `ocp perm diff <old> <new>` + `ocp perm approve <diff_report.json> --by <id> --date <YYYY-MM-DD>`.
4. `ocp build <project_dir> --attest` + `ocp verify --attest <artifact_dir>`.
5. `ocp run <project_dir>` + `ocp replay <artifact_dir>`.
6. Nếu có sự cố: `ocp doctor <project_dir>` -> `ocp fix --plan <project_dir>` -> `ocp fix --apply <project_dir> ...` -> `ocp dbg <artifact_dir>`.
7. Nếu nghi ngờ quá ngân sách: `ocp budget analyze <artifact_dir|audit.jsonl>`.

---

<a id="troubleshooting"></a>
## [troubleshooting] Troubleshooting

### 1) Lệnh không chạy được sau cài đặt
Triệu chứng:
- `ocp` không được nhận diện.

Xử lý:
1. Mở terminal mới.
2. Kiểm tra `PATH`.
3. Chạy lại `ocp --version`.

### 2) `perm diff` tăng quyền bất thường
Triệu chứng:
- Nhiều quyền mới xuất hiện không rõ lý do.

Xử lý:
1. Không approve ngay.
2. Rà lại thay đổi code/dependency.
3. Chạy lại `perm snapshot` và so sánh từng cụm.

### 3) `lock verify` không pass
Triệu chứng:
- Lock/trust mismatch.

Xử lý:
1. Chạy lại `ocp lock sync`.
2. Nếu thiếu `deps.lock.v3.sig`, chạy `ocp lock sign <project_dir> --key <keyid>`.
3. Verify lại.
4. Nếu vẫn lỗi, kiểm tra trust policy/metadata.

### 4) `verify --attest` fail
Triệu chứng:
- Attestation không khớp.

Xử lý:
1. Build lại với `ocp build --attest`.
2. Verify lại ngay sau build.
3. Nếu lệch tiếp, kiểm tra môi trường và artifact path.

### 5) `replay` fail do thiếu dữ liệu
Triệu chứng:
- Thiếu cassette/chunk.

Xử lý:
1. Ghi lại dữ liệu đúng flow.
2. Không dùng fallback IO thật.
3. Dùng `ocp dbg` để xác nhận điểm thiếu.

### 6) Tool IO bị chặn (`RC-*-PERMISSION-DENIED`)
Triệu chứng:
- `std.fs.*`, `std.kv.*`, hoặc `std.time.*` trả `INSUFFICIENT` do policy.

Xử lý:
1. Mở `Ocp.toml`, kiểm tra các section:
   - `[permissions.std_fs]`
   - `[permissions.std_kv]`
   - `[permissions.std_time]`
2. Đối chiếu key gọi thực tế và allow/deny.
3. Chạy lại `ocp check . --locked` để đọc hint policy từ diagnostics.
4. Chạy lại `ocp test . --golden fixtures/expected --clean` để xác nhận flow đã ổn.

### 7) Lane `quarantine` bị chặn khi run/replay
Triệu chứng:
- CLI báo lane `quarantine` bị từ chối hoặc replay không cho chạy.

Xử lý:
1. Xác nhận project/artifact đang ở lane nào (`locked_v071`, `locked_v06`, hay `quarantine`).
2. Nếu là `quarantine`, bật env gate trước khi chạy:
   - PowerShell: `$env:OCP_QUARANTINE="1"`
   - bash/zsh: `export OCP_QUARANTINE=1`
3. Không dùng artifact `quarantine` để replay như locked lane (signature/lane marker khác nhau).
4. Chạy lại `ocp run ...` rồi `ocp replay <artifact_dir>` trong cùng lane.

### 8) Cần hỗ trợ sâu hơn
- `docs/vi/troubleshooting/common-errors.md`
- `docs/vi/troubleshooting/editor.md`
- `docs/vi/troubleshooting/replay.md`

---

<a id="advanced-appendix"></a>
## [advanced-appendix] Phụ lục nâng cao (đọc sau phần vỡ lòng)

Từ đây trở xuống là nội dung nâng cao.  
Nếu bạn đang học lần đầu, chỉ cần đọc đến hết `Troubleshooting` là đủ để làm việc hằng ngày.

### 7) Tool-grade IO packs cần biết khi dùng (nâng cao)
Bạn có thể làm tool file/config trong lane locked với 3 nhóm key:
- `std.fs.*`: đọc/ghi/list/stat file trong sandbox policy.
- `std.kv.*`: lưu state local deterministic (checkpoint, marker, cache nhỏ).
- `std.time.*`: thời gian logic deterministic cho test/replay.

Khi làm tool có IO:
1. Khởi tạo bằng template:
   - `ocp init <project_dir> --template tool-cli`
2. Test với golden:
   - `ocp test <project_dir> --golden fixtures/expected --clean`
3. Replay từ artifact:
   - `ocp replay ./.ocp_artifacts/<run_id>/`

Artifacts IO metadata quan trọng:
- `io/fixtures_manifest.json`
- `state/kv_start.json`
- `replay.toml` (chứa `io_mode = "fixtures"` và đường dẫn fixture metadata)

### 8) Consumer packs cho app/game (nâng cao)
OCP có thêm nhóm capability cho app/game:
- `std.ui.*`: UI theo mô hình draw-list (không phải widget framework nặng trong core).
- `engine.game.*`: capability key family cho tick/rng/state deterministic; manifest thường cấp quyền qua `[permissions.std_game]`.
- `std.shadow.*`: chạy nhánh shadow và compare report.

Bắt đầu nhanh:
1. `ocp init <project_dir> --template mini-game`
2. `ocp run <project_dir>`
3. `ocp replay ./.ocp_artifacts/<run_id>/`

Hoặc:
1. `ocp init <project_dir> --template shadow-preview`
2. `ocp run <project_dir> --shadow <id> --shadow-policy forbid_commit`
3. `ocp replay ./.ocp_artifacts/<run_id>/`

Lưu ý lane:
- `quarantine` là lane tách biệt cho capability nondeterministic theo env gate.
- Khi chạy lane `quarantine`, cần bật `OCP_QUARANTINE=1`.
- Artifact/signature của `quarantine` không dùng lẫn như lane locked.

### 9) Quarantine + cassette record/replay (nâng cao)
Khi tool của bạn cần capability nondeterministic (HTTP/proc/wallclock), dùng lane `quarantine` thay vì cố chạy trong lane locked.

Checklist vận hành đúng:
1. Trong `Ocp.toml`, đặt lane `quarantine` và khai báo permission đúng capability.
2. Bật env gate trước khi chạy:

```powershell
$env:OCP_QUARANTINE="1"
```

```bash
export OCP_QUARANTINE=1
```

3. Chạy project như bình thường bằng `ocp run <project_dir>`.
4. Replay từ artifact dir của lần chạy:

```bash
ocp replay ./.ocp_artifacts/<run_id>/
```

5. Nếu cần vận hành cassette lâu dài, dùng bộ lệnh:
   - `ocp cassette stats`
   - `ocp cassette prune`
   - `ocp cassette gc`
   - `ocp cassette upgrade`

Mẫu project có sẵn cho v0.8 line:
- `tool-http`
- `tool-proc`
- `tool-wallclock`

Nguyên tắc quan trọng:
- Thiếu cassette/chunk bắt buộc => replay phải fail-honest, không tự gọi IO thật.
- Artifact/signature của lane `quarantine` không dùng lẫn với lane locked.
- `quarantine` chỉ dùng khi thật sự cần capability nondeterministic.

### 10) Schema + typing cho ctx/payload/effects (nâng cao)
OCP tăng kiểm tra hợp đồng theo hướng “bắt lỗi sớm”:
- `ctx` nên viết dạng record typed (không ưu tiên `ctx("k=v;...")` kiểu cũ).
- `match`/`try`/`guard` kỳ vọng đúng kiểu `Result4`.
- `commit(...)` chỉ hợp lệ với observe key có quyền commit theo contract.

Mẫu khuyến nghị:

```ocp
let req = { path: "./out/result.json", overwrite: true };
observe("std.fs.write_text", "tier2", req, budget(5)) -> wr;
match wr {
  OK => { commit(wr); }
  DEGRADED => { commit(wr); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}
```

Nếu bạn đang chuyển từ script cũ còn dùng `ctx("...")`, có thể bật compat tạm thời trong `Ocp.toml`:

```toml
[compat]
ctx_string = "warn"
ctx_extra_fields = "warn"
```

Lộ trình an toàn:
1. Đặt compat ở `warn` để thấy toàn bộ cảnh báo.
2. Sửa script dần sang `ctx` record typed.
3. Chuyển về `deny` để khóa drift.

Lệnh hữu ích khi làm việc với schema packs:
- `ocp doc packs`
- `ocp doc packs --json`

Lỗi thường gặp từ line v0.9:
- `RC-CTX-INVALID`: runtime báo `ctx` không khớp schema key.
- `T-TRY-NOT-RESULT4` / `T-GUARD-NOT-RESULT4`: dùng `try` hoặc `guard` sai kiểu.
- commit bị chặn ở compile-time khi key thuộc nhóm observe-only.

### 11) Packaging + lockfile + trust (nâng cao)
OCP có luồng dependency/package đầy đủ để tái sử dụng module giữa nhiều dự án mà vẫn giữ kiểm soát supply-chain.

Những điểm bạn cần nhớ khi vận hành:
- SoT lockfile là `deps.lock.v3`.
- Permission của dependency tính theo nguyên tắc least-privilege theo từng package.
- Lane `locked_v071` yêu cầu dependency non-builtin phải signed + trusted.

Luồng chuẩn khi thêm/cập nhật dependency:
1. Resolve graph:

```bash
ocp deps resolve <project_dir>
```

2. Verify lock + trust:

```bash
ocp deps verify <project_dir>
```

3. Chạy build/run như bình thường sau khi verify pass.

Nếu bạn đóng gói package để tái sử dụng:

```bash
ocp pack build <project_dir>
ocp pack sign <artifact.ocppkg>
ocp pack verify <artifact.ocppkg>
```

Permission dependency trong v0.10:
- Package chỉ “yêu cầu” quyền qua `requested_permissions` (không tự cấp quyền).
- Project là nơi “cấp” quyền thực tế.
- Effective permission được tính theo giao của `granted` và `requested`, rồi áp deny.

Tối thiểu bạn nên theo dõi các artifact:
- `deps.lock.v3`
- `permissions_effective.json`
- audit event `DepsResolved` (để truy vết trust/signature decision)

### 12) Debug replay-native: trace, diff, dbg, minimize (nâng cao)
Bạn có bộ công cụ debug dựa trên artifact replay thay vì thêm log thủ công vào code.

Luồng xử lý sự cố khuyến nghị:
1. Chạy và lấy artifact run như bình thường.
2. Xem trace để xác định điểm lỗi đầu tiên:

```bash
ocp trace view <artifact_dir|audit.jsonl> --tail 50
```

3. Nếu cần so với baseline:

```bash
ocp trace diff <artifactA|auditA> <artifactB|auditB> --mode align --out <out_dir>
```

4. Mở debugger replay:

```bash
ocp dbg <artifact_dir>
```

5. Khi đã xác định lỗi nhưng case còn quá lớn, rút gọn repro:

```bash
ocp minimize <artifact_dir> --goal <error_code:X|divergence|kind:KIND> --out <out_dir>
```

Các điểm vận hành quan trọng:
- Trace viewer/diff/dbg dùng `audit.jsonl` làm nguồn dữ liệu chính.
- `trace diff` nên ưu tiên `--mode align` khi so sánh hai run có khác nhẹ về cấu trúc event.
- Minimizer không sửa code; nó tạo artifact nhỏ hơn để tái hiện cùng lỗi/divergence.

### 13) Shadow scheduling + incremental reuse (nâng cao)
`shadow-preview` không chỉ chạy nhiều nhánh đơn giản mà có engine search bounded:
- policy: `round_robin`, `beam`, `portfolio`
- scoring deterministic
- report v2 để đọc kết quả nhánh tốt nhất và lý do bị prune.

Luồng dùng nhanh:
1. Khởi tạo template:

```bash
ocp init <project_dir> --template shadow-preview
```

2. Chạy và tạo artifact:

```bash
ocp run <project_dir>
```

3. Replay để xác nhận ổn định:

```bash
ocp replay ./.ocp_artifacts/<run_id>/
```

4. Khi cần so nhánh/bản build:
   - `ocp trace view <artifact_dir>`
   - `ocp trace diff <artifactA> <artifactB> --mode align`

Điểm quan trọng khi vận hành:
- Policy shadow phải nằm trong allowlist dự án; nếu không sẽ bị chặn fail-honest với `RC-SHADOW-POLICY-DENIED`.
- Reuse trong v0.12 tập trung cho lane `locked_v071` (quarantine reuse vẫn tách riêng).
- Report shadow v2 có ranking nhánh, divergence summary, cost/steps và reason breakdown.

Nếu bạn cần benchmark nội bộ để đo hiệu quả reuse:
- baseline: tắt memo/checkpoint reuse
- reuse: bật memo/checkpoint reuse
- so sánh `steps_executed_total` trên cùng dataset/seed/lane.

### 14) IR + cache tái lập được (nâng cao)
OCP thêm lớp tối ưu hiệu năng nhưng vẫn giữ mục tiêu “determinism trước, tốc độ sau”:
- compile cache (biên dịch/module)
- execution cache cho nhánh thuần (pure)
- observe cache theo policy an toàn.

Bộ lệnh vận hành cache:

```bash
ocp cache stats <project_dir> [--json]
ocp cache bench <project_dir> [--engine interpreter|bytecode|dual] [--locked] [--json]
ocp cache clean <project_dir> --yes [--json]
```

Khi dùng thực tế:
1. Chạy `ocp cache stats` để xem hit/miss compile/exec/observe.
2. Chạy `ocp cache bench` để so cold/warm/no-cache và kiểm tra `signature_equal`.
3. Chỉ chạy `ocp cache clean --yes` khi cần reset hoàn toàn cache.

Lưu ý quan trọng:
- `cache clean` yêu cầu `--yes` để tránh xóa nhầm.
- Lệnh clean sẽ dọn các thư mục cache/artifacts của project:
  - `.ocp_artifacts`
  - `.ocp_cache`
  - `target/ocp/cache`
- Benchmark v0.13 ghi report tại:
  - `target/ocp/v13/cache_benchmark.json`

Nguyên tắc an toàn:
- Cache là tối ưu, không được thay đổi nghĩa chương trình.
- Nếu bật cache mà signature lệch so với no-cache, coi như lỗi cần điều tra ngay.

### 15) Conformance suite + upgrade-check (nâng cao)
OCP có lớp kiểm chứng hành vi ở cấp “hợp đồng toàn hệ”, không chỉ unit test rời.

Hai nhóm lệnh chính:

```bash
ocp test --conformance list --manifest <manifest_file>
ocp test --conformance run --manifest <manifest_file> --out <report_file> [--locked]
```

Alias tương đương:

```bash
ocp conformance list --manifest <manifest_file>
ocp conformance run --manifest <manifest_file> --out <report_file> [--locked]
```

Ví dụ thực tế:

```bash
ocp test --conformance list --manifest projects/ocp/conformance/conformance.v5.toml
ocp test --conformance run --manifest projects/ocp/conformance/conformance.v5.toml --out target/ocp/w14/reports/conformance_report.json
```

Khi chạy conformance ở chế độ `--locked`:
- bắt buộc cung cấp key ký và signer id theo policy strict lane.
- nếu thiếu sẽ bị chặn fail-honest (không tự hạ chuẩn bảo mật).

Kiểm tra nâng cấp trước khi đổi runtime:

```bash
ocp upgrade-check <project_dir> --manifest <manifest_file> --out <report_file> [--locked]
```

Mục tiêu của `upgrade-check`:
- chọn subset conformance phù hợp với packs/project đang dùng,
- phát hiện divergence sớm,
- xuất report để quyết định nâng cấp hay giữ version hiện tại.

### 16) Luồng nâng cao cho maintainer/release
Để tránh trộn vai trò người dùng cuối và người vận hành phát hành, phần sau đã tách ra tài liệu riêng:
- trust + attestation + LTS production
- product-readiness audit/checklist
- migration/release packaging/signoff
- gate kiểm duyệt cuối (v0.20 line)

Xem tại:
- `docs/vi/MAINTAINER_GUIDE.md`
- `docs/plans/history/*` (lịch sử quyết định kỹ thuật theo version)

---

<a id="reference"></a>
## Reference (phụ trợ, không bắt buộc để học A->Z)

Tra cứu ngay trong file này (ưu tiên):
- [Tra cứu nhanh](#quick-reference)
- [Language Quickstart](#language-quickstart)
- [Nền tảng cần biết để dùng OCP](#concepts)
- [Permissions](#permissions)
- [Danh mục lệnh đầy đủ](#commands)
- [Troubleshooting](#troubleshooting)
- [Phụ lục nâng cao](#advanced-appendix)

Tài liệu ngoài file (chỉ dùng khi cần đào sâu):
- `docs/vi/reference/cli.md` (CLI đầy đủ)
- `docs/vi/ops/doctor-and-fix.md`
- `docs/vi/ops/budget-analyze.md`
- `docs/vi/security/verify-download.md`

---

<a id="maintainer-guide"></a>
## Maintainer Guide

Phần dành cho người vận hành release/gate đã tách riêng để USER_GUIDE tập trung cho người dùng viết/chạy OCP:
- [docs/vi/MAINTAINER_GUIDE.md](/docs/vi/MAINTAINER_GUIDE.md)
