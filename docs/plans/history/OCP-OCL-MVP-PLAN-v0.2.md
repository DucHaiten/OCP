# OCL v0.2 Master Plan + Execution Log

Ngày tạo: 2026-02-21
Mục tiêu: một file duy nhất để theo dõi kế hoạch + nhật ký triển khai v0.2 theo hướng core-first, đọc lại là nắm được ngay.

## Quy ước cập nhật bắt buộc (áp dụng từ 2026-02-21)
- Mọi thay đổi kế hoạch phải cập nhật file này trước khi triển khai.
- Mọi triển khai xong phải cập nhật file này ngay sau khi chạy test.
- Mỗi entry phải có: ngày, mục tiêu, phạm vi, files changed, lệnh test, kết quả, notes/risks.
- Không nhảy gate: gate sau chỉ mở khi gate hiện tại đã `DONE`.

### Template cập nhật kế hoạch (trước khi làm)
- Date:
- Gate/Step:
- Why:
- Scope:
- Expected tests:
- Exit criteria:

### Template cập nhật triển khai (sau khi làm)
- Date:
- Gate/Step:
- Implemented:
- Files changed:
- Commands run:
- Test results:
- Notes/risks:

## Quick Snapshot (BẮT BUỘC ĐỌC TRƯỚC)
- Mục tiêu phiên bản:
  - Khóa core semantics v0.2 theo hướng core-first, bounded, fail-honest.
- Trạng thái tổng quan:
  - Xem `Trạng thái Gate v0.2` ngay bên dưới.
- Trạng thái kiểm chứng mới nhất:
  - Re-verify + confidence hardening ngày `2026-03-04` (v0.6 audit): `PASS`.
- Trình tự làm việc chuẩn:
  - Planning freeze -> Implementation -> Test -> Closeout.
- Bằng chứng kỹ thuật phải có cho từng gate:
  - Files changed
  - Commands run
  - Test results (PASS/FAIL)
- Bước tiếp theo:
  - Không nhảy gate, chỉ mở gate mới khi gate hiện tại `DONE`.

## Trạng thái Gate v0.2
- Gate V2-A (Canonicalization Freeze): `DONE` (2026-03-03)
- Gate V2-B (P0 Core Alignment): `DONE` (2026-03-03)
- Gate V2-C (Bounded Condition Semantics): `DONE` (2026-03-03)
- Gate V2-D (Bounded Entangle Semantics): `DONE` (2026-03-03)
- Gate V2-E (Regression/Soak/Docs Closeout): `DONE` (2026-03-03)

---

## Scope khóa cho v0.2 (Core-first)
### In-scope bắt buộc
- Chốt canonical rule cho `condition`.
- Chốt canonical taxonomy mã lỗi: `P-* / T-* / X-* / R-* / RC-*`.
- Chia rõ profile `P0 Core` và `P1 Extended` để tránh overclaim.
- Ship semantics bounded thật cho `condition` (không no-op).
- Ship semantics bounded thật cho `entangle` (không no-op).
- Pass full regression + pilot/soak + lint/fmt + docs closeout.

### Out-of-scope / Deferred sang v0.2.1
- Typed context parser/checker đầy đủ theo AST (`ctx(field=...)`).
- `horizon=...` grammar/checker/exec/bridge mapping đầy đủ.
- Loop/function/module/class/object và codegen backend.

---

## Current Capability Snapshot (v0.2)
- OCL syntax đang hỗ trợ: `let`, `entangle`, `condition`, `observe`, `commit`, `match`, field-access narrowing.
- Runtime keys: `world.exists:*`, `world.bounds:*`, `render.prims:*`.
- Condition executor-visible surface:
- `PASS -> continue`
- `FAIL -> X-COND-FALSE`
- `DEFERRED -> X-COND-DEFERRED`
- `INSUFFICIENT -> X-COND-INSUFFICIENT + root_reason=RC-*`
- Entangle semantics:
- session-local constraint state
- edge cap + degree cap + propagation budget cap
- fail-honest, không hidden observe/commit/materialize
- Bridge context contract core semantic fields (numeric):
- `seed:u64`, `tick:u64`, `observer_id:u64`, `policy_id:u32`
- Tooling status:
- pilot + soak + compare vẫn regression-pass.

