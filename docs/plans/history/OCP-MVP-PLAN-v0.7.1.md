# OCP v0.7.1 — Expansion Plan (Foundation + DX + Replay-first)

Ngày tạo: 2026-03-03  
Trạng thái: `DONE` (2026-03-04)
Phạm vi: **OCP-only** (không có HAR, không có AI runtime).  
Mục tiêu: Xây nền để OCP trở nên “dùng được cho dev bình thường” (ít boilerplate, ít file sprawl, ít lỗi vặt, DX tốt) nhưng **giữ nguyên** cốt lõi OCP: capability/permission chặt, bounded, deterministic replay, 4-kind everywhere, commit-gated effects.

---

## 0) Split Plan (v0.7.x chia 3 bản)

- **v0.7.1**: Core + DX + Data model + JSON + Modules + Result4 sugar + Loops bounded + Manifest permissions + Replay-first artifacts + CLI tối thiểu.
- **v0.7.2**: Packs “tool-grade”: `std.fs` (sandbox), `std.kv`, `std.time` + template tool (CLI init) + fixture IO.
- **v0.7.3**: Packs “consumer wow”: `std.ui` (draw-list), `std.game`, `std.shadow` primitive + compare report + templates mini-game & shadow-preview; (quarantine lane nếu cần real IO).

### 0.1 Change Classification (LOCKED)

| Item | Tag | Compatibility | Evidence suite | Owner gate |
|---|---|---|---|---|
| Value model (`null/bool/int/string/list/record/map`) | `S` | `additive` | `tests/ocp_value.rs` | `7.1-B` |
| JSON contract (`std.json.parse/stringify`) | `S` | `additive` | `tests/std_json.rs` | `7.1-C` |
| `try/else` sugar | `S` | `additive` | `tests/ocp_sugar.rs` | `7.1-F` |
| `guard` sugar + `guard_mode` | `S` | `additive` | `tests/ocp_sugar.rs`, `tests/ocp_manifest.rs` | `7.1-F`, `7.1-G` |
| Bounded loops (`repeat`, `for ... cap`) | `S` | `additive` | `tests/ocp_loops.rs` | `7.1-E` |
| Module/import loader | `S` | `additive` | `tests/ocp_modules.rs` | `7.1-D` |
| Manifest/permissions + deny/allow precedence | `S` | `additive` | `tests/ocp_manifest.rs` | `7.1-G` |
| Replay-first artifacts + signature rules | `S` | `additive` | `tests/ocp_determinism.rs`, `tests/cli_e2e.rs` | `7.1-A`, `7.1-G` |
| CLI surface cho v0.7.1 (`init/run/test/replay/trace`) | `C` | `additive` | `tests/cli_e2e.rs` | `7.1-G` |
| Workspace mapping (module -> crate -> path) | `I` | `internal` | `docs` conformance review + gate checklist | `7.1-A` |

### 0.2 Trạng thái Workstreams/Gates v0.7.1 (TRACKING)

#### Workstreams
- WS-S (Semantics: value/json/modules/loops/sugar/commit): `DONE` (2026-03-04)
- WS-C (CLI surface + compatibility + artifacts contract): `DONE` (2026-03-04)
- WS-I (Implementation mapping internal: crate/path/doc alignment): `DONE` (2026-03-04)

#### Gate status (7.1-A .. 7.1-G)
- Gate 7.1-A (Core skeleton + diagnostics + audit): `DONE` (2026-03-04)
- Gate 7.1-B (Value model + pure ops + caps): `DONE` (2026-03-04)
- Gate 7.1-C (std.json pure + Result4 integration): `DONE` (2026-03-04)
- Gate 7.1-D (Modules/import loader): `DONE` (2026-03-04)
- Gate 7.1-E (Bounded loops): `DONE` (2026-03-04)
- Gate 7.1-F (try/else + guard sugar): `DONE` (2026-03-04)
- Gate 7.1-G (Manifest + permissions + CLI/replay): `DONE` (2026-03-04)

#### Quy tắc cập nhật trạng thái (bắt buộc)
- Chỉ chuyển `DONE` khi gate có đủ cặp log: `Planning Freeze` + `Implementation Closeout`.
- `Implementation Closeout` bắt buộc có đủ: `Files changed`, `Commands run`, `Test results`, `Notes/risks`.
- Nếu mới verify/chưa có code delta đúng phạm vi gate: chỉ được để `IN_PROGRESS` hoặc `TODO`, không được đánh `DONE`.

---

## 1) Goals v0.7.1 (LOCKED)

### 1.1 North Star

v0.7.1 phải đạt:
- Không còn bắt buộc “match 4-arm mọi chỗ” mới viết được chương trình.
- Có data model + JSON để tool/app xử lý dữ liệu cấu trúc thay vì serialize thủ công.
- Có module/import để tránh dự án nở file glue (mà vẫn kiểm soát determinism).
- Có CLI + template để tạo dự án **ít file**, chạy/test/replay được ngay.
- Permissions manifest rõ ràng; lỗi permission/ctx/type có hint giúp sửa nhanh.
- Mọi run phát `audit + signature + replay config` (replay-first).

### 1.2 KPI bắt buộc (định lượng)

**KPI-1: File sprawl control**
- `ocp init hello-tool` tạo dự án mới **≤ 4 file** (không tính thư mục artifacts):
  - `ocp.toml`
  - `main.ocp`
  - `README.md`
  - (optional) `fixtures/` (nếu demo cần input JSON mẫu; ưu tiên 0–1 file fixture)

**KPI-2: Replay-first**
- `ocp run` luôn tạo `./.ocp_artifacts/<run_id>/` gồm:
  - `audit.jsonl`
  - `signature.txt`
  - `replay.toml`
- `ocp replay <artifact_dir>` tái tạo `signature` **giống hệt** (lane locked).

**KPI-3: LOC reduction enabler**
- Demo xử lý JSON + loops bounded + try/else đạt **~200–300 LOC** (không tính stdlib/packs).

#### Evidence Checklist (machine-checkable)

**KPI-1: File sprawl control**
- Command:
  - `ocp init hello-tool ./tmp/hello-tool`
  - kiểm tra file bằng script lane (đếm file trong root project, không tính `.ocp_artifacts/`)
- Artifacts:
  - `tmp/hello-tool/ocp.toml`
  - `tmp/hello-tool/main.ocp`
  - `tmp/hello-tool/README.md`
  - `tmp/hello-tool/fixtures/*` (0..1 file, optional)
- Pass:
  - Tổng file root của template <= 4 theo rule trên.
- Fail:
  - Vượt 4 file hoặc tạo thêm file governance không nằm trong danh sách.

**KPI-2: Replay-first**
- Command:
  - `ocp run ./tmp/hello-tool --seed 42`
  - `ocp replay ./.ocp_artifacts/<run_id>/`
- Artifacts:
  - `.ocp_artifacts/<run_id>/audit.jsonl`
  - `.ocp_artifacts/<run_id>/signature.txt`
  - `.ocp_artifacts/<run_id>/replay.toml`
- Pass:
  - `ocp replay` exit code = 0 và in `signature match`.
- Fail:
  - signature mismatch hoặc thiếu bất kỳ artifact bắt buộc nào.

**KPI-3: LOC reduction enabler**
- Command:
  - script lane đếm LOC của `main.ocp` (không tính comments/blank lines theo rule tool đã khóa trong lane)
- Artifacts:
  - báo cáo LOC (`loc_report.json` hoặc output lane tương đương)
- Pass:
  - LOC nằm trong range 200..300.
- Fail:
  - LOC ngoài range hoặc thiếu báo cáo đo LOC.

---

## 2) Axis Lock v0.7.x (bất biến không mặc cả)

