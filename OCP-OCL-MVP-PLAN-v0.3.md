# OCL MVP PLAN v0.3

Ngày tạo: 2026-02-24  
Mục tiêu: đưa OCL thành ngôn ngữ lập trình đa dụng độc lập, ứng dụng viết và chạy trực tiếp bằng OCL, không phụ thuộc host code trong workspace ứng dụng.

## Quy ước cập nhật bắt buộc (áp dụng từ 2026-02-24)
- Mọi thay đổi kế hoạch phải cập nhật file này trước khi triển khai.
- Mọi triển khai xong phải cập nhật file này ngay sau khi chạy test.
- Mỗi entry bắt buộc có: ngày, gate/step, mục tiêu, phạm vi, files changed, lệnh test, kết quả, notes/risks.
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
- Gate M0-A (Boundary cứng + Toolchain skeleton OCL-only): `DONE` (đóng ngày 2026-02-24)
- Gate M1 (Language MVP): `DONE` (đóng ngày 2026-02-25; M1-A/B/C pass trên blocking + quarantine lanes; call-depth fail-honest test đã bật chính thức, không còn waiver)
- Gate M2 (Stdlib/Capabilities MVP): `DONE` (closed 2026-02-25; contract-clean M2 lane pass)
- Gate M3 (Packaging): `DONE` (closed 2026-02-25; reproducible source-bundle + exact-pin lock + workspace registry pass)
- Gate M4 (Genome Registry + Composer MVP): `DONE` (closed 2026-02-25; morphogenesis closure auto-fill + lock/verify fail-hard + CLI compose/verify integrated)
- Gate M5 (Conformance suite + demo apps): `DONE` (closed 2026-02-25; 5 demo apps + conformance + offline bundle verification pass)

## 1) Re-scope v0.3: Standalone Platform MVP

### 1.1. Tuyên bố mục tiêu v0.3

v0.3 MUST deliver:

1. OCL-only workspace: dự án ứng dụng chỉ chứa `.ocl` + manifest/lockfile; không cần viết host code để chạy.
2. Toolchain chuẩn: `ocl fmt/check/test/run/build` + diagnostics có cấu trúc để AI sửa theo lỗi.
3. Language đủ để viết chương trình thực: module/package, `fn`, structs/enums, collections cơ bản, `Result/error`, bounded loops.
4. Runtime model cho chương trình dài hạn: reactor/tick (server/app/game) để chạy vô hạn mà không cần loop vô hạn trong ngôn ngữ.
5. Stdlib/capabilities MVP: FS + HTTP client + JSON + time + log + args + basic hash (tùy chọn), đủ làm tool/web.
6. Genome + Morphogenesis MVP: registry + `ComponentSpec` + composer bounded (generate skeleton hợp lệ), verifier.
7. Axis lock giữ nguyên: non-smart orchestrator, hard stop, 4-kind observe, commit gating, no semantic routing theo payload, no implicit commit, no global solver/propagation.

### 1.2. Khóa 6 bất biến biologic v0.3 (bắt buộc)

Để tránh lệch kiến trúc khi triển khai, v0.3 khóa cứng 6 bất biến sau:

1. Mọi module phải có manifest rõ `requires/provides/ports/phase/budget/policy`.
2. Mọi I/O phải đi qua capability key; không có syscall trực tiếp trong language.
3. Mọi nhánh xử lý world-data phải đi qua 4-kind observe; không được “giả OK”.
4. Composer là đường mặc định để tạo skeleton/ghép hệ (`assemble-first` là mặc định khuyến nghị).
5. Verifier là cổng bắt buộc ở CI/pre-commit để chặn forbidden patterns.
6. Runtime chuẩn cho chương trình dài hạn là reactor/tick; tránh `while true` trong language, vẫn chạy mãi nhưng bounded theo tick.

Nếu 6 bất biến này được giữ, OCL vẫn là “mã gen sinh học” dù bề mặt người dùng là ngôn ngữ đa dụng độc lập.

### 1.3. Non-goals v0.3 (defer sang v0.4)

1. Concurrency model đầy đủ (`async/await`, actor runtime chuẩn).
2. Debugger/profiler hoàn chỉnh.
3. FFI mở rộng/ABI stable cho plugin third-party phức tạp.
4. Stdlib toàn năng kiểu C++/Rust (DB drivers, TLS stack đầy đủ, GUI toolkit full...).
5. Optimizer/JIT/bytecode VM phức tạp.

## 2) Kiến trúc sản phẩm v0.3 (ép dùng OCL)

### 2.1. Tách 2 repo / 2 bề mặt

1. `ocl-runtime` (engine): Rust/C++ triển khai parser/checker/executor, adapters, audit. Repo này không phải bề mặt phát triển ứng dụng.
2. `app-ocl` (workspace): chỉ `.ocl`, `Ocl.toml`, `deps.lock`, `tests/`.
3. Nguyên tắc ép đường ray: workspace không chứa `.rs/.cpp`, tránh bẻ lái sang host code.

### 2.2. OCL runtime là binary/SDK opaque

1. Người dùng cài `ocl` CLI (binary).
2. Adapters/stdlib đi kèm runtime dưới dạng package đã ký/đóng gói.

## 3) Toolchain v0.3 (xương sống)

### 3.1. CLI bắt buộc

`ocl` tối thiểu phải có:

1. `ocl init` tạo project.
2. `ocl fmt` format chuẩn (idempotent).
3. `ocl check` parse + typecheck + contract check + forbidden patterns.
4. `ocl test` chạy test `.ocl`.
5. `ocl run <target>` chạy (dev mode).
6. `ocl build` build artifact (bundle/bytecode/IR).
7. `ocl doc` generate docs (MVP có thể extract manifest + signatures).
8. `ocl lsp` (MVP: diagnostics + go-to definition).

### 3.2. Diagnostics đủ để AI tự sửa

`ocl check` MUST xuất:

1. `file`, `span`, `code (E0xxx)`, `message`, `hint`, `expected/got`.
2. Khuyến nghị: `fixit patch`.

Definition of Done:

1. Chạy được vòng lặp `generate -> ocl check -> sửa theo diagnostic -> check pass`.
2. Không cần đọc runtime source để sửa lỗi ứng dụng.

## 4) Language v0.3 (GPL MVP nhưng bounded)

### 4.1. Core syntax cần có (MVP)

1. `module`, `import/export`, versioning.
2. `fn`, local vars, expressions.
3. `struct`, `enum`, pattern matching.
4. `Result<T,E>`, toán tử `?`.
5. `List<T>`, `Map<K,V>` (có cap).
6. `for` bounded (`0..N`, N phải chứng minh bounded theo policy hoặc literal).
7. `match` trên observe-result vẫn bắt buộc đủ 4 arms với commit gating (kế thừa OCP axis).

### 4.2. Reactor/Tick entrypoint (thay loop vô hạn)

MVP định nghĩa 2 kiểu entrypoint:

1. CLI one-shot: `fn main(args) -> ExitCode`.
2. Reactor: `fn on_event(ctx, event) -> StateUpdate`.

Runtime gọi `on_event` theo vòng lặp request/event/frame; mỗi call bị budget caps để bounded mà vẫn chạy dài hạn.

### 4.3. Pure compute vs world I/O

1. Pure compute (structs/collections/algorithms) chạy trong executor, chịu compute budget cap.
2. World I/O đi qua capability keys (FS/HTTP/Time/Process...) theo observe/commit.

## 5) Stdlib + Capabilities v0.3 (đủ dùng đa dụng)

### 5.1. OCL stdlib modules (public surface)

1. `std.args` (parse args).
2. `std.fs` (read/write/list/stat).
3. `std.http` (client MVP).
4. `std.json` (parse/emit).
5. `std.time` (now, sleep qua timer event, deadlines).
6. `std.log`.
7. `std.rand/hash` (tối thiểu).

### 5.2. Native adapters (runtime nội bộ)

1. FS adapter.
2. HTTP adapter.
3. Timer adapter.
4. Crypto hash adapter (tùy chọn).

Axis lock:

1. Adapter phải trả `ObservationResult` đúng 4-kind.
2. Adapter phải obey caps.
3. Enumeration phải deterministic ordering.

## 6) Packaging/Manifest/Lock

### 6.1. `Ocl.toml` (manifest)

MVP cần:

1. `package name/version`.
2. `targets` (`cli/reactor`).
3. `dependencies` (stdlib + third-party packages).
4. `required adapters/capabilities`.
5. `default policy profile` (caps).
6. `module exports`.

### 6.2. `deps.lock`

1. pinned versions + hash.
2. registry source.
3. adapter compatibility snapshot.

Definition of Done:

1. `ocl build` reproducible theo lockfile.

## 7) Genome + Morphogenesis v0.3 (MVP bounded)

### 7.1. Genome Registry MVP

1. `ComponentSpec` schema chuẩn.
2. index theo `provides/requires`.
3. version + schema compatibility checks.

### 7.2. Composer MVP (bounded deterministic)

Input: `PhenotypeSpec (goal + constraints)`  
Output: `DAG + emitted .ocl skeleton + AssemblyProof`

Giới hạn MVP:

1. dựng pipeline hợp lệ + đúng phase order.
2. không tối ưu toàn cục.
3. fail-honest nếu không assemble được.

### 7.3. Verifier

1. static: type + contract + forbidden patterns.
2. runtime: audit hooks + cap enforcement.

## 8) Conformance Test Suite (điều kiện gọi là đa dụng độc lập)

### 8.1. Tests ngôn ngữ

1. parser/formatter roundtrip.
2. type errors (golden diagnostics).
3. bounded loop proofs (pass/fail).
4. `Result/?` behavior.
5. module import/version resolution.

### 8.2. Tests OCP invariants

1. `match` 4 arms bắt buộc.
2. `commit` bị chặn trong `INSUFFICIENT/DEFERRED`.
3. caps vượt -> `INSUFFICIENT + reason code`.
4. deterministic ordering enumeration.

### 8.3. Integration apps (MVP showcase)

Mỗi app là workspace chỉ chứa `.ocl`:

1. `hello-cli` (`args + fs`).
2. `web-fetch` (`http client + json parse`).
3. `mini-server` (`reactor: request -> response`).
4. `scheduler` (timer events).
5. `composer-demo` (`phenotype -> generate skeleton -> run`).

## 9) Milestones v0.3 (workstreams)

### M0 — Workspace & CLI skeleton

1. `ocl init/fmt/check/run/test/build` chạy được với 1 file `.ocl`.
2. diagnostic JSON schema ổn định.

### M1 — Language MVP

1. `module/import/export` + `fn` + `struct/enum` + `Result/?` + collections + bounded `for`.
2. reactor entrypoint.

### M2 — Stdlib/Capabilities MVP

1. `std.args/std.fs/std.http/std.json/std.time/std.log`.
2. adapters tương ứng, obey 4-kind + caps.

### M3 — Packaging

1. `Ocl.toml` + `deps.lock` + registry fetch (local registry đủ cho MVP).
2. build reproducible.

### M4 — Genome Registry + Composer MVP

1. `ComponentSpec` loader/index.
2. composer bounded + `AssemblyProof`.
3. verifier (static + runtime hooks).

### M5 — Conformance suite + 5 demo apps

1. pass toàn bộ tests.
2. demo projects chạy end-to-end mà không có host code.

## 10) Những thứ defer sang v0.4 (để v0.3 ship đúng trọng tâm)

1. Concurrency model chuẩn (`async/await` hoặc actor), IO multiplexing.
2. Debugger/profiler, trace viewer chuẩn hóa.
3. Bytecode/IR + optimizer pipeline (build nhanh, chạy nhanh).
4. FFI/plugin ABI (third-party native extensions).
5. Stdlib mở rộng (TLS, DB, GUI, game engine bindings...).
6. Composer nâng cấp: cost model, multi-objective assembly, caching plans, incremental re-assembly.

---

## Release gate v0.3 (đề xuất khóa)

v0.3 chỉ được `DONE` khi đồng thời đạt:

1. 100% app showcase chạy trong OCL-only workspace.
2. Conformance OCP invariants pass, không bypass observe/commit.
3. Toolchain CLI pass trên CI tối thiểu: `fmt/check/test/build`.
4. Build reproducible với `deps.lock`.
5. Không vi phạm axis lock (no semantic routing, no implicit commit, no global solver).

---

## Nhật ký triển khai

### 2026-02-24 - M0-A implementation closeout (Boundary cứng + Toolchain skeleton)

- Phạm vi:
  - Tách track OCL thành workspace crates độc lập.
  - Dựng `ocl-sdk` facade và `ocl-cli` skeleton (`init/check/run/fmt`).
  - Tạo `app-ocl` canary workspace OCL-only.
- Files/dirs đã thêm:
  - `crates/ocl-runtime-core/*`
  - `crates/ocl-sdk/*`
  - `crates/ocl-cli/*`
  - `app-ocl/*`
  - `tools/ci_ocl_lane.ps1`
- File đã cập nhật:
  - `Cargo.toml` (workspace members cho track OCL)
  - `OCL-MVP-PLAN-v0.3.md` (log này)
- Test/verify commands:
  - `cargo test -p ocl-runtime-core`
  - `cargo test -p ocl-sdk`
  - `cargo test -p ocl-cli`
  - `cargo check -p ocl-runtime-core -p ocl-sdk -p ocl-cli`
  - `cargo run -p ocl-cli -- init app-ocl/demo_m0a`
  - `cargo run -p ocl-cli -- check app-ocl/demo_m0a --json`
  - `cargo run -p ocl-cli -- run app-ocl/demo_m0a`
  - `cargo run -p ocl-cli -- fmt app-ocl/demo_m0a --check`
  - `cargo fmt -- --check`
  - `cargo clippy -p ocl-runtime-core -p ocl-sdk -p ocl-cli --all-targets -- -D warnings`
- Kết quả:
  - OCL lane pass.
  - `ocl` CLI chạy được skeleton commands.
  - Canary `app-ocl` check/run/fmt pass.

### 2026-02-24 - M0-A boundary hardening update
- Date:
  - 2026-02-24
- Gate/Step:
  - M0-A boundary hardening
- Implemented:
  - Khóa rule source-of-truth: OCL v0.3 nằm ở `projects/ocp-ocl/`; `src/ocp_ocl/` chỉ còn legacy bridge.
- Files changed:
  - `docs/PROJECT-BOUNDARY.md`
  - `tools/guard_project_boundaries.ps1`
  - `tools/ci_ocl_lane.ps1`
- Commands run:
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_lane.ps1`
- Test results:
  - OCL lane pass; boundary guard có khả năng bắt path ngoài track.
- Notes/risks:
  - Cần duy trì whitelist artifacts test để tránh nhiễu cảnh báo boundary.

### 2026-02-24 - M0-A repo structure hard split update
- Date:
  - 2026-02-24
- Gate/Step:
  - M0-A repo hard split
- Implemented:
  - Di chuyển OCP-OCL assets vào `projects/ocp-ocl/`.
  - Cập nhật root workspace trỏ OCL crates về `projects/ocp-ocl/crates/*`.
  - Cập nhật tests OCL dùng fixtures local dưới `projects/ocp-ocl/tests/fixtures`.
- Files changed:
  - `projects/ocp-ocl/crates/ocl-runtime-core/*`
  - `projects/ocp-ocl/crates/ocl-sdk/*`
  - `projects/ocp-ocl/crates/ocl-cli/*`
  - `projects/ocp-ocl/app-ocl/*`
  - `projects/ocp-ocl/docs/*`
  - `projects/ocp-ocl/OCL-MVP-PLAN-v0.1..v0.3.md`
  - `projects/ocp-ocl/OCP-OCL-v0.1..v0.2.md`
  - `projects/ocp-ocl/OCL-V0_1-GRAMMAR.md`
  - `Cargo.toml`
- Commands run:
  - `cargo test -p ocl-runtime-core -p ocl-sdk -p ocl-cli`
- Test results:
  - Crates OCL split pass.
- Notes/risks:
  - Cần giữ rõ phân tách artifacts theo track để không phát sinh cross-track noise.

### 2026-02-24 - M0-A verification sync (v0.1/v0.2 nối với v0.3)
- Date:
  - 2026-02-24
- Gate/Step:
  - M0-A verification sync
- Implemented:
  - Chạy lại full legacy OCL suites (`ocl_*`) để xác nhận không đứt semantic v0.1/v0.2 sau khi tách v0.3.
  - Chạy lại smoke toolchain v0.3 (`init/check/run/fmt`) trên project OCL-only.
  - Chạy lane OCL-only (`tools/ci_ocl_lane.ps1`) để xác nhận boundary + crates + canary.
- Files changed:
  - `projects/ocp-ocl/OCL-MVP-PLAN-v0.3.md`
- Commands run:
  - `cargo test --test ocl_ctx_validation --test ocl_diag_taxonomy --test ocl_exec --test ocl_parser --test ocl_pilot --test ocl_soak --test ocl_toy_programs --test ocl_typecheck`
  - `cargo run -p ocl-cli -- init artifacts/ocl_m0a_smoke_run1`
  - `cargo run -p ocl-cli -- check artifacts/ocl_m0a_smoke_run1 --json`
  - `cargo run -p ocl-cli -- run artifacts/ocl_m0a_smoke_run1`
  - `cargo run -p ocl-cli -- fmt artifacts/ocl_m0a_smoke_run1 --check`
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_lane.ps1`
- Test results:
  - Full legacy suite `ocl_*`: PASS.
  - OCL toolchain smoke `init/check/run/fmt`: PASS.
  - OCL lane: PASS; có cảnh báo boundary guard do thư mục smoke tạm `artifacts/ocl_m0a_smoke_run1` chưa nằm whitelist.
- Notes/risks:
  - Trạng thái `M0-A` giữ `DONE`.
  - Cần dọn/whitelist chuẩn artifacts test tạm để log lane sạch hoàn toàn.

### 2026-02-24 - M1-0 preflight anti-fragmentation (start)
- Date:
  - 2026-02-24
- Gate/Step:
  - M1-0 preflight anti-fragmentation
- Implemented:
  - Set `Gate M1 = IN PROGRESS`.
  - Bổ sung quarantine lane script cho legacy root `tests/ocl_*` (non-blocking).
  - Cập nhật boundary guard để:
    - cho phép `artifacts/` path test tạm.
    - chặn `tests/ocl_*.rs` trong OCL active gating track.
- Files changed:
  - `projects/ocp-ocl/OCL-MVP-PLAN-v0.3.md`
  - `tools/guard_project_boundaries.ps1`
  - `tools/ci_ocl_quarantine.ps1`
- Commands run:
  - `powershell -ExecutionPolicy Bypass -File tools/guard_project_boundaries.ps1 -Track ocl -AgainstRef HEAD`
- Test results:
  - Pending M1 runtime/parser implementation; M1 gate chưa close.
- Notes/risks:
  - Quarantine lane là non-blocking by policy; blocking lane vẫn là `tools/ci_ocl_lane.ps1`.

### 2026-02-25 - M1 implementation pass (M1-A/M1-B/M1-C)
- Date:
  - 2026-02-25
- Gate/Step:
  - M1-A module/function core
  - M1-B struct/enum + Result/?
  - M1-C bounded for + reactor/tick CLI path
- Implemented:
  - Runtime core:
    - Mở rộng AST/parser/checker/exec cho `module/import/const/fn/return/for/call/try/list/map/struct/enum`.
    - Thêm xử lý function call runtime, list/map operation (`len`, `push`, `get`), struct field access.
    - Giữ tương thích suite v0.1/v0.2 (`compat_v1_v2` vẫn pass).
  - SDK:
    - Khôi phục và mở rộng `ocl-sdk` với helper project-first:
      - `parse_project`
      - `type_check_project`
      - `run_project`
      - `run_reactor_ticks`
    - Khóa import canonicalization ở mức project resolver:
      - root = `<project>/src`
      - `import a.b;` -> `src/a/b.ocl`
      - cấm path traversal (`..`, absolute)
      - detect cycle (`T-MODULE-IMPORT-CYCLE`)
  - CLI:
    - `ocl check` chạy project-first qua resolver SDK.
    - `ocl run` thêm mode `--reactor --ticks N`.
    - Thêm `ocl test <project_dir>`.
    - `ocl init` cập nhật template có `src/lib.ocl` + `on_event(...) -> ReactorCtl`.
  - Lane scripts:
    - `tools/ci_ocl_lane.ps1` thêm canary `ocl test` + `ocl run --reactor --ticks 32`.
    - Chặn silent pass: thêm kiểm tra `$LASTEXITCODE` sau từng lệnh external.
    - `tools/ci_ocl_quarantine.ps1` cũng fail-fast theo exit code.
- Files changed:
  - `projects/ocp-ocl/crates/ocl-runtime-core/src/ast.rs`
  - `projects/ocp-ocl/crates/ocl-runtime-core/src/lexer.rs`
  - `projects/ocp-ocl/crates/ocl-runtime-core/src/parser.rs`
  - `projects/ocp-ocl/crates/ocl-runtime-core/src/checker.rs`
  - `projects/ocp-ocl/crates/ocl-runtime-core/src/exec.rs`
  - `projects/ocp-ocl/crates/ocl-runtime-core/src/types.rs`
  - `projects/ocp-ocl/crates/ocl-runtime-core/src/value.rs`
  - `projects/ocp-ocl/crates/ocl-runtime-core/src/diag.rs`
  - `projects/ocp-ocl/crates/ocl-runtime-core/tests/m1_language.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/tests/m1_project.rs`
  - `projects/ocp-ocl/crates/ocl-cli/src/lib.rs`
  - `tools/ci_ocl_lane.ps1`
  - `tools/ci_ocl_quarantine.ps1`
  - `projects/ocp-ocl/OCL-MVP-PLAN-v0.3.md`
- Commands run:
  - `cargo check -p ocl-runtime-core -p ocl-sdk -p ocl-cli`
  - `cargo test -p ocl-runtime-core -p ocl-sdk -p ocl-cli`
  - `cargo fmt -- --check`
  - `cargo clippy -p ocl-runtime-core -p ocl-sdk -p ocl-cli --all-targets -- -D warnings`
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_lane.ps1`
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_quarantine.ps1`
- Test results:
  - Blocking lane pass.
  - Quarantine lane pass.
  - M1 test additions:
    - `m1a_import_resolve_ok`: PASS
    - `m1a_import_cycle_fail`: PASS
    - `m1a_import_path_traversal_forbidden`: PASS
    - `m1a_fn_return_type_enforced`: PASS
    - `m1b_struct_ctor_and_field_access_ok`: PASS
    - `m1b_result_try_operator_ok`: PASS
    - `m1b_try_operator_context_mismatch_fail`: PASS
    - `m1c_for_range_literal_bound_ok`: PASS
    - `m1c_for_range_unproven_bound_fail`: PASS
    - `m1c_list_foreach_read_only_ok`: PASS
- Notes/risks:
  - `m1a_call_depth_cap_fail_honest` đã chạy chính thức (không còn `ignored`) và pass trong lane blocking.
  - Reactor path M1 hiện là minimal tick-mode trong CLI/SDK để giữ bounded + deterministic lane; optimization/runtime semantics sâu hơn để M2/M3 nếu cần.

### 2026-02-25 - Pre-M2 anti-drift locks + alignment audit
- Date:
  - 2026-02-25
- Gate/Step:
  - Pre-M2 lock bổ sung
- Decision locks added:
  - `No naked IO`: mọi I/O phải đi qua capability key + cert, không có direct syscall path.
  - `4-kind everywhere`: mọi world-data đi qua 4-kind; match 4 arms + commit gating enforced.
  - `Budgets in DNA`: module/package phải mang budget profile + policy constraints và được checker/verifier/runtime enforce.
- Anti-fragmentation checklist added:
  - `app-ocl` canary là đường kiểm tra bắt buộc trong lane blocking.
  - Root `tests/ocl_*` giữ ở quarantine lane non-blocking.
  - Giữ `Level-0 bounded proof` (không mở proof engine sớm).
  - Không bypass OCP biologic invariants để pass tạm.
- Alignment audit with current code:
  - A. Boundary đóng màng: PASS.
    - `cargo tree -p rgok_feasibility --edges normal` không có dependency OCL internals.
    - `ocl-cli` chỉ phụ thuộc `ocl-sdk` (`projects/ocp-ocl/crates/ocl-cli/Cargo.toml`).
  - B. Extract tối thiểu trước refactor sâu: PASS.
    - Legacy compat suite v0.1/v0.2 vẫn pass (`compat_v1_v2`).
  - C. CLI skeleton + canary: PASS.
    - `ocl init/check/run/test/fmt` chạy được trên `projects/ocp-ocl/app-ocl`.
- Commands run:
  - `cargo check --workspace`
  - `cargo test -p ocl-runtime-core -p ocl-sdk -p ocl-cli`
  - `cargo run -p ocl-cli -- check projects/ocp-ocl/app-ocl --json`
  - `cargo run -p ocl-cli -- run projects/ocp-ocl/app-ocl`
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_lane.ps1`
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_quarantine.ps1`
- Test results:
  - PASS.
- Notes/risks:
  - `m1a_call_depth_cap_fail_honest` đã bỏ `ignored` và pass; waiver call-depth được đóng.

### 2026-02-25 - Budgets in DNA enforcement (manifest-level) closure
- Date:
  - 2026-02-25
- Gate/Step:
  - Pre-M2 closure: `Budgets in DNA`
- Implemented:
  - Bổ sung manifest parser cứng cho `Ocl.toml` tại SDK:
    - bắt buộc `[policy].budget_profile`
    - bắt buộc section `[budget_profiles.<name>]` cho profile đã chọn
    - bắt buộc các khóa: `observe_default_ns`, `observe_max_ns`, `loop_max_steps`, `reactor_max_ticks`, `call_depth_max`
  - Thêm verifier manifest-level trong `type_check_project(...)`:
    - enforce `observe(...)` budget không vượt `observe_max_ns`
    - enforce bounded `for range` theo `loop_max_steps` (literal/const-known)
    - enforce projected function call depth theo `call_depth_max`
  - Thêm runtime gate cho reactor:
    - `run_reactor_ticks(...)` fail-honest khi `ticks > reactor_max_ticks`
  - Cập nhật `ocl init` template + canary `app-ocl/Ocl.toml` để luôn sinh/project hợp lệ policy-budget.
  - Cập nhật `ocl check` project-path đi qua `type_check_project(...)` để không bypass verifier.
- Files changed:
  - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/tests/m1_project.rs`
  - `projects/ocp-ocl/crates/ocl-cli/src/lib.rs`
  - `projects/ocp-ocl/app-ocl/Ocl.toml`
- Commands run:
  - `cargo test -p ocl-sdk --test m1_project`
  - `cargo test -p ocl-runtime-core -p ocl-sdk -p ocl-cli`
  - `cargo clippy -p ocl-runtime-core -p ocl-sdk -p ocl-cli --all-targets -- -D warnings`
  - `cargo fmt -- --check`
  - `cargo run -p ocl-cli -- check projects/ocp-ocl/app-ocl --json`
  - `cargo run -p ocl-cli -- run projects/ocp-ocl/app-ocl`
- Test results:
  - PASS.
  - Test mới:
    - `m1_budget_profile_manifest_required`: PASS
    - `m1_budget_profile_observe_budget_enforced`: PASS
    - `m1_budget_profile_reactor_ticks_enforced`: PASS
- Notes/risks:
  - Enforcement đang ở manifest-level (SDK checker/verifier + reactor entry gate), chưa mở expression-bound solver (đúng lock M1).

## Khóa Pre-M2 (đóng cứng)

Trước khi mở Gate M2, bắt buộc giữ các tiêu chí sau:

1. No naked IO
   - Mọi I/O phải đi qua capability key + cert.
   - Cấm direct host I/O path trong language.
2. 4-kind everywhere
   - Mọi world-data phải đi qua 4-kind observe.
   - Mọi nhánh xử lý world-data phải có 4-arm handling tương thích commit gating.
3. Budgets in DNA
   - Module/package phải có budget_profile + policy constraints.
   - Checker/verifier/runtime phải enforce cùng một bộ budget/policy.

Checklist anti-drift trước M2:

3. `projects/ocp-ocl/app-ocl` chạy pass đầy đủ `check/run/test/fmt` trong lane blocking.
4. Root `tests/ocl_*` chỉ chạy ở quarantine non-blocking.
5. Giữ `Level-0 bounded proof`; không mở expression-bound solver trước gate phù hợp.
6. Không bypass OCP biologic invariants để pass tạm.

Lệnh chốt bắt buộc trước M2:

1. `cargo check --workspace`
2. `cargo test -p ocl-runtime-core -p ocl-sdk -p ocl-cli`
3. `cargo run -p ocl-cli -- check projects/ocp-ocl/app-ocl --json`
4. `cargo run -p ocl-cli -- run projects/ocp-ocl/app-ocl`

### 2026-02-25 - M2 implementation closeout (contract-clean, anti-drift)
- Date:
  - 2026-02-25
- Gate/Step:
  - M2-A/B/C/D/E/F
- Implemented:
  - Key contract cleanup:
    - Added stable std capability keys (no param-in-key for std.*).
    - Added checker guard rejecting `std.*` keys containing `:`.
  - Args channel hardening:
    - Added `ArgsToken { schema_id, canonical, hash64 }` into `Req`.
    - Added canonical KVP encode/decode + stable hash helpers.
    - Runtime now builds `ArgsToken` before adapter observe path.
  - Typed payload path:
    - Added typed payload variants (`StdText`, `StdInt`, `StdBool`, `StdListText`, `StdFsStat`, `StdHttpResp`, `StdLogAck`).
    - Added payload field access path `r.payload.<field>`.
    - Kept legacy sugar (`http_status`, `fs_size`, `text_value`) as compatibility path for this gate.
  - std.json pure compute:
    - Added pure-call support in checker+executor:
      - `std.json.validate`
      - `std.json.get_string`
      - `std.json.get_int`
      - `std.json.emit_flat`
    - No observe/commit needed for std.json path.
  - Verifier + manifest hardening:
    - `Ocl.toml` parser now validates `[capabilities]` allowlist.
    - `type_check_project(...)` enforces used key-family must be present in capabilities.
  - CLI/runtime adapter behavior:
    - Added `ocl run --arg key=value` (repeatable) and runtime args map.
    - Added deterministic std adapters in local bridge (`std.args`, `std.fs`, `std.http`, `std.time`, `std.log`).
    - Enforced commit-gated effects for side effects (`std.fs.write`, sleep/log ack path) with cert binding + idempotent commit.
- Files changed:
  - `projects/ocp-ocl/crates/ocl-runtime-core/src/lib.rs`
  - `projects/ocp-ocl/crates/ocl-runtime-core/src/bridge.rs`
  - `projects/ocp-ocl/crates/ocl-runtime-core/src/types.rs`
  - `projects/ocp-ocl/crates/ocl-runtime-core/src/checker.rs`
  - `projects/ocp-ocl/crates/ocl-runtime-core/src/exec.rs`
  - `projects/ocp-ocl/crates/ocl-runtime-core/tests/m2_stdlib.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/tests/m1_project.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/tests/m2_project.rs`
  - `projects/ocp-ocl/crates/ocl-cli/src/lib.rs`
  - `projects/ocp-ocl/app-ocl/Ocl.toml`
  - `projects/ocp-ocl/OCL-MVP-PLAN-v0.3.md`
- Commands run:
  - `cargo check --workspace`
  - `cargo test -p ocl-runtime-core`
  - `cargo test -p ocl-sdk`
  - `cargo test -p ocl-cli`
  - `cargo clippy -p ocl-runtime-core -p ocl-sdk -p ocl-cli --all-targets -- -D warnings`
  - `cargo fmt -- --check`
  - `cargo run -p ocl-cli -- check projects/ocp-ocl/app-ocl --json`
  - `cargo run -p ocl-cli -- run projects/ocp-ocl/app-ocl --arg profile=demo`
  - `cargo run -p ocl-cli -- test projects/ocp-ocl/app-ocl`
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_lane.ps1`
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_quarantine.ps1`
- Test results:
  - Blocking lane: PASS.
  - Quarantine lane: PASS.
  - New M2 suites:
    - `m2_stdlib` (runtime-core): PASS
    - `m2_project` (sdk): PASS
    - `m2_cli_run_with_args_pass` (cli): PASS
- Notes/risks:
  - Legacy accessor compatibility still present for 1 gate; removal/deprecation hard-fail can be finalized in M3 if desired.
  - `std.json` path is pure compute and intentionally stays outside capability observe/commit lane.

### 2026-02-25 - M3 implementation closeout (Packaging MVP reproducible + offline-ready)
- Date:
  - 2026-02-25
- Gate/Step:
  - M3-A/B/C/D/E/F/G/H/I/J/K
- Implemented:
  - Hash contract hardening:
    - `deps.lock` hash field canonicalized as 16-char lowercase hex.
    - Compat parser accepts legacy decimal hash input for read path only.
    - Lock writer (`sync_deps_lock_v1`) always emits canonical hex.
  - Deterministic hash/runtime helpers:
    - Locked `FNV-1a 64-bit` implementation and canonical hash helpers.
    - Package hash computed from full content set (`package.toml`, `src/**/*.ocl`, `schemas/**`, `component_specs/**`) rather than manifest-only.
  - Canonical text bytes:
    - Added LF-only canonicalization pipeline (`CRLF/CR -> LF`) for text file extensions.
    - No trim/pretty-print/content rewrite beyond newline normalization.
  - Path/symlink safety:
    - Registry hash walk and vendoring reject symlink entries.
    - Reject traversal (`..`) and absolute paths in package-relative paths.
  - Packaging/build:
    - `ocl build` now vendors full dependency content set into `.oclbundle/vendor/...`.
    - Bundle `manifest.toml` includes resolved `[[deps]]` list (`name/version/source/hash64/vendor_path`).
    - Bundle verification fails fast when vendor completeness mismatches deps manifest.
  - Lock strictness policy:
    - `check/run/test --locked`: warn on extra lock entries.
    - `build --locked`: fail on extra lock entries.
    - `ocl lock sync`: prunes extras and rewrites canonical lock.
  - Workspace registry:
    - Added/updated MVP registry index + `std` package under `projects/ocp-ocl/registry`.
    - Canary `projects/ocp-ocl/app-ocl` moved to dependency-driven lock (`deps.lock.v1`) and synced hash.
- Files changed:
  - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/tests/m3_packaging.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/tests/m1_project.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/tests/m2_project.rs`
  - `projects/ocp-ocl/crates/ocl-cli/src/lib.rs`
  - `projects/ocp-ocl/app-ocl/Ocl.toml`
  - `projects/ocp-ocl/app-ocl/deps.lock`
  - `projects/ocp-ocl/registry/index.toml`
  - `projects/ocp-ocl/registry/packages/std/0.1.0/package.toml`
  - `projects/ocp-ocl/registry/packages/std/0.1.0/src/std.ocl`
  - `tools/ci_ocl_lane.ps1`
  - `projects/ocp-ocl/OCL-MVP-PLAN-v0.3.md`
- Commands run:
  - `cargo check --workspace`
  - `cargo test -p ocl-runtime-core -p ocl-sdk -p ocl-cli`
  - `cargo clippy -p ocl-runtime-core -p ocl-sdk -p ocl-cli --all-targets -- -D warnings`
  - `cargo fmt -- --check`
  - `cargo run -p ocl-cli -- lock sync projects/ocp-ocl/app-ocl`
  - `cargo run -p ocl-cli -- check projects/ocp-ocl/app-ocl --json --locked`
  - `cargo run -p ocl-cli -- run projects/ocp-ocl/app-ocl --locked`
  - `cargo run -p ocl-cli -- test projects/ocp-ocl/app-ocl --locked`
  - `cargo run -p ocl-cli -- build projects/ocp-ocl/app-ocl --locked --json`
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_lane.ps1`
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_quarantine.ps1`
- Test results:
  - Blocking lane: PASS.
  - Quarantine lane: PASS (non-blocking).
  - M3 mandatory suites:
    - `m3_hash64_hex_writer_canonical`: PASS
    - `m3_hash64_parser_accepts_decimal_compat`: PASS
    - `m3_text_canonicalization_lf_only_no_content_rewrite`: PASS
    - `m3_registry_symlink_rejected`: PASS
    - `m3_registry_traversal_path_rejected`: PASS
    - `m3_build_manifest_contains_resolved_deps`: PASS
    - `m3_bundle_missing_vendor_dep_fail_fast`: PASS
    - `m3_package_hash_changes_when_src_changes`: PASS
    - `m3_build_vendors_full_dependency_content`: PASS
    - `m3_build_reproducible_same_inputs_same_bundle_hash`: PASS
    - `m3_build_fails_on_lock_extra_entries`: PASS
    - `m3_lock_sync_prunes_extra_entries`: PASS
- Notes/risks:
  - `FNV-1a 64` is used only for deterministic reproducibility in MVP scope, not security integrity.
  - Decimal hash parsing remains compatibility-read path only; canonical writer always emits hex.

### 2026-02-25 - M3 hygiene closure (no-dang-do state)
- Date:
  - 2026-02-25
- Gate/Step:
  - M3 post-close hygiene hardening
- Implemented:
  - Changed `ocl build` default output root from `dist/` to hidden `.oclbundle/` (project-local, deterministic, less repo noise).
  - Added ignore rules for generated runtime/build artifacts under canary project:
    - `projects/ocp-ocl/app-ocl/.ocl_sandbox/`
    - `projects/ocp-ocl/app-ocl/.oclbundle/`
    - `projects/ocp-ocl/app-ocl/dist/` (legacy path compatibility)
  - Added CLI regression test ensuring default build no longer creates legacy `dist/`.
- Files changed:
  - `.gitignore`
  - `projects/ocp-ocl/crates/ocl-cli/src/lib.rs`
  - `projects/ocp-ocl/OCL-MVP-PLAN-v0.3.md`
- Commands run:
  - `cargo test -p ocl-cli`
  - `cargo run -p ocl-cli -- build projects/ocp-ocl/app-ocl --locked --json`
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_lane.ps1`
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_quarantine.ps1`
- Test results:
  - PASS (blocking lane + quarantine lane).
- Notes/risks:
  - Existing local `dist/` directories (nếu đã tạo từ run cũ) được bỏ theo dõi qua ignore policy; không ảnh hưởng deterministic gate.

### 2026-02-25 - M4 implementation closeout (Genome Registry + Composer MVP)
- Date:
  - 2026-02-25
- Gate/Step:
  - M4-A -> M4-N
- Implemented:
  - Added full M4 composer/registry module in SDK:
    - types: `ComponentSpecV1`, `ComponentRegistryIndexV1`, `PhenotypeSpecV1`, `CatalogLockV1`, `AssemblyProofV1`
    - APIs: `load_component_catalog`, `build_catalog_lock_v1`, `verify_catalog_lock_v1`, `compose_phenotype`, `verify_assembly`
  - Locked provider selection semantics:
    - capability edge matching is exact on `key + tier + type_id`
    - `default_versions` is version preference only (not provider whitelist)
    - deterministic tie-break by `phase_rank`, `name`, `version`, `hash64`
    - bounded closure auto-fill with hard caps (`MAX_COMPOSE_*`)
  - Locked catalog/proof integrity:
    - `catalog_hash64` computed from canonical merged entries (`source/name/version/spec_rel/spec_hash/template_rel/template_hash`)
    - stale-proof fail-hard codes:
      - `V-PROOF-PHENOTYPE-STALE`
      - `V-PROOF-CATALOG-STALE`
      - `V-PROOF-GENERATED-STALE`
      - `V-PROOF-HASH-INVALID`
  - Integration contract:
    - composer emits `src/generated/mod.ocl` as stable generated entry
    - generated `.ocl` files are now LF-normalized and parser-safe (no invalid `#` lines)
  - CLI integration:
    - new commands: `ocl compose`, `ocl verify`
    - `ocl check` + `ocl build` call verify fail-hard when phenotype exists
    - `ocl run` keeps non-blocking warning behavior on missing/stale proof (`W-RUN-PROOF` / `W-RUN-LOCKED-PROOF`)
  - CI lane hardening:
    - `tools/ci_ocl_lane.ps1` canary now runs `check/run/test/build` in `--locked` mode
- Files changed:
  - `projects/ocp-ocl/crates/ocl-sdk/src/m4.rs` (new)
  - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/tests/m4_composer.rs` (new)
  - `projects/ocp-ocl/crates/ocl-cli/src/lib.rs`
  - `tools/ci_ocl_lane.ps1`
  - `projects/ocp-ocl/app-ocl/phenotype.toml` (new)
  - `projects/ocp-ocl/app-ocl/catalog.lock` (new)
  - `projects/ocp-ocl/app-ocl/src/main.ocl`
  - `projects/ocp-ocl/registry/components/*` (new registry/components tree)
  - `projects/ocp-ocl/OCL-MVP-PLAN-v0.3.md`
- Commands run:
  - `cargo check --workspace`
  - `cargo test -p ocl-runtime-core -p ocl-sdk -p ocl-cli`
  - `cargo clippy -p ocl-runtime-core -p ocl-sdk -p ocl-cli --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo run -p ocl-cli -- compose app-ocl --phenotype app-ocl/phenotype.toml --locked --json`
  - `cargo run -p ocl-cli -- verify app-ocl --phenotype app-ocl/phenotype.toml --locked --json`
  - `cargo run -p ocl-cli -- check app-ocl --locked --json`
  - `cargo run -p ocl-cli -- run app-ocl --locked`
  - `cargo run -p ocl-cli -- test app-ocl --locked`
  - `cargo run -p ocl-cli -- run app-ocl --reactor --ticks 32 --locked`
  - `cargo run -p ocl-cli -- build app-ocl --locked --json`
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_lane.ps1`
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_quarantine.ps1`
- Test results:
  - Blocking lane: PASS
  - Quarantine lane: PASS
  - New M4 suite (`m4_composer.rs`): 18/18 PASS
  - CLI M4 behavior tests: PASS
- Notes/risks:
  - `ocl run` intentionally does not hard-fail on stale proof (warning-only by M4 lock); fail-hard remains on `verify/check/build`.
  - `app-ocl/src/generated/*` and `assembly_proof.toml` are compose outputs; CI generates/refreshes them deterministically via `ocl compose`.

### 2026-02-25 - M5 implementation closeout (Conformance + 5 demo apps, anti-flake + offline-verified)
- Date:
  - 2026-02-25
- Gate/Step:
  - M5-A -> M5-M
- Implemented:
  - Added full M5 app set under `projects/ocp-ocl/apps/*`:
    - `hello-cli`, `web-fetch`, `mini-server`, `scheduler`, `composer-demo`.
  - Locked anti-flake fixture/data contracts:
    - mini-server input moved to JSONL fixture schema (`requests.jsonl`).
    - mini-server outputs canonical JSONL responses in sandbox.
    - scheduler uses synthetic tick counter only (no wall-clock dependency in scenario logic).
  - Commit-gated proof path for hello-cli:
    - `do_commit=0` => observe path allowed but no file write side effect.
    - `do_commit=1` => commit applies deterministic write exactly once.
  - Added offline bundle regression path:
    - build locked bundle + run from bundle path when project registry is unavailable.
    - run succeeds using bundle vendor content.
  - Updated conformance harness:
    - always writes report at `target/ocl/m5/conformance_report.json` before final assert.
    - any failing scenario returns non-zero test exit.
    - report includes `report_written`, `offline_bundle_run_checked`, `offline_bundle_run_passed`, per-failure stage.
  - Updated CI lane script (`tools/ci_ocl_lane.ps1`) to include all M5 app and conformance scenarios.
- Files changed:
  - `projects/ocp-ocl/apps/hello-cli/*`
  - `projects/ocp-ocl/apps/web-fetch/*`
  - `projects/ocp-ocl/apps/mini-server/*`
  - `projects/ocp-ocl/apps/scheduler/*`
  - `projects/ocp-ocl/apps/composer-demo/*`
  - `projects/ocp-ocl/crates/ocl-cli/src/lib.rs`
  - `projects/ocp-ocl/crates/ocl-cli/tests/m5_conformance.rs`
  - `tools/ci_ocl_lane.ps1`
  - `.gitignore`
  - `projects/ocp-ocl/OCL-MVP-PLAN-v0.3.md`
- Commands run:
  - `cargo check --workspace`
  - `cargo test -p ocl-runtime-core -p ocl-sdk -p ocl-cli`
  - `cargo test -p ocl-cli --test m5_conformance`
  - `cargo clippy -p ocl-runtime-core -p ocl-sdk -p ocl-cli --all-targets -- -D warnings`
  - `cargo fmt -- --check`
  - `cargo run -p ocl-cli -- check projects/ocp-ocl/apps/hello-cli --locked --json`
  - `cargo run -p ocl-cli -- run projects/ocp-ocl/apps/hello-cli --locked --arg name=demo --arg do_commit=0`
  - `cargo run -p ocl-cli -- run projects/ocp-ocl/apps/hello-cli --locked --arg name=demo --arg do_commit=1`
  - `cargo run -p ocl-cli -- run projects/ocp-ocl/apps/mini-server --locked --reactor --ticks 64`
  - `cargo run -p ocl-cli -- run projects/ocp-ocl/apps/scheduler --locked --reactor --ticks 64 --arg period=4 --arg max_ticks=64`
  - `cargo run -p ocl-cli -- run projects/ocp-ocl/apps/web-fetch --locked --arg fixture_key=demo_user`
  - `cargo run -p ocl-cli -- compose projects/ocp-ocl/apps/composer-demo --phenotype projects/ocp-ocl/apps/composer-demo/phenotype.toml --locked --json`
  - `cargo run -p ocl-cli -- verify projects/ocp-ocl/apps/composer-demo --phenotype projects/ocp-ocl/apps/composer-demo/phenotype.toml --locked --json`
  - `cargo run -p ocl-cli -- build projects/ocp-ocl/apps/hello-cli --locked --json`
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_lane.ps1`
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_quarantine.ps1`
- Test results:
  - Blocking lane: PASS.
  - Quarantine lane: PASS (non-blocking).
  - `m5_conformance`: 15/15 PASS.
  - Report evidence present:
    - `target/ocl/m5/conformance_report.json` exists.
    - `offline_bundle_run_checked=true`, `offline_bundle_run_passed=true`, `checks_failed=0`.
- Notes/risks:
  - M5 keeps M4 lock: `ocl run` remains warning-mode for stale proof; fail-hard stays in `verify/check/build`.
  - Offline run check is coverage for M3 lock behavior; no new runtime feature was introduced.

### 2026-02-25 - Documentation refresh (User Guide sync to M5)
- Date:
  - 2026-02-25
- Gate/Step:
  - Post-M5 documentation sync
- Implemented:
  - Rewrote `projects/ocp-ocl/docs/OCL-USER-GUIDE.md` to match current toolchain and behavior through M5:
    - CLI surface (`init/check/run/test/fmt/lock sync/build/compose/verify`)
    - locked workflow + bundle offline run
    - typed payload usage (`r.payload.<field>`)
    - std.json pure path
    - 5 demo apps contracts (hello-cli, web-fetch, mini-server, scheduler, composer-demo)
    - updated troubleshooting with verify/proof/locked notes
- Files changed:
  - `projects/ocp-ocl/docs/OCL-USER-GUIDE.md`
- Commands run:
  - `rg -n "ocl compose|offline|do_commit|std.json" projects/ocp-ocl/docs/OCL-USER-GUIDE.md`
- Test results:
  - N/A (docs-only update; runtime/CI evidence already captured in M5 closeout above).
- Notes/risks:
  - Guide title/file đã chuẩn hóa không phiên bản để tránh drift naming; nội dung vẫn synchronized tới M5.

### 2026-02-25 - Documentation hardening (self-contained user textbook pass)
- Date:
  - 2026-02-25
- Gate/Step:
  - Post-M5 documentation quality hardening
- Implemented:
  - Reworked `projects/ocp-ocl/docs/OCL-USER-GUIDE.md` into self-contained guide:
    - no dependency on external docs for normal user flow
    - full end-to-end workflow (init/check/run/test/fmt/lock/build/compose/verify)
    - language usage section (observe/match/commit, fn/struct/result/for/reactor)
    - capability/std contracts + locked/offline bundle flow + troubleshooting
    - operational checklist for applying OCL to new projects
- Files changed:
  - `projects/ocp-ocl/docs/OCL-USER-GUIDE.md`
- Commands run:
  - `rg -n "xem|tham khảo|OCL-MVP-PLAN|tools/ci_ocl_lane" projects/ocp-ocl/docs/OCL-USER-GUIDE.md`
- Test results:
  - N/A (docs-only quality pass).
- Notes/risks:
  - Guide intentionally duplicates critical usage details to reduce context-switch and prevent documentation fragmentation.