## Versioned Language Surface
### v0.2 P0 Core (ĐÃ SHIP)
- Grammar như baseline v0.1/B+, không mở syntax lớn mới.
- Canonical diagnostics + alias migration path.
- Bounded condition + bounded entangle semantics.
- Legacy `ctx("...")` với strict/lenient bridge validation.

### v0.2 P1 Extended (DEFERRED v0.2.1)
- Typed context grammar (`ctx(field=...)`).
- `horizon=...` argument và semantics đầy đủ.

## Operational Commands
- `cargo test`
- `cargo test --test ocl_parser`
- `cargo test --test ocl_typecheck`
- `cargo test --test ocl_exec`
- `cargo test --test ocl_toy_programs`
- `cargo test --test ocl_pilot`
- `cargo test --bin soak_compare`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt -- --check`

---

## Gate V2-A - Canonicalization Freeze (`DONE`)

### V2-A.1 Condition canonical rule freeze
- Rule canonical đã chốt:
- evaluator nội bộ có thể cho `PASS/FAIL/DEFERRED/INSUFFICIENT`
- executor-visible bắt buộc map về `X-COND-*` cho mọi non-PASS
- policy MAY đổi strategy/threshold nhưng MUST NOT đổi result-shape public
- Exit: không còn wording mâu thuẫn ở spec.

### V2-A.2 Taxonomy freeze
- Namespace đã chốt:
- `P-*` parser
- `T-*` type checker
- `X-*` executor
- `R-*` bridge/runtime integration
- `RC-*` runtime reason code cho insufficient
- `E-*` giữ vai trò alias migration tạm thời.

### V2-A.3 Conformance profile freeze
- `P0` = core ship trong v0.2.
- `P1` = extension defer v0.2.1.
- Feature chưa code xong không được viết thành MUST của P0.

### V2-A.4 Typed context contract freeze
- Core semantic fields chốt numeric:
- `seed:u64`, `tick:u64`, `observer_id:u64`, `policy_id:u32`
- Label/metadata string là optional non-semantic.

### V2-A.5 Evidence
- Spec đã bổ sung section freeze normative:
- `OCP-OCL-v0.2.md` (phần release freeze core-first).

---

## Gate V2-B - P0 Core Alignment (`DONE`)

### Mục tiêu
Đồng bộ parser/checker/exec/bridge theo canonical contract P0, không mở scope lớn.

### V2-B.1 Diagnostics migration phase-1
- Implemented:
- `src/ocp_ocl/diag.rs` canonicalize code theo layer.
- thêm `root_reason` field vào `Diagnostic`/`ExecError`.
- canonical assertions trong tests đã đổi sang `P/T/X/R`.
- Alias migration được giữ trong code path (không lock lâu dài).

### V2-B.2 Bridge error/code alignment
- Strict context validation surfacing code canonical (`R-CTX-*`) qua `ExecError` path.
- Lenient mode vẫn backward-compatible.

### V2-B.3 Test/snapshot alignment
- Đã update:
- `tests/ocl_parser.rs`
- `tests/ocl_typecheck.rs`
- `tests/ocl_exec.rs`
- `tests/ocl_ctx_validation.rs`
- `tests/ocl_diag_taxonomy.rs`
- Exit: test mới assert canonical codes.

### V2-B.4 Exit evidence
- Targeted suites pass:
- parser/typecheck/exec/ctx validation/diag taxonomy.

---

## Gate V2-C - Bounded Condition Semantics (`DONE`)

### Mục tiêu
`condition` không còn no-op trace-only; có evaluator bounded + deterministic + fail-honest.

### Rule thực thi đã ship
- Internal outcome: `Pass/Fail/Deferred/Insufficient`.
- Executor mapping:
- `Pass -> continue`
- `Fail -> X-COND-FALSE`
- `Deferred -> X-COND-DEFERRED`
- `Insufficient -> X-COND-INSUFFICIENT + root_reason=RC-*`

### Caps đã dùng trong code
- `condition_time_budget_ns=20_000`
- `condition_max_steps=64`
- `condition_max_constraints=256`

### Fixtures/tests bổ sung
- `tests/fixtures/ok_v2_condition_pass.ocl`
- `tests/fixtures/fail_exec_v2_condition_false.ocl`
- `tests/fixtures/fail_exec_v2_condition_insufficient.ocl`
- Assertions mới trong `tests/ocl_exec.rs`:
- verify code `X-COND-FALSE`
- verify code `X-COND-INSUFFICIENT`
- verify `root_reason` không bị mất.

### Exit evidence
- Không còn marker no-op cho condition.
- Exec suite pass với condition pass/fail/insufficient paths.

---

## Gate V2-D - Bounded Entangle Semantics (`DONE`)

### Mục tiêu
`entangle` không còn no-op; có semantics local/session bounded và deterministic.

### Rule thực thi đã ship
- Session-local `ConstraintSession`.
- Runtime validate bindings tồn tại.
- Constraint phải bool (type-check + runtime guard).
- Enforce caps:
- `entangle_max_edges_per_session=128`
- `entangle_max_degree_per_binding=16`
- `entangle_propagation_budget_ns=20_000`
- Không hidden observe/commit/materialize.

### Bổ sung checker
- `entangle` constraint expression bắt buộc `Bool`.

### Fixtures/tests bổ sung
- `tests/fixtures/ok_v2_entangle_session_local.ocl`
- `tests/fixtures/fail_exec_v2_entangle_degree_cap.ocl`
- `tests/fixtures/fail_type_v2_entangle_constraint_type.ocl`
- Assertions mới trong `tests/ocl_exec.rs` + `tests/ocl_typecheck.rs`.

### Exit evidence
- Không còn marker no-op cho entangle.
- Determinism trace test pass.
- Cap violation surfaced bằng canonical error.

---

## Gate V2-E - Regression / Soak / Docs Closeout (`DONE`)

### Mục tiêu
Đảm bảo v0.2 core không phá trục v0.1 và có evidence closeout đầy đủ.

### Regression matrix đã chạy
- `cargo test`
- `cargo test --test ocl_parser`
- `cargo test --test ocl_typecheck`
- `cargo test --test ocl_exec`
- `cargo test --test ocl_toy_programs`
- `cargo test --test ocl_pilot`
- `cargo test --bin soak_compare`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt -- --check`

