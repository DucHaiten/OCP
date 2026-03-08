# OCP v0.3 Master Plan + Execution Log (Standalone OCP Platform)

Ngày tạo: 2026-02-24  
Mục tiêu: đưa OCP thành nền tảng lập trình độc lập cho ứng dụng OCP-only, có toolchain đầy đủ, packaging/lock reproducible, composer/verifier bounded deterministic, và conformance suite có demo end-to-end.

## Tóm tắt nhìn nhanh (đọc trước khi làm)
- Mục tiêu bản này:
  - Khóa baseline OCP-only platform MVP theo các gate M0-A -> M5.
- Trạng thái hiện tại:
  - `DONE` (chi tiết ở mục Trạng thái Gate).
  - Re-verify mới nhất: `2026-03-04` theo matrix v0.3, kết quả `PASS`.
- Luồng làm việc chuẩn:
  - Cập nhật kế hoạch -> code -> chạy test -> cập nhật closeout.
- Bằng chứng kỹ thuật bắt buộc:
  - `Files changed` + `Commands run` + `Test results` trong từng gate.
- Bước kế tiếp:
  - Dùng handoff cuối file để mở v0.4, không bỏ qua gate-order.

## Quy ước cập nhật bắt buộc (áp dụng từ 2026-02-24)
- Mọi thay đổi kế hoạch phải cập nhật file này trước khi code.
- Mọi triển khai xong phải cập nhật file này ngay sau khi chạy test.
- Mỗi entry bắt buộc có: ngày, gate/step, mục tiêu, phạm vi, files changed, lệnh chạy, kết quả, notes/risks.
- Không nhảy gate: chỉ mở gate kế tiếp khi gate hiện tại đạt `DONE`.

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

## Trạng thái Gate v0.3
- Gate M0-A (Boundary cứng + Toolchain skeleton OCP-only): `DONE` (đóng ngày 2026-03-03)
- Gate M1 (Language MVP): `DONE` (đóng ngày 2026-03-03)
- Gate M2 (Stdlib/Capabilities MVP): `DONE` (đóng ngày 2026-03-03)
- Gate M3 (Packaging): `DONE` (đóng ngày 2026-03-03)
- Gate M4 (Genome Registry + Composer MVP): `DONE` (đóng ngày 2026-03-03)
- Gate M5 (Conformance suite + 5 demo apps): `DONE` (đóng ngày 2026-03-03)

---

## Scope khóa cho v0.3
### In-scope bắt buộc
- Workspace OCP-only cho app: `.ocp`, `Ocp.toml`, `deps.lock`, `tests/`.
- Toolchain: `ocp init/fmt/check/run/test/build`, `ocp lock sync`, `ocp compose`, `ocp verify`.
- Language MVP: module/import/export, fn/return/call, for-range bounded, list/map literals, `?` trên Result4.
- Runtime reactor/tick cho luồng dài hạn.
- Stdlib/capability MVP: `std.args`, `std.fs`, `std.http`, `std.json`, `std.time`, `std.log`.
- Packaging/lock reproducible với chế độ `--locked`.
- Composer/verifier bounded deterministic với proof.
- Conformance suite + 5 demo apps end-to-end.

### Out-of-scope (defer v0.4+)
- Concurrency model đầy đủ (`async/await`, actor runtime).
- Optimizer/JIT/bytecode pipeline nâng cao.
- Debugger/profiler hoàn chỉnh.
- FFI/plugin ABI mở rộng.

---

## Release gate v0.3
v0.3 chỉ được `DONE` khi đồng thời đạt:
1. 100% app showcase chạy trong workspace OCP-only.
2. Conformance invariants pass (không bypass observe/commit).
3. CLI lane pass tối thiểu `check/run/fmt/test/build`.
4. Build reproducible theo `deps.lock`.
5. Không vi phạm axis lock.

Trạng thái: `PASS`.

---

## Operational Commands
- `cargo test -p ocp-runtime-core -p ocp-sdk -p ocp-cli`
- `cargo fmt -- --check`
- `cargo clippy -p ocp-runtime-core -p ocp-sdk -p ocp-cli --all-targets -- -D warnings`
- `powershell -ExecutionPolicy Bypass -File tools/ci_ocp_lane.ps1`

---

## Nhật ký triển khai

