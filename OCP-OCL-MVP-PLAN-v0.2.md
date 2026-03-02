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

## Trạng thái Gate v0.2
- Gate V2-A (Canonicalization Freeze): `TODO`
- Gate V2-B (P0 Core Alignment): `TODO`
- Gate V2-C (Bounded Condition Semantics): `TODO`
- Gate V2-D (Bounded Entangle Semantics): `TODO`
- Gate V2-E (Regression/Soak/Docs Closeout): `TODO`

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

## Gate V2-A - Canonicalization Freeze (`TODO`)

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

## Gate V2-B - P0 Core Alignment (`TODO`)

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

## Gate V2-C - Bounded Condition Semantics (`TODO`)

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

## Gate V2-D - Bounded Entangle Semantics (`TODO`)

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

## Gate V2-E - Regression / Soak / Docs Closeout (`TODO`)

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
- [ ] Gate `V2-A..V2-E` đều `TODO`.
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

### 2026-02-21 - V2-B + V2-C + V2-D implementation
- Date:
- 2026-02-21
- Gate/Step:
- V2-B, V2-C, V2-D
- Implemented:
- canonical diagnostics + root_reason metadata.
- condition bounded semantics + canonical X-COND mapping.
- entangle bounded semantics + session-local caps.
- update tests/snapshots sang canonical codes.
- update pilot commit-rejection recognition cho canonical codes.
- Files changed:
- `src/ocp_ocl/diag.rs`
- `src/ocp_ocl/checker.rs`
- `src/ocp_ocl/exec.rs`
- `src/pilot.rs`
- `tests/ocl_parser.rs`
- `tests/ocl_typecheck.rs`
- `tests/ocl_exec.rs`
- `tests/ocl_ctx_validation.rs`
- `tests/ocl_diag_taxonomy.rs`
- `tests/fixtures/ok_v2_condition_pass.ocl`
- `tests/fixtures/fail_exec_v2_condition_false.ocl`
- `tests/fixtures/fail_exec_v2_condition_insufficient.ocl`
- `tests/fixtures/ok_v2_entangle_session_local.ocl`
- `tests/fixtures/fail_exec_v2_entangle_degree_cap.ocl`
- `tests/fixtures/fail_type_v2_entangle_constraint_type.ocl`
- Commands run:
- `cargo test --test ocl_exec --test ocl_typecheck --test ocl_parser --test ocl_ctx_validation --test ocl_diag_taxonomy`
- `cargo test --test ocl_exec --test ocl_pilot --test ocl_soak`
- Test results:
- tất cả targeted suites pass.
- no-op markers cho condition/entangle bị loại bỏ trong behavior runtime.
- Notes/risks:
- một số literal E-* có thể vẫn tồn tại như alias migration trong code path cũ, nhưng tests mới đã lock canonical.