### Kết quả
- Full suite xanh.
- Invariants v0.1 vẫn giữ:
- boundedness
- failure-honest
- determinism scope
- no direct access compile-fail
- pilot/soak suites không regression.

### Docs/report đã bổ sung
- `reports/gate-v02-core-readiness.md`
- cập nhật freeze normative vào `OCP-OCL-v0.2.md`.

---

## Known Limits sau v0.2 close
- Typed context AST parser/checker chưa ship (defer v0.2.1).
- `horizon=...` chưa ship (defer v0.2.1).
- Không có global solver; entangle chỉ local/session-bounded.

## Release Close Checklist (v0.2)
- [x] Gate `V2-E` đã `DONE` (2026-03-03).
- [x] Gate `V2-D` đã `DONE` (2026-03-03).
- [x] Gate `V2-C` đã `DONE` (2026-03-03).
- [x] Gate `V2-B` đã `DONE` (2026-03-03).
- [x] Gate `V2-A` đã `DONE` (2026-03-03).
- [x] Canonical rule `condition` đã khóa trong spec và code.
- [x] `condition` và `entangle` không còn no-op runtime.
- [x] Diagnostics canonical `P/T/X/R/RC` đã được assert trong tests mới.
- [x] Full matrix test/lint/format pass:
- `cargo test`
- `cargo test --test ocl_parser`
- `cargo test --test ocl_typecheck`
- `cargo test --test ocl_exec`
- `cargo test --test ocl_toy_programs`
- `cargo test --test ocl_pilot`
- `cargo test --bin soak_compare`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt -- --check`
- [x] Báo cáo closeout tồn tại: `reports/gate-v02-core-readiness.md`.
- [x] Release notes tồn tại: `reports/release-notes-v0.2.0.md`.
- [x] Known limits/deferred của v0.2.1 đã ghi rõ, không overclaim.

---

## Nhật ký triển khai

### 2026-02-21 - V2-A planning freeze (pre-implementation)
- Date:
- 2026-02-21
- Gate/Step:
- V2-A planning freeze
- Why:
- Khóa semantics canonical trước khi sửa code để tránh drift spec/parser/checker/tests.
- Scope:
- Core-first V2-A..V2-E.
- Defer typed ctx/horizon sang v0.2.1.
- Expected tests:
- baseline parser/typecheck/exec.
- Exit criteria:
- có plan file v0.2 chính thức và scope lock.

### 2026-03-03 - V2-A implementation closeout
- Date:
- 2026-03-03
- Gate/Step:
- V2-A
- Implemented:
- Chuẩn hóa taxonomy runtime: thêm canonical nhóm `X-*` và `R-*`.
- Giữ alias migration `E-*` qua `ErrorCode::canonical()` và `ErrorCode::legacy_alias()`.
- Khóa contract profile:
- `ConformanceProfile::{P0Core,P1Extended}`.
- Khóa typed context core fields:
- `seed:u64`, `tick:u64`, `observer_id:u64`, `policy_id:u32`.
- Cập nhật test canonical taxonomy + compatibility.
- Files changed:
- `src/ocp_ocl/diag.rs`
- `src/ocp_ocl/mod.rs`
- `src/ocp_ocl/ctx_contract.rs`
- `tests/ocl_diag.rs`
- `tests/ocl_exec.rs`
- `tests/ocl_commit_policy.rs`
- `tests/ocl_ctx_contract.rs`
- `OCP-OCL-MVP-PLAN-v0.2.md`
- Commands run:
- `cargo test --test ocl_diag --test ocl_ctx_contract --test ocl_exec --test ocl_commit_policy`
- `cargo test`
- Test results:
- PASS:
- `ocl_diag`: 4/4
- `ocl_ctx_contract`: 2/2
- `ocl_exec`: 5/5
- `ocl_commit_policy`: 4/4
- full `cargo test`: tất cả suite pass.
- Notes/risks:
- Executor vẫn còn tạo một số `ErrorCode::E*` nội bộ; output canonical đã được khóa qua `as_str()`.

### 2026-03-03 - V2-B planning freeze
- Date:
- 2026-03-03
- Gate/Step:
- V2-B
- Why:
- Đồng bộ parser/typecheck/exec/bridge theo taxonomy canonical vừa chốt ở V2-A.
- Scope:
- Chuẩn hóa mã lỗi runtime/context sang canonical `X-*`/`R-*`.
- Bổ sung `root_reason` cho nhánh fail-honest liên quan context/runtime.
- Cập nhật test taxonomy + ctx validation theo contract canonical.
- Expected tests:
- `cargo test --test ocl_parser`
- `cargo test --test ocl_typecheck`
- `cargo test --test ocl_exec`
- `cargo test --test ocl_ctx_validation`
- `cargo test --test ocl_diag_taxonomy`
- `cargo test`
- Exit criteria:
- Runtime/context không còn surfacing `E-*` ở output public.
- Tests taxonomy/context pass theo canonical contract.

### 2026-03-03 - V2-B implementation closeout
- Date:
- 2026-03-03
- Gate/Step:
- V2-B
- Implemented:
- Đồng bộ taxonomy runtime theo canonical:
- thêm `R-CTX-INVALID` cho strict ctx validation.
- giữ alias migration `E-*` nhưng output canonical luôn trả `X-*`/`R-*`.
- Bổ sung `root_reason` vào `Diagnostic` để giữ nguyên nhân gốc `RC-*`.
- Executor:
- chuyển các lỗi runtime từ `E-*` sang canonical `X-*`/`R-*`.
- thêm validate `ctx("k=v;...")` (duplicate key, key/value rỗng, charset key).
- lỗi ctx invalid surface `R-CTX-INVALID` + `root_reason=RC-CTX-INVALID`.
- Bổ sung test alignment:
- `tests/ocl_ctx_validation.rs`
- `tests/ocl_diag_taxonomy.rs`
- update `tests/ocl_diag.rs`, `tests/ocl_exec.rs`, `tests/ocl_commit_policy.rs`
- Files changed:
- `src/ocp_ocl/diag.rs`
- `src/ocp_ocl/exec.rs`
- `tests/ocl_diag.rs`
- `tests/ocl_exec.rs`
- `tests/ocl_commit_policy.rs`
- `tests/ocl_ctx_validation.rs`
- `tests/ocl_diag_taxonomy.rs`
- `OCP-OCL-MVP-PLAN-v0.2.md`
- Commands run:
- `cargo test --test ocl_parser --test ocl_typecheck --test ocl_exec --test ocl_ctx_validation --test ocl_diag_taxonomy`
- `cargo test`
- Test results:
- PASS:
- Targeted:
- `ocl_parser`: 4/4
- `ocl_typecheck`: 5/5
- `ocl_exec`: 5/5
- `ocl_ctx_validation`: 2/2
- `ocl_diag_taxonomy`: 4/4
- Full `cargo test`: toàn bộ suite pass.
- Notes/risks:
- Alias `E-*` vẫn còn ở mức migration enum; output canonical đã khóa theo `as_str()`.

### 2026-03-03 - V2-C planning freeze
- Date:
- 2026-03-03
- Gate/Step:
- V2-C
- Why:
- Chuyển `condition` từ no-op trace-only sang evaluator bounded deterministic, fail-honest.
- Scope:
- Ship internal outcome `Pass/Fail/Deferred/Insufficient`.
- Map canonical executor-visible `X-COND-*` + `root_reason=RC-*` cho nhánh insufficient.
- Bổ sung fixtures/test cho pass/fail/deferred/insufficient.
- Expected tests:
- `cargo test --test ocl_exec`
- `cargo test --test ocl_diag`
- `cargo test`
- Exit criteria:
- `condition` không còn no-op.
- Có bằng chứng pass/fail/deferred/insufficient qua test.

### 2026-03-03 - V2-C implementation closeout
- Date:
- 2026-03-03
- Gate/Step:
- V2-C
- Implemented:
- Nâng `condition` từ bool-check trực tiếp sang evaluator bounded có 4 outcome nội bộ:
- `Pass | Fail | Deferred | Insufficient`.
- Bổ sung caps bounded trong executor:
- `condition_time_budget_ns=20_000` (pseudo-time deterministic)
- `condition_max_steps=64`
- `condition_max_constraints=256`
- Mapping executor-visible canonical:
- `Fail -> X-COND-FALSE`
- `Deferred -> X-COND-DEFERRED`
- `Insufficient -> X-COND-INSUFFICIENT + root_reason=RC-*`
- Bổ sung mã lỗi canonical cho V2-C trong diagnostics:
- `X-COND-DEFERRED`, `X-COND-INSUFFICIENT`
- Chuẩn hóa alias cũ:
- `E-CONDITION-FALSE` map về `X-COND-FALSE`
- Bổ sung tests và fixtures cho V2-C:
- update `tests/ocl_diag.rs`
- update `tests/ocl_exec.rs`
- thêm fixtures:
- `tests/fixtures/ok_v2_condition_pass.ocl`
- `tests/fixtures/fail_exec_v2_condition_false.ocl`
- `tests/fixtures/fail_exec_v2_condition_insufficient.ocl`
- `tests/fixtures/fail_exec_v2_condition_deferred.ocl`
- Files changed:
- `src/ocp_ocl/diag.rs`
- `src/ocp_ocl/exec.rs`
- `tests/ocl_diag.rs`
- `tests/ocl_exec.rs`
- `tests/fixtures/ok_v2_condition_pass.ocl`
- `tests/fixtures/fail_exec_v2_condition_false.ocl`
- `tests/fixtures/fail_exec_v2_condition_insufficient.ocl`
- `tests/fixtures/fail_exec_v2_condition_deferred.ocl`
- `OCP-OCL-MVP-PLAN-v0.2.md`
- Commands run:
- `cargo test --test ocl_exec --test ocl_diag`
- `cargo test`
- Test results:
- PASS:
- `ocl_exec`: 7/7
- `ocl_diag`: 4/4
- full `cargo test`: toàn bộ suite pass.
- Notes/risks:
- Hai nhánh `DEFERRED/INSUFFICIENT` có thể xuất hiện cả khi bỏ qua typecheck (exec-only path), đây là hành vi chủ đích để test runtime bounded evaluator.

### 2026-03-03 - V2-D planning freeze
- Date:
- 2026-03-03
- Gate/Step:
- V2-D
- Why:
- Đưa `entangle` từ no-op sang semantics bounded/session-local, deterministic, fail-honest.
- Scope:
- Mở syntax/parser/typecheck/runtime cho `entangle(left,right,constraint)`.
- Enforce caps session-local: edge cap, degree cap, propagation budget cap.
- Thêm canonical error path `X-ENTANGLE-*`.
- Expected tests:
- `cargo test --test ocl_parser`
- `cargo test --test ocl_typecheck`
- `cargo test --test ocl_exec`
- `cargo test`
- Exit criteria:
- `entangle` có semantics runtime thật, không còn no-op marker.
- Cap violation surfaced bằng canonical errors.

### 2026-03-03 - V2-D implementation closeout
- Date:
- 2026-03-03
- Gate/Step:
- V2-D
- Implemented:
- Bổ sung syntax/runtime `entangle` end-to-end:
- lexer keyword `entangle`
- parser statement `entangle(left, right, constraint);`
- typecheck:
- binding trái/phải phải tồn tại
- constraint bắt buộc `Bool`
- runtime session-local `ConstraintSession` bounded:
- `entangle_max_edges_per_session=128`
- `entangle_max_degree_per_binding=16`
- `entangle_propagation_budget_ns=20_000` (pseudo-time deterministic)
- runtime guards fail-honest:
- `X-ENTANGLE-BINDING-UNKNOWN`
- `X-ENTANGLE-CONSTRAINT-TYPE`
- `X-ENTANGLE-CONSTRAINT-FALSE`
- `X-ENTANGLE-EDGE-CAP`
- `X-ENTANGLE-DEGREE-CAP`
- `X-ENTANGLE-DEFERRED`
- Thêm fixtures và test cho V2-D.
- Files changed:
- `src/ocp_ocl/ast.rs`
- `src/ocp_ocl/lex.rs`
- `src/ocp_ocl/parse.rs`
- `src/ocp_ocl/typecheck.rs`
- `src/ocp_ocl/exec.rs`
- `src/ocp_ocl/diag.rs`
- `tests/ocl_parser.rs`
- `tests/ocl_typecheck.rs`
- `tests/ocl_exec.rs`
- `tests/fixtures/ok_v2_entangle_session_local.ocl`
- `tests/fixtures/fail_exec_v2_entangle_degree_cap.ocl`
- `tests/fixtures/fail_type_v2_entangle_constraint_type.ocl`
- `OCP-OCL-MVP-PLAN-v0.2.md`
- Commands run:
- `cargo test --test ocl_parser --test ocl_typecheck --test ocl_exec`
- `cargo test`
- Test results:
- PASS:
- `ocl_parser`: 4/4
- `ocl_typecheck`: 6/6
- `ocl_exec`: 9/9
- full `cargo test`: toàn bộ suite pass.
- Notes/risks:
- Propagation budget dùng pseudo-time deterministic để giữ replay ổn định; chưa dùng đồng hồ thực.

### 2026-03-03 - V2-E planning freeze
- Date:
- 2026-03-03
- Gate/Step:
- V2-E
- Why:
- Chốt regression/docs closeout để xác nhận v0.2 không phá trục v0.1 và đủ evidence release.
- Scope:
- Chạy full test matrix + lint/fmt.
- Bổ sung artifacts còn thiếu cho closeout (`soak_compare`, `ocl_toy_programs`, reports/spec freeze).
- Cập nhật checklist và log closeout cuối v0.2.
- Expected tests:
- `cargo test`
- `cargo test --test ocl_parser`
- `cargo test --test ocl_typecheck`
- `cargo test --test ocl_exec`
- `cargo test --test ocl_toy_programs`
- `cargo test --test ocl_pilot`
- `cargo test --test ocl_confidence`
- `cargo test --bin soak_compare`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt -- --check`
- Exit criteria:
- Matrix pass đầy đủ, docs/reports closeout tồn tại, known limits ghi rõ.