### 2026-03-03 - M0-A planning freeze (Boundary + Toolchain skeleton)
- Date:
  - 2026-03-03
- Gate/Step:
  - M0-A
- Why:
  - Dựng lại xương sống OCP v0.3 theo hướng app OCP-only với CLI/toolchain riêng.
- Scope:
  - Tách workspace crates: `ocp-runtime-core`, `ocp-sdk`, `ocp-cli`.
  - Dựng canary app OCP-only: `projects/ocp/app-ocp`.
  - Dựng lane script `tools/ci_ocp_lane.ps1`.
- Expected tests:
  - `cargo check --workspace`
  - `cargo test -p ocp-runtime-core`
  - `cargo test -p ocp-sdk`
  - `cargo test -p ocp-cli`
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocp_lane.ps1`
- Exit criteria:
  - `ocp init/check/run/fmt/test/build` chạy được trên canary.

### 2026-03-03 - M0-A implementation closeout
- Date:
  - 2026-03-03
- Gate/Step:
  - M0-A
- Implemented:
  - Dựng xong track crates + canary app + lane script.
  - SDK/CLI có luồng project-first cơ bản.
- Files changed:
  - `Cargo.toml`
  - `projects/ocp/crates/ocp-runtime-core/src/lib.rs`
  - `projects/ocp/crates/ocp-sdk/src/lib.rs`
  - `projects/ocp/crates/ocp-cli/src/main.rs`
  - `projects/ocp/app-ocp/Ocp.toml`
  - `projects/ocp/app-ocp/src/main.ocp`
  - `projects/ocp/app-ocp/tests/smoke.ocp`
  - `tools/ci_ocp_lane.ps1`
- Commands run:
  - `cargo check --workspace`
  - `cargo test -p ocp-runtime-core`
  - `cargo test -p ocp-sdk`
  - `cargo test -p ocp-cli`
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocp_lane.ps1`
- Test results:
  - PASS
- Notes/risks:
  - M0-A dùng backend engine hiện có để mở nhanh bề mặt toolchain.
  - Cần giữ boundary cứng giữa app OCP-only và host/tooling crates để tránh drift quay về host-code app pattern.
  - Lane script là điểm kiểm soát chính; mọi thay đổi CLI sau này phải cập nhật lane cùng lúc.

### 2026-03-03 - M1 planning freeze (Language MVP)
- Date:
  - 2026-03-03
- Gate/Step:
  - M1
- Why:
  - Mở bề mặt ngôn ngữ đủ cho app OCP-only thực dụng.
- Scope:
  - Bổ sung parser/typecheck/exec cho:
    - `module/import/export`
    - `fn/return/call`
    - `for-range` bounded
    - list/map literals
    - `?` trên `Result4`
  - Bổ sung reactor mode `ocp run --reactor --ticks N`.
- Expected tests:
  - `cargo test --workspace`
  - `cargo fmt -- --check`
  - `cargo clippy -p ocp-runtime-core -p ocp-sdk -p ocp-cli --all-targets -- -D warnings`
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocp_lane.ps1`
- Exit criteria:
  - Syntax/semantics M1 pass trên canary + suite.

### 2026-03-03 - M1 implementation closeout
- Date:
  - 2026-03-03
- Gate/Step:
  - M1
- Implemented:
  - Đóng language MVP theo scope M1 + reactor run mode.
- Files changed:
  - `src/ocp/ast.rs`
  - `src/ocp/lex.rs`
  - `src/ocp/parse.rs`
  - `src/ocp/typecheck.rs`
  - `src/ocp/exec.rs`
  - `src/ocp/types.rs`
  - `src/ocp/value.rs`
  - `projects/ocp/crates/ocp-sdk/src/lib.rs`
  - `projects/ocp/crates/ocp-cli/src/main.rs`
- Commands run:
  - `cargo test --workspace`
  - `cargo fmt -- --check`
  - `cargo clippy -p ocp-runtime-core -p ocp-sdk -p ocp-cli --all-targets -- -D warnings`
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocp_lane.ps1`
- Test results:
  - PASS
- Notes/risks:
  - M1 chốt contract language MVP, chưa đi vào optimization.
  - `struct/enum` đang ở mức MVP cho parser/typecheck/runtime flow; các pattern khai thác sâu defer sang v0.4.
  - Reactor mode dựa trên tick bounded; không dùng infinite loop trong DSL.

