# OCL MVP PLAN v0.6 (Stabilization-Only)

Ngày tạo: 2026-02-26  
Mục tiêu: đóng hết nợ kỹ thuật/xung đột/rủi ro còn tồn đọng sau v0.5, không thêm tính năng mới.

## Cập nhật chốt cuối (2026-02-27)
- Trạng thái canonical đã chốt tại `projects/ocp-ocl/OCL-PLAN.md`: `OPEN=0` cho `001..017`.
- Unified release gate đã PASS trên clean checkout tại commit `1de5a285e7f16456cba0d2a39cef19c0435749ad`.
- Evidence signoff cuối: `projects/ocp-ocl/release/evidence/u10-final-signoff.md`.
- File này được giữ làm hồ sơ lịch sử của v0.6; trạng thái điều hành active chỉ đọc từ `OCL-PLAN.md`.
- Ghi chú bổ sung (2026-03-04): sau snapshot `001..017`, đã bổ sung các issue audit `018..023`; toàn bộ đều đã `DONE`, nên trạng thái tổng hiện tại vẫn `OPEN=0`.

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

### V6-ISSUE-018 - Drift tương thích test v0.1 sau thay đổi `ExecConfig`
- Severity: `MAJOR`
- Status: `DONE`
- Evidence:
  - Khi rerun matrix v0.1, nhiều suite fail compile `E0063`:
    - `tests/ocl_exec.rs`
    - `tests/ocl_registry.rs`
    - `tests/ocl_determinism.rs`
    - `tests/ocl_pilot.rs`
  - Nguyên nhân: `ExecConfig` hiện có thêm field `commit_policy` (`src/ocp_ocl/budget.rs`) nhưng test literals vẫn dùng dạng cũ chỉ có `step_cap`.
- Impact:
  - Mất khả năng verify lại baseline v0.1 một cách ổn định.
  - Tăng rủi ro drift “doc DONE nhưng test matrix không chạy lại được”.
- Exit criteria:
  1. Toàn bộ test literals `ExecConfig { step_cap: ... }` trong nhóm v0.1 được cập nhật hợp lệ.
  2. Rerun matrix v0.1 pass đầy đủ:
     - targeted suites,
     - `cargo test` full,
     - `cargo clippy --all-targets -- -D warnings`,
     - `cargo fmt -- --check`.
  3. Ghi nhận thay đổi vào `OCP-OCL-MVP-PLAN-v0.1.md` và giữ dấu vết trong v0.6.
- Resolution (2026-03-03):
  - Đã vá test literals sang:
    - `ExecConfig { step_cap: ..., ..ExecConfig::default() }`
  - Đã rerun matrix và pass:
    - `cargo test --test ocl_parser`
    - `cargo test --test ocl_typecheck`
    - `cargo test --test ocl_exec`
    - `cargo test --test ocl_registry`
    - `cargo test --test ocl_determinism`
    - `cargo test --test ocl_pilot`
    - `cargo test`
    - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
  - Đã cập nhật bổ sung audit entry vào `OCP-OCL-MVP-PLAN-v0.1.md`.

### V6-ISSUE-019 - Re-verify v0.2: chuẩn hóa full-log và đối chiếu thực thi
- Severity: `MAJOR`
- Status: `DONE`
- Evidence:
  - Matrix v0.2 rerun PASS trong cùng phiên:
    - `cargo test --test ocl_parser` (4/4)
    - `cargo test --test ocl_typecheck` (6/6)
    - `cargo test --test ocl_exec` (9/9)
    - `cargo test --test ocl_toy_programs` (3/3)
    - `cargo test --test ocl_pilot` (4/4)
    - `cargo test --bin soak_compare` (1/1)
    - `cargo test` (full suite PASS)
    - `cargo clippy --all-targets -- -D warnings` (PASS)
    - `cargo fmt -- --check` (PASS)
  - Đối chiếu code trục khóa v0.2:
    - `src/ocp_ocl/exec.rs` có caps bounded `condition`/`entangle` (`CONDITION_*`, `ENTANGLE_*`).
    - `src/ocp_ocl/diag.rs` và `src/ocp_ocl/exec.rs` giữ canonical `X-*`/`R-*` + `root_reason`.
    - `tests/ocl_ctx_validation.rs` assert `R-CTX-INVALID` + `RC-CTX-INVALID`.
  - Chuẩn hóa tài liệu:
    - `OCP-OCL-MVP-PLAN-v0.2.md` đã bổ sung planning freeze bị thiếu cho V2-B..V2-E.
    - Thêm entry `V0.6 audit v0.2 consistency re-verify` ở nhật ký v0.2.
