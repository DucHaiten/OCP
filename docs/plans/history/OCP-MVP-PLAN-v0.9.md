# OCP v0.9 — Verifiable Contracts: Schema + Typing for ctx/payload/effects (Policy + Correctness)

Ngày tạo: 2026-03-03  
Trạng thái: `DONE` (2026-03-04)  
Phạm vi: **OCP-only**.  
Tiền đề: v0.7.x đã có Value/JSON/import/sugar/loops + packs; v0.8 đã có quarantine+cassette nondet IO.  
Mục tiêu v0.9: biến “capability boundary” thành **contract có thể kiểm chứng** (compile-time + runtime), giảm mạnh lỗi vặt và boilerplate khi packs nhiều.

> Trục chính:  
> - Trước v0.9: ctx/payload chủ yếu là Value + runtime validation (nếu có).  
> - v0.9: có **schema** và **typing** đủ mạnh để:
>   1) validate ctx/payload chuẩn ngay từ typecheck,  
>   2) tạo error/hint tốt hơn,  
>   3) effect policy (commit allowed/forbidden) có thể chặn sớm,  
>   4) vẫn giữ tính linh hoạt (structural/row types) không khóa cứng như OOP.

---

## 0) Governance + Tracking v0.9

### 0.1 Quy ước cập nhật bắt buộc
- Mọi thay đổi kế hoạch phải cập nhật file này trước khi code.
- Mọi triển khai xong phải cập nhật log ngay sau khi chạy test.
- Không nhảy gate: gate sau chỉ mở khi gate hiện tại `DONE`.
- Chỉ chuyển gate sang `DONE` khi có đủ:
  - `Planning Freeze` + `Implementation Closeout`
  - `Files changed`, `Commands run`, `Test results`, `Notes/risks`
  - targeted tests pass đúng phạm vi gate.
- Nếu targeted tests chưa pass hoặc còn thiếu code delta:
  - bắt buộc giữ trạng thái `IN_PROGRESS` hoặc `PARTIAL` (không ghi `DONE`).
- `verification-only` snapshot (không có code delta) không được dùng để đóng gate.
- Với block closeout đã `DONE`:
  - không để placeholder `PASS/FAIL`
  - không để marker `FAIL` trong chính block closeout đó.

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

### 0.2 Quick Snapshot (BẮT BUỘC ĐỌC TRƯỚC)
- Mục tiêu phiên bản:
  - Khóa schema + typing contracts để capability boundary kiểm chứng được ở compile-time/runtime.
- Trạng thái tổng quan:
  - `DONE`: đã hoàn tất Gate `9-A`, `9-B`, `9-C`, `9-D`, `9-E1`, `9-E2`, `9-E3`, `9-F`, `9-G`.
- Gate đang làm/đã xong/chưa làm:
  - `9-A`, `9-B`, `9-C`, `9-D`, `9-E1`, `9-E2`, `9-E3`, `9-F`, `9-G` đã `DONE`.
- Bước kế tiếp ngay:
  - Rà soát close checklist v0.9 và chuẩn bị handoff sang v0.10 theo contract đã khóa.
- Lệnh kiểm chứng chuẩn:
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
- Danh sách file code trọng yếu đã thay đổi:
  - `src/ocp/schema.rs`
  - `src/ocp/ast.rs`
  - `src/ocp/parse.rs`
  - `src/ocp/typecheck.rs`
  - `src/ocp/types.rs`
  - `src/ocp/exec.rs`
  - `src/ocp/diag.rs`
  - `src/ocp/registry.rs`
  - `src/ocp/mod.rs`
  - `projects/ocp/crates/ocp-cli/src/main.rs`
  - `projects/ocp/crates/ocp-runtime-core/src/lib.rs`
  - `projects/ocp/crates/ocp-sdk/src/lib.rs`
  - `tests/schema_validate.rs`
  - `tests/schema_pretty.rs`
  - `tests/parser_destructure.rs`
  - `tests/type_record.rs`
  - `tests/type_result4.rs`
  - `tests/type_ctx_schema.rs`
  - `tests/type_effects.rs`
  - `tests/ocp_stdlib.rs`
  - `tests/pack_schema_conformance_core.rs`
  - `tests/pack_schema_conformance_consumer.rs`
  - `tests/pack_schema_conformance_quarantine.rs`
  - `tests/compat_ctx_string.rs`
  - `tests/compat_ctx_extra_fields.rs`
  - `tests/cli_doc_packs.rs`
  - `tests/cli_tool_http_e2e.rs`
  - `tests/cli_tool_proc_e2e.rs`
  - `tests/cli_tool_wallclock_e2e.rs`

### 0.3 Trạng thái Workstreams/Gates v0.9
#### Workstreams:
- WS-S (schema + type system contracts): `DONE` (2026-03-04)
- WS-C (CLI/SDK compatibility + warnings): `DONE` (2026-03-04)
- WS-I (runtime/core integration + pack rollout): `DONE` (2026-03-04)

#### Gate status (9-A .. 9-G):
- Gate 9-A (Schema core): `DONE` (2026-03-04)
- Gate 9-B (Type system + destructuring parser/AST): `DONE` (2026-03-04)
- Gate 9-C (Ctx schema typecheck): `DONE` (2026-03-04)
- Gate 9-D (Effect typing): `DONE` (2026-03-04)
- Gate 9-E1 (Pack schema core): `DONE` (2026-03-04)
- Gate 9-E2 (Pack schema consumer): `DONE` (2026-03-04)
- Gate 9-E3 (Pack schema quarantine): `DONE` (2026-03-04)
- Gate 9-F (Migration compatibility controls): `DONE` (2026-03-04)
- Gate 9-G (Docs/examples/benchmark): `DONE` (2026-03-04)