### 2026-03-03 - M2 planning freeze (Stdlib/Capabilities MVP)
- Date:
  - 2026-03-03
- Gate/Step:
  - M2
- Why:
  - Mở lớp capability `std.*` để app OCP dùng được luồng thực dụng.
- Scope:
  - Mở keyspace `std.*` trong registry.
  - Observe adapters deterministic cho `std.args/fs/http/json/time/log`.
  - Giữ commit gating hiện hành.
- Expected tests:
  - `cargo test --test ocp_stdlib --test ocp_registry --test ocp_exec --test ocp_commit_policy`
  - `cargo test`
- Exit criteria:
  - Observe `std.*` trả 4-kind hợp lệ, deterministic.

### 2026-03-03 - M2 implementation closeout
- Date:
  - 2026-03-03
- Gate/Step:
  - M2
- Implemented:
  - Đóng stdlib/capability MVP theo scope M2.
- Files changed:
  - `src/ocp/registry.rs`
  - `src/ocp/exec.rs`
  - `tests/ocp_stdlib.rs`
- Commands run:
  - `cargo test --test ocp_stdlib --test ocp_registry --test ocp_exec --test ocp_commit_policy`
  - `cargo test`
- Test results:
  - PASS
- Notes/risks:
  - M2 adapter là stub deterministic để khóa semantics.
  - `std.time.sleep` giữ DEFERRED để không tạo blocking side-effect trong runtime loop.
  - `std.http.post` trả DEGRADED để test rõ path degrade + commit discipline.

### 2026-03-03 - M3 planning freeze (Packaging)
- Date:
  - 2026-03-03
- Gate/Step:
  - M3
- Why:
  - Đảm bảo reproducible build qua lockfile và mode `--locked`.
- Scope:
  - Dependencies parser từ `Ocp.toml`.
  - `deps.lock` schema v1 canonical.
  - `sync_deps_lock_v1(...)`.
  - `--locked` cho check/run/test/build.
  - CLI `ocp lock sync`.
- Expected tests:
  - `cargo test -p ocp-runtime-core -p ocp-sdk -p ocp-cli`
  - `cargo fmt -- --check`
  - smoke `lock/check/build` ở mode locked.
- Exit criteria:
  - Lock sync deterministic, locked-mode fail-honest khi mismatch.

### 2026-03-03 - M3 implementation closeout
- Date:
  - 2026-03-03
- Gate/Step:
  - M3
- Implemented:
  - Đóng packaging MVP + lock discipline.
- Files changed:
  - `projects/ocp/crates/ocp-sdk/src/lib.rs`
  - `projects/ocp/crates/ocp-sdk/tests/m3_packaging.rs`
  - `projects/ocp/crates/ocp-cli/src/main.rs`
- Commands run:
  - `cargo test -p ocp-runtime-core -p ocp-sdk -p ocp-cli`
  - `cargo fmt -- --check`
  - `cargo run -p ocp-cli -- lock sync projects/ocp/app-ocp`
  - `cargo run -p ocp-cli -- check projects/ocp/app-ocp --json --locked`
  - `cargo run -p ocp-cli -- build projects/ocp/app-ocp --locked`
- Test results:
  - PASS
- Notes/risks:
  - Hash lock dùng cho reproducibility MVP.
  - `--locked` là cổng an toàn chính cho reproducible lane và build artifact consistency.
  - Build manifest phải chứa đủ thông tin deps để phục vụ trace/replay và audit sau này.

### 2026-03-03 - M4 planning freeze (Genome Registry + Composer MVP)
- Date:
  - 2026-03-03
- Gate/Step:
  - M4
- Why:
  - Mở composer/verifier cho pipeline assemble-first.
- Scope:
  - Thêm model: `ComponentSpecV1`, `PhenotypeSpecV1`, `AssemblyProofV1`.
  - Thêm API compose/verify trong SDK.
  - Thêm CLI: `ocp compose`, `ocp verify`.
- Expected tests:
  - `cargo test -p ocp-runtime-core -p ocp-sdk -p ocp-cli`
  - `cargo fmt -- --check`
  - smoke compose/verify.