- Impact:
  - Xác nhận `DONE` của v0.2 là done thực, có bằng chứng rerun mới nhất.
  - Loại bỏ rủi ro “log đủ closeout nhưng thiếu planning pair” trong chuẩn full-log hiện hành.
- Exit criteria:
  1. Toàn bộ matrix v0.2 pass lại trong một phiên audit thống nhất.
  2. v0.2 plan có đủ cặp planning freeze + implementation closeout cho mọi gate.
  3. Có entry audit mới trong v0.2 và issue log tương ứng trong v0.6.
- Resolution (2026-03-03):
  - Đã đạt đủ 3 tiêu chí trên.

### V6-ISSUE-020 - Mức tự tin cuối cho v0.2 chưa có confidence-suite chuyên biệt
- Severity: `MAJOR`
- Status: `DONE`
- Evidence:
  - Trước khi vá, v0.2 đã có matrix targeted/full pass nhưng chưa có suite chuyên cho:
    - lặp deterministic nhiều vòng trong một test,
    - lock hành vi `commit_policy` theo mode,
    - assert explicit “không rò rỉ legacy `E-*`” ở runtime surface.
  - Đã thêm file test mới:
    - `tests/ocl_confidence.rs`.
  - Nội dung suite mới:
    - `confidence_v1_core_flow_is_deterministic_over_many_runs` (64 runs),
    - `confidence_v2_condition_reason_mapping_is_strict`,
    - `confidence_commit_policy_modes_are_explicit`,
    - `confidence_runtime_error_surface_is_canonical_not_legacy_e_prefix`.
- Impact:
  - Tăng mức tự tin “không quay lại nữa” cho v0.2 nhờ một lớp kiểm thử tập trung vào invariant cốt lõi.
  - Giảm rủi ro lệch hành vi do thay đổi runtime nhỏ nhưng chưa lộ qua targeted suites cũ.
- Exit criteria:
  1. Suite confidence riêng chạy pass độc lập.
  2. Full regression sau khi thêm suite vẫn pass.
  3. Lint/format vẫn pass.
  4. Có log cập nhật tương ứng trong `OCP-OCL-MVP-PLAN-v0.2.md`.
- Resolution (2026-03-04):
  - Đã chạy pass:
    - `cargo test --test ocl_confidence` (4/4)
    - `cargo test` (full suite pass)
    - `cargo clippy --all-targets -- -D warnings` (pass)
    - `cargo fmt -- --check` (pass sau `cargo fmt`)
  - Đã cập nhật entry `2026-03-04 - V0.6 confidence hardening append` trong `OCP-OCL-MVP-PLAN-v0.2.md`.