1) **No naked IO**: IO/effect chỉ qua capability (observe/commit).  
2) **4-kind everywhere**: `OK | DEGRADED | INSUFFICIENT | DEFERRED`.  
3) **Commit-gated effects**: effect chỉ xảy ra khi commit hợp lệ + policy.  
4) **Boundedness**: step cap + enumerate cap + per-call budget.  
5) **Determinism-by-default**: trace signature ổn định.  
6) **Diagnostics structured**: mọi lỗi parse/type/exec/policy có `code + span + hint`.

---

## 3) Scope v0.7.1

### 3.1 In-scope (bắt buộc ship)

A) **Core language foundation**
- Parser/AST/Span/Diagnostics chuẩn hóa.
- Typechecker đủ cho data model + imports + sugar + loops.
- Executor bounded (step cap) + deterministic (stable iteration order).

B) **Data model phổ thông (Value)**
- `null`, `bool`, `int`, `string`
- `record`, `list`, `map`
- access: `x.a`, `x["k"]`, `x[i]`
- pure ops tối thiểu: `len`, `keys cap`, `merge cap`, (optional) `has/get`, `push` (persistent)

C) **JSON stdlib (pure)**
- `std.json.parse(str)` -> `Result4<Value>`
- `std.json.stringify(value)` -> `String`

D) **Module/import tối thiểu**
- `import "std.json" as json;`
- `import "./lib.ocp" as lib;`
- cycle detection; path literal only.

E) **Result4 sugar để giảm boilerplate**
- `try <expr> else { ... }` (desugar xuống match 4-arm)
- `guard <result4>;` (early-exit chuẩn, policy configurable)

F) **Bounded loops**
- `repeat n { ... }`
- `for item in xs cap 100 { ... }`

G) **Permissions manifest + locked lane**
- `ocp.toml` bắt buộc trong lane locked.
- deny/allow precedence rõ.
- Permission errors phải chỉ rõ “bị deny bởi rule nào” + hint sửa manifest.

H) **Replay-first artifacts + CLI tối thiểu**
- `ocp init hello-tool`
- `ocp run`
- `ocp test`
- `ocp replay <artifact_dir>`
- `ocp trace view <audit.jsonl>` (text)

### 3.2 Out-of-scope (defer sang v0.7.2+)

- Packs có IO thật: `std.fs`, `std.kv`, `std.time`, `std.ui`, `std.game`
- Shadow primitive + compare report
- Quarantine lane (real IO env-gated)
- Package registry online / solver phức tạp
- Async/concurrency, optimizer/JIT
- Widget UI framework (OCP chỉ nên xuất draw-list)

---

## 4) Deliverables v0.7.1

1) `docs/OCP-v0.7.1.md` (tài liệu này).  
2) Workspace crates (Rust):
- `ocp-runtime-core`: span/diag/lex/parse/ast/types/typecheck/value/result4/budget/exec/runtime/audit/determinism
- `ocp-sdk`: manifest/permissions/resolver wiring
- `ocp-cli`: command surface `init/run/test/replay/trace`  
3) Pure stdlib:
- `std.json` (in-tree module)
- `std.core` (pure helpers: len/keys/merge) nếu cần  
4) CLI `ocp`:
- init/run/test/replay/trace view  
5) Test suites:
- parser/type/exec
- module/import + cycle detection
- loops cap enforcement
- sugar desugaring equivalence
- determinism signatures
- manifest/permission diagnostics
- CLI e2e (template run/replay)

---

## 5) Architecture v0.7.1 (code layout & interfaces)

### 5.1 Workspace mapping (Module -> Crate -> Path)

| Module | Crate | Path (workspace hiện tại) |
|---|---|---|
| span/diag/lex/ast/parse/types/typecheck | `ocp-runtime-core` | `projects/ocp/crates/ocp-runtime-core/src/*` |
| value/result4/budget/exec/audit/determinism | `ocp-runtime-core` | `projects/ocp/crates/ocp-runtime-core/src/*` |
| runtime observe/commit boundary | `ocp-runtime-core` | `projects/ocp/crates/ocp-runtime-core/src/*` |
| stdlib pure execution (`std.json`, `std.core`) | `ocp-runtime-core` | `projects/ocp/crates/ocp-runtime-core/src/*` |
| manifest parser + permissions + resolver wiring | `ocp-sdk` | `projects/ocp/crates/ocp-sdk/src/*` |
| CLI command UX (`init/run/test/replay/trace`) | `ocp-cli` | `projects/ocp/crates/ocp-cli/src/main.rs` |
| public SDK exports + integration glue | `ocp-sdk` | `projects/ocp/crates/ocp-sdk/src/lib.rs` |

Quy tắc khóa: **Nếu chưa map crate/path thì chưa được triển khai**.
Ghi chú phân loại: mapping workspace ở v0.7.1 là `I/internal` cho mục đích doc alignment, không ép refactor code nếu không cần.

### 5.2 Runtime interface (v0.7.1 minimal)

v0.7.1 chưa ship IO packs, nhưng core vẫn giữ boundary để v0.7.2+ cắm vào không refactor lớn:

- `Runtime::observe(key, tier, ctx, budget) -> Result4<Value>`
- `Runtime::commit(result: &Result4<Value>) -> ExecOutcome`

Gợi ý triển khai v0.7.1:
- `std.json.*` và `std.core.*` là **pure** -> thực thi trực tiếp trong executor.
- Registry/permissions vẫn tồn tại để enforce “default deny” và test DX; resolver/path policy nằm ở `ocp-sdk`.

### 5.3 Migration Contract v0.6 -> v0.7.1

#### 5.3.1 Compatibility mode (lane-based duy nhất)
- Mặc định v0.7.1:
  - `[project] lane = "locked_v071"`
- Compat path v0.6:
  - `[project] lane = "locked_v06"`
- Không mở thêm cơ chế `--profile legacy_v06` trong giai đoạn này.

#### 5.3.2 Additive vs Breaking matrix
| Hạng mục | Phân loại | Ghi chú |
|---|---|---|
| Value model mở rộng + JSON pure + sugar + bounded loops + module/import | Additive | Không phá lane `locked_v06` |
| Permissions DX + replay artifact fields bổ sung | Additive | Giữ reader cũ đọc được artifact hiện hữu |
| CLI v0.7.1 commands | Additive | Không silent break lệnh đang có |
| Thay đổi đường dẫn artifact hiện hữu | Breaking (không áp dụng ở 7.1) | Chỉ được làm nếu có compat reader rõ ràng |

#### 5.3.3 Rollback/compat vận hành
- Cùng project chạy được bằng lane `locked_v06` để replay/đối chiếu hành vi cũ.
- Nếu gate v0.7.1 fail, baseline vận hành quay về lane `locked_v06` mà không cần refactor code runtime.

#### 5.3.4 Deprecation policy
- Mọi thay đổi command/output chỉ được deprecate bằng alias + hint.
- Cấm silent break (đổi hành vi mặc định mà không có thông báo).

---

## 6) Language spec v0.7.1 (cú pháp + semantics)

### 6.1 Value literals

- null: `null`
- bool: `true | false`
- int: `0 | [1-9][0-9]*`
- string: `"..."` (escape tối thiểu `\" \\ \n \t`)
- list: `[expr, expr, ...]`
- record: `{ a: expr, b: expr }`
- map: `{ "k": expr, "k2": expr }` (v0.7.1: key string literal only)

### 6.2 Statements

- `let name = expr;`
- `import "path" as alias;`
- `match r { OK => {...} DEGRADED => {...} INSUFFICIENT => {...} DEFERRED => {...} }`
- `repeat n { ... }`
- `for item in xs cap 100 { ... }`
- `commit(r);` (v0.7.1 đã khóa validation + audit; effect adapters đầy đủ defer v0.7.2+)
- `guard r;` (sugar)
- `return expr;` (v0.7.1: return-from-program, top-level)
- block dùng `{ ... }`