- Exit criteria:
  - Generate `src/generated/mod.ocp` + `assembly_proof.toml`.
  - Verify phát hiện stale/tamper proof fail-honest.

### 2026-03-03 - M4 implementation closeout
- Date:
  - 2026-03-03
- Gate/Step:
  - M4
- Implemented:
  - Đóng composer/verifier MVP bounded deterministic.
- Files changed:
  - `projects/ocp/crates/ocp-sdk/src/m4.rs`
  - `projects/ocp/crates/ocp-sdk/src/lib.rs`
  - `projects/ocp/crates/ocp-sdk/tests/m4_composer.rs`
  - `projects/ocp/crates/ocp-cli/src/main.rs`
- Commands run:
  - `cargo test -p ocp-runtime-core -p ocp-sdk -p ocp-cli`
  - `cargo fmt -- --check`
  - `cargo run -p ocp-cli -- compose ...`
  - `cargo run -p ocp-cli -- verify ...`
- Test results:
  - PASS
- Notes/risks:
  - M4 chưa mở planner sâu, chỉ bounded closure theo phenotype.
  - Verify fail-honest nếu stale/tamper proof, không tự “heal”.
  - Generated source được xem là build artifact logic, không là source-of-truth business logic.

### 2026-03-03 - M5 planning freeze (Conformance + 5 demo apps)
- Date:
  - 2026-03-03
- Gate/Step:
  - M5
- Why:
  - Chốt v0.3 bằng bằng chứng end-to-end cho app OCP-only và suite conformance khóa regression.
- Scope:
  - Thêm 5 demo apps:
    - `hello-cli`
    - `web-fetch`
    - `mini-server`
    - `scheduler`
    - `composer-demo`
  - Thêm conformance suite M5.
  - Mở rộng lane script để chạy toàn bộ flow lock/check/run/reactor/test/build + compose/verify.