### V6-ISSUE-021 - Re-verify v0.3: xác nhận `DONE` thực thi trên workspace hiện tại
- Severity: `MAJOR`
- Status: `DONE`
- Evidence:
  - Rerun đúng matrix `Operational Commands` của v0.3:
    - `cargo test -p ocl-runtime-core -p ocl-sdk -p ocl-cli`
    - `cargo fmt -- --check`
    - `cargo clippy -p ocl-runtime-core -p ocl-sdk -p ocl-cli --all-targets -- -D warnings`
    - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_lane.ps1`
  - Kết quả lane e2e:
    - conformance JSON: `scenarios_total=10`, `scenarios_passed=10`, `scenarios_failed=0`,
    - `required_digest=10deff71c24ef729`.
- Impact:
  - Xác nhận trạng thái `DONE` của M0-A..M5 là done thực ở baseline hiện tại, không chỉ log lịch sử.
  - Giảm rủi ro sai lệch giữa plan v0.3 và toolchain/runtime đã tích lũy sau các bản v0.4/v0.5.
- Exit criteria:
  1. Matrix v0.3 pass lại đầy đủ trong một phiên audit.
  2. Có entry re-verify trong `OCP-OCL-MVP-PLAN-v0.3.md`.
  3. Không phát sinh regression mới ở lane e2e và conformance.
- Resolution (2026-03-04):
  - Đã đạt đủ 3 tiêu chí trên.
  - Đã cập nhật entry `2026-03-04 - V0.6 audit v0.3 consistency re-verify` trong `OCP-OCL-MVP-PLAN-v0.3.md`.

### V6-ISSUE-022 - Re-verify v0.4: xác nhận toàn bộ W0..W9 `DONE` thực trên baseline hiện tại
- Severity: `MAJOR`
- Status: `DONE`
- Evidence:
  - Rerun matrix v0.4:
    - `cargo test -p ocl-runtime-core -p ocl-sdk -p ocl-cli`
    - `cargo fmt -- --check`
    - `cargo clippy -p ocl-runtime-core -p ocl-sdk -p ocl-cli --all-targets -- -D warnings`
    - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_lane.ps1`
  - Chạy thêm W9 explicit:
    - `cargo test -p ocl-sdk --test w9_conformance` (4/4)
    - `cargo test -p ocl-cli w9_cli_` (2/2)
    - `cargo run -p ocl-cli -- test --conformance --locked --runtime deterministic --engine dual --manifest projects/ocp-ocl/conformance/conformance.v1.toml --out target/ocl/w9/reports/conformance_report.json --trust-store projects/ocp-ocl/security/trust.store.toml --signer-id dev-root-1 --sign-key projects/ocp-ocl/security/dev-root-1.signing.key.toml --json` (10/10, `required_digest=10deff71c24ef729`)
- Impact:
  - Xác nhận trạng thái `DONE` v0.4 là done thực thi được, không phụ thuộc log cũ.
  - Khóa lại bằng chứng W9 theo cả lane tổng và command explicit.
- Exit criteria:
  1. Matrix v0.4 pass lại đầy đủ trong một phiên audit.
  2. W9 explicit pass độc lập (SDK test, CLI test, conformance command).
  3. Có entry re-verify mới trong `OCP-OCL-MVP-PLAN-v0.4.md`.
- Resolution (2026-03-04):
  - Đã đạt đủ 3 tiêu chí trên.
  - Đã cập nhật entry `2026-03-04 - V0.6 audit v0.4 consistency re-verify` trong `OCP-OCL-MVP-PLAN-v0.4.md`.