### 0.4 Operational commands
- `cargo test`
- `cargo test --test schema_validate`
- `cargo test --test type_record`
- `cargo test --test type_result4`
- `cargo test --test type_ctx_schema`
- `cargo test --test type_effects`
- `cargo test --test parser_destructure`
- `cargo test --test pack_schema_conformance_core`
- `cargo test --test pack_schema_conformance_consumer`
- `cargo test --test pack_schema_conformance_quarantine`
- `cargo test --test compat_ctx_string`
- `cargo test --test compat_ctx_extra_fields`
- `cargo test --test cli_doc_packs`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt -- --check`

### 0.5 Change Classification (S/C/I)
| Item | Tag | Compatibility | Evidence suite | Owner gate |
|---|---|---|---|---|
| SchemaType + validator + pretty-printer | `S` | additive | `tests/schema_validate.rs`, `tests/schema_pretty.rs` | `9-A` |
| Result4<T> + record typing + destructuring | `S` | additive | `tests/type_record.rs`, `tests/type_result4.rs`, `tests/parser_destructure.rs` | `9-B` |
| ctx schema enforcement (literal-key static checks) | `S` | additive | `tests/type_ctx_schema.rs` | `9-C` |
| Effect typing (ObserveOnly/ObserveAndCommit) | `S` | additive | `tests/type_effects.rs` | `9-D` |
| Pack schema rollout core/consumer/quarantine | `I` | internal | `tests/pack_schema_conformance_*` | `9-E1..9-E3` |
| compat knobs `ctx_string`/`ctx_extra_fields` | `C` | additive | `tests/compat_ctx_string.rs`, `tests/compat_ctx_extra_fields.rs` | `9-F` |
| CLI docs/examples (`ocp doc packs`, templates) | `C` | additive | `tests/cli_doc_packs.rs` | `9-G` |

---

## 1) Goals v0.9 (LOCKED)

### 1.1 North Star
- Dev có thể thêm packs/engines mà không “bị vỡ vì ctx/payload sai”.
- Mọi capability key có schema ctx/payload rõ ràng.
- Typechecker bắt được phần lớn lỗi trước runtime.
- Runtime vẫn validate (defense-in-depth) nhưng lỗi runtime giảm mạnh.
- Policy/effects trở thành “typed & checkable”: script không thể compile nếu commit vào key forbidden (khi thông tin tĩnh đủ).

### 1.2 KPI bắt buộc (định lượng)
**KPI-1: Compile-time catches**
- >= 70% lỗi ctx thiếu field / sai type / sai key name trong fixtures phải bị bắt ở typecheck (`T-CTX-*`) thay vì runtime.

**KPI-2: Boilerplate reduction**
- Các chương trình dùng packs không phải thủ công `match` để “extract fields” nhiều; destructuring + record typing giúp giảm LOC.

**KPI-3: Error quality**
- Lỗi ctx/payload phải có hint tự động:
  - missing required fields
  - unknown fields (typo)
  - expected type vs got
  - example ctx skeleton (auto-generated)

**KPI-4: Effect correctness**
- Ít nhất 1 class error “commit forbidden” phải bị bắt compile-time trong case tĩnh (ví dụ commit `std.fs.read_text` hoặc commit key declared NeverCommit).

---

## 2) Axis Lock v0.9 (kế thừa, không đổi)
- Capability/permission/budget/determinism/4-kind/commit-gate giữ nguyên.
- v0.9 chỉ tăng “verifiability” và “correctness ergonomics”.

---

## 3) Scope v0.9

### 3.1 In-scope (ship)
A) **Schema system cho ctx/payload**
- Schema language (internal) để mô tả:
  - record fields, optional fields, defaults
  - primitive types, list/map, bounded list/map (cap)
  - enum (string enum)
  - constraints (min/max/regex) mức tối thiểu
- Schema attached to capability keys trong registry.

B) **Type system nâng cấp**
- Structural typing cho record (row types nhẹ).
- Optional/nullable chuẩn.
- Typed Result4: `Result4<T>` với `T` là schema-backed payload type, không chỉ Value.
- Narrowing trong `match`/`try`:
  - `try` trả payload typed.
- Destructuring typed:
  - `let {a,b} = rec;` có check.

C) **Ctx builder typed**
- Thay vì `ctx("k=v;...")` string, v0.9 chuẩn hóa:
  - ctx là record typed: `{ path:"...", cap:100 }`
- Vẫn hỗ trợ legacy `ctx("...")` dưới chế độ compatibility (warning), nhưng packs v0.9 yêu cầu ctx record typed.
- Observe surface giữ nguyên baseline, không đổi grammar:
  - `observe(<key>, <tier>, <ctx>, <budget>) -> <ident>;`

D) **Effect typing / commit policy check**
- Registry đánh dấu key:
  - `ObserveOnly` (commit forbidden)
  - `ObserveAndCommit`
- Typechecker enforce:
  - `commit()` chỉ nhận Result4 từ key commit-capable
  - không mở primitive/effect token mới trong v0.9
- Optional: “phase typing” (defer) nếu bạn có phases; không bắt buộc v0.9.

E) **Schema-first registry**
- Mọi std pack keys phải khai schema ctx/payload trong code.
- Auto-generate docs từ schema (optional).

F) **Runtime validation**
- Runtime validate ctx against schema:
  - in locked: invalid ctx => INSUFFICIENT `RC-CTX-INVALID` + details
- But goal: 대부분 caught at typecheck.

### 3.2 Out-of-scope (defer)
- Full dependent types / theorem proving.
- Generics system full-power (đủ dùng thôi).
- Refinement types phức tạp (chỉ basic constraints).
- Full JSON schema interop.
- Module version solver (thuộc v0.10).

---

## 4) Schema Model (v0.9)

### 4.1 Type universe (SchemaType)
- `SBool`, `SInt`, `SString`
- `SNull` (rare)
- `SList(elem: SchemaType, cap?: Int)`
- `SMap(key: SStringOnly, val: SchemaType, cap?: Int)`
- `SRecord(fields: Map<Name, FieldSpec>, open_row?: Bool)`
- `SEnum(values: List<String>)`
- `SUnion(types: List<SchemaType>)` (optional minimal; dùng cho nullable/variant)
- `SBytes` (optional for fs/proc net; can represent as string base64 if not)

FieldSpec:
- `required: Bool`
- `ty: SchemaType`
- `default?: Value`
- `constraints?: { min, max, regex, nonempty }` (minimal set)

### 4.2 Open vs closed record (row types)
- **Closed record**: unknown fields => type error.
- **Open record**: allow extra fields (captured row), nhưng fields required vẫn check.

Default v0.9:
- ctx records should be **closed** (to catch typos).
- payload records can be open (forward compatibility), configurable.
- compatibility knob cho ctx closed:
  - `compat.ctx_extra_fields=warn` mặc định locked để không gãy cứng script cũ ngay.
  - templates mới đặt `compat.ctx_extra_fields=deny`.

### 4.3 Bounded schema
- `cap` không chỉ runtime; schema có thể yêu cầu ctx có cap field và enforce bounds.

Example:
- `std.fs.list_dir` ctx schema requires:
  - `path: String`
  - `cap: Int` (1..=perm.max_list_entries)

---

## 5) Typing rules (v0.9)

### 5.1 Result4<T>
- `observe(key, tier, ctx, budget) -> bind`:
  - `tier` giữ hợp đồng hiện tại của baseline (string-tier), không đổi kiểu ở v0.9
  - nếu `key` là literal: returns `Result4<PayloadType(key)>`
  - nếu `key` là dynamic expression: returns `Result4<Value>` + `T-KEY-NOT-LITERAL-FOR-SCHEMA` (warning/error theo policy)
- `try` extracts `T` from `Result4<T>` (OK/DEGRADED path)

### 5.2 Record typing
- `{ a: 1, b: "x" }` typed as `Record{a:Int,b:String}` (closed by default literal)
- `let r: Record{a:Int,b:String} = ...` supported (optional annotation)
- Field access `r.a` typed.

### 5.3 Destructuring
- `let {a, b} = r;`:
  - r must have fields a,b with compatible types
  - missing => type error with hint

### 5.4 Schema check for ctx
- `observe("std.fs.read_text", "default", { path: "./x.txt", max_bytes: 100 }, budget(1000)) -> r;`
- Typechecker:
  - key literal-only path: map key -> schema và enforce compile-time đầy đủ
  - key dynamic expression: downgrade runtime-only validation + warning/error theo policy (`T-KEY-NOT-LITERAL-FOR-SCHEMA`)
  - validates ctx record matches schema (names, types, required)
  - emits `T-CTX-MISSING-FIELD`, `T-CTX-UNKNOWN-FIELD`, `T-CTX-TYPE-MISMATCH`
  - optionally checks constraints if literals (e.g., cap literal > max => type error)

### 5.5 Effect typing
- `commit(r)` allowed only if:
  - r is `Result4<T>` where origin key is commit-capable
  - and kind constraints satisfied (OK/DEGRADED)
- commit on observe-only keys:
  - compile-time error `T-COMMIT-FORBIDDEN-KEY`
- compile-time forbidden-key checks chỉ áp dụng đầy đủ cho origin từ observe với key literal.
- origin key dynamic expression:
  - typechecker không kết luận được key-kind tĩnh
  - runtime registry/policy check vẫn bắt buộc và là source of truth

---

## 6) Registry contract (v0.9)

### 6.1 CapabilityKeySpec
Each key in registry must declare:
- `key_name`
- `kind`: ObserveOnly | ObserveAndCommit
- `ctx_schema: SchemaType` (record)
- `payload_schema: SchemaType` (record or primitive)
- `permission_class`: e.g. `std_fs.read`, `std_net_http.request`
- `budget_class`: compute cost model (optional)
- `determinism_class`: deterministic | cassette-based | nondet-forbidden (locked)
- `notes`: docstring (optional)

### 6.2 Schema versioning
- Each pack has `schema_version`.
- Payload records recommended `open_row=true` for forward compatibility.
- Typechecker can allow open payload (unknown fields) but still require known fields for your code.

---

## 7) Migration: ctx("...") -> ctx record (v0.9)

### 7.1 Policy
- v0.9 keeps `ctx("k=v;...")` only for legacy keys or internal use.
- For std packs, v0.9 requires ctx record.
- Grammar không đổi ở v0.9: tham số `ctx` của `observe(...)` vẫn là expression/value như baseline.
- `ctx typed` là policy ở typecheck/CLI + schema validation, không phải grammar fork.
- đường chuyển tiếp khóa cứng:
  - `ctx("k=v;...")` được parse như legacy value rồi desugar/convert sang record value trước schema validation.
  - từ điểm validate schema trở đi chỉ dùng một đường `ctx` record.
- Typechecker:
  - if sees `ctx("...")` used for std pack => warning or error (configurable).
- CLI can offer `ocp fix ctx` (optional) to convert trivial cases.

### 7.2 Compatibility gate
- `compat.ctx_string = allow|warn|deny` in manifest.
- `compat.ctx_extra_fields = allow|warn|deny` in manifest (cho ctx closed policy).
Manifest snippet:
```toml
[compat]
ctx_string = "warn"        # allow|warn|deny
ctx_extra_fields = "warn"  # allow|warn|deny
```
Default:
- locked: `warn`
- new templates: `deny`

---

## 8) Error taxonomy v0.9 (additive, DX-focused)

New type errors:
- `T-CTX-MISSING-FIELD`
- `T-CTX-UNKNOWN-FIELD`
- `T-CTX-TYPE-MISMATCH`
- `T-CTX-CONSTRAINT-VIOLATION` (literals only)
- `T-PAYLOAD-TYPE-MISMATCH` (when destructuring payload)
- `T-COMMIT-FORBIDDEN-KEY`
- `T-KEY-NOT-LITERAL-FOR-SCHEMA`

Compatibility note:
- `T-OBSERVE-FORBIDDEN-KEY` reserved for future scope khi/neu có `CommitOnly` model; out-of-scope v0.9.

New reasons (runtime):
- `RC-CTX-INVALID` (with structured details)
- `RC-PAYLOAD-INVALID` (rare)

DX requirements:
- include list of missing fields
- include nearest-match suggestions for unknown field names (edit distance)
- include expected schema skeleton snippet

---

## 9) Packs updates required in v0.9

All v0.7/v0.8 std packs must add schemas:

- `std.fs.*` ctx/payload schema typed
- `std.kv.*` ctx/payload schema typed
- `std.time.*` ctx/payload schema typed
- `std.ui.*` ctx/payload schema typed
- `std.game.*` ctx/payload schema typed
- `std.shadow.*` ctx/payload schema typed
- `std.net.http.*` ctx/payload schema typed
- `std.proc.exec` ctx/payload schema typed
- `std.time.wallclock.*` ctx/payload schema typed

Note:
- For nondet packs (quarantine), determinism_class = cassette-based.

---

## 10) Tooling support (v0.9)

### 10.1 Auto-doc generation (optional but high value)
- `ocp doc packs` outputs:
  - key list
  - ctx schema
  - payload schema
  - permission class
  - examples

### 10.2 Schema conformance tests
- Each pack has tests that:
  - validate schema matches runtime encoder/decoder
  - ensure payload serialization stable (for signature)

### 10.3 Module -> Crate -> Path mapping (LOCKED)
- Schema types/validator/typecheck/runtime enforcement:
  - crate: `projects/ocp/crates/ocp-runtime-core`
  - source of truth: core engine modules re-exported by runtime-core
- Manifest compatibility knobs (`compat.ctx_string`, `compat.ctx_extra_fields`):
  - crate: `projects/ocp/crates/ocp-sdk`
- CLI warnings/hints/fixers and developer UX:
  - crate: `projects/ocp/crates/ocp-cli`
- Rule:
  - Không map được Module -> Crate -> Path thì không mở implementation gate tương ứng.

---

## 11) Execution gates v0.9 (triển khai tuần tự)

### Gate 9-A — Schema core (types + validator + pretty-printer)
Scope:
- implement SchemaType + FieldSpec
- implement runtime validator (Value -> SchemaType)
- implement schema pretty printer (for hints/docs)
Tests:
- `tests/schema_validate.rs`
- `tests/schema_pretty.rs`
Exit criteria:
- validator catches missing/unknown/type mismatch
- pretty printer outputs stable skeleton

### Gate 9-B — Type system: structural records + Result4<T>
Scope:
- extend types: Record types, open rows
- extend parser grammar for destructuring:
  - `let {a, b} = expr;`
- add AST pattern nodes for destructuring binding
- typed Result4
- destructuring typing
Tests:
- `tests/type_record.rs`
- `tests/type_result4.rs`
- `tests/parser_destructure.rs`
Exit criteria:
- record access/destructure works
- try extracts typed payload

### Gate 9-C — Ctx schema enforcement at typecheck
Scope:
- link registry key -> ctx schema
- enforce required/unknown/type mismatch
- constraint checks for literals
Tests:
- `tests/type_ctx_schema.rs`
Exit criteria:
- KPI-1 progress: most ctx errors caught at typecheck

### Gate 9-D — Effect typing / commit policy checks
Scope:
- registry key kind (ObserveOnly/ObserveAndCommit)
- typecheck commit/observe usage
Tests:
- `tests/type_effects.rs`
Exit criteria:
- commit observe-only key rejected compile-time

### Gate 9-E1 — Registry schema-first refactor (fs/kv/time)
Scope:
- add ctx/payload schema cho `std.fs.*`, `std.kv.*`, `std.time.*`
- update runtime adapters to use validator (defense-in-depth)
Tests:
- `tests/pack_schema_conformance_core.rs`
Exit criteria:
- core tool-grade packs pass schema conformance

### Gate 9-E2 — Registry schema-first refactor (ui/game/shadow)
Scope:
- add ctx/payload schema cho `std.ui.*`, `std.game.*`, `std.shadow.*`
- update runtime adapters to use validator (defense-in-depth)
Tests:
- `tests/pack_schema_conformance_consumer.rs`
Exit criteria:
- consumer packs pass schema conformance

### Gate 9-E3 — Registry schema-first refactor (net/proc/wallclock)
Scope:
- add ctx/payload schema cho `std.net.http.*`, `std.proc.exec`, `std.time.wallclock.*`
- quarantine/cassette path vẫn giữ deterministic-by-recording contracts
Tests:
- `tests/pack_schema_conformance_quarantine.rs`
Exit criteria:
- nondet packs pass schema conformance + quarantine contracts

### Gate 9-F — Migration controls + warnings
Scope:
- manifest `compat.ctx_string`
- manifest `compat.ctx_extra_fields`
- warnings/hints for ctx("...") on std keys
Tests:
- `tests/compat_ctx_string.rs`
- `tests/compat_ctx_extra_fields.rs`
Exit criteria:
- new templates default deny ctx-string for std keys

### Gate 9-G — Docs & examples + benchmark
Scope:
- optional `ocp doc packs`
- update templates to typed ctx records
- run benchmark: ctx errors caught compile-time
Tests:
- `tests/cli_doc_packs.rs` (optional)
Exit criteria:
- KPI-2/3 improved: less runtime breakage, better hints

---

## 12) CI Commands (v0.9)
- `cargo test`
- `cargo test --test schema_validate`
- `cargo test --test type_record`
- `cargo test --test type_result4`
- `cargo test --test type_ctx_schema`
- `cargo test --test type_effects`
- `cargo test --test parser_destructure`
- `cargo test --test pack_schema_conformance_core`
- `cargo test --test pack_schema_conformance_consumer`
- `cargo test --test pack_schema_conformance_quarantine`
- `cargo test --test compat_ctx_string`
- `cargo test --test compat_ctx_extra_fields`
- `cargo test --test cli_doc_packs`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt -- --check`