- Expected tests:
  - `cargo test -p ocp-runtime-core -p ocp-sdk -p ocp-cli`
  - `cargo fmt -- --check`
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocp_lane.ps1`
- Exit criteria:
  - 5 demo apps pass end-to-end ở mode `--locked`.
  - `composer-demo` pass `compose + verify`.
  - Có test guard OCP-only (không host code file).

### 2026-03-03 - M5 implementation closeout
- Date:
  - 2026-03-03
- Gate/Step:
  - M5
- Implemented:
  - Đóng M5 với 5 app + conformance suite + lane full-flow.
- Files changed:
  - `projects/ocp/apps/hello-cli/*`
  - `projects/ocp/apps/web-fetch/*`
  - `projects/ocp/apps/mini-server/*`
  - `projects/ocp/apps/scheduler/*`
  - `projects/ocp/apps/composer-demo/*`
  - `projects/ocp/crates/ocp-sdk/tests/m5_conformance.rs`
  - `tools/ci_ocp_lane.ps1`
- Commands run:
  - `cargo test -p ocp-sdk --test m5_conformance`
  - `cargo test -p ocp-runtime-core -p ocp-sdk -p ocp-cli`
  - `cargo fmt -- --check`
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocp_lane.ps1`
- Test results:
  - PASS
- Notes/risks:
  - Demo app intentionally minimal để khóa contract pipeline/tooling.
  - Mục tiêu M5 là conformance và luồng e2e ổn định, không phải benchmark feature depth.
  - `composer-demo` được xem là app chứng minh assemble-first + verify-first.

### 2026-03-03 - Post-M5 lane hygiene update
- Date:
  - 2026-03-03
- Gate/Step:
  - Post-M5 lane hygiene
- Implemented:
  - Chạy lane trên bản copy tạm `%TEMP%` cho canary và demo apps, tránh làm bẩn source workspace bởi artifacts runtime.
- Files changed:
  - `tools/ci_ocp_lane.ps1`
- Commands run:
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocp_lane.ps1`
- Test results:
  - PASS
- Notes/risks:
  - Không tự động cleanup temp folder để tránh thao tác xóa tự động.
  - Temp copies làm tăng disk usage tạm thời; chấp nhận được để đổi lấy an toàn source workspace.

### 2026-03-04 - V0.6 audit v0.3 consistency re-verify
- Date:
  - 2026-03-04
- Gate/Step:
  - M0-A..M5 re-verify (v0.6 audit)
- Implemented:
  - Rerun đầy đủ ma trận `Operational Commands` của v0.3 trên baseline hiện tại.
  - Xác nhận lại lane e2e cho canary + demo apps + conformance locked flow.
- Files changed:
  - `OCP-MVP-PLAN-v0.3.md`
- Commands run:
  - `cargo test -p ocp-runtime-core -p ocp-sdk -p ocp-cli`
  - `cargo fmt -- --check`
  - `cargo clippy -p ocp-runtime-core -p ocp-sdk -p ocp-cli --all-targets -- -D warnings`
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocp_lane.ps1`
- Test results:
  - PASS:
  - `cargo test -p ocp-runtime-core -p ocp-sdk -p ocp-cli`: PASS toàn bộ suite liên quan.
  - `cargo fmt -- --check`: PASS.
  - `cargo clippy -p ocp-runtime-core -p ocp-sdk -p ocp-cli --all-targets -- -D warnings`: PASS.
  - `tools/ci_ocp_lane.ps1`: PASS toàn bộ lane;
    - conformance report: `scenarios_total=10`, `scenarios_passed=10`, `scenarios_failed=0`,
    - `required_digest=10deff71c24ef729`.
- Notes/risks:
  - Lane tạo workspace tạm trong `%TEMP%` để chạy e2e; không cleanup tự động theo policy an toàn hiện hành.
  - Không phát hiện drift thực thi giữa plan v0.3 và code/toolchain hiện tại.

---

## Phụ lục A - Change Ledger chi tiết theo gate

### A.1 M0-A - Ledger chi tiết
- Runtime core:
  - wrapper check/run/format bridge sang engine nội bộ.
- SDK:
  - project layout model.
  - init/check/run/fmt/test/build APIs.
- CLI:
  - skeleton commands `init/check/run/fmt/test/build`.
- Workspace canary:
  - `app-ocp` bootstrap + smoke test.
- Lane:
  - luồng chuẩn canary từ check -> run -> reactor -> fmt -> test -> build.

### A.2 M1 - Ledger chi tiết
- Parser/AST:
  - thêm statements: module/import/export/fn/return/for-range.
  - thêm expressions: list/map/try postfix.
- Typecheck:
  - function symbol resolution + arity checks.
  - boundedness checks cho for-range.
- Runtime:
  - function invocation path.
  - return propagation qua block/match/for.
  - reactor ticks integration.

### A.3 M2 - Ledger chi tiết
- Registry:
  - enable family `std`.
  - ctx-required rules cho `std.args/fs/http/json/time/log`.
- Runtime observe:
  - map key `std.*` -> 4-kind deterministic results.
- Test coverage:
  - gate-by-gate behavior cho OK/DEGRADED/DEFERRED.
  - commit-forbidden path cho non-committable result.

### A.4 M3 - Ledger chi tiết
- Manifest dependencies:
  - parse section `[dependencies]` và canonical sort.
- Lockfile:
  - schema v1 stable (`version=1`, `dep=name|version|hash64`).
- Locked mode:
  - verify lock consistency trước check/run/test/build.
- Build bundle:
  - manifest ghi cả files + deps đã resolve.

### A.5 M4 - Ledger chi tiết
- Catalog + phenotype:
  - loader for `registry/components/*.toml`.
  - phenotype components parser.
- Compose:
  - deterministic phase order.
  - emits `src/generated/*.ocp` + `src/generated/mod.ocp`.
  - emits `assembly_proof.toml`.
- Verify:
  - so khớp component list + hash catalog/phenotype/generated.
  - fail-honest qua lock mismatch errors.

### A.6 M5 - Ledger chi tiết
- Demo app set:
  - `hello-cli`, `web-fetch`, `mini-server`, `scheduler`, `composer-demo`.
- Conformance tests:
  - OCP-only file guard.
  - e2e locked flows.
  - reactor flows cho app long-running pattern.
  - compose/verify integration cho composer app.
- Lane integration:
  - chạy toàn bộ app matrix trong một lane.
  - lock sync trước mọi run/check/test/build.

---

## Phụ lục B - Evidence matrix (để đọc nhanh mà không cần mở code)

### B.1 Matrix gate -> test evidence
- M0-A:
  - `cargo check --workspace`
  - `cargo test -p ocp-runtime-core`
  - `cargo test -p ocp-sdk`
  - `cargo test -p ocp-cli`
- M1:
  - `cargo test --workspace`
  - `cargo clippy -p ocp-runtime-core -p ocp-sdk -p ocp-cli --all-targets -- -D warnings`
  - lane canary pass
- M2:
  - `cargo test --test ocp_stdlib --test ocp_registry --test ocp_exec --test ocp_commit_policy`
- M3:
  - `cargo run -p ocp-cli -- lock sync ...`
  - `cargo run -p ocp-cli -- check ... --locked`
  - `cargo run -p ocp-cli -- build ... --locked`
- M4:
  - `cargo run -p ocp-cli -- compose ...`
  - `cargo run -p ocp-cli -- verify ...`
- M5:
  - `cargo test -p ocp-sdk --test m5_conformance`
  - full lane with demo matrix pass

### B.2 Matrix gate -> deliverable chính
- M0-A: workspace + SDK/CLI skeleton + canary lane.
- M1: language MVP + reactor run mode.
- M2: std capability contract.
- M3: lock discipline + reproducible bundle metadata.
- M4: compose/verify + assembly proof.
- M5: conformance suite + 5 demo apps end-to-end.

---

## Phụ lục C - Legacy notes (nhập từ chu kỳ triển khai trước)
- Có chu kỳ log cũ (2026-02-24/2026-02-25) với mức granularity rất cao cho M3-M5.
- Trong v0.3 bản hiện tại, các thông tin cốt lõi đã được hợp nhất và chuẩn hóa theo full-log format mới.
- Nếu cần forensic sâu theo từng vòng thử-sửa nhỏ, dùng Git history để truy vết nguyên văn theo timestamp.

---

## Phụ lục D - Demo app matrix chi tiết (M5)

### D.1 hello-cli
- Mục tiêu:
  - kiểm chứng luồng CLI one-shot + std.args.
- Luồng kiểm chứng:
  - `lock sync`
  - `check --locked`
  - `run --locked`
  - `fmt --check`
  - `test --locked`
  - `build --locked`
- Kết quả:
  - PASS full flow.

### D.2 web-fetch
- Mục tiêu:
  - kiểm chứng std.http + std.json trong flow OCP-only.
- Luồng kiểm chứng:
  - `lock sync`
  - `check/run/fmt/test/build` ở mode `--locked`.
- Kết quả:
  - PASS full flow.

### D.3 mini-server
- Mục tiêu:
  - kiểm chứng reactor-tick cho app dạng server.
- Luồng kiểm chứng:
  - `lock sync`
  - `check/run --locked`
  - `run --reactor --ticks N --locked`
  - `fmt/test/build --locked`
- Kết quả:
  - PASS full flow + reactor path.

### D.4 scheduler
- Mục tiêu:
  - kiểm chứng luồng timer/deferred trong pattern scheduler.
- Luồng kiểm chứng:
  - `lock sync`
  - `check/run --locked`
  - `run --reactor --ticks N --locked`
  - `fmt/test/build --locked`
- Kết quả:
  - PASS full flow + reactor path.

### D.5 composer-demo
- Mục tiêu:
  - kiểm chứng assemble-first và verify-first.
- Luồng kiểm chứng:
  - `lock sync`
  - `compose --locked`
  - `verify --locked`
  - `check/run/fmt/test/build --locked`
- Kết quả:
  - PASS full flow + compose/verify integration.

---

## Phụ lục E - Tiêu chí DONE checklist (v0.3)
- [x] M0-A DONE + có evidence test.
- [x] M1 DONE + có evidence test.
- [x] M2 DONE + có evidence test.
- [x] M3 DONE + có evidence test.
- [x] M4 DONE + có evidence test.
- [x] M5 DONE + có evidence test.
- [x] Lane full-flow PASS.
- [x] Handoff v0.4 đã ghi.
- [x] Tài liệu có Quick Snapshot + execution log + phụ lục evidence.

---

## Handoff v0.3 -> v0.4
- v0.3 đã đóng đủ M0-A -> M5 và có lane conformance pass.
- v0.4 có thể tập trung vào mở rộng semantics/runtime chiều sâu mà không phá baseline OCP-only đã khóa.
