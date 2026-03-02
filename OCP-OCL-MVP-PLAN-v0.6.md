> [!WARNING]
> ARCHIVED PLAN: file này chỉ dùng cho mục đích lịch sử/audit.
> Nguồn sự thật active: `projects/ocp-ocl/OCL-PLAN.md`.

# OCL MVP PLAN v0.6 (Stabilization-Only)

Ngày tạo: 2026-02-26  
Mục tiêu: đóng hết nợ kỹ thuật/xung đột/rủi ro còn tồn đọng sau v0.5, không thêm tính năng mới.

## Cập nhật chốt cuối (2026-02-27)
- Trạng thái canonical đã chốt tại `projects/ocp-ocl/OCL-PLAN.md`: `OPEN=0` cho `001..017`.
- Unified release gate đã PASS trên clean checkout tại commit `1de5a285e7f16456cba0d2a39cef19c0435749ad`.
- Evidence signoff cuối: `projects/ocp-ocl/release/evidence/u10-final-signoff.md`.
- File này được giữ làm hồ sơ lịch sử của v0.6; trạng thái điều hành active chỉ đọc từ `OCL-PLAN.md`.

## Quy ước cập nhật bắt buộc (áp dụng từ v0.6)
- Mọi thay đổi kế hoạch phải cập nhật file này trước khi triển khai.
- Mọi issue xử lý xong phải cập nhật evidence ngay sau khi chạy xác minh.
- Không được đóng issue nếu chưa có bằng chứng trước/sau rõ ràng.
- Không thêm feature mới trong v0.6; chỉ xử lý các vấn đề hiện hữu.

### Template cập nhật kế hoạch (trước khi làm)
- Date:
- Issue:
- Why:
- Scope:
- Expected checks:
- Exit criteria:

### Template cập nhật triển khai (sau khi làm)
- Date:
- Issue:
- Implemented:
- Files changed:
- Commands run:
- Results:
- Residual risks:

## 0) Phạm vi v0.6 (khóa cứng)
1. v0.6 chỉ là bản ổn định hóa, không mở feature mới, không mở grammar mới.
2. Mỗi mục trong file này là issue cần xử lý để chốt release readiness.
3. Chỉ khi tất cả issue `OPEN` về `DONE` mới được xem là “xong hẳn” cho pre-1.0.

## 1) Current Issue Register (Snapshot chốt)

### V6-ISSUE-001 - Runtime artifacts bị track trong repo
- Severity: `BLOCKER`
- Status: `DONE`
- Evidence:
  - `git ls-files "projects/ocp-ocl/crates/ocl-cli/target/**"` => `221` files tracked.
  - Ví dụ: `projects/ocp-ocl/crates/ocl-cli/target/ocl/v5/w3/reports/shadow_compare.run.json`
  - Ví dụ: `projects/ocp-ocl/crates/ocl-cli/target/ocl/v5/w3/reports/shadow_compare.reactor.json`
- Impact:
  - Dirty worktree sau mỗi lần run/test.
  - Dễ drift evidence, khó phân biệt code change và runtime artifacts.
- Exit criteria:
  1. Toàn bộ files dưới `projects/ocp-ocl/crates/ocl-cli/target/**` được untrack.
  2. `.gitignore` cover đủ path nested `crates/*/target/**`.
  3. Chạy lại full lane không sinh tracked artifacts mới.

### V6-ISSUE-002 - Release-grade gate chưa wire end-to-end trong CI lane
- Severity: `BLOCKER`
- Status: `DONE`
- Evidence:
  - SDK có gate: `projects/ocp-ocl/crates/ocl-sdk/src/v5_w6_organs.rs:21` (`OCL_RELEASE_GRADE`).
  - Lane script chưa set/check env `OCL_RELEASE_GRADE`.
  - `tools/ci_ocl_lane.ps1:30` chỉ in “release-grade” theo signer inputs.
- Impact:
  - Hardening release-grade có thể không được bật thật trong lane chính.
- Exit criteria:
  1. Lane blocking set rõ `OCL_RELEASE_GRADE=1` (hoặc flag tương đương có enforce thật).
  2. Có test/evidence chứng minh release-grade denylist đang active trên lane.