Program return contract (LOCKED):
- Program return type của v0.7.1 là `Result4<Value>`.
- `return <Value>;` được wrap thành `return OK(payload=<Value>);`.
- `return <Result4<Value>>;` giữ nguyên (dùng cho `guard_mode="return"`).

### 6.3 Expressions

- literals, ident
- field access: `x.a`
- index:
  - list: `x[i]`
  - map: `x["k"]`
- calls (v0.7.1 giới hạn):
  - `len(x)`
  - `keys(x, cap)`
  - `merge(a, b, cap)`
  - `budget(n)` -> Budget (giữ shape)
  - `ctx("k=v;...")` -> Ctx (giữ shape)
  - `std.json.parse(s)`
  - `std.json.stringify(v)`

### 6.4 Value semantics (boundedness)

- `keys(map, cap)`:
  - trả list keys với `min(len(keys), cap)`
  - nếu `cap > value_keys_cap` => `X-KEYS-CAP-EXCEEDED`
- `merge(a, b, cap)`:
  - merge deterministic và tính kết quả đầy đủ trước khi kiểm tra cap.
  - nếu `len(keys(result)) > cap` => fail-hard với canonical `X-KEYS-CAP-EXCEEDED`, alias machine-readable `X-LIMIT-EXCEEDED` với `meta.limit_kind="value_keys_cap"`.
  - v0.7.1 không truncate silent kết quả merge.

### 6.5 Result4 model (v0.7.1)

`Result4<Value>` gồm:
- `kind: Kind` (`OK | DEGRADED | INSUFFICIENT | DEFERRED`)
- `payload: Value` (OK/DEGRADED meaningful; kind khác thì payload = null)
- `reason_code: String?` (INSUFFICIENT/DEFERRED meaningful)
- `audit: record` (seed/tick/steps/budget_used… tối thiểu)
- `origin_id: u64` (để commit discipline về sau)

### 6.6 JSON contract (pure)

- `std.json.parse(str)`:
  - ok => `OK(payload=value)`
  - fail => `INSUFFICIENT(reason_code="RC-JSON-INVALID")` (không panic)
- `std.json.stringify(value)`:
  - output deterministic (stable key order)

### 6.7 try/else sugar (giảm boilerplate match)

Syntax:
- `let x = try expr else { ... };`

Rules:
- `expr` phải có type `Result4<Value>`.
- Nếu kind OK/DEGRADED: `x = expr.payload`
- Nếu INSUFFICIENT/DEFERRED: chạy else block, block yield `Value` hoặc `return`.
- else block **luôn** có implicit binding `r` (read-only):
  - `r.kind`, `r.reason_code`, `r.audit`
- Nếu else block yield `Value` => `x` nhận đúng `Value` đó.
- Else block không được rơi ra cuối block mà không yield/return; vi phạm => `T-TRY-ELSE-NO-VALUE`.
Desugar:
- compile-time transform thành match 4-arm.

### 6.8 guard sugar (early-exit)

Syntax:
- `guard r;`

Rules:
- `r` phải có type `Result4<Value>`.
- Nếu OK/DEGRADED: continue.
- Nếu INSUFFICIENT/DEFERRED: early exit theo config từ `ocp.toml`:
  - `guard_mode="return"`: `return r;`
  - `guard_mode="error"`: raise `X-GUARD-FAILED` (include reason_code)
Desugar:
- thành match + return/error.

### 6.9 Bounded loops

- `repeat n { ... }`
  - `n <= loop_cap` (else `X-LOOP-CAP-EXCEEDED`)
  - mỗi iteration tăng step
- `for item in xs cap C { ... }`
  - `xs` phải là list
  - `C` bắt buộc int literal trong v0.7.1
  - iterate `min(len(xs), C)`
  - nếu `C > loop_cap` => `X-LOOP-CAP-EXCEEDED`

### 6.10 Sugar invariance contract (LOCKED)

- Mọi chương trình dùng `try/else` hoặc `guard` phải có canonical form sau desugar.
- Tiêu chí pass của gate sugar:
  - chương trình sugar và canonical-desugared phải cho cùng `signature`.
- Nếu signature khác nhau:
  - coi là FAIL semantics invariance, không được đóng gate.

### 6.11 Commit discipline v0.7.1 (không mâu thuẫn baseline)

- `commit(r)` chỉ hợp lệ khi:
  - `r: Result4<Value>`
  - `r.kind ∈ {OK, DEGRADED}`
  - `origin_id` hợp lệ theo observe-origin contract.
- Pure origins (`origin_id = 0`) mặc định `commit_allowed = false`.
- Dù chưa có side-effect adapters ở v0.7.1, commit vẫn phải emit audit event:
  - `CommitAttempt`
  - `CommitDenied` hoặc `CommitAccepted(no_effect)`
- Quy tắc này giữ tương thích với trục `commit-gated effects` từ các bản trước.

---

## 7) Permissions + Manifest (locked lane)

### 7.1 `ocp.toml` schema (v0.7.1 minimal)

Ví dụ:

```toml
[project]
name = "hello-tool"
entry = "main.ocp"
lane = "locked_v071"

[limits]
step_cap = 20000
loop_cap = 10000
value_keys_cap = 2000

[language]
guard_mode = "return"

[permissions]
std_json = true
std_core = true

[deny]
patterns = ["std.net.*", "std.fs.*"]
```

### 7.2 Yêu cầu

- Lane `locked_v071` và `locked_v06` đều bắt buộc có `ocp.toml`.
- Default lane của v0.7.1 là `locked_v071`.
- Compat lane để replay hành vi cũ là `locked_v06`.
- Default deny: nếu capability không explicit allow => deny.
- deny patterns ưu tiên cao hơn allow.
- `guard_mode` đọc từ `[language]`:
  - `return` hoặc `error`
  - default = `return`.

### 7.3 Precedence rules

- deny > allow
- explicit project deny override mọi allow
- unknown capability => deny với `RC-CAPABILITY-DENIED`

### 7.4 Permission error DX requirements

Permission error phải render:
- capability requested (pattern hoặc key)
- deny rule matched (pattern nào)
- allow status (có/không)
- vị trí manifest (line/col nếu có)
- hint:
  - “Add to [permissions] …” hoặc “Remove from [deny] …”
  - nếu deny mặc định do missing allow => hint add allow

---

## 8) Replay-first artifacts (bắt buộc)

### 8.1 Output directory convention

`ocp run` tạo:
- `./.ocp_artifacts/<run_id>/audit.jsonl`
- `./.ocp_artifacts/<run_id>/signature.txt`
- `./.ocp_artifacts/<run_id>/replay.toml`

run_id dùng counter timestamp-agnostic để determinism không phụ thuộc thời gian. Không dùng timestamp trong signature input.

### 8.2 Audit format (JSONL)

Mỗi event 1 dòng JSON, canonical fields:
- `t` (event type)
- `i` (incrementing index)
- `tick` (u64)
- `seed` (u64)
- `data` (event payload, record/map stable order)

Event set tối thiểu:
- `ProgramStart`
- `ModuleLoad` (path, hash)
- `StmtLet` (name)
- `CallPure` (fn, args_summary)
- `MatchSelect` (kind selected)
- `LoopIter` (loop_id, iter_index)
- `ProgramEnd` (status)
- `Error` (code, span, message)

### 8.3 Signature definition

- `signature = hash(canonical_audit_bytes)`

canonicalization rules:
- stable map key order: sort lexicographic theo UTF-8 bytes
- exclude wallclock timestamps
- normalize module/path:
  - dùng `/` cho mọi path separator
  - strip prefix `./`
  - cấm absolute path trong signature input
  - module id là logical path tương đối từ entry root

`signature.txt` chứa hex string.

### 8.4 replay.toml