---

## 13) Execution Log (template)

### YYYY-MM-DD — 9-X Planning Freeze
- Date:
- Gate/Step:
- Why:
- Scope:
- Expected tests:
- Exit criteria:

### YYYY-MM-DD — 9-X Implementation Closeout
- Date:
- Gate/Step:
- Implemented:
- Files changed:
- Commands run:
- Test results:
  - Targeted tests (must-pass for gate):
    - `PASS` / `IN_PROGRESS`
  - Regression tests (supporting only):
    - `PASS` / `IN_PROGRESS`
  - Kết luận gate:
    - `DONE` chỉ khi targeted tests pass đúng phạm vi gate.
- Design alignment:
  - `FULL` hoặc `PARTIAL` (nêu rõ phần bất khả thi + phương án thay thế).
- Notes/risks:

---

### 2026-03-04 — 9-A Planning Freeze
- Date:
  - 2026-03-04
- Gate/Step:
  - 9-A
- Why:
  - Mở triển khai schema core để khóa contract validator/pretty-printer trước khi đi vào type-system của 9-B.
- Scope:
  - Thêm `SchemaType`, `FieldSpec`, constraint model tối thiểu.
  - Thêm runtime validator `Value -> SchemaType`.
  - Thêm pretty-printer + skeleton renderer ổn định.
  - Thêm test targeted cho validator/pretty.