### 2026-03-03 - V2-E implementation closeout
- Date:
- 2026-03-03
- Gate/Step:
- V2-E
- Implemented:
- Hoàn tất regression/docs closeout cho v0.2 theo matrix command chuẩn.
- Bổ sung artifacts còn thiếu để matrix chạy đủ:
- thêm test suite `tests/ocl_toy_programs.rs`
- thêm bin `src/bin/soak_compare.rs`
- tạo báo cáo:
- `reports/gate-v02-core-readiness.md`
- `reports/release-notes-v0.2.0.md`
- thêm spec freeze:
- `OCP-OCL-v0.2.md`
- Chuẩn hóa format toàn repo bằng `cargo fmt` trước khi check lại.
- Files changed:
- `src/bin/soak_compare.rs`
- `tests/ocl_toy_programs.rs`
- `reports/gate-v02-core-readiness.md`
- `reports/release-notes-v0.2.0.md`
- `OCP-OCL-v0.2.md`
- `OCP-OCL-MVP-PLAN-v0.2.md`
- (format-only) nhiều file `src/` và `tests/` được chuẩn hóa bởi `cargo fmt`.
- Commands run:
- `cargo test`
- `cargo test --test ocl_parser`
- `cargo test --test ocl_typecheck`
- `cargo test --test ocl_exec`
- `cargo test --test ocl_toy_programs`
- `cargo test --test ocl_pilot`
- `cargo test --bin soak_compare`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt`
- `cargo fmt -- --check`
- Test results:
- PASS toàn bộ matrix:
- `cargo test`: pass toàn bộ suite.
- từng suite riêng (`ocl_parser`, `ocl_typecheck`, `ocl_exec`, `ocl_toy_programs`, `ocl_pilot`) đều pass.
- `cargo test --bin soak_compare`: 1/1 pass.
- `cargo clippy --all-targets -- -D warnings`: pass.
- `cargo fmt -- --check`: pass (sau khi chạy `cargo fmt`).
- Notes/risks:
- Không có regression mới ở các suite đã tồn tại.
- Soak compare hiện là binary baseline tối thiểu cho matrix v0.2; có thể mở rộng logic ở v0.2.1+ nếu cần so sánh artifact chi tiết.

### 2026-03-03 - V0.6 audit v0.2 consistency re-verify
- Date:
- 2026-03-03
- Gate/Step:
- V2-A..V2-E re-verify (v0.6 audit)
- Implemented:
- Rerun đầy đủ matrix kiểm chứng v0.2 theo `Operational Commands`.
- Đối chiếu trực tiếp code cho các trục khóa v0.2:
- caps bounded `condition` và `entangle`,
- taxonomy canonical `X-*`/`R-*`,
- `root_reason=RC-*` cho runtime fail-honest.
- Chuẩn hóa nhật ký để đủ cặp planning freeze + implementation closeout cho từng gate V2-B..V2-E.
- Files changed:
- `OCP-OCL-MVP-PLAN-v0.2.md`
- Commands run:
- `cargo test --test ocl_parser`
- `cargo test --test ocl_typecheck`
- `cargo test --test ocl_exec`
- `cargo test --test ocl_toy_programs`
- `cargo test --test ocl_pilot`
- `cargo test --bin soak_compare`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt -- --check`
- `rg -n "COND|ENTANGLE|condition|entangle" src/ocp_ocl/exec.rs`
- `rg -n "R-CTX-INVALID|RC-CTX-INVALID|root_reason" src/ocp_ocl/exec.rs src/ocp_ocl/diag.rs tests/ocl_ctx_validation.rs tests/ocl_diag_taxonomy.rs`
- Test results:
- PASS:
- `ocl_parser`: 4/4
- `ocl_typecheck`: 6/6
- `ocl_exec`: 9/9
- `ocl_toy_programs`: 3/3
- `ocl_pilot`: 4/4
- `soak_compare` (bin test): 1/1
- `cargo test` full: PASS toàn bộ suite hiện có.
- `cargo clippy --all-targets -- -D warnings`: PASS.
- `cargo fmt -- --check`: PASS.
- Notes/risks:
- Không phát hiện drift thực thi giữa plan v0.2 và code hiện tại.
- Các số test trong closeout lịch sử của từng gate là ảnh chụp theo thời điểm gate đóng; trạng thái mới nhất được khóa bởi entry audit này.