### 2026-02-21 - V2-E closeout
- Date:
- 2026-02-21
- Gate/Step:
- V2-E closeout
- Implemented:
- chạy full regression + lint + format.
- tạo report closeout core readiness.
- bổ sung section freeze canonical vào spec v0.2.
- Files changed:
- `OCP-OCL-v0.2.md`
- `reports/gate-v02-core-readiness.md`
- `OCL-MVP-PLAN-v0.2.md`
- Commands run:
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt -- --check`
- Test results:
- all pass.
- Notes/risks:
- typed ctx parser path + horizon được ghi rõ là deferred v0.2.1, không overclaim trong core DoD.

### 2026-02-21 - RC.1 -> RC.7 release close pass (pre-tag)
- Date:
- 2026-02-21
- Gate/Step:
- RC.1 -> RC.7
- Implemented:
- Soát lại inventory diff theo đúng phạm vi core-close.
- Sửa blocker encoding mojibake của `OCP-OCL-v0.2.md`.
- Bổ sung coverage deferred path cho `condition` (`X-COND-DEFERRED`) bằng bridge test chuyên dụng.
- Chỉnh pilot match script generators để tương thích semantics `condition` v0.2 (không còn giả định no-op).
- Tạo release notes chính thức cho v0.2.0.
- Files changed:
- `OCP-OCL-v0.2.md`
- `src/pilot.rs`
- `tests/ocl_exec.rs`
- `tests/fixtures/fail_exec_v2_condition_deferred.ocl`
- `reports/release-notes-v0.2.0.md`
- `OCL-MVP-PLAN-v0.2.md`
- Commands run:
- `cargo test`
- `cargo test --test ocl_parser`
- `cargo test --test ocl_typecheck`
- `cargo test --test ocl_exec`
- `cargo test --test ocl_toy_programs`
- `cargo test --test ocl_pilot`
- `cargo test --bin soak_compare`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt -- --check`
- `cargo run --bin soak_runner -- --suite game-default --ticks 1000 --seeds 99,101 --mode both --out artifacts/soak --build-id local --git-commit dev`
- `cargo run --bin soak_compare -- --inputs artifacts/soak --format table --gate soft`
- Test results:
- Full validation matrix pass.
- Smoke soak runner: `summaries=20`, `unstable=0`.
- Soft gate compare: không có hard failure, warnings: none.
- Notes/risks:
- Legacy `E-*` vẫn được giữ như alias migration trong code path cũ; baseline test/snapshot mới đã khóa canonical namespace.

### 2026-02-21 - DOCS planning freeze (OCL user guide v0.2)
- Date:
- 2026-02-21
- Gate/Step:
- Post-release docs / user onboarding
- Why:
- Bổ sung tài liệu hướng dẫn dùng OCL cho người viết script sản phẩm thử nghiệm; hiện repo có spec/plan nhưng chưa có user guide gom một chỗ.
- Scope:
- Tạo file hướng dẫn người dùng mới cho OCL v0.2 core (P0), bao gồm cú pháp, ký tự/token, quy tắc type/runtime, ví dụ, lỗi thường gặp.
- Không mở syntax mới, không đổi semantics runtime.
- Expected tests:
- docs-only, không yêu cầu chạy test code.
- Exit criteria:
- Có tài liệu user guide riêng trong `docs/`, nội dung bám implementation hiện tại.

### 2026-02-21 - DOCS implementation (OCL user guide v0.2)
- Date:
- 2026-02-21
- Gate/Step:
- Post-release docs / user onboarding
- Implemented:
- Tạo tài liệu `docs/OCL-USER-GUIDE.md` cho người dùng mới:
- quickstart API chạy script
- keyword/tokens/ký tự hợp lệ
- grammar P0 implementation-aligned
- quy tắc cho `observe/commit/match/field-access/condition/entangle`
- context string contract + strict/lenient validation
- taxonomy mã lỗi
- ví dụ script + lỗi thường gặp
- files changed:
- `docs/OCL-USER-GUIDE.md`
- `OCL-MVP-PLAN-v0.2.md`
- Commands run:
- `rg -n "OCL|grammar|syntax|guide|ctx\\(|condition|entangle" -S .`
- `Get-Content src/ocp_ocl/mod.rs`
- `Get-Content src/ocp_ocl/parser.rs`
- `Get-Content src/ocp_ocl/checker.rs`
- `Get-Content src/ocp_ocl/exec.rs`
- `Get-Content src/ocp_ocl/bridge.rs`
- Test results:
- docs-only change, không chạy lại test suite.
- Notes/risks:
- Guide khóa theo release `v0.2.0` (P0 Core). Các mục deferred `v0.2.1` (`ctx(field=...)`, `horizon`) đã ghi rõ là ngoài scope.

- Date:
- 2026-02-21
- Gate/Step:
- Post-v0.2 planning handoff
- Implemented:
- Files changed:
- `OCL-MVP-PLAN-v0.2.md`
- Commands run:
- docs-only update
- Test results:
- không chạy test (không đổi code runtime)
- Notes/risks:



