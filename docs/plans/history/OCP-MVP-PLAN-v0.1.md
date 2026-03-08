OCP v0.1 Master Plan + Execution Log (Core Language + Runtime Kernel)

Ngày tạo: 2026-02-21
Trạng thái: `DONE` (historical baseline; re-verified 2026-03-03)
Mục tiêu: Kế hoạch + nhật ký triển khai cho **OCP v0.1**. Đây là dự án **ngôn ngữ/runtime OCP thuần**.

---

0) Quick Snapshot (BẮT BUỘC ĐỌC TRƯỚC)

- Mục tiêu phiên bản:
  - Dựng core language + runtime kernel OCP v0.1 theo gate V1-A -> V1-H.
- Trạng thái tổng quan:
  - Xem `2) Gate Plan (v0.1)` để biết gate nào DONE/chưa DONE.
- Trình tự làm việc chuẩn:
  - Planning freeze -> Implement -> Test -> Implementation closeout.
- Bằng chứng kỹ thuật phải có cho từng gate:
  - Files changed
  - Commands run
  - Test results (PASS/FAIL)
- Bước tiếp theo:
  - Chỉ mở gate mới khi gate hiện tại `DONE`.

---

0) Constitution (LOCKED)

0.1 Mục tiêu bắt buộc
1. **Truth-first qua 4-kind**: mọi IO/capability trả về đúng hợp đồng `OK | DEGRADED | INSUFFICIENT | DEFERRED` (không bịa).
2. **Observe-only reads**: mọi read từ thế giới đi qua `observe(...)`.
3. **Commit-gated effects**: mọi effect phải đi qua `commit(...)` và pass policy.
4. **Boundedness**: runtime có step cap; mọi enumerate/budget đều bounded.
5. **Determinism-by-default**: cùng program + inputs + seed/config => cùng trace signature.
6. **Structured diagnostics**: parse/type/exec error đều có `code`, `span`, `message`, `hint`.

0.2 Out-of-scope v0.1
- Module system/package manager hoàn chỉnh.
- Concurrency/async.
- Optimizer/JIT.
- Typed ctx AST đầy đủ (v0.1 dùng `ctx("k=v;...")` string với validation).
- Generic function system (fn/call stack) nếu chưa thật cần.

---

1) Governance

- Mọi thay đổi kế hoạch phải cập nhật file này trước khi code.
- Mọi triển khai xong phải cập nhật log ngay sau khi chạy test.
- Không nhảy gate: gate sau chỉ mở khi gate hiện tại `DONE`.

Template cập nhật kế hoạch (trước khi làm)
- Date:
- Gate/Step:
- Why:
- Scope:
- Expected tests:
- Exit criteria:

Template cập nhật triển khai (sau khi làm)
- Date:
- Gate/Step:
- Implemented:
- Files changed:
- Commands run:
- Test results:
- Notes/risks:

---

2) Gate Plan (v0.1)

- Gate V1-A (Repo Skeleton + Diagnostics + Error Codes): `DONE` (2026-03-03)
- Gate V1-B (Lexer + Parser + AST + Span): `DONE` (2026-03-03)
- Gate V1-C (Types + Typechecker Core): `DONE` (2026-03-03)
- Gate V1-D (Executor Core + Bounded Step + Env): `DONE` (2026-03-03)
- Gate V1-E (Keyspace + Capability Registry + Permission Gate): `DONE` (2026-03-03)
- Gate V1-F (observe/match/commit Semantics + Commit Policies): `DONE` (2026-03-03)
- Gate V1-G (Audit/Trace + Fixture Runner + Determinism Signature): `DONE` (2026-03-03)
- Gate V1-H (Pilot/Replay + Closeout Report): `DONE` (2026-03-03)

---

3) Code Architecture (Rust baseline)

3.1 Source layout (khuyến nghị)
- `src/ocp/span.rs`            (Span, SourceMap)
- `src/ocp/diag.rs`            (Diagnostic, ErrorCode)
- `src/ocp/lex.rs`             (Lexer)
- `src/ocp/ast.rs`             (AST)
- `src/ocp/parse.rs`           (Parser)
- `src/ocp/types.rs`           (Type definitions)
- `src/ocp/typecheck.rs`       (Typechecker)
- `src/ocp/value.rs`           (Runtime values)
- `src/ocp/result_kind.rs`     (4-kind result model)
- `src/ocp/budget.rs`          (Budget)
- `src/ocp/ctx_contract.rs`    (`ctx("k=v;...")` contract parse/validate)
- `src/ocp/keys.rs`            (Key parse/canonicalize)
- `src/ocp/registry.rs`        (Capabilities/permissions/schema/policies)
- `src/ocp/exec.rs`            (Executor)
- `src/ocp/audit.rs`           (TraceEvent + Signature)
- `src/ocp/runner.rs`          (Fixture runner)
- `src/ocp/pilot.rs`           (Pilot/replay + closeout report)
- `src/ocp/mod.rs`
- `src/lib.rs`