- Expected tests:
  - `cargo test --test schema_validate --test schema_pretty`
  - `cargo test`
  - `cargo fmt -- --check`
- Exit criteria:
  - Validator bắt đúng `missing/unknown/type mismatch`.
  - Pretty/skeleton output ổn định, deterministic.
  - Targeted tests pass và không làm vỡ full regression.

### 2026-03-04 — 9-A Implementation Closeout
- Date:
  - 2026-03-04
- Gate/Step:
  - 9-A
- Implemented:
  - Thêm module schema core:
    - `SchemaType`, `FieldSpec`, `FieldConstraints`.
    - `SchemaIssue`, `SchemaIssueCode`.
    - `validate_schema_value(...)`.
    - `pretty_schema(...)` và `schema_skeleton(...)`.
  - Export schema API qua:
    - `src/ocp/mod.rs`
    - `projects/ocp/crates/ocp-runtime-core/src/lib.rs`
  - Bổ sung test targeted cho gate:
    - `tests/schema_validate.rs`
    - `tests/schema_pretty.rs`
- Files changed:
  - `src/ocp/schema.rs`
  - `src/ocp/mod.rs`
  - `projects/ocp/crates/ocp-runtime-core/src/lib.rs`
  - `tests/schema_validate.rs`
  - `tests/schema_pretty.rs`
  - `OCP-MVP-PLAN-v0.9.md`
- Commands run:
  - `cargo test --test schema_validate --test schema_pretty`
  - `cargo test`
  - `cargo fmt`
  - `cargo fmt -- --check`
- Test results:
  - Targeted tests (must-pass for gate):
    - `schema_pretty`: 2/2 pass.
    - `schema_validate`: 3/3 pass.
  - Regression tests (supporting only):
    - `cargo test`: toàn bộ suite pass.
  - Kết luận gate:
    - `DONE`.
- Design alignment:
  - `FULL`
- Notes/risks:
  - Regex constraint ở 9-A dùng matching tối thiểu theo pattern string để giữ zero-dependency; có thể nâng lên regex engine đầy đủ ở gate sau nếu cần.

### 2026-03-04 — 9-B Planning Freeze
- Date:
  - 2026-03-04
- Gate/Step:
  - 9-B
- Why:
  - Khóa parser/AST/type system cho record + destructuring để mở đường cho ctx schema enforcement ở 9-C mà không phải đoán shape dữ liệu.
- Scope:
  - Thêm AST pattern cho `let` destructuring.
  - Thêm parse record literal `{ a: expr }` song song với map literal key-string.
  - Nâng typecheck cho record typing + destructuring typing.
  - Nâng executor để bind destructuring runtime và evaluate record literal.
  - Thêm test targeted cho parser/type record/type result4.
- Expected tests:
  - `cargo test --test parser_destructure --test type_record --test type_result4`
  - `cargo test --test ocp_parser --test ocp_typecheck --test ocp_exec`
  - `cargo test`
  - `cargo fmt -- --check`
- Exit criteria:
  - Destructuring parse/type/runtime chạy đúng trên case hợp lệ.
  - Case sai (thiếu field, destructure non-record) bị chặn ở typecheck.
  - Full regression không bị vỡ.

### 2026-03-04 — 9-B Implementation Closeout
- Date:
  - 2026-03-04
- Gate/Step:
  - 9-B
- Implemented:
  - AST:
    - thêm `LetPattern::{Ident,Record}`.
    - đổi `Stmt::Let` sang `pattern`.
    - thêm `Expr::Record`.
  - Parser:
    - parse destructuring `let {a, b} = expr;`.
    - parse record literal `{ a: expr }`.
    - giữ map literal string-key `{ "a": expr }` để tương thích ngược.
  - Type system + typecheck:
    - thêm `Type::Record`.
    - bind destructuring theo kiểu record/map.
    - field access trên `Type::Record` trả đúng field type khi có.
    - `len/keys/merge` chấp nhận `record` như map-compatible value.
  - Executor:
    - bind runtime cho `LetPattern::Record`.
    - evaluate `Expr::Record` thành `Value::Map`.
    - cập nhật condition budget walker để bao phủ `Expr::Record`.
  - Test targeted mới:
    - `tests/parser_destructure.rs`
    - `tests/type_record.rs`
    - `tests/type_result4.rs`
- Files changed:
  - `src/ocp/ast.rs`
  - `src/ocp/parse.rs`
  - `src/ocp/typecheck.rs`
  - `src/ocp/types.rs`
  - `src/ocp/exec.rs`
  - `src/ocp/mod.rs`
  - `tests/parser_destructure.rs`
  - `tests/type_record.rs`
  - `tests/type_result4.rs`
  - `OCP-MVP-PLAN-v0.9.md`
- Commands run:
  - `cargo test --test parser_destructure --test type_record --test type_result4`
  - `cargo test --test ocp_parser --test ocp_typecheck --test ocp_exec`
  - `cargo test`
  - `cargo fmt`
  - `cargo fmt -- --check`
- Test results:
  - Targeted tests (must-pass for gate):
    - `parser_destructure`: 2/2 pass.
    - `type_record`: 3/3 pass.
    - `type_result4`: 3/3 pass.
  - Regression tests (supporting only):
    - `ocp_parser`: 6/6 pass.
    - `ocp_typecheck`: 6/6 pass.
    - `ocp_exec`: 9/9 pass.
    - `cargo test`: toàn bộ suite pass.
  - Kết luận gate:
    - `DONE`.
- Design alignment:
  - `FULL`
- Notes/risks:
  - 9-B giữ dạng structural typing tối thiểu cho record; open-row và policy nâng cao sẽ tiếp tục được khóa chi tiết ở các gate sau.