### V6-ISSUE-023 - Re-verify v0.5: chuẩn hóa validation matrix và xác nhận `DONE` thực thi
- Severity: `MAJOR`
- Status: `DONE`
- Evidence:
  - Đã rerun đầy đủ nhóm lệnh v0.5 hiện hành:
    - `cargo check --workspace`
    - `cargo test -p ocl-runtime-core -p ocl-sdk -p ocl-cli`
    - `cargo clippy -p ocl-runtime-core -p ocl-sdk -p ocl-cli --all-targets -- -D warnings`
    - `cargo fmt -- --check`
    - `cargo test -p ocl-sdk --test v5_w2_domain_resolution` (6/6)
    - `cargo test -p ocl-sdk --test v5_w3_shadow` (6/6)
    - `cargo test -p ocl-sdk --test v5_w4_hive` (3/3)
    - `cargo run -p ocl-cli -- test --conformance --locked --runtime deterministic --engine dual --manifest projects/ocp-ocl/conformance/conformance.v1.toml --out target/ocl/w9/reports/conformance_report.json --trust-store projects/ocp-ocl/security/trust.store.toml --signer-id dev-root-1 --sign-key projects/ocp-ocl/security/dev-root-1.signing.key.toml --json` (10/10, `required_digest=10deff71c24ef729`)
    - `cargo run -p ocl-cli -- test --conformance --locked --runtime deterministic --engine dual --manifest projects/ocp-ocl/conformance/conformance.v5.toml --out target/ocl/w9/reports/conformance_report.v5.json --trust-store projects/ocp-ocl/security/trust.store.toml --signer-id dev-root-1 --sign-key projects/ocp-ocl/security/dev-root-1.signing.key.toml --json` (5/5, `required_digest=9e97cc112c567170`)
    - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_lane.ps1` (PASS)
    - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_quarantine.ps1` (PASS)
  - Đã tái hiện drift trong matrix cũ:
    - `-p ocl-runtime-rt` fail vì package không tồn tại.
    - conformance locked command thiếu signer/trust fail `W9-CONFORMANCE-SIGN-REQUIRED`.
  - Đã cập nhật thủ công `OCP-OCL-MVP-PLAN-v0.5.md` section `Validation matrix v0.5` để phản ánh command thực chạy.
- Impact:
  - Xác nhận trạng thái `DONE` của v0.5 là done thực, không phụ thuộc log cũ.
  - Loại bỏ rủi ro “matrix trong plan lệch workspace” gây PASS giả hoặc tái hiện sai.
- Exit criteria:
  1. Matrix v0.5 hiện hành pass lại đầy đủ trên workspace hiện tại.
  2. Drift command cũ được ghi nhận rõ và thay bằng command chuẩn.
  3. Có entry audit mới trong `OCP-OCL-MVP-PLAN-v0.5.md` theo cặp planning/closeout.
- Resolution (2026-03-04):
  - Đã đạt đủ 3 tiêu chí trên.
  - Đã cập nhật entry:
    - `2026-03-04 - V0.6 audit v0.5 consistency re-verify (planning freeze)`
    - `2026-03-04 - V0.6 audit v0.5 consistency re-verify (implementation closeout)`
    trong `OCP-OCL-MVP-PLAN-v0.5.md`.

### V0.6 Consolidated Implementation Closeout (2026-03-04)
- Date:
  - 2026-03-04
- Gate/Step:
  - v0.6 stabilization register sync
- Implemented:
  - Đồng bộ register `V6-ISSUE-001..V6-ISSUE-023` về trạng thái đã đóng.
  - Bổ sung chú thích snapshot để tránh hiểu nhầm giữa mốc `001..017` và phần issue bổ sung `018..023`.
- Files changed:
  - `OCP-OCL-MVP-PLAN-v0.6.md`
- Commands run:
  - `cargo check --workspace`
  - `cargo test -p ocl-runtime-core -p ocl-sdk -p ocl-cli`
  - `cargo clippy -p ocl-runtime-core -p ocl-sdk -p ocl-cli --all-targets -- -D warnings`
  - `cargo fmt -- --check`
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_lane.ps1`
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_quarantine.ps1`
- Test results:
  - PASS:
    - Matrix kỹ thuật hiện hành pass trên workspace.
    - Không còn issue `OPEN/BLOCKER` trong register v0.6.
- Notes/risks:
  - Đây là closeout tổng hợp cho file register v0.6 để đáp ứng guard tài liệu; chi tiết kỹ thuật của từng issue vẫn nằm trong block issue tương ứng.

## 2) Rules khi đóng issue
1. Mỗi issue phải có command evidence trước/sau.
2. Không đóng issue nếu chỉ sửa docs mà chưa có verification tương ứng.
3. Không thêm issue “nice to have” nếu không ảnh hưởng release readiness.

## 3) Trạng thái tổng
- Open issues: `0`
- Blocker issues: `0`
- v0.6 readiness: `READY`