Chứa tối thiểu:
- project name
- entry
- lane
- seed
- limits
- module hashes (optional)
- fixture pointers (optional)

### 8.5 Artifact compatibility contract

- v0.7.1 giữ nguyên layout hiện hữu:
  - `.ocp_artifacts/<run_id>/audit.jsonl`
  - `.ocp_artifacts/<run_id>/signature.txt`
  - `.ocp_artifacts/<run_id>/replay.toml`
- Chỉ được additive:
  - thêm field trong JSON/TOML
  - thêm file phụ không phá reader hiện tại
- Không được đổi path bắt buộc ở v0.7.1.

---

## 9) CLI v0.7.1 (minimal but complete)

### 9.1 Commands

- `ocp init hello-tool <dir>`
  - tạo `ocp.toml`, `main.ocp`, `README.md`
  - tạo `fixtures/` nếu demo cần input JSON mẫu

- `ocp run [--entry main.ocp] [--seed N]`
  - chạy theo lane trong `ocp.toml` (default `locked_v071`), emit artifacts như mục 8

- `ocp test`
  - chạy fixtures: parse/type/exec
  - assert expected success hoặc expected error code + (optional) signature

- `ocp replay <artifact_dir>`
  - rerun theo replay.toml và compare signature

- `ocp trace view <audit.jsonl>`
  - in summary (counts, last N events, errors)

### 9.2 Template hello-tool (v0.7.1)

Yêu cầu demo không IO thật:
- Input JSON lấy từ fixture file hoặc string literal trong code.
- Parse -> transform -> stringify -> print (hoặc output vào artifacts).
- Demonstrate:
  - import std.json
  - record/list/map
- loop bounded
- try/else + guard
- Project file count: phải thỏa KPI-1.

### 9.3 CLI Compatibility Contract (LOCKED)

**Unchanged**
- Không silent break các command/flag đã tồn tại trong workspace hiện tại.
- Lane `locked_v06` phải chạy được để replay baseline.
- Nguồn chuẩn command set hiện có lấy từ `ocp --help` khi chạy ở lane `locked_v06`; mọi command/flag xuất hiện ở đây được coi là baseline compatibility.

**Additive**
- Các command nêu trong mục 9.1 là additive cho profile/lane v0.7.1.
- Có thể thêm subcommand/flag mới nếu không đổi behavior mặc định của command cũ.

**Deprecated**
- Nếu đổi tên command/flag/output:
  - phải giữ alias tương thích
  - phải in hint deprecation rõ ràng trong CLI output
  - không được xóa alias ngay trong v0.7.1.

**Artifact compatibility**
- Giữ layout `.ocp_artifacts/<run_id>/...` hiện hữu.
- Nếu có file mới, reader cũ vẫn phải đọc được bộ file bắt buộc cũ.

---

## 10) Error taxonomy v0.7.1 (DX-critical)

### 10.1 Parse (P-*)

- `P-LEX-INVALID-CHAR`
- `P-LEX-UNTERMINATED-STRING`
- `P-PARSE-EXPECTED-TOKEN`
- `P-PARSE-EXPECTED-SEMICOLON`
- `P-PARSE-IMPORT-SYNTAX`
- `P-PARSE-LOOP-SYNTAX`
- `P-PARSE-RECORD-SYNTAX`
- `P-PARSE-LIST-SYNTAX`

### 10.2 Type (T-*)

- `T-UNBOUND-NAME`
- `T-TYPE-MISMATCH`
- `T-INDEX-NOT-LIST`
- `T-MAP-KEY-NOT-STRING-LIT`
- `T-TRY-NOT-RESULT4`
- `T-TRY-ELSE-NO-VALUE`
- `T-GUARD-NOT-RESULT4`
- `T-FOR-NOT-LIST`
- `T-CAP-NOT-INT-LIT`
- `T-IMPORT-NOT-FOUND`
- `T-IMPORT-CYCLE`

### 10.3 Exec (X-*)

- `X-STEP-LIMIT`
- `X-LOOP-CAP-EXCEEDED`
- `X-KEYS-CAP-EXCEEDED`
- `X-GUARD-FAILED` (nếu guard_mode=error)
- `X-LIMIT-EXCEEDED` (alias chung machine-readable cho nhóm cap errors)

### 10.4 Root reasons (RC-*) dùng trong Result4

- `RC-JSON-INVALID`
- `RC-CAPABILITY-DENIED`
- `RC-NOT-IMPLEMENTED`
- `RC-LIMIT-EXCEEDED`

Rule DX:
- permission/limit errors phải có hint actionable.
- parse/type errors phải có span đúng.

### 10.5 Error Code Policy v0.7.1 (machine-readable)

- Namespace giữ nguyên: `P-*`, `T-*`, `X-*`, `R-*`, `RC-*` (không tạo namespace mới).
- Cap errors dùng canonical code riêng + alias code chung:
  - `X-LOOP-CAP-EXCEEDED` -> alias `X-LIMIT-EXCEEDED` với `meta.limit_kind="loop_cap"`
  - `X-KEYS-CAP-EXCEEDED` -> alias `X-LIMIT-EXCEEDED` với `meta.limit_kind="value_keys_cap"`
- `X-LIMIT-EXCEEDED` không bao giờ là canonical code ở v0.7.1; chỉ xuất hiện trong `aliases[]`.
- JSON invalid input:
  - `std.json.parse` trả `INSUFFICIENT + RC-JSON-INVALID`
  - không ném `X-*` cho JSON input invalid thông thường.

Diagnostic shape tối thiểu:
- `code: String` (canonical)
- `aliases: [String]` (optional)
- `meta: { limit_kind?: String, ... }`
- `root_reason: String?` (dùng cho `RC-*` khi cần giữ nguyên nhân gốc)

---

## 11) Execution gates v0.7.1 (triển khai tuần tự)

### Gate 7.1-A — Core skeleton + diagnostics + audit

Change tag:
- `S`, `I`

Compatibility:
- `additive` (không phá lane `locked_v06`)

Scope:
- span/diag/lex/parse minimal
- audit writer + signature hash skeleton

Command:
- `cargo test --test ocp_diag --test ocp_audit`

Artifacts:
- `.ocp_artifacts/<run_id>/audit.jsonl`
- `.ocp_artifacts/<run_id>/signature.txt`
- `.ocp_artifacts/<run_id>/replay.toml`

Pass/Fail:
- PASS:
  - parse error có `code + span + hint`
  - run fail vẫn emit audit + error event + signature
- FAIL:
  - thiếu một trong ba artifact bắt buộc
  - diagnostics thiếu code/span/hint

### Gate 7.1-B — Value model + pure ops + caps

Change tag:
- `S`

Compatibility:
- `additive`

Scope:
- Value: null/bool/int/string/list/record/map
- ops: len, keys(cap), merge(cap), push
- caps: value_keys_cap enforced

Command:
- `cargo test --test ocp_value`

Artifacts:
- test report `ocp_value`
- audit events cho case cap violation (nếu suite ghi ra artifact)

Pass/Fail:
- PASS:
  - keys/merge bounded deterministic
  - cap violation surfacing canonical code + alias metadata
- FAIL:
  - merge/keys không bounded hoặc code không đúng policy alias

### Gate 7.1-C — std.json (pure) + Result4 integration

Change tag:
- `S`

Compatibility:
- `additive`

Scope:
- json.parse -> Result4<Value> (fail => INSUFFICIENT + RC-JSON-INVALID)
- json.stringify deterministic (stable key order)

Command:
- `cargo test --test std_json`

Artifacts:
- test report `std_json`
- deterministic snapshots (nếu suite có)

Pass/Fail:
- PASS:
  - invalid JSON không panic
  - stringify stable giữa runs
- FAIL:
  - JSON invalid bị map sang `X-*`
  - output stringify không ổn định