### V6-ISSUE-003 - Quarantine conformance throughput fail sớm vì thiếu signer/trust
- Severity: `MAJOR`
- Status: `DONE`
- Evidence:
  - `tools/ci_ocl_quarantine.ps1:148` gọi:
    - `ocl test --conformance --locked --runtime throughput ... --json`
    - không truyền `--trust-store/--signer-id/--sign-key`.
  - Khi chạy thực tế, fail sớm `V-W9-CONFORMANCE-SIGNER-MISSING`.
- Impact:
  - Quarantine throughput không test đủ scenario path như kỳ vọng (fail preflight sớm).
- Exit criteria:
  1. Quarantine throughput command đủ signer/trust inputs nếu mục tiêu là smoke scenario.
  2. Hoặc khóa rõ đây là negative smoke có chủ đích và assert đúng error code.

### V6-ISSUE-004 - Key dev signing đang được commit trong repo
- Severity: `MAJOR`
- Status: `DONE`
- Evidence:
  - File tracked: `projects/ocp-ocl/security/dev-root-1.signing.key.toml`
  - Chứa private key field: `projects/ocp-ocl/security/dev-root-1.signing.key.toml:3`
- Impact:
  - Release posture không sạch nếu lane phát hành vẫn dựa vào key trong repo.
  - Có rủi ro nhầm lẫn giữa fixture key và release key.
- Exit criteria:
  1. Định nghĩa rõ policy: fixture-only key và release key tách biệt.
  2. Release lane bắt buộc key ngoài repo.
  3. Docs/plan/CI thống nhất 1 chuẩn signer cho release.

### V6-ISSUE-005 - Governance doc chưa tuyệt đối “single canonical plan” sau v0.5
- Severity: `MINOR`
- Status: `DONE`
- Evidence:
  - Có đầy đủ plan `v0.1..v0.5`, nhưng chưa có dòng explicit “v0.5 là canonical final, v0.1-v0.4 là archive”.
- Impact:
  - Đội người dùng/developer mới có thể hiểu nhầm nguồn sự thật vận hành.
- Exit criteria:
  1. Thêm section governance rõ ràng:
     - `v0.5` là nguồn canonical cho release readiness.
     - `v0.1..v0.4` là lịch sử/archived.
  2. User guide tham chiếu 1 canonical source duy nhất.

### V6-ISSUE-006 - Drift đường dẫn evidence trong plan v0.1
- Severity: `MAJOR`
- Status: `DONE`
- Evidence:
  - `projects/ocp-ocl/OCL-MVP-PLAN-v0.1.md` tham chiếu nhiều path kiểu:
    - `tests/ocl_toy_programs.rs` (vd dòng 307/378/1371)
    - `tests/ocl_pilot.rs` (vd dòng 488/560)
    - `docs/GATE-E-BENCHMARK-PROTOCOL.md` (vd dòng 660)
    - `reports/gate-e-roi-v0.1.md` (vd dòng 664)
  - Trong tree hiện tại, các file này không nằm cùng base path với plan v0.1:
    - tồn tại ở root/archive hoặc path khác (`projects/ocp-ocl/OCL-V0_1-GRAMMAR.md`, `docs/GATE-E-BENCHMARK-PROTOCOL.md`, `reports/gate-e-roi-v0.1.md`, `_archive/...`).
- Impact:
  - Khó replay evidence theo đúng ngữ cảnh path hiện tại.
  - Reviewer mới dễ hiểu nhầm “file thiếu” hoặc “evidence không tồn tại”.
- Exit criteria:
  1. Chuẩn hóa chú thích path base cho plan v0.1 (legacy-root hay workspace-relative).
  2. Bổ sung mapping table “path cũ -> path hiện tại” cho các evidence quan trọng.
  3. Check script/doc check không còn false-missing cho path evidence lịch sử.

### V6-ISSUE-007 - Mục “Known limits” trong v0.1 chưa gắn nhãn lịch sử/superseded
- Severity: `MINOR`
- Status: `DONE`
- Evidence:
  - `projects/ocp-ocl/OCL-MVP-PLAN-v0.1.md` còn các đoạn:
    - `Known limits sau Gate B` (dòng 243-248)
    - `Known limits sau Gate C` (dòng 383-387)
    - `Known limits sau Gate D` (dòng 564-568)
    - `Known limits sau Gate E` (dòng 666-670)
  - Các mô tả “chưa có branch/match”, “entangle/condition no-op”, “ctx opaque...” là trạng thái lịch sử, nhưng chưa có marker “đã được xử lý ở phiên bản sau”.