3.2 Test layout (khuyến nghị)
- `tests/ocp_parser.rs`
- `tests/ocp_typecheck.rs`
- `tests/ocp_exec.rs`
- `tests/ocp_registry.rs`
- `tests/ocp_determinism.rs`
- `tests/fixtures/v1/parse_ok/*.ocp`
- `tests/fixtures/v1/parse_fail/*.ocp`
- `tests/fixtures/v1/type_ok/*.ocp`
- `tests/fixtures/v1/type_fail/*.ocp`
- `tests/fixtures/v1/exec_ok/*.ocp`
- `tests/fixtures/v1/exec_fail/*.ocp`

---

4) Language Surface v0.1 (cú pháp + semantics)

4.1 Statements
- `let <ident> = <expr>;`
- `observe(<key>, <tier>, <ctx>, <budget>) -> <ident>;`
- `commit(<ident_or_expr>);`
- `match <expr> { OK => { ... } DEGRADED => { ... } INSUFFICIENT => { ... } DEFERRED => { ... } }`
- `condition(<bool_expr>);`

4.2 Expressions
- Literals: int/bool/string
- Ident reference
- Field access: `x.y` (narrowing tối thiểu)
- Calls (allowlist): `budget(<int>)`, `ctx(<string>)`

4.3 Type rules tối thiểu
- `observe(...) -> r` tạo `r: Result4<Payload>`
- `match r` bắt buộc đủ 4 arms
- `commit(r)` chỉ hợp lệ với `Result4<_>` originate từ observe và kind ∈ {OK, DEGRADED} (default)
- `condition(b)` yêu cầu bool; false => exec error

4.4 4-kind result model (runtime contract)
- `OK`: payload đủ
- `DEGRADED`: payload có nhưng degrade
- `INSUFFICIENT`: không đủ dữ liệu, có `reason_code`
- `DEFERRED`: hoãn, có `reason_code`

Reason codes baseline:
- `RC-BUDGET-EXCEEDED`
- `RC-CAPABILITY-DENIED`
- `RC-KEY-UNKNOWN`
- `RC-CTX-INVALID`
- `RC-POLICY-DENIED`
- `RC-ADAPTER-FAILED`
- `RC-NOT-IMPLEMENTED`

---

5) Runtime Kernel Semantics v0.1 (đủ chi tiết để code)

5.1 Budget + Step cap
- `ExecConfig.step_cap` giới hạn số bước.
- `Budget.requested_units` giới hạn mỗi observe/effect.
- Vượt cap => trả `DEFERRED` với `RC-BUDGET-EXCEEDED` (khuyến nghị).

5.2 `ctx("k=v;...")` validation
- Parse `;` thành pairs `k=v`.
- Reject duplicate keys, key/value rỗng.
- Restrict charset keys (recommend): `[A-Za-z0-9_.-]`.

5.3 Key + Registry + Permission
- Key literal exact (v0.1) dạng `"family.name[.sub]"`.
- Registry có:
  - enabled families
  - allow patterns (wildcard chỉ trong policy)
  - deny patterns (optional)
  - ctx schema tối thiểu (required keys)
  - per-key `commit_allowed`
- Observe path:
  1) parse/canonicalize key
  2) family enabled?
  3) allow/deny?
  4) ctx schema pass?
  5) dispatch adapter => Result4

5.4 Commit discipline
- Commit chỉ chấp nhận observe-result có `origin_id` hợp lệ (gắn khi observe).
- Default: chỉ `OK/DEGRADED` mới commit được.
- Commit policy per-key có thể deny => `E-COMMIT-FORBIDDEN`.

5.5 Audit/Trace
- Trace events tối thiểu: ObserveStart/End, MatchArmSelected, CommitAttempt/Result, ConditionCheck, ProgramEnd.
- Signature: hash ổn định của chuỗi events (ordered, deterministic).

---