### 2026-03-04 — 9-C Planning Freeze
- Date:
  - 2026-03-04
- Gate/Step:
  - 9-C
- Why:
  - Đưa ctx schema check lên typecheck cho key literal để chặn sớm lỗi thiếu field/sai field/sai kiểu trước runtime.
- Scope:
  - Nối `registry key -> ctx schema`.
  - Typecheck `observe(...)` chấp nhận `ctx(...)` legacy và record/map, nhưng enforce schema khi có thông tin tĩnh đủ.
  - Ánh xạ lỗi schema sang error code `T-CTX-*`.
  - Thêm test targeted `type_ctx_schema`.
- Expected tests:
  - `cargo test --test type_ctx_schema`
  - `cargo test --test ocp_parser --test ocp_typecheck --test ocp_exec`
  - `cargo test`
  - `cargo fmt -- --check`
- Exit criteria:
  - Key literal + ctx literal sai schema bị chặn ở typecheck với mã `T-CTX-*`.
  - Dynamic key path giữ runtime validation (không over-block compile-time).
  - Full regression không vỡ.

### 2026-03-04 — 9-C Implementation Closeout
- Date:
  - 2026-03-04
- Gate/Step:
  - 9-C
- Implemented:
  - Registry:
    - thêm `ctx_schemas` và API:
      - `set_ctx_schema_for_key(...)`
      - `ctx_schema_for_key(...)`
    - seed schema cho các key đại diện (`std.fs.read_text`, `std.fs.list_dir`, `std.net.http.request`).
  - Typecheck:
    - `observe(...)` cho phép ctx ở dạng `ctx(...)` legacy hoặc record/map.
    - nếu `key` literal và có schema: enforce ctx schema tại typecheck.
    - map schema issues sang canonical codes:
      - `T-CTX-MISSING-FIELD`
      - `T-CTX-UNKNOWN-FIELD`
      - `T-CTX-TYPE-MISMATCH`
      - `T-CTX-CONSTRAINT-VIOLATION`
    - bổ sung hint schema skeleton cho lỗi.
    - giữ dynamic-key path ở runtime-validation (không đánh dấu DONE giả bằng over-block).
  - Diagnostics:
    - thêm error codes `T-CTX-*` và `T-KEY-NOT-LITERAL-FOR-SCHEMA`.
  - Tests:
    - thêm `tests/type_ctx_schema.rs` (6 cases: pass + 5 nhánh lỗi/compat).
- Files changed:
  - `src/ocp/diag.rs`
  - `src/ocp/registry.rs`
  - `src/ocp/typecheck.rs`
  - `tests/type_ctx_schema.rs`
  - `OCP-MVP-PLAN-v0.9.md`
- Commands run:
  - `cargo test --test type_ctx_schema`
  - `cargo test --test ocp_parser --test ocp_typecheck --test ocp_exec`
  - `cargo test --test type_ctx_schema --test ocp_diag --test ocp_typecheck --test ocp_exec`
  - `cargo test`
  - `cargo fmt`
  - `cargo fmt -- --check`
  - `cargo test --test type_ctx_schema --test ocp_typecheck --test ocp_exec`
- Test results:
  - Targeted tests (must-pass for gate):
    - `type_ctx_schema`: 6/6 pass.
  - Regression tests (supporting only):
    - `ocp_diag`: 5/5 pass.
    - `ocp_parser`: 6/6 pass.
    - `ocp_typecheck`: 6/6 pass.
    - `ocp_exec`: 9/9 pass.
    - `cargo test`: toàn bộ suite pass.
  - Kết luận gate:
    - `DONE`.
- Design alignment:
  - `FULL`
- Notes/risks:
  - `ctx("...")` legacy vẫn được giữ để không phá tương thích ở lane hiện tại; policy siết mạnh hơn theo `compat.ctx_string` đã được hoàn thiện tại gate `9-F`.

### 2026-03-04 — 9-D Planning Freeze
- Date:
  - 2026-03-04
- Gate/Step:
  - 9-D
- Why:
  - Khóa effect typing để chặn sớm `commit(...)` trên observe-only key, giảm lệ thuộc runtime deny path và tăng tính kiểm chứng ở compile-time.
- Scope:
  - Bổ sung key-kind API cho registry (`ObserveOnly` / `ObserveAndCommit`).
  - Mở rộng metadata của binding từ `observe(...)` trong typecheck (key literal + commit capability).
  - Enforce `T-COMMIT-FORBIDDEN-KEY` khi `commit(...)` dùng kết quả từ observe-only key literal.
  - Giữ dynamic-key path ở runtime validation (không over-block compile-time).
  - Thêm test targeted `tests/type_effects.rs`.
- Expected tests:
  - `cargo test --test type_effects`
  - `cargo test --test ocp_typecheck --test ocp_commit_policy --test ocp_exec`
  - `cargo test`
  - `cargo fmt -- --check`
- Exit criteria:
  - `commit` trên observe-only key literal bị chặn ở typecheck với mã `T-COMMIT-FORBIDDEN-KEY`.
  - `commit` trên key commit-capable vẫn pass typecheck.
  - Dynamic-key path vẫn compile-time pass và để runtime quyết định.
  - Full regression không bị vỡ.

### 2026-03-04 — 9-D Implementation Closeout
- Date:
  - 2026-03-04
- Gate/Step:
  - 9-D
- Implemented:
  - Registry:
    - thêm `KeyCapabilityKind::{ObserveOnly, ObserveAndCommit}`.
    - thêm API:
      - `set_key_kind_for_key(...)`
      - `key_kind_for_key(...)`
    - giữ tương thích ngược qua `commit_allowed_for_key(...)`.
  - Typecheck:
    - mở rộng `VarInfo` để giữ metadata observe:
      - `observe_key_literal`
      - `observe_commit_allowed`
    - tại `Stmt::Observe`, với key literal:
      - gắn key literal,
      - gắn commit capability từ registry.
    - tại `check_commit_expr(...)`:
      - chặn compile-time bằng `T-COMMIT-FORBIDDEN-KEY` khi key observe-only.
      - giữ dynamic-key path không chặn sớm.
  - Diagnostics + exports:
    - thêm `ErrorCode::TCommitForbiddenKey` (`T-COMMIT-FORBIDDEN-KEY`).
    - export `KeyCapabilityKind` qua `mod.rs`.
  - Tests:
    - thêm `tests/type_effects.rs` (4 cases targeted).
    - cập nhật `tests/ocp_stdlib.rs` để case `std.db.query_int` commit-forbidden được assert ở typecheck (khớp contract 9-D).
- Files changed:
  - `src/ocp/registry.rs`
  - `src/ocp/typecheck.rs`
  - `src/ocp/diag.rs`
  - `src/ocp/mod.rs`
  - `tests/type_effects.rs`
  - `tests/ocp_stdlib.rs`
  - `OCP-MVP-PLAN-v0.9.md`
- Commands run:
  - `cargo test --test type_effects`
  - `cargo test --test ocp_typecheck --test ocp_commit_policy --test ocp_exec`
  - `cargo test --test type_effects --test ocp_stdlib`
  - `cargo test`
  - `cargo fmt`
  - `cargo fmt -- --check`
- Test results:
  - Targeted tests (must-pass for gate):
    - `type_effects`: 4/4 pass.
    - `ocp_stdlib`: 13/13 pass.
  - Regression tests (supporting only):
    - `ocp_typecheck`: 6/6 pass.
    - `ocp_commit_policy`: 4/4 pass.
    - `ocp_exec`: 9/9 pass.
    - `cargo test`: toàn bộ suite pass.
    - `cargo fmt -- --check`: pass.
  - Kết luận gate:
    - `DONE`.