- Impact:
  - Người đọc có thể nhầm đây là hạn chế hiện tại của hệ thống.
  - Tăng ma sát khi audit chéo v0.1 với v0.5.
- Exit criteria:
  1. Gắn marker rõ cho từng block lịch sử: `Historical snapshot (superseded)`.
  2. Nêu link tham chiếu tới gate/version đã xử lý tiếp theo.
  3. Không còn phát sinh hiểu nhầm trạng thái hiện tại khi đọc riêng v0.1.

### V6-ISSUE-008 - Drift tham chiếu path trong plan v0.2 (legacy-root vs workspace hiện tại)
- Severity: `MAJOR`
- Status: `DONE`
- Evidence:
  - `projects/ocp-ocl/OCL-MVP-PLAN-v0.2.md` tham chiếu nhiều path theo legacy-root:
    - `docs/OCL-USER-GUIDE.md` (dòng 414/423)
    - `src/ocp_ocl/*.rs` (dòng 427-431)
    - `tests/ocl_*.rs`, `soak_compare` (nhiều dòng trong matrix)
  - Một số path hiện đã chuyển vùng hoặc chỉ còn trong archive:
    - user guide canonical đang ở `projects/ocp-ocl/docs/OCL-USER-GUIDE.md` (root `docs/OCL-USER-GUIDE.md` chỉ là file MOVED).
- Impact:
  - Khó replay evidence của v0.2 nếu dùng workspace hiện tại.
  - Dễ hiểu nhầm đường dẫn “không tồn tại” dù thực tế là đã chuyển.
- Exit criteria:
  1. Bổ sung chú thích “legacy-root context” cho các block execution log v0.2.
  2. Có bảng map path legacy -> path hiện tại/archived.

### V6-ISSUE-009 - Deferred v0.2.1 chưa có chỉ mục liên kết trạng thái cuối
- Severity: `MINOR`
- Status: `DONE`
- Evidence:
  - `projects/ocp-ocl/OCL-MVP-PLAN-v0.2.md` còn các cụm deferred:
    - `Out-of-scope / Deferred sang v0.2.1` (dòng 47)
    - `v0.2 P1 Extended (DEFERRED v0.2.1)` (dòng 78)
    - ghi chú deferred tại dòng 256/257/354/435.
  - Chưa có “resolution index” chỉ rõ từng mục deferred đã:
    - được xử lý ở version nào, hoặc
    - được hủy scope chính thức.
- Impact:
  - Audit dọc version tốn công và dễ bỏ sót mục “đã/ chưa xử lý”.
  - Người đọc mới có thể hiểu nhầm deferred vẫn là backlog hiện hành.
- Exit criteria:
  1. Tạo bảng “Deferred resolution” cho v0.2 (item -> trạng thái -> version xử lý/huỷ).
  2. Gắn link chéo từ v0.2 sang plan canonical hiện tại.
  3. Không còn mục deferred “trôi nổi” không biết đã kết thúc ở đâu.

### V6-ISSUE-010 - Lệch contract CLI trong plan v0.3 (`ocl doc`/`ocl lsp`)
- Severity: `MAJOR`
- Status: `DONE`
- Evidence:
  - `projects/ocp-ocl/OCL-MVP-PLAN-v0.3.md:87-98` ghi `CLI bắt buộc`, gồm:
    - `ocl doc`
    - `ocl lsp`
  - CLI hiện tại không có hai lệnh này:
    - `cargo run -p ocl-cli` in usage chỉ gồm `init|check|run|trace|profile|test|fmt|...`, không có `doc|lsp`.
- Impact:
  - Mâu thuẫn giữa tài liệu gate đã `DONE` và bề mặt CLI thực tế.
  - Gây nhầm lẫn cho người dùng mới khi đọc lịch sử v0.3.
- Exit criteria:
  1. Chuẩn hóa trạng thái: hoặc bổ sung rõ là mục đã hạ scope/defer, hoặc ghi rõ bị loại khỏi release contract.
  2. Không còn câu chữ “bắt buộc” cho lệnh không tồn tại.
  3. Có tham chiếu tới quyết định thay thế tương ứng trong plan mới hơn.