### 2026-03-04 - V0.6 confidence hardening append (final-confidence pass)
- Date:
- 2026-03-04
- Gate/Step:
- V2-A..V2-E confidence hardening (v0.6 audit append)
- Implemented:
- Thêm suite `tests/ocl_confidence.rs` để tăng mức tự tin cuối, tập trung vào:
- determinism lặp nhiều vòng cho core flow v1 (`64` runs),
- mapping strict `condition` deferred/insufficient + `root_reason`,
- hành vi `commit_policy` theo từng mode (`Normal/ForbidCommit/ShadowCommitLog`),
- kiểm tra runtime surface không rò rỉ mã legacy `E-*`.
- Rerun full regression + lint + format sau khi thêm suite mới.
- Files changed:
- `tests/ocl_confidence.rs`
- `OCP-OCL-MVP-PLAN-v0.2.md`
- Commands run:
- `cargo test --test ocl_confidence`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt`
- `cargo fmt -- --check`
- Test results:
- PASS:
- `ocl_confidence`: 4/4
- `cargo test` full: PASS toàn bộ suite hiện có.
- `cargo clippy --all-targets -- -D warnings`: PASS.
- `cargo fmt -- --check`: PASS (sau khi chạy `cargo fmt`).
- Notes/risks:
- Suite confidence tăng độ bao phủ hành vi quan trọng nhưng vẫn là kiểm thử hữu hạn; chưa phải chứng minh hình thức (formal proof).