### Gate 7.1-D — Modules/import loader

Change tag:
- `S`, `I`

Compatibility:
- `additive`

Scope:
- import syntax
- module resolver (std + relative)
- cycle detection
- path canonicalization cho signature input

Command:
- `cargo test --test ocp_modules`

Artifacts:
- test report `ocp_modules`
- module resolution trace (nếu suite emit)

Pass/Fail:
- PASS:
  - cycle rejected `T-IMPORT-CYCLE` với hint actionable
  - missing module => `T-IMPORT-NOT-FOUND` với hint actionable
- FAIL:
  - resolver nhận absolute path vào signature input
  - thiếu hints cho lỗi import

### Gate 7.1-E — Bounded loops

Change tag:
- `S`

Compatibility:
- `additive`

Scope:
- repeat / for cap
- loop cap enforcement + step counting

Command:
- `cargo test --test ocp_loops`

Artifacts:
- test report `ocp_loops`
- audit loop events (`LoopIter`, `Error`) cho cases cap/step limit

Pass/Fail:
- PASS:
  - loop cap violation => `X-LOOP-CAP-EXCEEDED`
  - step cap enforced => `X-STEP-LIMIT`
- FAIL:
  - loop không bị chặn đúng cap
  - code lỗi không đúng taxonomy đã khóa

### Gate 7.1-F — try/else + guard sugar (desugar pass)

Change tag:
- `S`

Compatibility:
- `additive`

Scope:
- AST desugar transforms
- type rules cho try/guard
- sugar invariance (signature equality)

Command:
- `cargo test --test ocp_sugar`

Artifacts:
- cặp artifacts sugar/canonical:
  - `audit.jsonl`
  - `signature.txt`

Pass/Fail:
- PASS:
  - sugar program có cùng signature với canonical desugared
  - implicit binding `r` luôn có trong else block
- FAIL:
  - signature mismatch giữa sugar và canonical
  - guard_mode không theo config manifest

### Gate 7.1-G — Manifest + permissions + CLI init/run/test/replay

Change tag:
- `S`, `C`, `I`

Compatibility:
- `additive`

Scope:
- ocp.toml parse (`lane`, `limits`, `[language].guard_mode`)
- deny/allow precedence
- permission error hints
- CLI commands + hello-tool template + replay compat

Command:
- `cargo test --test ocp_manifest --test cli_e2e`

Artifacts:
- `.ocp_artifacts/<run_id>/*` bộ file bắt buộc
- template `hello-tool` generated tree
- permission diagnostics snapshots

Pass/Fail:
- PASS:
  - KPI-1 pass (project file count)
  - KPI-2 pass (replay signature stable)
  - lane `locked_v06` vẫn chạy được trong compat path
  - permission errors show matched deny rule + manifest hint
- FAIL:
  - đổi behavior CLI cũ theo kiểu silent break
  - replay mismatch ở cùng seed/config
  - thiếu compat lane `locked_v06`

---

## 12) Operational commands