### V6-ISSUE-011 - Drift path evidence trong plan v0.3 sau repo split
- Severity: `MAJOR`
- Status: `DONE`
- Evidence:
  - Nhiều command/log còn path legacy hoặc mơ hồ sau khi chuyển sang `projects/ocp-ocl/`, ví dụ:
    - `cargo run -p ocl-cli -- compose app-ocl --phenotype app-ocl/phenotype.toml --locked --json` (`projects/ocp-ocl/OCL-MVP-PLAN-v0.3.md:815`)
    - `cargo run -p ocl-cli -- verify app-ocl --phenotype app-ocl/phenotype.toml --locked --json` (`projects/ocp-ocl/OCL-MVP-PLAN-v0.3.md:816`)
    - file range không hợp lệ: `projects/ocp-ocl/OCL-MVP-PLAN-v0.1..v0.3.md` (`projects/ocp-ocl/OCL-MVP-PLAN-v0.3.md:363`)
  - Có mix giữa root cũ và workspace mới trong cùng log.
- Impact:
  - Replay evidence khó lặp đúng khi audit lại.
  - Tăng rủi ro đọc sai ngữ cảnh do đường dẫn không chuẩn.
- Exit criteria:
  1. Bổ sung chú thích rõ cho từng block: legacy command hay workspace command.
  2. Chuẩn hóa path examples về format thống nhất (`projects/ocp-ocl/...`) hoặc gắn nhãn lịch sử.
  3. Loại bỏ/đổi các entry path không hợp lệ kiểu range.

### V6-ISSUE-012 - Danh sách “defer sang v0.4” trong v0.3 chưa có bảng đóng vòng
- Severity: `MINOR`
- Status: `DONE`
- Evidence:
  - `projects/ocp-ocl/OCL-MVP-PLAN-v0.3.md:270-277` có mục defer sang v0.4:
    - concurrency/debugger/bytecode-IR/FFI-ABI/stdlib mở rộng/composer nâng cấp.
  - Chưa có bảng trong chính v0.3 chỉ rõ từng item defer đã được xử lý ở gate nào của v0.4/v0.5.
- Impact:
  - Audit chuỗi version phải dò tay qua nhiều file.
  - Dễ bỏ sót mục “defer” khi review completeness trước phát hành.
- Exit criteria:
  1. Tạo bảng “Defer v0.3 -> resolution gate” ngay trong v0.3 hoặc tài liệu chỉ mục chung.
  2. Mỗi mục defer có trạng thái rõ: resolved / dropped / replaced.
  3. Có link chéo đến entry gate tương ứng đã `DONE`.

### V6-ISSUE-013 - Mâu thuẫn policy ký trong bản ghi v0.4 (repo key vs external key)
- Severity: `MAJOR`
- Status: `DONE`
- Evidence:
  - `projects/ocp-ocl/OCL-MVP-PLAN-v0.4.md:433` ghi lane mặc định signed/release-grade theo repo signer baseline.
  - `projects/ocp-ocl/OCL-MVP-PLAN-v0.4.md:522` ghi lock rằng sign key không default từ repo path.
  - `projects/ocp-ocl/OCL-MVP-PLAN-v0.4.md:559` dùng command evidence với `--sign-key projects/ocp-ocl/security/dev-root-1.signing.key.toml`.
  - Script hiện tại:
    - `tools/ci_ocl_lane.ps1:14-18` chỉ lấy sign key từ env/flag, không auto default repo key.
    - `tools/ci_ocl_release.ps1:35-37` mặc định từ chối fixture key repo trong release lane.
- Impact:
  - Mâu thuẫn giữa narrative lịch sử và policy release hiện hành.
  - Gây nhầm lẫn khi tái hiện lane v0.4 hoặc audit security posture.
- Exit criteria:
  1. Chuẩn hóa chú thích “historical evidence dùng fixture signer” vs “release policy hiện tại”.
  2. Không còn câu chữ có thể hiểu là repo key là baseline release mặc định.
  3. Link rõ tới policy phát hành hiện hành.

### V6-ISSUE-014 - Drift mặc định conformance giữa v0.4 và hiện trạng
- Severity: `MINOR`
- Status: `DONE`
- Evidence:
  - `projects/ocp-ocl/OCL-MVP-PLAN-v0.4.md:540`/`559` bám `conformance.v1.toml`.
  - Mặc định hiện tại ở SDK là v5:
    - `projects/ocp-ocl/crates/ocl-sdk/src/w9.rs:157-166` (`OCL_CONFORMANCE_DEFAULT`, fallback `conformance.v5.toml`).