- Design alignment:
  - `FULL`
- Notes/risks:
  - Có một khác biệt hành vi so với trước gate 9-D: một số case `commit-forbidden` được chặn sớm ở typecheck thay vì đợi runtime; đây là thay đổi chủ đích theo effect-typing contract.

### 2026-03-04 — 9-E1 Planning Freeze
- Date:
  - 2026-03-04
- Gate/Step:
  - 9-E1
- Why:
  - Khóa schema-first cho pack core (`std.fs.*`, `std.kv.*`, `std.time.*`) để chuẩn hóa contract ctx/payload và thêm defense-in-depth ở runtime cho lane hiện tại.
- Scope:
  - Bổ sung ctx schema + payload schema cho nhóm core keys trong registry.
  - Bổ sung API payload schema lookup/set theo key pattern.
  - Runtime observe path:
    - validate ctx theo schema trước khi gọi adapter core,
    - validate payload theo schema sau khi adapter core trả kết quả.
  - Giới hạn phạm vi runtime schema enforcement cho core packs của gate `9-E1`, không mở rộng sang nondet packs.
  - Thêm test targeted `tests/pack_schema_conformance_core.rs`.
- Expected tests:
  - `cargo test --test pack_schema_conformance_core`
  - `cargo test --test std_fs_observe --test std_fs_commit --test std_kv --test std_time`
  - `cargo test`
  - `cargo fmt -- --check`
- Exit criteria:
  - Registry có đủ ctx/payload schema cho core keys theo scope.
  - Dynamic-key path vẫn được runtime schema validation cho core keys.
  - Core pack suites vẫn xanh, không gây regression ngoài scope gate.

### 2026-03-04 — 9-E1 Implementation Closeout
- Date:
  - 2026-03-04
- Gate/Step:
  - 9-E1
- Implemented:
  - Registry schema-first core rollout:
    - thêm `payload_schemas` vào `CapabilityRegistry`.
    - thêm API:
      - `set_payload_schema_for_key(...)`
      - `payload_schema_for_key(...)`
    - thêm helper chọn schema theo pattern dài nhất (`best_schema_for_key(...)`), tái dùng cho cả ctx/payload.
    - seed ctx/payload schema cho nhóm core:
      - `std.fs.*` (read_text/list_dir/stat/write_text/mkdir/remove/rename/read/write/list)
      - `std.kv.*` (get/keys/put/del/clear)
      - `std.time.*` core (`now`, `tick_info`, `now_logical`, `sleep` cho ctx schema)
  - Runtime defense-in-depth cho core packs:
    - thêm runtime ctx schema validator trên observe path.
    - thêm runtime payload schema validator sau adapter result.
    - normalize `Value::Payload` về schema value trước khi validate.
    - giới hạn enforcement cho key thuộc core packs (`std.fs.*`, `std.kv.*`, `std.time.*`) để tránh ảnh hưởng gate khác.
  - Tests:
    - thêm `tests/pack_schema_conformance_core.rs` (4 cases targeted):
      - registry schema presence
      - dynamic-key runtime ctx validation
      - payload schema conformance cho `std.time.tick_info`
      - payload schema conformance cho `std.kv.keys`
- Files changed:
  - `src/ocp/registry.rs`
  - `src/ocp/exec.rs`
  - `tests/pack_schema_conformance_core.rs`
  - `OCP-MVP-PLAN-v0.9.md`
- Commands run:
  - `cargo test --test pack_schema_conformance_core`
  - `cargo test --test std_fs_observe --test std_fs_commit --test std_kv --test std_time`
  - `cargo test --test cli_tool_http_e2e`
  - `cargo test`
  - `cargo fmt`
  - `cargo fmt -- --check`
- Test results:
  - Targeted tests (must-pass for gate):
    - `pack_schema_conformance_core`: 4/4 pass.
    - core pack suites (`std_fs_observe`, `std_fs_commit`, `std_kv`, `std_time`): 16/16 pass.
  - Regression tests (supporting only):
    - `cli_tool_http_e2e`: 1/1 pass.
    - `cargo test`: toàn bộ suite pass.
    - `cargo fmt -- --check`: pass.
  - Kết luận gate:
    - `DONE`.
- Design alignment:
  - `FULL`
- Notes/risks:
  - Runtime schema enforcement được khóa theo scope core của `9-E1`; schema enforcement cho nhóm nondet/consumer tiếp tục ở `9-E2/9-E3`.

### 2026-03-04 — 9-E2 Planning Freeze
- Date:
  - 2026-03-04
- Gate/Step:
  - 9-E2
- Why:
  - Mở rộng schema-first rollout sang nhóm consumer (`std.ui.*`, `std.game.*`, `std.shadow.*`) để contract ctx/payload nhất quán và giảm lỗi runtime do payload shape drift.
- Scope:
  - Bổ sung ctx schema + payload schema cho toàn bộ key consumer trong registry.
  - Mở runtime schema enforcement sang nhóm consumer keys.
  - Thêm test targeted `tests/pack_schema_conformance_consumer.rs`.
  - Giữ nguyên semantics runtime hiện có, chỉ khóa contract schema theo payload thực tế của adapter.
- Expected tests:
  - `cargo test --test pack_schema_conformance_consumer`
  - `cargo test --test std_ui --test std_game --test std_shadow --test engine_ui --test engine_game --test pack_schema_conformance_consumer`
  - `cargo test`
  - `cargo fmt -- --check`
- Exit criteria:
  - Registry có đủ ctx/payload schema cho nhóm consumer.
  - Runtime schema enforcement cho consumer không gây regression các suite `std_ui/std_game/std_shadow/engine_ui/engine_game`.
  - `pack_schema_conformance_consumer` pass đầy đủ.

### 2026-03-04 — 9-E2 Implementation Closeout
- Date:
  - 2026-03-04
- Gate/Step:
  - 9-E2
- Implemented:
  - Registry schema rollout cho consumer packs:
    - thêm ctx/payload schema cho `std.ui.frame_info`, `std.ui.input`, `std.ui.draw`, `std.ui.present`,
      `std.game.tick_info`, `std.game.rng`, `std.game.state_delta`,
      `std.shadow.run`, `std.shadow.compare`.
  - Runtime schema enforcement:
    - mở `is_core_pack_schema_key(...)` để validate runtime cho nhóm `std.ui.*`, `std.game.*`, `std.shadow.*`.
  - Bổ sung test targeted:
    - thêm `tests/pack_schema_conformance_consumer.rs` (schema presence + runtime ctx/payload validation).
  - Sửa tương thích payload thực tế của `std.shadow.run`:
    - mở rộng `state_summary` value union để chấp nhận scalar + map/list bounded, khớp payload branch variant hiện tại.
- Files changed:
  - `src/ocp/registry.rs`
  - `src/ocp/exec.rs`
  - `tests/pack_schema_conformance_consumer.rs`
  - `OCP-MVP-PLAN-v0.9.md`
- Commands run:
  - `cargo test --test std_shadow -- --nocapture`
  - `cargo test --test std_shadow`
  - `cargo test --test pack_schema_conformance_consumer`
  - `cargo test --test std_ui --test std_game --test std_shadow --test engine_ui --test engine_game --test pack_schema_conformance_consumer`
  - `cargo test`
  - `cargo fmt`
  - `cargo fmt -- --check`
  - `cargo test --test std_shadow --test pack_schema_conformance_consumer`