- `cargo test`
- `cargo test --test ocp_parser`
- `cargo test --test ocp_typecheck`
- `cargo test --test ocp_exec`
- `cargo test --test ocp_determinism`
- `cargo test --test cli_e2e`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt -- --check`

---

## 13) Execution Log (template)

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

---

### 2026-03-04 — 7.1-A planning freeze
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.1-A
- Why:
  - Đóng Gate A bằng code delta thật cho diagnostics + audit artifacts theo chuẩn v0.7.1, tránh DONE giả chỉ dựa vào test cũ.
- Scope:
  - Bổ sung parse error hint mặc định để đảm bảo parse error có `code + span + hint`.
  - Bổ sung test suite `ocp_audit` để khóa hợp đồng audit/signature ở mức core.
  - Bổ sung artifact writer cho `ocp run` (non-reactor project path) luôn ghi:
    - `.ocp_artifacts/<run_id>/audit.jsonl`
    - `.ocp_artifacts/<run_id>/signature.txt`
    - `.ocp_artifacts/<run_id>/replay.toml`
    kể cả khi run fail.
  - Bổ sung CLI tests chứng minh artifact được ghi ở cả success/fail path.
- Expected tests:
  - `cargo test --test ocp_diag --test ocp_audit`
  - `cargo test -p ocp-cli v7_a_cli_run_emits_artifacts_on_success`
  - `cargo test -p ocp-cli v7_a_cli_run_emits_artifacts_on_fail`
  - `cargo test`
- Exit criteria:
  - Parse error thực tế có đủ `code + span + hint`.
  - `ocp run` (non-reactor project path) ghi đủ 3 artifact kể cả khi fail.
  - Targeted tests và full regression pass.

### 2026-03-04 — 7.1-A implementation closeout
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.1-A
- Implemented:
  - Bổ sung hint mặc định cho parse diagnostics để khóa hợp đồng `code + span + hint` trên lỗi parser thực tế.
  - Bổ sung suite `ocp_audit` để kiểm chứng trace/signature ổn định ở lớp core.
  - Nâng `ocp-cli` (non-reactor project run path) để luôn ghi đủ replay-first artifacts:
    - `.ocp_artifacts/<run_id>/audit.jsonl`
    - `.ocp_artifacts/<run_id>/signature.txt`
    - `.ocp_artifacts/<run_id>/replay.toml`
    kể cả success/fail path.
  - Bổ sung 2 CLI tests xác nhận artifact emission cho cả success và fail.
- Files changed:
  - `src/ocp/parse.rs`
  - `tests/ocp_diag.rs`
  - `tests/ocp_audit.rs`
  - `projects/ocp/crates/ocp-cli/src/main.rs`
  - `OCP-MVP-PLAN-v0.7.1.md`
- Commands run:
  - `cargo test --test ocp_diag --test ocp_audit`
  - `cargo test -p ocp-cli v7_a_cli_run_emits_artifacts_on_success`
  - `cargo test -p ocp-cli v7_a_cli_run_emits_artifacts_on_fail`
  - `cargo test`
- Test results:
  - PASS:
    - `ocp_diag`: 5/5
    - `ocp_audit`: 2/2
    - `ocp-cli` targeted:
      - `v7_a_cli_run_emits_artifacts_on_success`: 1/1
      - `v7_a_cli_run_emits_artifacts_on_fail`: 1/1
    - Full `cargo test`: pass toàn bộ suites hiện có.
- Notes/risks:
  - Gate A chỉ khóa core diagnostics/audit + artifact emission ở non-reactor project path theo đúng scope.
  - Ghi chú theo thời điểm đóng Gate A: lúc đó `7.1-B..7.1-G` chưa triển khai; trạng thái hiện tại xem mục `0.2`.

### 2026-03-04 — 7.1-B planning freeze
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.1-B
- Why:
  - Đóng phần Value model/pure ops bounded cho v0.7.1 để giảm boilerplate xử lý dữ liệu trước khi mở JSON/modules/sugar ở các gate sau.
- Scope:
  - Mở rộng runtime/typecheck cho pure ops:
    - `len(x)`
    - `keys(map, cap)`
    - `merge(a, b, cap)`
  - Enforce cap cho keys/merge và surfacing lỗi canonical.
  - Bổ sung test suite `tests/ocp_value.rs` cho pass/fail paths của Gate B.
- Expected tests:
  - `cargo test --test ocp_value`
  - `cargo test`
- Exit criteria:
  - `len/keys/merge` chạy đúng semantics đã khóa trong v0.7.1.
  - Vi phạm cap trả lỗi canonical ổn định.
  - Targeted + full regression pass.

### 2026-03-04 — 7.1-B implementation closeout
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.1-B
- Implemented:
  - Bổ sung pure ops trong runtime:
    - `len(x)` cho `list/map/payload/string`
    - `keys(map, cap)` trả danh sách key đã cap deterministic
    - `merge(a, b, cap)` merge deterministic, right overwrite left
  - Enforce cap `VALUE_KEYS_CAP=2000` cho `keys/merge`.
  - Khi cap vi phạm:
    - canonical code `X-KEYS-CAP-EXCEEDED`
    - alias machine-readable `X-LIMIT-EXCEEDED`
    - `limit_kind="value_keys_cap"`
    - `root_reason=RC-POLICY-DENIED`
  - Mở rộng typechecker allowlist cho `len/keys/merge` với rule kiểu rõ ràng.
  - Bổ sung test suite mới `tests/ocp_value.rs`.
- Files changed:
  - `src/ocp/diag.rs`
  - `src/ocp/typecheck.rs`
  - `src/ocp/exec.rs`
  - `tests/ocp_diag.rs`
  - `tests/ocp_value.rs`
  - `OCP-MVP-PLAN-v0.7.1.md`
- Commands run:
  - `cargo test --test ocp_value`
  - `cargo test`
- Test results:
  - PASS:
    - `ocp_value`: 6/6
    - Full `cargo test`: pass toàn bộ suites hiện có (bao gồm suite mới `ocp_value`).
- Notes/risks:
  - Gate B hiện dùng cap runtime hằng số nội bộ (`VALUE_KEYS_CAP`) để giữ thay đổi nhỏ và ổn định.
  - Mapping cap theo `limits.value_keys_cap` từ manifest sẽ được mở ở gate cấu hình/manifest sau.

### 2026-03-04 — 7.1-C planning freeze
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.1-C
- Why:
  - Mở pure JSON contract trong expression layer để `std.json.parse/stringify` chạy trực tiếp qua typecheck+exec, không lệ thuộc observe-key path.
- Scope:
  - Parser: hỗ trợ namespaced call dạng `std.json.parse(...)` và `std.json.stringify(...)`.
  - Typechecker:
    - `std.json.parse(string) -> Result4<Value>`
    - `std.json.stringify(Value) -> String`
  - Executor:
    - parse thành `Result4<Value>` (invalid -> `INSUFFICIENT + RC-JSON-INVALID`)
    - stringify deterministic với stable key order.
  - Test suite mới `tests/std_json.rs`.
- Expected tests:
  - `cargo test --test std_json`
  - `cargo test`
- Exit criteria:
  - `std.json.parse/stringify` chạy được ở pure call path.
  - Invalid JSON không panic, map đúng `RC-JSON-INVALID`.
  - stringify ổn định deterministic qua nhiều lần chạy cùng input.

### 2026-03-04 — 7.1-C implementation closeout
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.1-C
- Implemented:
  - Parser:
    - hỗ trợ namespaced call target dạng `a.b.c(...)` cho pure call path.
  - Typechecker:
    - `std.json.parse(string) -> Result4<Value>`
    - `std.json.stringify(value) -> String`
  - Executor:
    - thêm pure call `std.json.parse(...)`:
      - parse JSON thành `Result4::ok(Value)`
      - invalid JSON trả `Result4::insufficient(RC-JSON-INVALID)`
    - thêm pure call `std.json.stringify(...)` với output deterministic.
    - bổ sung JSON parser/stringifier nội bộ cho `Value`.
  - Diagnostics/reason:
    - thêm reason code `RC-JSON-INVALID`.
  - Tests:
    - thêm suite mới `tests/std_json.rs` cho parse/stringify/typecheck.
- Files changed:
  - `src/ocp/parse.rs`
  - `src/ocp/typecheck.rs`
  - `src/ocp/exec.rs`
  - `src/ocp/diag.rs`
  - `tests/std_json.rs`
  - `tests/ocp_diag.rs`
  - `OCP-MVP-PLAN-v0.7.1.md`
- Commands run:
  - `cargo test --test std_json`
  - `cargo test --test ocp_diag`
  - `cargo test`
- Test results:
  - PASS:
    - `std_json`: 4/4
    - `ocp_diag`: 5/5
    - Full `cargo test`: pass toàn bộ suites hiện có.
- Notes/risks:
  - Gate C đang tập trung pure JSON call path trong runtime core; observe-key path `std.json.parse` cũ vẫn giữ để tương thích baseline.
  - JSON number ở parser nội bộ hiện giới hạn int (`i64`) để giữ deterministic và scope gọn cho v0.7.1.

### 2026-03-04 — 7.1-D planning freeze
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.1-D
- Why:
  - Đóng module/import path cơ bản để tách code đa file có kiểm soát, đồng thời khóa lỗi rõ ràng cho missing/cycle trước khi mở gate loops/sugar.
- Scope:
  - Thêm module loader ở fixture runner theo root entry:
    - resolve import module path -> file tương ứng
    - load đệ quy module local
    - detect cycle
    - chuẩn hóa module id/path nội bộ theo rule deterministic
  - Bổ sung diagnostics:
    - `T-IMPORT-NOT-FOUND`
    - `T-IMPORT-CYCLE`
    với hint actionable.
  - Bổ sung test suite mới `tests/ocp_modules.rs`.
- Expected tests:
  - `cargo test --test ocp_modules`
  - `cargo test`
- Exit criteria:
  - Import local module chạy pass end-to-end.
  - Missing module trả `T-IMPORT-NOT-FOUND` + hint.
  - Import cycle trả `T-IMPORT-CYCLE` + hint.

### 2026-03-04 — 7.1-D implementation closeout
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.1-D
- Implemented:
  - Bổ sung module loader cho `run_fixture_file`:
    - load đệ quy theo `import` local module path.
    - detect import cycle bằng DFS (`visiting/loaded`).
    - resolve file theo root entry, canonicalize và chặn path thoát root.
  - Bổ sung diagnostics mới:
    - `T-IMPORT-NOT-FOUND`
    - `T-IMPORT-CYCLE`
    kèm hint actionable ở fail path.
  - Bổ sung test suite `tests/ocp_modules.rs`:
    - import local module pass.
    - missing module fail đúng code + hint.
    - import cycle fail đúng code + hint.
- Files changed:
  - `src/ocp/diag.rs`
  - `src/ocp/runner.rs`
  - `tests/ocp_modules.rs`
  - `OCP-MVP-PLAN-v0.7.1.md`
- Commands run:
  - `cargo test --test ocp_modules`
  - `cargo test --test ocp_diag`
  - `cargo test`
- Test results:
  - PASS:
    - `ocp_modules`: 3/3
    - `ocp_diag`: 5/5
    - Full `cargo test`: pass toàn bộ suites hiện có.
- Notes/risks:
  - Gate D hiện đang dùng import syntax hiện có (`import a.b;`) trong parser hiện tại; alias/import-string path sẽ khóa ở bước syntax mở rộng sau.
  - Loader xử lý `std.*` import theo hướng virtual (bỏ qua file resolution) để không phá compatibility của pure/runtime builtins.

### 2026-03-04 — 7.1-E planning freeze
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.1-E
- Why:
  - Đóng loop semantics bounded theo thiết kế v0.7.1 để thay thế trạng thái TODO bằng code delta thật, có test chứng minh rõ parse/typecheck/exec.
- Scope:
  - Mở rộng syntax loop:
    - `repeat n { ... }`
    - `for item in xs cap N { ... }`
    - vẫn giữ `for-range` cũ để tương thích ngược.
  - Typecheck:
    - iterable của `for ... cap` bắt buộc là list.
    - `cap` bắt buộc int literal không âm.
  - Executor:
    - enforce loop cap runtime (`LOOP_CAP`).
    - trả lỗi canonical `X-LOOP-CAP-EXCEEDED` + alias `X-LIMIT-EXCEEDED` + `limit_kind="loop_cap"`.
  - Bổ sung test suite `tests/ocp_loops.rs` và cập nhật parser/diag tests liên quan.
- Expected tests:
  - `cargo test --test ocp_loops`
  - `cargo test --test ocp_parser --test ocp_exec --test ocp_diag`
  - `cargo test`
- Exit criteria:
  - Loop forms mới parse + typecheck + exec đúng contract.
  - Vi phạm loop cap surfaced đúng taxonomy/alias metadata.
  - Targeted + full regression pass.

### 2026-03-04 — 7.1-E implementation closeout
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.1-E
- Implemented:
  - Lexer/AST/Parser:
    - thêm token `repeat`, `cap`.
    - thêm AST statements `Repeat`, `ForEachCap`.
    - parser hỗ trợ `repeat ...` và `for ... cap ...`, giữ `for-range` cũ.
  - Typechecker:
    - thêm rule `for ... cap`:
      - iterable phải là list (`T-FOR-NOT-LIST`).
      - cap phải là int literal không âm (`T-CAP-NOT-INT-LIT`).
    - thêm rule `repeat` count phải là int.
  - Executor:
    - thêm runtime execution cho `Repeat` và `ForEachCap`.
    - đổi cap enforcement của loop sang canonical `X-LOOP-CAP-EXCEEDED` (kèm alias `X-LIMIT-EXCEEDED`, `limit_kind="loop_cap"`).
    - thêm audit event `LoopIter` cho các loop path.
  - Diagnostics/tests:
    - thêm error codes mới:
      - `T-FOR-NOT-LIST`
      - `T-CAP-NOT-INT-LIT`
      - `X-LOOP-CAP-EXCEEDED`
    - thêm suite mới `tests/ocp_loops.rs`.
    - cập nhật `tests/ocp_parser.rs` và `tests/ocp_diag.rs`.
- Files changed:
  - `src/ocp/ast.rs`
  - `src/ocp/lex.rs`
  - `src/ocp/parse.rs`
  - `src/ocp/typecheck.rs`
  - `src/ocp/exec.rs`
  - `src/ocp/audit.rs`
  - `src/ocp/diag.rs`
  - `tests/ocp_loops.rs`
  - `tests/ocp_parser.rs`
  - `tests/ocp_diag.rs`
  - `OCP-MVP-PLAN-v0.7.1.md`
- Commands run:
  - `rustfmt projects/ocp/crates/ocp-cli/src/main.rs src/ocp/exec.rs src/ocp/parse.rs src/ocp/typecheck.rs src/ocp/runner.rs tests/ocp_audit.rs tests/ocp_loops.rs`
  - `cargo fmt -- --check`
  - `cargo test --test ocp_loops`
  - `cargo test --test ocp_parser --test ocp_exec --test ocp_diag`
  - `cargo test`
- Test results:
  - PASS:
    - `cargo fmt -- --check`: pass
    - `ocp_loops`: 5/5
    - `ocp_parser`: 5/5
    - `ocp_exec`: 9/9
    - `ocp_diag`: 5/5
    - Full `cargo test`: pass toàn bộ suites hiện có.
- Notes/risks:
  - `for-range` cũ vẫn được giữ để tương thích ngược; Gate E chỉ mở rộng thêm loop forms mới, không xóa behavior cũ.
  - `step cap` hiện vẫn surface bằng `X-BUDGET-EXCEEDED` từ meter hiện có; Gate E chỉ khóa taxonomy cho loop-cap riêng (`X-LOOP-CAP-EXCEEDED`).

### 2026-03-04 — 7.1-F planning freeze
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.1-F
- Why:
  - Đóng sugar semantics `try/else` + `guard` theo thiết kế v0.7.1 với tiêu chí bắt buộc: sugar và canonical desugar phải cho cùng signature audit.
- Scope:
  - Mở keyword/token/parser/AST cho:
    - `let x = try r else { ... };`
    - `guard r;`
  - Typechecker:
    - `try/else` và `guard` bắt buộc nhận `Result4`.
    - mở top-level `return` theo contract return-from-program.
    - implicit binding `r` trong nhánh `else`.
  - Executor:
    - chạy semantics sugar cho `TryLet` và `Guard`.
    - honor `guard_mode` (`return` | `error`) từ `ExecConfig`.
    - khóa sugar invariance bằng parity step/trace với canonical.
  - Tests:
    - thêm suite `tests/ocp_sugar.rs`.
    - cập nhật parser/diag tests liên quan.
- Expected tests:
  - `cargo test --test ocp_sugar`
  - `cargo test --test ocp_parser --test ocp_typecheck --test ocp_exec --test ocp_diag --test ocp_sugar`
  - `cargo test`
- Exit criteria:
  - Hai ca invariance (try/else, guard) pass signature equality.
  - Guard type/runtime errors surfaced đúng taxonomy (`T-GUARD-NOT-RESULT4`, `X-GUARD-FAILED`).
  - Targeted tests và full regression pass.

### 2026-03-04 — 7.1-F implementation closeout
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.1-F
- Implemented:
  - Lexer/AST/Parser:
    - thêm token `try`, `else`, `guard`.
    - thêm `Stmt::TryLet` và `Stmt::Guard`.
    - parser hỗ trợ `let x = try r else { ... };` và `guard r;`.
  - Typechecker:
    - thêm rule `try/else` phải nhận `Result4` (`T-TRY-NOT-RESULT4`).
    - thêm rule `guard` phải nhận `Result4` (`T-GUARD-NOT-RESULT4`).
    - cho phép top-level `return` theo contract 7.1.
    - hỗ trợ binding ngầm `r` trong `else`.
  - Runtime/exec:
    - thêm `GuardMode` vào `ExecConfig` (default `return`).
    - implement `exec_try_let` và `exec_guard`.
    - thêm `X-GUARD-FAILED`.
    - bổ sung parity tick trong sugar path để signature trùng canonical desugar.
    - mở field access cho `Result4`: `r.kind`, `r.reason_code`, `r.audit`.
  - Tests:
    - thêm `tests/ocp_sugar.rs` (invariance + type/runtime contracts).
    - cập nhật `tests/ocp_parser.rs`, `tests/ocp_diag.rs`.
    - vá `tests/ocp_confidence.rs` để tương thích `ExecConfig` có thêm `guard_mode`.
- Files changed:
  - `src/ocp/lex.rs`
  - `src/ocp/ast.rs`
  - `src/ocp/parse.rs`
  - `src/ocp/typecheck.rs`
  - `src/ocp/budget.rs`
  - `src/ocp/exec.rs`
  - `src/ocp/diag.rs`
  - `src/ocp/mod.rs`
  - `tests/ocp_sugar.rs`
  - `tests/ocp_parser.rs`
  - `tests/ocp_diag.rs`
  - `tests/ocp_confidence.rs`
  - `OCP-MVP-PLAN-v0.7.1.md`
- Commands run:
  - `cargo test --test ocp_sugar`
  - `cargo test --test ocp_parser --test ocp_typecheck --test ocp_exec --test ocp_diag --test ocp_sugar`
  - `cargo test`
- Test results:
  - PASS:
    - `ocp_sugar`: 6/6 pass.
    - targeted group (`ocp_parser`, `ocp_typecheck`, `ocp_exec`, `ocp_diag`, `ocp_sugar`) pass toàn bộ.
    - full `cargo test`: pass toàn bộ suites hiện có.
  - FAIL tạm thời đã xử lý:
    - full `cargo test` lần đầu fail compile ở `tests/ocp_confidence.rs` vì thiếu field `guard_mode` trong `ExecConfig`.
    - đã vá bằng `..ExecConfig::default()`, sau đó full `cargo test` pass.
- Notes/risks:
  - Gate F đã đóng theo scope sugar/runtime.
  - `T-TRY-ELSE-NO-VALUE` đã khai báo taxonomy để giữ ổn định contract; enforcement parser/typecheck có thể siết sâu hơn ở gate sau nếu mở rộng else-block phức hợp.

### 2026-03-04 — 7.1-G planning freeze
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.1-G
- Why:
  - Đóng phần Manifest + permissions + CLI replay theo đúng contract v0.7.1 để tránh drift giữa thiết kế và code thực thi.
- Scope:
  - `ocp-sdk`:
    - parse `[project].lane` và `[language].guard_mode` từ manifest.
    - nối `guard_mode` vào run/test/trace/reactor execution config.
    - siết permission deny message có `matched rule` + `section` + `hint` sửa manifest.
  - `ocp-cli`:
    - thêm command `replay <artifact_dir>`.
    - parse `replay.toml`, rerun theo `root/lane/engine`, compare signature.
    - lane compat rule cho replay: `locked_v06` đi compat path.
  - Regression bridge:
    - vá compile drift do AST/ExecConfig mới giữa `ocp` và `ocp-runtime-core`/`ocp-sdk`.
  - Tests:
    - thêm `tests/ocp_manifest.rs`.
    - thêm `tests/cli_e2e.rs`.
    - thêm test replay trong `ocp-cli/src/main.rs`.
- Expected tests:
  - `cargo test --test ocp_manifest --test cli_e2e`
  - `cargo test -p ocp-cli v7_g_cli_replay_`
  - `cargo test -p ocp-cli`
  - `cargo test`
- Exit criteria:
  - Manifest lane + guard_mode parse và runtime wiring pass.
  - Permission denied hiển thị matched rule + hint actionable.
  - CLI replay pass khi signature match, fail khi mismatch.
  - Targeted tests và full regression pass.

### 2026-03-04 — 7.1-G implementation closeout
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.1-G
- Implemented:
  - `ocp-sdk` manifest/runtime wiring:
    - thêm `ProjectLanguageConfigV071` và parser `parse_project_language_config_v071(...)`.
    - thêm `default_exec_config_for_layout(...)` để inject `guard_mode` từ manifest.
    - nối config này vào `run_project_with_engine_and_lock`, `run_reactor_service_with_lock`, `test_project_with_lock`.
    - các path trace/reactor trace merge `guard_mode` vào runtime config trước khi thực thi.
  - `ocp-sdk` permissions DX:
    - giữ precedence `deny > allow`.
    - nâng error `V-PERMISSION-DENIED` để nêu rõ:
      - `matched deny rule`,
      - source section (`permissions.package` hoặc `permissions.module.*`),
      - hint chỉnh manifest.
  - `ocp-cli` replay:
    - thêm command `replay <artifact_dir>`.
    - thêm parser `replay.toml` (root/lane/engine/signature).
    - replay rerun + compare signature:
      - match => exit 0,
      - mismatch => fail-hard.
    - thêm help text cho command replay.
  - Compatibility hardening (compile blockers):
    - `ocp-runtime-core`:
      - cập nhật `ExecConfig` initializers theo struct mới.
      - mở rộng IR/bytecode mapping cho statement mới (`TryLet`, `Guard`, `Repeat`, `ForEachCap`).
      - re-export `GuardMode`.
    - `ocp-sdk/w3.rs`:
      - cập nhật `ExecConfig` initializers.
    - `ocp-sdk/lib.rs`:
      - cập nhật trace mapper cho `TraceEvent::LoopIter`.
  - Tests mới:
    - `tests/ocp_manifest.rs`:
      - parse lane/guard_mode.
      - deny-over-allow + hint message.
      - `guard_mode=error` được enforce runtime.
    - `tests/cli_e2e.rs`:
      - `guard_mode=return` non-breaking path.
      - locked run yêu cầu `[permissions.package]`.
    - `ocp-cli` unit tests:
      - `v7_g_cli_replay_signature_match_pass`
      - `v7_g_cli_replay_signature_mismatch_fail`
- Files changed:
  - `Cargo.toml`
  - `projects/ocp/crates/ocp-runtime-core/src/lib.rs`
  - `projects/ocp/crates/ocp-runtime-core/src/ir.rs`
  - `projects/ocp/crates/ocp-runtime-core/src/bytecode.rs`
  - `projects/ocp/crates/ocp-sdk/src/lib.rs`
  - `projects/ocp/crates/ocp-sdk/src/w3.rs`
  - `projects/ocp/crates/ocp-cli/src/main.rs`
  - `tests/ocp_manifest.rs`
  - `tests/cli_e2e.rs`
  - `OCP-MVP-PLAN-v0.7.1.md`
- Commands run:
  - `cargo test --test ocp_manifest --test cli_e2e`
  - `cargo test -p ocp-cli v7_g_cli_replay_`
  - `cargo test -p ocp-cli`
  - `cargo test`
- Test results:
  - PASS:
    - `ocp_manifest`: 3/3
    - `cli_e2e`: 2/2
    - `ocp-cli` replay tests: 2/2
    - full `cargo test -p ocp-cli`: 33/33
    - full `cargo test`: pass toàn bộ suites hiện có.
  - FAIL tạm thời đã xử lý:
    - compile fail do `ExecConfig` thêm `guard_mode` ở `ocp-runtime-core`/`ocp-sdk/w3`.
    - compile fail do AST/trace enum mở rộng nhưng runtime-core/sdk mapper chưa cover.
    - đã vá xong và rerun pass toàn bộ.
- Notes/risks:
  - Gate G đã đóng với replay command mức project artifact-dir; replay cho artifact package (`.ocppkg`) chưa mở trong scope này.
  - Lane compat cho replay đang khóa rule: `locked_v06` dùng compat unlocked path; các lane khác mặc định locked path.

---

## 14) Public interfaces/types (LOCKED)

### 14.1 `ocp.toml`
- `[project].lane = "locked_v071" | "locked_v06"`
- `[language].guard_mode = "return" | "error"` (default = `return`)
- Lane mặc định v0.7.1: `locked_v071`.

### 14.2 Diagnostic schema tối thiểu
- `code: String` (canonical)
- `aliases: [String]` (optional)
- `meta: { limit_kind?: String, ... }`
- `root_reason: String?` (dùng cho `RC-*` theo contract hiện hành)

### 14.3 Replay artifacts contract
- Giữ:
  - `.ocp_artifacts/<run_id>/audit.jsonl`
  - `.ocp_artifacts/<run_id>/signature.txt`
  - `.ocp_artifacts/<run_id>/replay.toml`
- Chỉ additive field/file, không phá reader cũ.

---

## 15) Test/Scenario Matrix (phải có trước khi code gate)

1) Sugar invariance:
- `try/else` sugar vs canonical
- `guard` sugar vs canonical
- assert signature equality

2) Commit discipline:
- pure origin commit deny + audit event
- observe origin OK/DEGRADED -> accepted(no_effect) + audit event

3) Compat lane:
- cùng project chạy được `locked_v06` và `locked_v071`
- behavior cũ không vỡ ngoài phạm vi additive đã khai báo

4) Error taxonomy:
- cap canonical code + alias parse được
- JSON invalid trả `RC-JSON-INVALID`

5) Replay determinism:
- cross-run signature stable cùng seed/config
- path normalization không lệch OS

---

## 16) Design Freeze Checklist (must pass before code)

- [x] Có `Change Classification` đầy đủ.
- [x] Có `Module -> Crate -> Path` rõ ràng.
- [x] Có `Migration Contract` với lane legacy cụ thể.
- [x] Có `CLI Compatibility Contract`.
- [x] Có `Error Code Policy` machine-readable.
- [x] Có KPI evidence `Command/Artifacts/Pass/Fail`.
- [x] Mỗi gate có `Change tag + Compatibility + Command + Artifacts + Pass/Fail`.
- [x] Không còn câu “nếu thiết kế…” hoặc “có thể…” ở các mục semantics cốt lõi.

---