- Impact:
  - Người đọc v0.4 dễ chạy sai default nếu không biết đây là lịch sử.
- Exit criteria:
  1. Đánh dấu rõ trong v0.4 rằng conformance v1 là historical snapshot.
  2. Link đến rule mặc định hiện hành (v5 + env rollback).

### V6-ISSUE-015 - Mục defer trong v0.4 chưa có bảng resolution nội bộ
- Severity: `MINOR`
- Status: `DONE`
- Evidence:
  - `projects/ocp-ocl/OCL-MVP-PLAN-v0.4.md:91` còn ghi “Single-actor bounded mailbox cho W1, multi-actor defer.”
  - Không có bảng trong chính v0.4 chỉ rõ mục defer này được đóng ở gate/version nào.
- Impact:
  - Audit theo từng phiên bản phải dò tay sang file khác để biết mục defer đã kết thúc hay chưa.
- Exit criteria:
  1. Bổ sung bảng “defer -> resolved-at” cho v0.4 (hoặc trong chỉ mục chung v0.6) cho mọi defer còn lại.
  2. Mỗi defer có trạng thái rõ ràng (resolved/dropped).

### V6-ISSUE-016 - Gate log v0.5 còn dùng repo fixture signer trong evidence `DONE`
- Severity: `MAJOR`
- Status: `DONE`
- Evidence:
  - `projects/ocp-ocl/OCL-MVP-PLAN-v0.5.md:1021` ghi pass lane với:
    - `-SignKey projects/ocp-ocl/security/dev-root-1.signing.key.toml`
  - `projects/ocp-ocl/OCL-MVP-PLAN-v0.5.md:1071` ghi pass conformance với:
    - `--sign-key projects/ocp-ocl/security/dev-root-1.signing.key.toml`
- Impact:
  - Plan canonical v0.5 dễ bị hiểu là repo fixture key là baseline hợp lệ cho release evidence.
  - Mâu thuẫn ngầm với mục tiêu tách release key khỏi repo-fixture key.
- Exit criteria:
  1. Đánh dấu rõ các command này là local/fixture evidence, không phải release baseline.
  2. Bổ sung command tương đương dùng external sign key path cho release-grade evidence.
  3. Link rõ tới release lane policy hiện hành trong cùng section gate log.

### V6-ISSUE-017 - CI lane đang generate/delete `cosmos` cho `app-ocl` tại runtime (baseline không pin trong repo)
- Severity: `MAJOR`
- Status: `DONE`
- Evidence:
  - `projects/ocp-ocl/OCL-MVP-PLAN-v0.5.md:746` ghi rõ `cosmos.toml/cosmos.lock.v1` không commit mặc định cho `app-ocl`, lane tự `cosmos init`.
  - `tools/ci_ocl_lane.ps1:94-95` xóa `app-ocl/cosmos.toml` và `app-ocl/cosmos.lock.v1` trước run.
  - `tools/ci_ocl_lane.ps1:116-118` sinh lại `cosmos init` + `cosmos lock sync`.
  - `tools/ci_ocl_lane.ps1:143-144` xóa lại sau run.
- Impact:
  - Baseline locked/universe của app canary phụ thuộc mutation trong CI thay vì artifact đã pin.
  - Khó audit replay vì input governance quan trọng không tồn tại cố định trong tree dự án.
- Exit criteria:
  1. Chốt một baseline rõ ràng:
     - hoặc commit `cosmos.toml/cosmos.lock.v1` cho canary app,
     - hoặc chuyển lane sang fixture copy hoàn toàn ngoài working tree và ghi evidence path cố định.
  2. Không còn bước mutate/xóa trực tiếp file governance trong source app khi chạy lane chính.
  3. Replay lane từ clean checkout cho cùng commit cho ra cùng input governance mà không cần generate in-place.

## 2) Rules khi đóng issue
1. Mỗi issue phải có command evidence trước/sau.
2. Không đóng issue nếu chỉ sửa docs mà chưa có verification tương ứng.
3. Không thêm issue “nice to have” nếu không ảnh hưởng release readiness.

## 3) Trạng thái tổng
- Open issues: `0`
- Blocker issues: `0`
- v0.6 readiness: `READY`