- Test results:
  - Targeted tests (must-pass for gate):
    - `pack_schema_conformance_consumer`: 5/5 pass.
    - `std_ui`: 4/4 pass.
    - `std_game`: 5/5 pass.
    - `std_shadow`: 5/5 pass.
    - `engine_ui`: 2/2 pass.
    - `engine_game`: 3/3 pass.
  - Regression tests (supporting only):
    - `cargo test`: toàn bộ suite pass.
    - `cargo fmt -- --check`: pass.
  - Kết luận gate:
    - `DONE`.
- Design alignment:
  - `FULL`
- Notes/risks:
  - Có lỗi tạm thời ở nhánh `std.shadow.run` khi payload schema chưa bao quát `state_summary.variant` dạng object; đã xử lý bằng schema union bounded map/list và re-run targeted suites.

### 2026-03-04 — 9-E3 Planning Freeze
- Date:
  - 2026-03-04
- Gate/Step:
  - 9-E3
- Why:
  - Hoàn tất schema-first rollout cho nhóm quarantine (`std.net.http.*`, `std.proc.exec`, `std.time.wallclock.*`) để contract ctx/payload nhất quán với runtime nondet-by-recording.
- Scope:
  - Bổ sung ctx/payload schema còn thiếu cho key quarantine.
  - Mở runtime schema enforcement sang `std.proc.*` và `std.net.http.*`.
  - Cập nhật ctx schema `std.net.http.request` để bao quát `max_body_bytes` đã dùng ở runtime.
  - Thêm test targeted `tests/pack_schema_conformance_quarantine.rs`.
- Expected tests:
  - `cargo test --test pack_schema_conformance_quarantine`
  - `cargo test --test std_time_wallclock --test std_proc_exec --test std_net_http --test pack_schema_conformance_quarantine`
  - `cargo test`
  - `cargo fmt -- --check`
- Exit criteria:
  - Registry có đủ ctx/payload schema cho toàn bộ key quarantine trong scope.
  - Runtime schema enforcement không tạo regression cho suite wallclock/proc/net.
  - `pack_schema_conformance_quarantine` pass đầy đủ.

### 2026-03-04 — 9-E3 Implementation Closeout
- Date:
  - 2026-03-04
- Gate/Step:
  - 9-E3
- Implemented:
  - Registry quarantine schema rollout:
    - thêm ctx schema cho `std.time.wallclock.now`.
    - thêm ctx schema cho `std.proc.exec`.
    - mở rộng ctx schema `std.net.http.request` với `max_body_bytes`.
    - thêm payload schema cho:
      - `std.time.wallclock.now`
      - `std.proc.exec`
      - `std.net.http.request`
  - Runtime schema enforcement:
    - mở `is_core_pack_schema_key(...)` cho `std.proc.*` và `std.net.http.*` để validate ctx/payload runtime theo schema.
  - Tests:
    - thêm `tests/pack_schema_conformance_quarantine.rs`:
      - schema presence cho key quarantine,
      - runtime ctx schema validation ở dynamic-key path,
      - payload schema conformance cho wallclock/proc/net.
- Files changed:
  - `src/ocp/registry.rs`
  - `src/ocp/exec.rs`
  - `tests/pack_schema_conformance_quarantine.rs`
  - `OCP-MVP-PLAN-v0.9.md`
- Commands run:
  - `cargo test --test pack_schema_conformance_quarantine`
  - `cargo test --test std_time_wallclock --test std_proc_exec --test std_net_http --test pack_schema_conformance_quarantine`
  - `cargo fmt -- --check`
  - `cargo test`
- Test results:
  - Targeted tests (must-pass for gate):
    - `pack_schema_conformance_quarantine`: 5/5 pass.
    - `std_time_wallclock`: 1/1 pass.
    - `std_proc_exec`: 2/2 pass.
    - `std_net_http`: 3/3 pass.
  - Regression tests (supporting only):
    - `cargo test`: toàn bộ suite pass.
    - `cargo fmt -- --check`: pass.
  - Kết luận gate:
    - `DONE`.
- Design alignment:
  - `FULL`
- Notes/risks:
  - Có lệch tạm thời ở test net do dùng sai tên env var allowlist; đã sửa theo runtime contract (`OCP_STD_NET_ALLOW_HOSTS`, `OCP_STD_NET_ALLOW_METHODS`, `OCP_STD_NET_TIMEOUT_MS`, `OCP_STD_NET_MAX_BODY_BYTES`) và re-run targeted suites.

### 2026-03-04 — 9-F Planning Freeze
- Date:
  - 2026-03-04
- Gate/Step:
  - 9-F
- Why:
  - Hoàn thiện migration controls để chuyển dần từ `ctx("...")` legacy sang typed ctx record mà không gây gãy flow hiện có.
- Scope:
  - Parse manifest compatibility knobs:
    - `compat.ctx_string = allow|warn|deny`
    - `compat.ctx_extra_fields = allow|warn|deny`
  - Nâng typecheck compatibility policy:
    - deny `ctx("...")` trên std keys khi policy `ctx_string=deny`
    - unknown ctx field cho closed schema theo `ctx_extra_fields`
  - Nối policy từ `ocp-sdk` vào `ocp-runtime-core` check/run paths.
  - Cập nhật templates `tool-http|tool-proc|tool-wallclock`:
    - mặc định `[compat].ctx_string = "deny"`
    - chuyển observe ctx sang typed record để tương thích policy deny.
  - Thêm tests targeted:
    - `tests/compat_ctx_string.rs`
    - `tests/compat_ctx_extra_fields.rs`
- Expected tests:
  - `cargo test --test compat_ctx_string --test compat_ctx_extra_fields`
  - `cargo test --test compat_ctx_string --test compat_ctx_extra_fields --test cli_tool_http_e2e --test cli_tool_proc_e2e --test cli_tool_wallclock_e2e`
  - `cargo test --test ocp_manifest --test compat_ctx_string --test compat_ctx_extra_fields`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
- Exit criteria:
  - Knobs compat được parse từ manifest và có hiệu lực trên check/run flow.
  - `ctx_string=deny` chặn legacy `ctx("...")` cho std key với diagnostic canonical.
  - `ctx_extra_fields` điều khiển được closed-schema unknown field path.
  - Targeted tests pass, regression không vỡ.

### 2026-03-04 — 9-F Implementation Closeout
- Date:
  - 2026-03-04
- Gate/Step:
  - 9-F
- Implemented:
  - Typecheck compatibility policy:
    - thêm `CompatMode` + `TypecheckCompatConfig`.
    - thêm `typecheck_program_with_compat(...)`.
    - thêm diagnostics `T-CTX-STRING-COMPAT`.
    - enforce `ctx_string=deny` cho std keys có ctx schema.
    - enforce/relax unknown ctx fields theo `ctx_extra_fields` cho cả literal schema validation và typed-record path.
  - Runtime core wiring:
    - thêm check/run/compile variants nhận `TypecheckCompatConfig`.
    - giữ API cũ để backward compatibility, route qua default compat config.
  - Runtime observe ctx:
    - mở `observe(...)` executor để nhận `ctx(...)` hoặc record/map-compatible value (canonical hóa về ctx literal deterministic trước khi vào adapter).
  - SDK wiring:
    - mở `ProjectLanguageConfigV071` với:
      - `compat_ctx_string`
      - `compat_ctx_extra_fields`
    - parse `[compat]` từ `Ocp.toml`.
    - nối compat policy vào `check_project_with_lock`, `run_project_with_*`, `test_project_with_lock`, reactor/trace paths.
  - Tests targeted:
    - thêm `tests/compat_ctx_string.rs` (parse + warn path + deny path).
    - thêm `tests/compat_ctx_extra_fields.rs` (default warn + allow + deny path).
  - CLI templates:
    - cập nhật `tool-http`, `tool-proc`, `tool-wallclock` manifest mặc định `compat.ctx_string="deny"`.
    - cập nhật source template sang typed ctx record.
    - bổ sung assert trong e2e để khóa template contract.