6) Operational Commands
- `cargo test`
- `cargo test --test ocp_parser`
- `cargo test --test ocp_typecheck`
- `cargo test --test ocp_exec`
- `cargo test --test ocp_registry`
- `cargo test --test ocp_determinism`
- `cargo test --test ocp_commit_policy`
- `cargo test --test ocp_fixture_runner`
- `cargo test --test ocp_pilot`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt -- --check`

---

7) Execution Log

- Lưu ý:
  - Các entry `implementation update (chưa verify test)` là snapshot trung gian theo thời điểm triển khai.
  - Trạng thái chốt của từng gate phải đọc ở entry `implementation closeout` tương ứng.
  - Trạng thái current-head sau audit v0.6 được chốt ở entry `V0.6 audit v0.1 consistency patch` ở cuối file.

### 2026-03-03 - V1-A planning freeze
- Date:
- 2026-03-03
- Gate/Step:
- V1-A
- Why:
- Khởi động triển khai v0.1 theo thiết kế mới OCP (core language/runtime thuần), bắt đầu từ skeleton + diagnostics + error codes.
- Scope:
- Tạo skeleton repo Rust tối thiểu cho v0.1.
- Implement `span` + `diag` + `error codes` theo chuẩn structured diagnostics.
- Thêm test cơ bản cho shape diagnostics và mapping error code.
- Expected tests:
- `cargo test`
- Exit criteria:
- Có project build/test chạy được.
- Có module diagnostics dùng được từ `src/lib.rs`.
- Gate V1-A chuyển từ `TODO` sang `IN_PROGRESS` khi bắt đầu code.

### 2026-03-03 - V1-A implementation closeout
- Date:
- 2026-03-03
- Gate/Step:
- V1-A
- Implemented:
- Khởi tạo skeleton Rust cho OCP v0.1 (`Cargo.toml`, `src/lib.rs`, `src/ocp/mod.rs`).
- Implement module `Span` trong `src/ocp/span.rs`.
- Implement structured diagnostics trong `src/ocp/diag.rs` gồm:
  - `DiagPhase`
  - `ErrorCode` + mapping stable wire code
  - `ReasonCode` + mapping theo baseline `RC-*`
  - `Diagnostic` (code, phase, span, message, hint)
- Thêm test `tests/ocp_diag.rs` để khóa contract V1-A.
- Files changed:
- `Cargo.toml`
- `src/lib.rs`
- `src/ocp/mod.rs`
- `src/ocp/span.rs`
- `src/ocp/diag.rs`
- `tests/ocp_diag.rs`
- `OCP-MVP-PLAN-v0.1.md`
- Commands run:
- `cargo test`
- Test results:
- PASS. Tổng cộng 3 test integration cho diagnostics/reason-codes đều pass.
- Notes/risks:
- V1-A mới khóa phần skeleton + diagnostics/error-codes; parser/typechecker/executor sẽ triển khai ở V1-B/V1-C/V1-D.

### 2026-03-03 - V1-B planning freeze
- Date:
- 2026-03-03
- Gate/Step:
- V1-B
- Why:
- Mở lexer/parser/ast cho bề mặt ngôn ngữ v0.1 để có parse contract ổn định cho các gate typecheck/exec.
- Scope:
- Implement `lex.rs`, `ast.rs`, `parse.rs`.
- Nối module vào `src/ocp/mod.rs` và re-export kiểu chính.
- Thêm test parser pass/fail tối thiểu.
- Expected tests:
- `cargo test --test ocp_parser`
- `cargo test`
- Exit criteria:
- Parse được các statement cốt lõi (`let/observe/commit/condition/match`).
- Parse fail trả `Diagnostic` có `code/phase/span/message`.
- Test parser cơ bản pass.

### 2026-03-03 - V1-B implementation closeout
- Date:
- 2026-03-03
- Gate/Step:
- V1-B
- Implemented:
- Thêm AST nền tảng trong `src/ocp/ast.rs`:
  - `Program`, `Stmt`, `Expr`, `MatchStmt`
  - Span gắn trên statement/expression để phục vụ diagnostics.
- Thêm lexer trong `src/ocp/lex.rs`:
  - Token hóa keyword, literal, punctuation, arrow/fat-arrow.
  - Trả `Diagnostic` parse-phase cho lỗi ký tự/literal/string chưa đóng.
- Thêm parser trong `src/ocp/parse.rs`:
  - Parse statement: `let`, `observe`, `commit`, `condition`, `match`.
  - Parse expression: int/bool/string/ident/call/field-access.
  - Enforce `match` đủ 4 arm `OK|DEGRADED|INSUFFICIENT|DEFERRED`.
- Nối module và re-export ở `src/ocp/mod.rs`.
- Thêm test parser ở `tests/ocp_parser.rs` (pass/fail cases).
- Files changed:
- `src/ocp/mod.rs`
- `src/ocp/ast.rs`
- `src/ocp/lex.rs`
- `src/ocp/parse.rs`
- `tests/ocp_parser.rs`
- `OCP-MVP-PLAN-v0.1.md`
- Commands run:
- `cargo test --test ocp_parser`
- `cargo test`
- Test results:
- PASS. `ocp_parser`: 4/4 test pass. Full suite hiện có pass.
- Notes/risks:
- V1-B mới khóa parser contract cốt lõi; chưa có typechecker/runtime semantics (thuộc V1-C/V1-D trở đi).

### 2026-03-03 - V1-C planning freeze
- Date:
- 2026-03-03
- Gate/Step:
- V1-C
- Why:
- Bổ sung tầng kiểu và typechecker core để khóa hợp đồng ngữ nghĩa trước khi mở executor.
- Scope:
- Implement `src/ocp/types.rs`.
- Implement `src/ocp/typecheck.rs`.
- Nối module qua `src/ocp/mod.rs`.
- Thêm test `tests/ocp_typecheck.rs` cho các rule cốt lõi.
- Expected tests:
- `cargo test --test ocp_typecheck`
- `cargo test`
- Exit criteria:
- `condition(...)` bắt buộc bool.
- `commit(...)` chỉ chấp nhận identifier bind từ `observe(...)`.
- `observe(...)` kiểm tra kiểu tham số cơ bản.
- `match` yêu cầu biểu thức có kiểu `Result4<_>`.

### 2026-03-03 - V1-C implementation update (chưa verify test)
- Date:
- 2026-03-03
- Gate/Step:
- V1-C
- Implemented:
- Thêm mô hình kiểu trong `src/ocp/types.rs`:
  - `Type::{Int,Bool,String,Budget,Ctx,Payload,Result4,Unit,Unknown}`.
- Thêm typechecker trong `src/ocp/typecheck.rs`:
  - infer kiểu expression cơ bản.
  - validate call allowlist `budget(int)` và `ctx(string)`.
  - enforce `condition(bool)`.
  - enforce `commit(ident)` và ident phải là binding từ `observe(...)`.
  - enforce `observe(key,tier,ctx,budget)` theo kiểu đầu vào tối thiểu.
  - enforce `match` chỉ nhận `Result4<_>`.
- Nối export trong `src/ocp/mod.rs`:
  - `types`, `typecheck`, `typecheck_program`, `TypeChecker`, `Type`.
- Thêm test `tests/ocp_typecheck.rs` cho happy-path + fail-path chính.
- Files changed:
- `src/ocp/mod.rs`
- `src/ocp/types.rs`
- `src/ocp/typecheck.rs`
- `tests/ocp_typecheck.rs`
- `OCP-MVP-PLAN-v0.1.md`
- Commands run:
- Chưa chạy lệnh test trong bước này.
- Test results:
- Chưa verify.
- Notes/risks:
- Cần chạy `cargo test --test ocp_typecheck` + `cargo test` để chốt `V1-C = DONE`.

### 2026-03-03 - V1-C implementation closeout
- Date:
- 2026-03-03
- Gate/Step:
- V1-C
- Implemented:
- Hoàn tất `types + typechecker core` cho v0.1.
- Sửa lỗi compile trong `types.rs`: đổi `result4_payload` từ `const fn` sang `fn`.
- Files changed:
- `src/ocp/types.rs`
- `src/ocp/typecheck.rs`
- `src/ocp/mod.rs`
- `tests/ocp_typecheck.rs`
- `OCP-MVP-PLAN-v0.1.md`
- Commands run:
- `cargo test --test ocp_typecheck`
- `cargo test`
- Test results:
- PASS:
  - `ocp_typecheck`: 5/5 pass.
  - Full suite hiện có: `ocp_diag` 3/3, `ocp_parser` 4/4, `ocp_typecheck` 5/5 pass.
- Notes/risks:
- V1-C đã đóng. Chưa có runtime exec semantics (thuộc V1-D trở đi).

### 2026-03-03 - V1-D planning freeze
- Date:
- 2026-03-03
- Gate/Step:
- V1-D
- Why:
- Mở executor core có bounded step + env để chương trình OCP có thể chạy được theo semantics nền.
- Scope:
- Implement `result_kind.rs`, `value.rs`, `budget.rs`, `exec.rs`.
- Nối export trong `src/ocp/mod.rs`.
- Thêm test `tests/ocp_exec.rs`.
- Expected tests:
- `cargo test --test ocp_exec`
- `cargo test`
- Exit criteria:
- Có executor chạy được `let/condition/match` cơ bản.
- Có bounded step cap.
- Có env lưu biến sau thực thi.

### 2026-03-03 - V1-D implementation update (chưa verify test)
- Date:
- 2026-03-03
- Gate/Step:
- V1-D
- Implemented:
- Thêm mô hình 4-kind runtime:
  - `src/ocp/result_kind.rs` (`ResultKind`, `Result4<T>`, `origin_id`).
- Thêm runtime value model:
  - `src/ocp/value.rs` (`Value`).
- Thêm budget meter:
  - `src/ocp/budget.rs` (`ExecConfig`, `BudgetMeter`).
- Thêm executor:
  - `src/ocp/exec.rs` (`Executor`, `execute_program`, `ExecOutput`).
  - Step-cap check mỗi bước (`EBudgetExceeded`).
  - Runtime env cho biến.
  - `observe(...)` stub fail-honest: trả `DEFERRED + RC-NOT-IMPLEMENTED`.
  - `match` chọn arm theo `ResultKind`.
  - `condition(false)` trả `EConditionFalse`.
  - `commit` chỉ cho phép `OK/DEGRADED`, còn lại `ECommitForbidden`.
- Nối module/export qua `src/ocp/mod.rs`.
- Thêm test `tests/ocp_exec.rs` cho happy/fail path chính.
- Files changed:
- `src/ocp/mod.rs`
- `src/ocp/result_kind.rs`
- `src/ocp/value.rs`
- `src/ocp/budget.rs`
- `src/ocp/exec.rs`
- `tests/ocp_exec.rs`
- `OCP-MVP-PLAN-v0.1.md`
- Commands run:
- Chưa chạy lệnh test trong bước này.
- Test results:
- Chưa verify.
- Notes/risks:
- `observe` vẫn là stub trong V1-D; runtime adapter thật thuộc các gate sau.

### 2026-03-03 - V1-D implementation closeout
- Date:
- 2026-03-03
- Gate/Step:
- V1-D
- Implemented:
- Hoàn tất executor core + bounded step + env.
- Files changed:
- `src/ocp/result_kind.rs`
- `src/ocp/value.rs`
- `src/ocp/budget.rs`
- `src/ocp/exec.rs`
- `src/ocp/mod.rs`
- `tests/ocp_exec.rs`
- `OCP-MVP-PLAN-v0.1.md`
- Commands run:
- `cargo test --test ocp_exec`
- `cargo test`
- Test results:
- PASS:
  - `ocp_exec`: 5/5 pass.
  - Full suite hiện có: `ocp_diag` 3/3, `ocp_parser` 4/4, `ocp_typecheck` 5/5, `ocp_exec` 5/5 pass.
- Notes/risks:
- V1-D đã đóng với runtime core nền.
- `observe` vẫn là stub fail-honest (`DEFERRED + RC-NOT-IMPLEMENTED`), sẽ mở rộng ở các gate sau.

### 2026-03-03 - V1-E planning freeze
- Date:
- 2026-03-03
- Gate/Step:
- V1-E
- Why:
- Bổ sung keyspace parser + capability registry + permission gate để observe-path có kiểm soát quyền truy cập.
- Scope:
- Implement `src/ocp/keys.rs`.
- Implement `src/ocp/registry.rs`.
- Nối registry vào `observe` runtime path trong `src/ocp/exec.rs`.
- Thêm test `tests/ocp_registry.rs`.
- Expected tests:
- `cargo test --test ocp_registry`
- `cargo test`
- Exit criteria:
- Key parser enforce shape `"family.name[.sub]"`.
- Registry check đủ: family enabled, allow/deny, ctx-required.
- Observe trả `INSUFFICIENT` với reason tương ứng khi bị gate deny.

### 2026-03-03 - V1-E implementation update (chưa verify test)
- Date:
- 2026-03-03
- Gate/Step:
- V1-E
- Implemented:
- Thêm key parser:
  - `src/ocp/keys.rs` (`KeyRef`, `parse_key`).
- Thêm capability registry:
  - `src/ocp/registry.rs` (`CapabilityRegistry`, `KeyPattern`, `RegistryCheckError`).
  - Rule hỗ trợ: enabled families, allow patterns, deny patterns, ctx-required, commit_allowed.
- Nối module/export ở `src/ocp/mod.rs`.
- Tích hợp permission gate vào `observe` runtime path trong `src/ocp/exec.rs`:
  - pass registry -> `DEFERRED + RC-NOT-IMPLEMENTED` (stub adapter).
  - fail registry -> `INSUFFICIENT + reason`.
- Thêm test registry:
  - `tests/ocp_registry.rs`.
- Files changed:
- `src/ocp/mod.rs`
- `src/ocp/keys.rs`
- `src/ocp/registry.rs`
- `src/ocp/exec.rs`
- `tests/ocp_registry.rs`
- `OCP-MVP-PLAN-v0.1.md`
- Commands run:
- Chưa chạy lệnh test trong bước này.
- Test results:
- Chưa verify.
- Notes/risks:
- Commit policy per-key đã có data shape trong registry, enforcement chi tiết commit sẽ khóa sâu hơn ở V1-F.

### 2026-03-03 - V1-E implementation closeout
- Date:
- 2026-03-03
- Gate/Step:
- V1-E
- Implemented:
- Hoàn tất key parser + capability registry + permission gate cho observe-path runtime.
- Files changed:
- `src/ocp/keys.rs`
- `src/ocp/registry.rs`
- `src/ocp/exec.rs`
- `src/ocp/mod.rs`
- `tests/ocp_registry.rs`
- `OCP-MVP-PLAN-v0.1.md`
- Commands run:
- `cargo test --test ocp_registry`
- `cargo test`
- Test results:
- PASS:
  - `ocp_registry`: 7/7 pass.
  - Full suite hiện có: `ocp_diag` 3/3, `ocp_parser` 4/4, `ocp_typecheck` 5/5, `ocp_exec` 5/5, `ocp_registry` 7/7 pass.
- Notes/risks:
- V1-E đã đóng.
- Commit policy per-key mới ở mức dữ liệu trong registry; logic enforce commit-policy chi tiết sẽ khóa ở V1-F.

### 2026-03-03 - V1-F planning freeze
- Date:
- 2026-03-03
- Gate/Step:
- V1-F
- Why:
- Khóa semantics runtime cho `observe/match/commit` và enforce commit policy theo origin/key.
- Scope:
- Bổ sung commit event/log trong executor output.
- Enforce `commit`:
  - phải có `origin_id` từ observe.
  - chỉ nhận `OK/DEGRADED`.
  - pass `commit_allowed_for_key`.
- Nâng observe stub để có đủ nhánh `OK/DEGRADED/DEFERRED` cho test semantics.
- Thêm test policy/semantics.
- Expected tests:
- `cargo test --test ocp_commit_policy`
- `cargo test --test ocp_exec`
- `cargo test`
- Exit criteria:
- Commit policy deny theo key hoạt động.
- Match chọn đúng arm theo result kind.
- Commit event ghi nhận đầy đủ `origin_id/key/kind`.

### 2026-03-03 - V1-F implementation update (chưa verify test)
- Date:
- 2026-03-03
- Gate/Step:
- V1-F
- Implemented:
- Mở rộng executor output với commit events:
  - `CommitEvent { origin_id, key, kind }`
  - `ExecOutput.commits`.
- Bổ sung observe provenance store (`origin_id -> key`) trong runtime.
- Enforce commit runtime policy trong `exec_commit`:
  - fail nếu thiếu/không nhận diện được `origin_id`.
  - fail nếu kind là `INSUFFICIENT/DEFERRED`.
  - fail nếu registry deny `commit_allowed_for_key`.
  - pass thì append commit event.
- Nâng observe stub result:
  - `world.ok*` -> `OK(payload)`
  - `world.degraded*` -> `DEGRADED(payload, RC-ADAPTER-FAILED)`
  - key khác (pass gate) -> `DEFERRED(RC-NOT-IMPLEMENTED)`
- Nối export thêm `CommitEvent` qua `mod.rs`.
- Thêm test `tests/ocp_commit_policy.rs`.
- Files changed:
- `src/ocp/mod.rs`
- `src/ocp/exec.rs`
- `tests/ocp_commit_policy.rs`
- `OCP-MVP-PLAN-v0.1.md`
- Commands run:
- Chưa chạy lệnh test trong bước này.
- Test results:
- Chưa verify.
- Notes/risks:
- Observe hiện vẫn dùng stub deterministic, chưa có adapter IO thật (đúng scope v0.1).

### 2026-03-03 - V1-F implementation closeout
- Date:
- 2026-03-03
- Gate/Step:
- V1-F
- Implemented:
- Hoàn tất runtime semantics `observe/match/commit` cho scope v0.1 và commit policy theo `origin_id/key/kind`.
- Files changed:
- `src/ocp/exec.rs`
- `src/ocp/mod.rs`
- `tests/ocp_commit_policy.rs`
- `OCP-MVP-PLAN-v0.1.md`
- Commands run:
- `cargo test --test ocp_commit_policy`
- `cargo test`
- Test results:
- PASS:
  - `ocp_commit_policy`: 4/4 pass.
  - Full suite hiện có: `ocp_commit_policy` 4/4, `ocp_diag` 3/3, `ocp_parser` 4/4, `ocp_typecheck` 5/5, `ocp_exec` 5/5, `ocp_registry` 7/7 pass.
- Notes/risks:
- V1-F đã đóng theo scope v0.1.
- Observe adapter vẫn là stub deterministic; chưa có world IO adapter thật (để gate sau mở rộng).

### 2026-03-03 - V1-G planning freeze
- Date:
- 2026-03-03
- Gate/Step:
- V1-G
- Why:
- Bổ sung trace/audit và deterministic signature để replay/evidence có thể kiểm chứng.
- Scope:
- Implement `src/ocp/audit.rs`.
- Tích hợp trace vào `exec.rs`.
- Thêm fixture runner module `src/ocp/runner.rs`.
- Thêm test `ocp_determinism` và `ocp_fixture_runner`.
- Expected tests:
- `cargo test --test ocp_determinism`
- `cargo test --test ocp_fixture_runner`
- `cargo test`
- Exit criteria:
- Exec output có trace + signature ổn định.
- Cùng input/config cho cùng signature.
- Có API chạy fixture từ source và từ file.

### 2026-03-03 - V1-G implementation update (chưa verify test)
- Date:
- 2026-03-03
- Gate/Step:
- V1-G
- Implemented:
- Thêm audit module:
  - `src/ocp/audit.rs` với `TraceEvent`, `TraceLog`, `signature_hex()` (FNV-1a ổn định).
- Thêm fixture runner:
  - `src/ocp/runner.rs` (`run_fixture_source`, `run_fixture_file`, `FixtureRunnerError`).
- Nối export tại `src/ocp/mod.rs`.
- Tích hợp trace vào executor (`src/ocp/exec.rs`):
  - emit events: ObserveStart/ObserveEnd, MatchArmSelected, CommitAttempt/CommitResult, ConditionCheck, ProgramEnd.
  - `ExecOutput` có thêm `trace` và `signature`.
- Thêm test:
  - `tests/ocp_determinism.rs`
  - `tests/ocp_fixture_runner.rs`
  - fixture file `tests/fixtures/v1/exec_ok/determinism_basic.ocp`
- Files changed:
- `src/ocp/mod.rs`
- `src/ocp/audit.rs`
- `src/ocp/runner.rs`
- `src/ocp/exec.rs`
- `tests/ocp_determinism.rs`
- `tests/ocp_fixture_runner.rs`
- `tests/fixtures/v1/exec_ok/determinism_basic.ocp`
- `OCP-MVP-PLAN-v0.1.md`
- Commands run:
- Chưa chạy lệnh test trong bước này.
- Test results:
- Chưa verify.
- Notes/risks:
- Signature hiện dựa trên trace string canonical; thay đổi schema event sẽ đổi signature (chấp nhận trong v0.1).

### 2026-03-03 - V1-G implementation closeout
- Date:
- 2026-03-03
- Gate/Step:
- V1-G
- Implemented:
- Hoàn tất audit/trace + fixture runner + determinism signature theo scope v0.1.
- Files changed:
- `src/ocp/audit.rs`
- `src/ocp/runner.rs`
- `src/ocp/exec.rs`
- `src/ocp/mod.rs`
- `tests/ocp_determinism.rs`
- `tests/ocp_fixture_runner.rs`
- `tests/fixtures/v1/exec_ok/determinism_basic.ocp`
- `OCP-MVP-PLAN-v0.1.md`
- Commands run:
- `cargo test --test ocp_determinism --test ocp_fixture_runner`
- `cargo test`
- Test results:
- PASS:
  - `ocp_determinism`: 2/2 pass.
  - `ocp_fixture_runner`: 3/3 pass.
  - Full suite hiện có: `ocp_commit_policy` 4/4, `ocp_determinism` 2/2, `ocp_diag` 3/3, `ocp_exec` 5/5, `ocp_fixture_runner` 3/3, `ocp_parser` 4/4, `ocp_registry` 7/7, `ocp_typecheck` 5/5 pass.
- Notes/risks:
- V1-G đã đóng.
- Signature ổn định theo trace schema hiện tại; thay đổi event schema trong tương lai sẽ đổi signature.

### 2026-03-03 - V1-H planning freeze
- Date:
- 2026-03-03
- Gate/Step:
- V1-H
- Why:
- Hoàn tất vòng v0.1 bằng pilot/replay và closeout report để có bằng chứng vận hành end-to-end.
- Scope:
- Implement module pilot/replay.
- Thêm API tạo closeout report dạng markdown.
- Thêm test pilot deterministic + report shape.
- Expected tests:
- `cargo test --test ocp_pilot`
- `cargo test`
- Exit criteria:
- Có pilot sequence runner chạy nhiều turn.
- Có replay check determinism cho cùng input/config.
- Có closeout report text từ summary/replay.

### 2026-03-03 - V1-H implementation update (chưa verify test)
- Date:
- 2026-03-03
- Gate/Step:
- V1-H
- Implemented:
- Thêm module pilot:
  - `src/ocp/pilot.rs`
  - API:
    - `run_pilot_sequence`
    - `replay_pilot_sequence`
    - `build_closeout_report`
  - model:
    - `PilotTurnStatus`, `PilotTurnResult`, `PilotSummary`, `PilotReplayResult`.
- Nối export pilot vào `src/ocp/mod.rs`.
- Thêm test:
  - `tests/ocp_pilot.rs`
  - kiểm tra pilot run, replay deterministic, report section, error turn.
- Files changed:
- `src/ocp/mod.rs`
- `src/ocp/pilot.rs`
- `tests/ocp_pilot.rs`
- `OCP-MVP-PLAN-v0.1.md`
- Commands run:
- Chưa chạy lệnh test trong bước này.
- Test results:
- Chưa verify.
- Notes/risks:
- Closeout report hiện ở dạng string markdown trong memory; chưa ghi file tự động (có thể thêm ở bước sau nếu cần).

### 2026-03-03 - V1-H implementation closeout
- Date:
- 2026-03-03
- Gate/Step:
- V1-H
- Implemented:
- Hoàn tất pilot/replay + closeout report theo scope v0.1.
- Files changed:
- `src/ocp/pilot.rs`
- `src/ocp/mod.rs`
- `tests/ocp_pilot.rs`
- `OCP-MVP-PLAN-v0.1.md`
- Commands run:
- `cargo test --test ocp_pilot`
- `cargo test`
- Test results:
- PASS:
  - `ocp_pilot`: 4/4 pass.
  - Full suite hiện có:
    - `ocp_commit_policy` 4/4
    - `ocp_determinism` 2/2
    - `ocp_diag` 3/3
    - `ocp_exec` 5/5
    - `ocp_fixture_runner` 3/3
    - `ocp_parser` 4/4
    - `ocp_pilot` 4/4
    - `ocp_registry` 7/7
    - `ocp_typecheck` 5/5
    - tất cả pass.
- Notes/risks:
- V1-H đã đóng; v0.1 gate A→H hoàn tất theo kế hoạch hiện tại.

### 2026-03-03 - V0.6 audit v0.1 consistency patch
- Date:
- 2026-03-03
- Gate/Step:
- V0.6 / Audit v0.1
- Implemented:
- Rà soát lại v0.1 theo code thực tế và matrix test hiện hành.
- Phát hiện drift compile ở test do `ExecConfig` đã thêm field `commit_policy` nhưng nhiều test v0.1 còn dùng literal cũ chỉ có `step_cap`.
- Vá toàn bộ test affected sang cú pháp có default:
  - `ExecConfig { step_cap: ..., ..ExecConfig::default() }`.
- Cập nhật tài liệu v0.1:
  - đổi trạng thái tổng sang `DONE` (đã re-verify),
  - sửa kiến trúc `ctx.rs` -> `ctx_contract.rs`,
  - bỏ dòng `runtime.rs` không còn đúng với cấu trúc hiện tại,
  - bổ sung `runner.rs` và `pilot.rs`,
  - mở rộng mục Operational Commands cho `ocp_commit_policy`, `ocp_fixture_runner`, `ocp_pilot`.
- Files changed:
- `tests/ocp_exec.rs`
- `tests/ocp_registry.rs`
- `tests/ocp_determinism.rs`
- `tests/ocp_pilot.rs`
- `tests/ocp_toy_programs.rs`
- `tests/ocp_stdlib.rs`
- `tests/ocp_fixture_runner.rs`
- `tests/ocp_ctx_validation.rs`
- `tests/ocp_commit_policy.rs`
- `tests/ocp_diag_taxonomy.rs`
- `OCP-MVP-PLAN-v0.1.md`
- Commands run:
- `cargo test --test ocp_parser`
- `cargo test --test ocp_typecheck`
- `cargo test --test ocp_exec`
- `cargo test --test ocp_registry`
- `cargo test --test ocp_determinism`
- `cargo test --test ocp_pilot`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt`
- `cargo fmt -- --check`
- Test results:
- PASS (sau khi vá):
  - `ocp_parser`: 4/4
  - `ocp_typecheck`: 6/6
  - `ocp_exec`: 9/9
  - `ocp_registry`: 8/8
  - `ocp_determinism`: 2/2
  - `ocp_pilot`: 4/4
  - `cargo test` full suite: PASS
  - `cargo clippy --all-targets -- -D warnings`: PASS
  - `cargo fmt -- --check`: PASS
- FAIL tạm thời đã xử lý:
  - compile fail nhiều suite với `E0063` (`ExecConfig` thiếu `commit_policy` trong test literals).
- Notes/risks:
- Đây là drift tương thích test, không thay đổi semantics runtime.
- Log closeout lịch sử của v0.1 vẫn giữ nguyên theo mốc thời gian gốc; entry này là lớp audit bổ sung từ v0.6.