- Files changed:
  - `src/ocp/typecheck.rs`
  - `src/ocp/diag.rs`
  - `src/ocp/exec.rs`
  - `src/ocp/mod.rs`
  - `projects/ocp/crates/ocp-cli/src/main.rs`
  - `projects/ocp/crates/ocp-runtime-core/src/lib.rs`
  - `projects/ocp/crates/ocp-sdk/src/lib.rs`
  - `tests/cli_tool_http_e2e.rs`
  - `tests/cli_tool_proc_e2e.rs`
  - `tests/cli_tool_wallclock_e2e.rs`
  - `tests/compat_ctx_string.rs`
  - `tests/compat_ctx_extra_fields.rs`
  - `OCP-MVP-PLAN-v0.9.md`
- Commands run:
  - `cargo test --test compat_ctx_string --test compat_ctx_extra_fields`
  - `cargo test --test compat_ctx_string --test compat_ctx_extra_fields --test cli_tool_http_e2e --test cli_tool_proc_e2e --test cli_tool_wallclock_e2e`
  - `cargo test --test ocp_manifest --test compat_ctx_string --test compat_ctx_extra_fields`
  - `cargo fmt --all`
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`
- Test results:
  - Targeted tests (must-pass for gate):
    - `compat_ctx_string`: 3/3 pass.
    - `compat_ctx_extra_fields`: 3/3 pass.
    - `cli_tool_http_e2e`: 1/1 pass.
    - `cli_tool_proc_e2e`: 1/1 pass.
    - `cli_tool_wallclock_e2e`: 1/1 pass.
  - Regression tests (supporting only):
    - `ocp_manifest`: 3/3 pass.
    - `cargo test`: toàn bộ suite pass.
    - `cargo fmt --all -- --check`: pass.
    - `cargo clippy --all-targets -- -D warnings`: pass.
  - Kết luận gate:
    - `DONE`.
- Design alignment:
  - `FULL`
- Notes/risks:
  - Sự cố tạm thời trong quá trình đóng gate:
    - rustfmt check lúc đầu phản ánh format chưa đồng bộ.
    - clippy nhắc nested-if ở nhánh unknown ctx field.
    - compile-time mismatch do nhầm nhánh scalar (`Value::Null`) trong converter ctx runtime.
  - Đã xử lý dứt điểm toàn bộ các nhánh này, sau đó re-run `fmt` + `clippy` + full test matrix.

### 2026-03-04 — 9-G Planning Freeze
- Date:
  - 2026-03-04
- Gate/Step:
  - 9-G
- Why:
  - Đóng vòng docs/examples/benchmark của v0.9 để giảm runtime breakage bằng tài liệu capability sinh từ schema thật và evidence compile-time catches cho ctx lỗi.
- Scope:
  - Triển khai CLI command `ocp doc packs` (text + `--json`) sinh docs từ `CapabilityRegistry::v1_baseline()`.
  - Bổ sung metadata docs:
    - key list
    - key kind (`observe_only`/`observe_and_commit`)
    - permission class
    - ctx/payload schema
    - example observe + ctx skeleton
  - Bổ sung test targeted:
    - `tests/cli_doc_packs.rs`
  - Re-verify benchmark compile-time ctx catches bằng suites compat:
    - `compat_ctx_string`
    - `compat_ctx_extra_fields`
- Expected tests:
  - `cargo test --test cli_doc_packs --test compat_ctx_string --test compat_ctx_extra_fields`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo test`
- Exit criteria:
  - `ocp doc packs` chạy ổn định, output có đủ fields docs theo scope.
  - `ocp doc packs --json` parseable và chứa known keys quan trọng.
  - Targeted tests pass và full regression không vỡ.

### 2026-03-04 — 9-G Implementation Closeout
- Date:
  - 2026-03-04
- Gate/Step:
  - 9-G
- Implemented:
  - Registry docs surface:
    - thêm `documented_keys()` để lấy key list deterministic.
    - thêm `ctx_required_for_key()` để phục vụ docs/hints.
  - Runtime-core export:
    - re-export `CapabilityRegistry` và `KeyCapabilityKind` để CLI có thể sinh docs trực tiếp từ registry baseline.
  - CLI `doc packs`:
    - thêm command `ocp doc packs [--json]`.
    - output text và JSON đều gồm:
      - key
      - key kind
      - permission class
      - ctx required fields
      - ctx schema
      - payload schema
      - example observe + ctx skeleton.
    - cập nhật `print_help()` thêm mục `doc packs`.
  - Targeted tests:
    - thêm `tests/cli_doc_packs.rs` với 2 test:
      - text output contains required sections.
      - json output contains known pack fields.
  - Benchmark evidence (compile-time catches):
    - re-run `compat_ctx_string` + `compat_ctx_extra_fields` để chứng minh lỗi ctx được bắt sớm theo contract 9-F/9-G.
- Files changed:
  - `src/ocp/registry.rs`
  - `projects/ocp/crates/ocp-runtime-core/src/lib.rs`
  - `projects/ocp/crates/ocp-cli/src/main.rs`
  - `tests/cli_doc_packs.rs`
  - `OCP-MVP-PLAN-v0.9.md`
- Commands run:
  - `cargo fmt --all`
  - `cargo test --test cli_doc_packs --test compat_ctx_string --test compat_ctx_extra_fields`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
  - `cargo test`
- Test results:
  - Targeted tests (must-pass for gate):
    - `cli_doc_packs`: 2/2 pass.
    - `compat_ctx_string`: 3/3 pass.
    - `compat_ctx_extra_fields`: 3/3 pass.
  - Regression tests (supporting only):
    - `cargo clippy --all-targets -- -D warnings`: pass.
    - `cargo fmt --all -- --check`: pass.
    - `cargo test`: toàn bộ suite pass.
  - Kết luận gate:
    - `DONE`.
- Design alignment:
  - `FULL`
- Notes/risks:
  - `ocp doc packs` hiện sinh docs từ baseline registry của runtime; nếu registry schema đổi ở các phiên bản sau thì output docs sẽ đổi tương ứng.
  - Mapping `permission_class` hiện theo prefix rules trong CLI, cần giữ đồng bộ khi mở rộng họ capability mới.

## 14) Checklist khóa trước khi đóng gate
- [x] Gate status đã cập nhật đúng (`TODO/IN_PROGRESS/PARTIAL/DONE`).
- [x] Có đủ `Planning Freeze` + `Implementation Closeout`.
- [x] Có code delta thật cho gate (không chỉ sửa plan).
- [x] `Files changed` khớp code delta thực tế.
- [x] `Commands run` là lệnh đã chạy thật.
- [x] `Targeted tests` đã pass cho đúng phần vừa làm.
- [x] `Regression tests` chỉ ghi vai trò phụ trợ.
- [x] Không còn placeholder `PASS/FAIL` trong closeout.
- [x] Closeout `DONE` không còn marker `FAIL`.
- [x] `Design alignment` đã ghi `FULL` hoặc `PARTIAL` rõ ràng.

---

## 15) Handoff v0.9 -> v0.10
- Chỉ mở v0.10 khi `9-A..9-G` đều `DONE` và KPI v0.9 đã có evidence đầy đủ.
- Nếu còn gate `IN_PROGRESS/PARTIAL`, không mở scope v0.10 ngoài việc xử lý blocker tồn đọng.
- Mọi thay đổi breaking sau v0.9 phải đi kèm migration contract rõ (compat flags, default policy, rollback path).

---
