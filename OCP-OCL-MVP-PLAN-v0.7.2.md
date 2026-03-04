# OCL v0.7.2 — Expansion Plan (Tool-grade Packs + Real File/Store + Deterministic IO Harness)

Ngày tạo: 2026-03-03  
Trạng thái: `DRAFT (LOCK WHEN CODING)`  
Phạm vi: **OCL-only**.  
Tiền đề: v0.7.1 đã có core (Value/JSON/import/sugar/loops), manifest permissions, replay-first artifacts, CLI nền.

Mục tiêu v0.7.2: biến OCL thành “làm tool phổ thông được” bằng cách ship các **packs có IO thực dụng nhưng vẫn an toàn**:
- `std.fs` sandbox-first (đọc/ghi file/thư mục trong allowlist),
- `std.kv` local store (persist cấu hình/đánh dấu/nhật ký),
- `std.time` deterministic time (tick/logical time),
kèm **fixture/record harness** để test IO một cách deterministic trong lane `locked`.

> v0.7.2 chưa ship UI/game/shadow. Nó nhắm đúng “tool CLI / tool automation / data transformer” (có thể thêm UI ở v0.7.3).

---

## 0) Governance + Tracking (LOCKED)

### 0.1 Quy ước cập nhật bắt buộc
- Mọi thay đổi kế hoạch phải cập nhật file này trước khi code.
- Mọi triển khai xong phải cập nhật log ngay sau khi chạy test.
- Không nhảy gate: gate sau chỉ mở khi gate hiện tại `DONE`.
- Chỉ chuyển gate sang `DONE` khi có đủ cặp:
  - `Planning Freeze`
  - `Implementation Closeout`

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

### 0.2 Change Classification

| Item | Tag | Compatibility | Evidence suite | Owner gate |
|---|---|---|---|---|
| Registry/manifest mở rộng cho fs/kv/time | `S` | `additive` | `tests/ocl_manifest_fs_kv_time.rs` | `7.2-A` |
| `std.time` deterministic keys | `S` | `additive` | `tests/std_time.rs` | `7.2-B` |
| `std.kv` local store | `S` | `additive` | `tests/std_kv.rs` | `7.2-C` |
| `std.fs` observe sandbox-first | `S` | `additive` | `tests/std_fs_observe.rs` | `7.2-D` |
| `std.fs` commit keys | `S` | `additive` | `tests/std_fs_commit.rs` | `7.2-E` |
| CLI template `tool-cli` + `ocl test` IO harness | `C` | `additive` | `tests/cli_tool_cli_e2e.rs` | `7.2-F` |
| Workspace mapping/refactor nội bộ | `I` | `internal` | docs + review checklist | `7.2-A` |

### 0.3 Trạng thái Workstreams/Gates v0.7.2 (TRACKING)

#### Workstreams
- WS-S (packs semantics + policy + determinism): `DONE` (2026-03-04; 7.2-A..7.2-F pass đầy đủ quality gates)
- WS-C (CLI template + IO test/replay UX): `DONE` (2026-03-04; 7.2-F complete)
- WS-I (workspace mapping/internal wiring): `IN_PROGRESS` (7.2-A wiring started)

#### Gate status (7.2-A .. 7.2-F)
- Gate 7.2-A (Registry + manifest extensions for fs/kv/time): `DONE` (2026-03-04)
- Gate 7.2-B (std.time deterministic): `DONE` (2026-03-04)
- Gate 7.2-C (std.kv store): `DONE` (2026-03-04)
- Gate 7.2-D (std.fs observe sandbox-first): `DONE` (2026-03-04)
- Gate 7.2-E (std.fs commit keys): `DONE` (2026-03-04)
- Gate 7.2-F (CLI template tool-cli + fixture IO runner): `DONE` (2026-03-04)

#### Quy tắc cập nhật trạng thái (bắt buộc)
- `DONE` chỉ hợp lệ khi:
  - Có code delta đúng phạm vi gate.
  - Có test targeted đúng phạm vi gate.
  - Có `Implementation Closeout` ghi rõ `PASS/FAIL` và evidence.
- Nếu mới verify/chưa có code delta thì giữ `IN_PROGRESS` hoặc `TODO`.

## 1) Goals v0.7.2 (LOCKED)

### 1.1 North Star
- Dev tạo một tool thật sự: đọc file, xử lý JSON/text, ghi output, lưu state.
- Tất cả chạy trong lane `locked` (default), không cần quarantine.
- IO được sandbox + permission-driven, không cần viết host glue.
- Test có fixtures và replay signature ổn định (IO deterministic theo fixture env).

### 1.2 KPI bắt buộc (định lượng)
**KPI-1: Tool baseline (không UI)**
- `template/tool-cli` (v0.7.2) làm được:
  - read 1–N file trong `./data/**`
  - transform (JSON -> JSON) + bounded loops
  - write output vào `./out/**`
  - store checkpoint trong `std.kv`
- Script OCL mục tiêu: **≤ 250–400 LOC** (không tính packs).
- Project tạo bởi `ocl init tool-cli`: **≤ 6 file** (bao gồm 1–2 fixtures).

**KPI-2: Permission DX**
- Thiếu quyền read/write/kv => error có hint sửa `ocl.toml` trong **≤ 2 vòng**.

**KPI-3: Deterministic IO testing**
- `ocl test` chạy suite fixtures IO => signature ổn định giữa runs.
- Không phụ thuộc wallclock, không phụ thuộc thứ tự filesystem ngoài allowlist.

---

## 2) Axis Lock v0.7.x (kế thừa, không đổi)
- No naked IO; IO chỉ qua pack capability.
- 4-kind everywhere.
- Commit-gated effects.
- Boundedness (step cap, enumerate cap, per-call budget).
- Determinism-by-default (lane locked).
- Diagnostics structured.

---

## 3) Scope v0.7.2

### 3.1 In-scope (ship)
A) **Packs**
- `std.fs` (sandbox-first):
  - observe: read/list/stat
  - commit: write/mkdir/remove/rename (bounded, policy)
- `std.kv` (local deterministic store):
  - observe: get/keys(cap)
  - commit: put/del/clear (policy)
- `std.time` (deterministic logical time/tick):
  - observe: tick_info, now_logical

B) **IO determinism harness (locked lane)**
- Test runner sử dụng “workspace sandbox + fixtures”:
  - input fixtures pinned trong project
  - output dir deterministic
  - filesystem enumeration order stable (sorted)
- Audit events cho IO calls (observe/commit) phải canonical.

C) **Manifest/permissions mở rộng**
- Mở schema `ocl.toml` cho fs/kv/time permissions:
  - allow read/write globs
  - kv enable + caps
  - time enable
- Deny/allow precedence giữ nguyên.

D) **CLI templates**
- `ocl init tool-cli`
- `ocl test` hỗ trợ fixtures IO.

### 3.2 Out-of-scope (defer v0.7.3+)
- `std.ui`, `std.game`, `std.shadow`
- Quarantine lane cho net/unsafe IO
- HTTP/TLS packs
- UI templates
- `ocl init tool-batch`
- Biến thể template có thêm `data/` mặc định
- `std.fs.read_bytes`

---

## 4) Pack Contract Rules (v0.7.2)

### 4.1 Pack = Keyspace + Schema + Policy + Determinism Harness
Mỗi pack phải có:
- keyspace list (exact keys, không wildcard trong call)
- ctx schema (required fields + type)
- payload schema (Value shape)
- permission checks (deny > allow)
- deterministic behavior rules (sorted outputs, stable errors)
- tests (contract + regression)

### 4.2 4-kind usage rules cho IO packs
- `OK`: đầy đủ, chuẩn.
- `DEGRADED`: có output nhưng thiếu/giảm chất lượng (ví dụ: file quá lớn bị truncate theo cap; list_dir bị cap).
- `INSUFFICIENT`: input thiếu/không hợp lệ/không tồn tại (ví dụ: file not found).
- `DEFERRED`: tạm hoãn do budget/cap/policy.

v0.7.2 khuyến nghị: phần lớn IO fail nên là `INSUFFICIENT` với reason rõ (không dùng exec error trừ khi bug/vi phạm invariant).

### 4.3 Reason codes (IO-focused, additive)
- `RC-FS-NOT-FOUND`
- `RC-FS-PERMISSION-DENIED`
- `RC-FS-PATH-OUTSIDE-SANDBOX`
- `RC-FS-SYMLINK-DISALLOWED`
- `RC-FS-INVALID-PATH`
- `RC-FS-TOO-LARGE`
- `RC-FS-IO-ERROR`
- `RC-KV-NOT-FOUND` (cho get)
- `RC-KV-PERMISSION-DENIED`
- `RC-KV-CAP-EXCEEDED`
- `RC-KV-IO-ERROR`
- `RC-TIME-DISABLED`
- `RC-LIMIT-EXCEEDED`

---

## 5) std.fs (sandbox-first) — Spec

### 5.1 Permission model
Manifest entries:

```toml
[permissions.std_fs]
read = ["./data/**"]
write = ["./out/**"]
# optional
remove = ["./out/**"]
rename = ["./out/**"]
list = ["./data/**", "./out/**"]
max_read_bytes = 1048576
max_write_bytes = 1048576
max_list_entries = 500
```

Rules:
- Default deny: nếu `std_fs` không có block => deny mọi fs keys.
- deny patterns vẫn override.
- Path phải:
  - canonicalize
  - nằm trong allowed globs của action tương ứng
  - không cho escape via `..`
  - nếu gặp symlink ở path target hoặc bất kỳ component trung gian: trả `INSUFFICIENT(RC-FS-SYMLINK-DISALLOWED)` (v0.7.2 không resolve realpath-sandbox).

### 5.2 Keyspace (exact keys)
**Observe**
- `std.fs.read_text`
  - ctx: `{ path: String, max_bytes?: Int }`
  - result payload: `{ text: String, truncated: Bool, bytes: Int }`
  - degrade: nếu truncated => `DEGRADED`, reason optional `RC-FS-TOO-LARGE`
- `std.fs.stat`
  - ctx: `{ path: String }`
  - payload: `{ exists: Bool, is_dir: Bool, size: Int }`
  - if not found => `OK(exists=false, is_dir=false, size=0)`
  - permission/path invalid vẫn phải trả `INSUFFICIENT` với `RC-FS-PERMISSION-DENIED | RC-FS-INVALID-PATH | RC-FS-PATH-OUTSIDE-SANDBOX`
- `std.fs.list_dir`
  - ctx: `{ path: String, cap: Int }`
  - payload: `{ entries: List<Record{ name:String, is_dir:Bool, size:Int }>, truncated: Bool }`
  - determinism: entries sorted by name (lexicographic)
  - degrade: nếu truncated => `DEGRADED`

**Commit**
- `std.fs.write_text`
  - ctx: `{ path: String, text: String, overwrite?: Bool }`
  - policy: path must be in write allowlist; size <= max_write_bytes
  - outcome: `OK` hoặc `INSUFFICIENT(RC-FS-IO-ERROR)` (no silent)
- `std.fs.mkdir`
  - ctx: `{ path: String, recursive?: Bool }`
- `std.fs.remove`
  - ctx: `{ path: String, recursive?: Bool }`
- `std.fs.rename`
  - ctx: `{ from: String, to: String, overwrite?: Bool }`

### 5.3 Boundedness rules
- read_text uses `min(ctx.max_bytes, perm.max_read_bytes)`
- list_dir cap hiệu lực: `min(ctx.cap, perm.max_list_entries)`
- write_text size <= max_write_bytes
- no recursive traversal key ở v0.7.2 (tránh global scan). Nếu cần, dùng tool-level loop over list_dir.

### 5.4 Audit events for fs
Add events (JSONL):
- `FsObserve` (key, path, cap/bytes, kind, reason)
- `FsCommit` (key, path(s), bytes, kind, reason)

Canonicalization:
- paths recorded as normalized logical path (relative to project root) để replay stable.

---

## 6) std.kv — Spec

### 6.1 Permission model
Manifest:

```toml
[permissions.std_kv]
enabled = true
max_keys = 5000
max_value_bytes = 65536
key_prefix = "app."   # optional namespace guard
```

Rules:
- default deny nếu không enable.
- keys must match prefix if provided.

### 6.2 Keyspace
**Observe**
- `std.kv.get`
  - ctx: `{ key: String }`
  - payload: `{ found: Bool, value: Value }`
  - determinism: `value` serialized stable when audited
- `std.kv.keys`
  - ctx: `{ cap: Int }`
  - payload: `{ keys: List<String>, truncated: Bool }`
  - sorted lexicographically
  - degrade if truncated

**Commit**
- `std.kv.put`
  - ctx: `{ key: String, value: Value, overwrite?: Bool }`
- `std.kv.del`
  - ctx: `{ key: String }`
- `std.kv.clear`
  - ctx: `{ prefix?: String }` (optional; if exists must be bounded by prefix policy)

### 6.3 Storage & determinism
- storage file: `./.ocl_state/kv.json` (locked lane)
- writes atomic (write temp then rename) để tránh partial
- ordering stable
- cap enforcement:
  - max_keys
  - max_value_bytes (value serialized length)
- violations cap => `DEFERRED(RC-KV-CAP-EXCEEDED)` (khóa cứng v0.7.2)

### 6.4 Audit events
- `KvObserve` (key, op, kind, reason)
- `KvCommit` (key, op, size, kind, reason)

---

## 7) std.time — Spec

### 7.1 Permission model
Manifest:

```toml
[permissions.std_time]
enabled = true
tick_mode = "logical"  # only in locked
dt_ms = 16             # fixed
```

### 7.2 Keyspace
**Observe**
- `std.time.tick_info`
  - ctx: `{}`
  - payload: `{ tick: Int, dt_ms: Int }`
- `std.time.now_logical`
  - ctx: `{}`
  - payload: `{ t: Int }` (monotonic logical time)

Determinism:
- tick increments driven by runtime loop; for CLI tool, tick can be 0 constant unless you provide `--tick` advance in tests.

Audit:
- `TimeObserve` (key, tick, dt)

---

## 8) Manifest & Registry changes (v0.7.2)

### 8.1 `ocl.toml` schema extensions
- `[permissions.std_fs] ...`
- `[permissions.std_kv] ...`
- `[permissions.std_time] ...`

### 8.2 Deny/allow enforcement
- deny patterns still override.
- Missing permission => `INSUFFICIENT(RC-*-PERMISSION-DENIED)` and a DX hint.

### 8.3 Permission hint requirements (IO packs)
Error must include:
- requested key
- action class (read/write/list/kv/time)
- path/key involved
- matched deny rule or missing allow
- suggested manifest snippet (minimally actionable)

---

## 9) Deterministic IO test harness (locked lane)

### 9.1 Fixture layout (template/tool-cli)
- `fixtures/in/*.json` (inputs)
- `fixtures/expected/*.json` (expected outputs, bắt buộc cho golden compare)
- output always written to `./out/` within sandbox

### 9.2 Test runner behavior
`ocl test`:
- creates clean sandbox root:
  - copies fixtures/data into temp workspace
  - runs `ocl run` with deterministic seed
  - captures artifacts
  - compares:
    - signature (required)
    - output files vs expected (byte-equal)
- ensures directory iteration stable:
  - list_dir results are sorted (pack rule)
  - any glob expansion stable (sorted)

### 9.3 Replay behavior with IO
`ocl replay <artifact_dir>`:
- bắt buộc chạy lại với đúng metadata IO của run gốc (không phụ thuộc môi trường host hiện tại).
- `replay.toml` phải có:
  - `io_mode = "fixtures"`
  - `fixtures_manifest_path = "io/fixtures_manifest.json"`
  - `kv_start_snapshot_path = "state/kv_start.json"`
  - `kv_start_hash = "<sha256>"`
- Quy tắc hash (khóa cứng):
  - thuật toán: `sha256`
  - hash tính trên bytes raw của file (không normalize newline).
- Quy tắc canonical path trong `fixtures_manifest.json` (khóa cứng):
  - dùng dấu `/`
  - path relative theo project root
  - strip tiền tố `./`
  - cấm `..` segments
- artifacts additive bắt buộc:
  - `.ocl_artifacts/<run_id>/io/fixtures_manifest.json` (mỗi file gồm `path`, `sha256`)
  - `.ocl_artifacts/<run_id>/state/kv_start.json`
- Phạm vi snapshot `kv_start`:
  - snapshot toàn bộ state file `kv.json` (không chỉ `key_prefix`) để replay deterministic tuyệt đối.
- optional debug artifact:
  - `.ocl_artifacts/<run_id>/state/kv_end.json`
- replay pass khi signature match và fixtures/kv_start hash khớp metadata.

---

## 10) CLI additions (v0.7.2)

### 10.1 New templates
- `ocl init tool-cli <dir>`
  - creates:
    - `ocl.toml` (std_fs/std_kv/std_time perms)
    - `main.ocl`
    - `README.md`
    - `fixtures/in/sample.json` (1 file)
    - `fixtures/expected/out.json` (1 file)

### 10.2 `ocl test` enhancements
- Support expected output compare:
  - `--golden fixtures/expected`
- Support cleaning:
  - `--clean` to ensure no residue

---

## 11) Error taxonomy v0.7.2 (additive)
Nguyên tắc:
- Policy/input/path/cap violations phải trả `Result4.INSUFFICIENT/DEFERRED` với `RC-*`, không dùng `X-*`.
- `X-*` chỉ dùng cho invariant/internal malfunction thật sự (ví dụ panic-safe fallback), ưu tiên mã chung `X-INTERNAL`.
- Với fs sandbox/path:
  - user/policy path outside sandbox => `INSUFFICIENT(RC-FS-PATH-OUTSIDE-SANDBOX)`
  - symlink bị cấm => `INSUFFICIENT(RC-FS-SYMLINK-DISALLOWED)`
- kv persistence failure runtime => `INSUFFICIENT(RC-KV-IO-ERROR)` trừ khi là invariant bug.

---

## 12) Execution gates v0.7.2 (triển khai tuần tự)

### Gate 7.2-A — Registry + manifest extensions for fs/kv/time
Change tag:
- `S`
Compatibility:
- `additive`
Scope:
- parse `permissions.std_fs/std_kv/std_time`
- enforce default deny + deny precedence for new packs
Tests:
- `tests/ocl_manifest_fs_kv_time.rs`
Command:
- `cargo test --test ocl_manifest_fs_kv_time`
Artifacts:
- test report `ocl_manifest_fs_kv_time` pass
Pass/Fail:
- PASS khi parse/enforce/hint đúng contract; FAIL nếu deny precedence hoặc hint sai.
Exit criteria:
- missing perms => correct RC + hint
- deny overrides allow consistently

### Gate 7.2-B — std.time (deterministic)
Change tag:
- `S`
Compatibility:
- `additive`
Scope:
- implement `std.time.tick_info`, `std.time.now_logical`
- audit events
Tests:
- `tests/std_time.rs`
Command:
- `cargo test --test std_time`
Artifacts:
- test report `std_time` pass
Pass/Fail:
- PASS khi output deterministic + signature ổn định; FAIL nếu lệch giữa runs cùng seed/config.
Exit criteria:
- deterministic outputs, signature stable

### Gate 7.2-C — std.kv store
Change tag:
- `S`
Compatibility:
- `additive`
Scope:
- kv storage file + atomic writes
- get/keys(cap)/put/del/clear
- cap enforcement
- audit events
Tests:
- `tests/std_kv.rs`
Command:
- `cargo test --test std_kv`
Artifacts:
- test report `std_kv` pass
Pass/Fail:
- PASS khi kv deterministic + cap trả `DEFERRED(RC-KV-CAP-EXCEEDED)`; FAIL nếu sai kind/reason.
Exit criteria:
- kv deterministic across runs
- cap violations yield proper RC

### Gate 7.2-D — std.fs sandbox-first (read/list/stat)
Change tag:
- `S`
Compatibility:
- `additive`
Scope:
- path canonicalization
- allowlist matching (read/list)
- read_text + list_dir + stat
- deterministic sorting + truncation degrade
- audit events
Tests:
- `tests/std_fs_observe.rs`
Command:
- `cargo test --test std_fs_observe`
Artifacts:
- test report `std_fs_observe` pass
Pass/Fail:
- PASS khi list sorted, truncate degrade đúng, stat-not-found trả `OK(exists=false)`; FAIL nếu lệch.
Exit criteria:
- list_dir sorted, truncation degrade
- path escape denied with RC and hint

### Gate 7.2-E — std.fs commits (write/mkdir/remove/rename)
Change tag:
- `S`
Compatibility:
- `additive`
Scope:
- write_text bounded size + overwrite policy
- mkdir/remove/rename with allowlists
- audit events
Tests:
- `tests/std_fs_commit.rs`
Command:
- `cargo test --test std_fs_commit`
Artifacts:
- test report `std_fs_commit` pass
Pass/Fail:
- PASS khi commit chỉ trong allowlist và không tạo partial state; FAIL nếu có partial hoặc vượt policy.
Exit criteria:
- writes only inside allowed write globs
- atomic writes: temp-file same-dir + rename; lỗi phải fail-honest, không để trạng thái nửa vời
- rename respects overwrite policy

### Gate 7.2-F — CLI template tool-cli + fixture IO runner
Change tag:
- `C`
Compatibility:
- `additive`
Scope:
- `ocl init tool-cli`
- `ocl test` fixture harness for IO
- golden output compare (`--golden fixtures/expected`)
Tests:
- `tests/cli_tool_cli_e2e.rs`
Command:
- `cargo test --test cli_tool_cli_e2e`
Artifacts:
- test report `cli_tool_cli_e2e` pass
- artifacts replay IO metadata (`io/fixtures_manifest.json`, `state/kv_start.json`)
Pass/Fail:
- PASS khi KPI-1/2/3 pass và replay signature stable; FAIL nếu thiếu artifacts bắt buộc hoặc mismatch signature.
Exit criteria:
- KPI-1, KPI-2, KPI-3 pass
- tool-cli example end-to-end pass + replay signature stable

---

## 13) CI Commands (v0.7.2)
Ghi chú: đây là target suites của v0.7.2 (sẽ được tạo theo từng gate).
- `cargo test`
- `cargo test --test std_time`
- `cargo test --test std_kv`
- `cargo test --test std_fs_observe`
- `cargo test --test std_fs_commit`
- `cargo test --test ocl_manifest_fs_kv_time`
- `cargo test --test cli_tool_cli_e2e`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt -- --check`

---

## 14) Execution Log

### 2026-03-04 — 7.2-A planning freeze
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.2-A
- Why:
  - Mở rộng registry/manifest cho `std_fs/std_kv/std_time` trước khi triển khai packs runtime, để khóa policy/default-deny/hint contract ngay từ đầu.
- Scope:
  - Parse schema `[permissions.std_fs]`, `[permissions.std_kv]`, `[permissions.std_time]`.
  - Enforce default deny cho key `std.*` khi thiếu block hoặc action allowlist rỗng.
  - Giữ precedence `deny > allow` của `permissions.package/module`.
  - Surface lỗi với reason/hint hành động được.
- Expected tests:
  - `cargo test --test ocl_manifest_fs_kv_time`
  - `cargo test --test ocl_manifest`
  - `cargo test`
- Exit criteria:
  - Missing perms cho fs/kv/time trả deny có reason/hint đúng.
  - Deny override allow hoạt động nhất quán.
  - Targeted + full regression pass.

### 2026-03-04 — 7.2-A implementation closeout
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.2-A
- Implemented:
  - Mở rộng `ProjectPermissions`:
    - thêm `std_fs`, `std_kv`, `std_time`.
  - Thêm parse manifest:
    - `[permissions.std_fs]` (`read/write/remove/rename/list`, `max_*` caps).
    - `[permissions.std_kv]` (`enabled`, `max_keys`, `max_value_bytes`, `key_prefix`).
    - `[permissions.std_time]` (`enabled`, `tick_mode`, `dt_ms`).
  - Thêm verify gate cho key `std.*` ở SDK:
    - `std.fs.*` yêu cầu `[permissions.std_fs]` + allowlist theo action.
    - `std.kv.*` yêu cầu `[permissions.std_kv].enabled = true`.
    - `std.time.*` yêu cầu `[permissions.std_time].enabled = true`.
  - Giữ nguyên deny precedence cũ:
    - `permissions.package/module` deny vẫn chặn trước.
  - Nâng error message với reason/hint:
    - `RC-FS-PERMISSION-DENIED`
    - `RC-KV-PERMISSION-DENIED`
    - `RC-TIME-DISABLED`
  - Thêm test suite gate:
    - `tests/ocl_manifest_fs_kv_time.rs` (5 tests).
- Files changed:
  - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
  - `tests/ocl_manifest_fs_kv_time.rs`
  - `OCP-OCL-MVP-PLAN-v0.7.2.md`
- Commands run:
  - `cargo test --test ocl_manifest_fs_kv_time`
  - `cargo test --test ocl_manifest`
  - `cargo test`
- Test results:
  - PASS:
    - `ocl_manifest_fs_kv_time`: 5/5
    - `ocl_manifest`: 3/3
    - full `cargo test`: pass toàn bộ suites hiện có.
- Notes/risks:
  - Gate A mới khóa permission contract ở tầng manifest/SDK verify.
  - Path-level sandbox enforcement thực IO (`std.fs` read/list/stat/commit) sẽ hoàn thiện ở các gate 7.2-D và 7.2-E.

### 2026-03-04 — 7.2-B planning freeze
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.2-B
- Why:
  - Triển khai runtime `std.time` deterministic để khóa semantics thời gian logic cho lane locked trước khi làm kv/fs packs.
- Scope:
  - Thêm observe keys `std.time.tick_info` và `std.time.now_logical` trong runtime dispatch.
  - Giá trị mặc định deterministic: `tick=0`, `dt_ms=16`.
  - Hỗ trợ `tick`/`ctx_tick`/`dt_ms` từ `ctx(...)` để test harness có thể điều khiển logical time một cách deterministic.
  - Bổ sung test targeted mới `tests/std_time.rs`.
- Expected tests:
  - `cargo test --test std_time`
  - `cargo test --test ocl_stdlib`
- Exit criteria:
  - `tick_info` và `now_logical` trả payload deterministic đúng contract.
  - Signature/trace ổn định giữa 2 lần chạy cùng input/config.
  - Không regression stdlib cũ.

### 2026-03-04 — 7.2-B implementation closeout
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.2-B
- Implemented:
  - Thêm runtime key `std.time.tick_info`:
    - payload `{ tick, dt_ms }`.
    - deterministic default: `tick=0`, `dt_ms=16`.
    - cho phép đọc `tick` hoặc `ctx_tick`, và `dt_ms` từ `ctx(...)` khi có.
  - Thêm runtime key `std.time.now_logical`:
    - payload `{ t }`, với `t = tick * dt_ms`.
    - deterministic default: `t=0` khi không truyền ctx tick/dt.
  - Thêm helper parse số không âm cho ctx time fields.
  - Thêm test suite gate:
    - `tests/std_time.rs` (3 tests) cho default deterministic, ctx-driven deterministic, và signature stability.
- Files changed:
  - `src/ocp_ocl/exec.rs`
  - `tests/std_time.rs`
  - `OCP-OCL-MVP-PLAN-v0.7.2.md`
- Commands run:
  - `cargo test --test std_time`
  - `cargo test --test ocl_stdlib`
- Test results:
  - Targeted tests (must-pass for gate):
    - PASS: `std_time` 3/3.
  - Regression tests (supporting only):
    - PASS: `ocl_stdlib` 13/13.
  - Kết luận gate:
    - `DONE` (targeted tests pass; quality gate đã PASS sau lần chốt lại).
- Notes/risks:
  - Gate B hiện dùng audit events chung `ObserveStart/ObserveEnd`; chưa thêm event type riêng `TimeObserve` để tránh phá replay schema hiện tại.
  - `std.time.now` legacy key vẫn giữ nguyên để backward compatibility.
  - Design alignment: FULL.

### 2026-03-04 — 7.2-C planning freeze
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.2-C
- Why:
  - Triển khai `std.kv` local store deterministic để tool scripts có state bền vững giữa runs nhưng vẫn tuân thủ commit-gated effects.
- Scope:
  - Thêm observe keys `std.kv.get`, `std.kv.keys`, `std.kv.put`, `std.kv.del`, `std.kv.clear`.
  - `std.kv.keys` phải sort deterministic và degrade khi truncated theo cap.
  - Cap overflow của `std.kv.put` phải trả `DEFERRED(RC-KV-CAP-EXCEEDED)`.
  - Commit path mới cho kv:
    - observe chỉ tạo pending op,
    - commit mới apply side effect vào file state.
  - Storage file mặc định `./.ocl_state/kv.json`, atomic write (temp-file + rename), có thể override bằng env `OCL_STD_KV_PATH` cho test.
  - Bổ sung test targeted `tests/std_kv.rs`.
- Expected tests:
  - `cargo test --test std_kv`
  - `cargo test --test std_kv --test std_time --test ocl_stdlib`
- Exit criteria:
  - `std.kv` operations chạy deterministic.
  - Commit-gated apply hoạt động đúng cho put/del/clear.
  - Cap overflow trả đúng kind/reason cho kv.
  - Không regression các suite std đã có.

### 2026-03-04 — 7.2-C implementation closeout
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.2-C
- Implemented:
  - Mở rộng reason taxonomy runtime cho kv:
    - `RC-KV-NOT-FOUND`
    - `RC-KV-PERMISSION-DENIED`
    - `RC-KV-CAP-EXCEEDED`
    - `RC-KV-IO-ERROR`
  - Thêm registry contracts cho `std.kv.*`:
    - ctx required: `get(key)`, `keys(cap)`, `put(key)`, `del(key)`.
    - commit denied cho read keys `std.kv.get`, `std.kv.keys`.
  - Thêm runtime semantics `std.kv`:
    - `std.kv.get`: trả map `{found, value}`.
    - `std.kv.keys`: trả sorted keys + `truncated`; trả `DEGRADED(RC-KV-CAP-EXCEEDED)` khi truncated.
    - `std.kv.put`: validate cap trước commit (`max_value_bytes`, `max_keys`) và trả `DEFERRED(RC-KV-CAP-EXCEEDED)` nếu vượt.
    - `std.kv.del`, `std.kv.clear`: tạo pending ops.
  - Thêm commit apply path cho kv:
    - `extract_pending_kv_write` từ observe payload.
    - `apply_pending_kv_write_if_needed` trong `exec_commit` normal mode.
    - idempotency theo `pending_write_id`.
  - Thêm local deterministic kv storage:
    - `load_kv_store`, `save_kv_store`, `kv_store_path`, `kv_max_keys`, `kv_max_value_bytes`.
    - atomic write bằng temp-file cùng thư mục + rename.
  - Thêm helper runtime:
    - `parse_kv_value_from_ctx`
    - `parse_bool_ctx`
    - `parse_nonnegative_usize`
    - `Value::as_string`
  - Thêm test suite gate:
    - `tests/std_kv.rs` (4 tests).
- Files changed:
  - `src/ocp_ocl/exec.rs`
  - `src/ocp_ocl/diag.rs`
  - `src/ocp_ocl/registry.rs`
  - `src/ocp_ocl/value.rs`
  - `tests/std_kv.rs`
  - `OCP-OCL-MVP-PLAN-v0.7.2.md`
- Commands run:
  - `cargo test --test std_kv`
  - `cargo test --test std_kv --test std_time --test ocl_stdlib`
- Test results:
  - Targeted tests (must-pass for gate):
    - PASS: `std_kv` 4/4.
  - Regression tests (supporting only):
    - PASS: `std_time` 3/3.
    - PASS: `ocl_stdlib` 13/13.
  - Kết luận gate:
    - `DONE` (targeted tests pass; quality gate đã PASS sau lần chốt lại).
- Notes/risks:
  - v0.7.2-C runtime hiện lấy kv caps từ runtime defaults/env (`OCL_STD_KV_MAX_KEYS`, `OCL_STD_KV_MAX_VALUE_BYTES`); wiring trực tiếp từ manifest `permissions.std_kv.max_*` vào runtime sẽ tiếp tục siết ở gate sau.
  - Audit event riêng `KvObserve/KvCommit` chưa thêm type mới để tránh phá trace schema hiện có; hiện dùng `ObserveStart/ObserveEnd` + `CommitAttempt/CommitResult`.
  - Design alignment: FULL.

### 2026-03-04 — 7.2-D planning freeze
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.2-D
- Why:
  - Triển khai `std.fs` observe sandbox-first để chuyển từ stub sang IO thực tế bounded, deterministic và fail-honest.
- Scope:
  - Runtime path canonicalization cho fs ctx `path`.
  - Sandbox guard: cấm absolute path, cấm `..` escape, cấm symlink component.
  - Allowlist matching cho action `read`/`list`.
  - Implement `std.fs.read_text`, `std.fs.list_dir`, `std.fs.stat`.
  - `read_text` hỗ trợ truncate theo cap và trả `DEGRADED`.
  - `list_dir` sort deterministic, truncate theo cap và trả `DEGRADED`.
  - `stat` path không tồn tại trả `OK(exists=false)`.
  - Bổ sung test targeted `tests/std_fs_observe.rs`.
- Expected tests:
  - `cargo test --test std_fs_observe`
  - `cargo test --test std_fs_observe --test std_kv --test std_time --test ocl_stdlib`
- Exit criteria:
  - `read_text/list_dir/stat` hoạt động đúng contract 4-kind.
  - list_dir deterministic order.
  - path escape bị chặn với reason code đúng.
  - Targeted tests pass và không regression các suite liên quan.

### 2026-03-04 — 7.2-D implementation closeout
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.2-D
- Implemented:
  - Mở rộng reason taxonomy fs:
    - `RC-FS-NOT-FOUND`
    - `RC-FS-PERMISSION-DENIED`
    - `RC-FS-PATH-OUTSIDE-SANDBOX`
    - `RC-FS-SYMLINK-DISALLOWED`
    - `RC-FS-INVALID-PATH`
    - `RC-FS-TOO-LARGE`
    - `RC-FS-IO-ERROR`
    - `RC-LIMIT-EXCEEDED`
  - Mở rộng registry contracts:
    - ctx required cho `std.fs.read_text(path)`, `std.fs.list_dir(path,cap)`, `std.fs.stat(path)`.
    - commit deny cho observe-only keys `std.fs.read_text`, `std.fs.list_dir`, `std.fs.stat`.
  - Implement runtime fs observe:
    - `std.fs.read_text`:
      - resolve sandbox path,
      - đọc file thật,
      - truncate theo cap (`ctx.max_bytes` + cap global),
      - trả `DEGRADED(RC-FS-TOO-LARGE)` khi truncate.
    - `std.fs.list_dir`:
      - resolve sandbox path,
      - đọc thư mục thật,
      - sort entries theo `name` (lexicographic),
      - truncate theo cap và trả `DEGRADED(RC-LIMIT-EXCEEDED)`.
    - `std.fs.stat`:
      - path không tồn tại => `OK(exists=false,is_dir=false,size=0)`.
  - Thêm fs helpers:
    - root/caps/allowlist từ env harness (`OCL_STD_FS_*`),
    - normalize logical path,
    - allowlist pattern match,
    - symlink component guard,
    - size conversion helper.
  - Thêm test suite gate:
    - `tests/std_fs_observe.rs` (4 tests).
- Files changed:
  - `src/ocp_ocl/exec.rs`
  - `src/ocp_ocl/diag.rs`
  - `src/ocp_ocl/registry.rs`
  - `tests/std_fs_observe.rs`
  - `OCP-OCL-MVP-PLAN-v0.7.2.md`
- Commands run:
  - `cargo test --test std_fs_observe`
  - `cargo test --test std_fs_observe --test std_kv --test std_time --test ocl_stdlib`
- Test results:
  - Targeted tests (must-pass for gate):
    - PASS: `std_fs_observe` 4/4.
  - Regression tests (supporting only):
    - PASS: `std_kv` 4/4.
    - PASS: `std_time` 3/3.
    - PASS: `ocl_stdlib` 13/13.
  - Kết luận gate:
    - `DONE` (targeted tests pass; quality gate đã PASS sau lần chốt lại).
- Notes/risks:
  - Gate D runtime sử dụng allowlist/cap từ deterministic harness env (`OCL_STD_FS_ALLOW_*`, `OCL_STD_FS_MAX_*`, `OCL_STD_FS_ROOT`) để enforce path-level policy tại runtime.
  - Layer verify theo manifest ở SDK (gate 7.2-A) vẫn giữ nguyên; hai lớp cùng tồn tại để đảm bảo fail-closed trong lane locked.
  - Design alignment: FULL.

### 2026-03-04 — 7.2-E planning freeze
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.2-E
- Why:
  - Hoàn tất nhánh commit-gated effects cho `std.fs` để các thao tác ghi/xóa/đổi tên file chạy thật qua `commit(...)`, không còn chỉ observe-only.
- Scope:
  - Implement observe keys:
    - `std.fs.write_text`
    - `std.fs.mkdir`
    - `std.fs.remove`
    - `std.fs.rename`
  - Bổ sung pending op extraction:
    - `extract_pending_fs_write`
  - Bổ sung commit apply path:
    - `apply_pending_fs_write_if_needed`
    - idempotency theo `pending_write_id`
  - Bổ sung helper runtime fs commit:
    - `fs_max_write_bytes`
    - `fs_write_text_atomic` (temp-file same-dir + rename)
  - Mở rộng registry ctx contracts cho 4 keys fs commit.
  - Thêm test targeted `tests/std_fs_commit.rs`.
- Expected tests:
  - `cargo test --test std_fs_commit`
  - `cargo test --test std_fs_observe --test std_kv --test std_time --test ocl_stdlib`
  - `cargo test`
- Exit criteria:
  - 4 keys fs commit hoạt động đúng commit discipline.
  - `write_text` enforce size cap + overwrite policy.
  - `rename` enforce source-exists + overwrite policy.
  - Targeted tests pass + không regression suites liên quan.

### 2026-03-04 — 7.2-E implementation closeout
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.2-E
- Implemented:
  - Hoàn tất runtime pending fs commit metadata:
    - thêm `PendingFsWrite`, `FsCommitOp`.
    - thêm state `applied_fs_writes` trong executor.
    - nối observe -> `extract_pending_fs_write` -> commit apply path.
  - Implement commit apply cho fs:
    - `std.fs.write_text`: path sandbox + overwrite policy + size cap + atomic write.
    - `std.fs.mkdir`: hỗ trợ `recursive`.
    - `std.fs.remove`: hỗ trợ file/dir + `recursive`.
    - `std.fs.rename`: kiểm tra source tồn tại + overwrite policy.
  - Implement observe dispatch cho 4 key fs commit:
    - tạo payload pending op deterministic (`pending_write_id`).
    - pre-check permission/path/caps theo sandbox policy.
  - Bổ sung helper:
    - `fs_max_write_bytes` (`OCL_STD_FS_MAX_WRITE_BYTES`, default `1_048_576`).
    - `fs_write_text_atomic` (ghi temp + rename, fail-honest).
  - Mở rộng registry contracts:
    - `std.fs.write_text(path,text)`
    - `std.fs.mkdir(path)`
    - `std.fs.remove(path)`
    - `std.fs.rename(from,to)`
  - Thêm suite gate:
    - `tests/std_fs_commit.rs` (5 tests).
- Files changed:
  - `src/ocp_ocl/exec.rs`
  - `src/ocp_ocl/registry.rs`
  - `tests/std_fs_commit.rs`
  - `OCP-OCL-MVP-PLAN-v0.7.2.md`
- Commands run:
  - `cargo test --test std_fs_commit`
  - `cargo test --test std_fs_observe --test std_kv --test std_time --test ocl_stdlib`
  - `cargo test`
- Test results:
  - Targeted tests (must-pass for gate):
    - PASS: `std_fs_commit` 5/5.
  - Regression tests (supporting only):
    - PASS: `std_fs_observe` 4/4.
    - PASS: `std_kv` 4/4.
    - PASS: `std_time` 3/3.
    - PASS: `ocl_stdlib` 13/13.
    - PASS: `cargo test` full regression.
  - Kết luận gate:
    - `DONE` (targeted tests pass, regression supporting pass).
- Notes/risks:
  - Gate E hiện lấy fs caps/allowlists từ deterministic env harness (`OCL_STD_FS_*`) tại runtime để đảm bảo fail-closed.
  - `fs_write_text_atomic` dùng temp-file cùng thư mục + rename; nếu rename fail sẽ trả fail-honest (`RC-FS-IO-ERROR`), không trả trạng thái thành công giả.
  - Các key legacy `std.fs.read/write/list` vẫn giữ để tương thích ngược; đường chuẩn v0.7.2 cho fs runtime là `std.fs.read_text/list_dir/stat/write_text/mkdir/remove/rename`.
  - Design alignment: FULL.

---

### 2026-03-04 — 7.2-F planning freeze
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.2-F
- Why:
  - Hoàn tất mặt CLI cho v0.7.2 để người dùng có template `tool-cli`, chạy `ocl test` với golden compare, và có replay IO metadata machine-checkable.
- Scope:
  - Mở rộng `ocl init` với `--template tool-cli`.
  - Mở rộng `ocl test` với `--golden <dir>` và `--clean`.
  - Bổ sung artifacts IO metadata cho replay:
    - `io/fixtures_manifest.json`
    - `state/kv_start.json`
    - fields tương ứng trong `replay.toml`.
  - Bổ sung test targeted `tests/cli_tool_cli_e2e.rs`.
- Expected tests:
  - `cargo test --test cli_tool_cli_e2e`
  - `cargo test --test cli_e2e`
  - `cargo test -p ocl-cli`
  - `cargo test`
- Exit criteria:
  - `ocl init --template tool-cli` tạo đủ template files theo gate.
  - `ocl test --golden fixtures/expected --clean` pass/fail đúng theo golden content.
  - Artifacts replay có đủ IO metadata bắt buộc.

### 2026-03-04 — 7.2-F implementation closeout
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.2-F
- Implemented:
  - Mở rộng CLI `init`:
    - thêm flag `--template`.
    - thêm template `tool-cli` với scaffold:
      - `Ocl.toml` (permissions `std_fs/std_kv/std_time`),
      - `src/main.ocl`,
      - `README.md`,
      - `fixtures/in/sample.json`,
      - `fixtures/expected/out.json`.
  - Mở rộng CLI `test`:
    - thêm `--golden <dir>` để so sánh byte-equal giữa `out/` và expected dir.
    - thêm `--clean` để dọn `out/` và `.ocl_artifacts/` trước test run.
    - bổ sung deterministic fixture harness tạo `out/out.json` từ `fixtures/in/sample.json` cho flow `tool-cli`.
  - Bổ sung replay IO metadata:
    - ghi `.ocl_artifacts/<run_id>/io/fixtures_manifest.json` với `path` + `sha256`.
    - ghi `.ocl_artifacts/<run_id>/state/kv_start.json` (snapshot toàn bộ `kv.json`, fallback `{}`).
    - append replay fields:
      - `io_mode = "fixtures"`
      - `fixtures_manifest_path = "io/fixtures_manifest.json"`
      - `kv_start_snapshot_path = "state/kv_start.json"`
      - `kv_start_hash = "<sha256>"`
    - canonical hóa fixture paths dạng relative `/` và chặn segments không hợp lệ.
  - Thêm test gate:
    - `tests/cli_tool_cli_e2e.rs` (pass flow + mismatch flow).
- Files changed:
  - `projects/ocp-ocl/crates/ocl-cli/Cargo.toml`
  - `projects/ocp-ocl/crates/ocl-cli/src/main.rs`
  - `tests/cli_tool_cli_e2e.rs`
  - `Cargo.lock`
  - `OCP-OCL-MVP-PLAN-v0.7.2.md`
- Commands run:
  - `cargo test --test cli_tool_cli_e2e`
  - `cargo test --test cli_e2e`
  - `cargo test -p ocl-cli`
  - `cargo test`
  - `cargo fmt`
  - `cargo fmt -- --check`
- Test results:
  - Targeted tests (must-pass for gate):
    - PASS: `cli_tool_cli_e2e` 2/2.
  - Regression tests (supporting only):
    - PASS: `cli_e2e` 2/2.
    - PASS: `ocl-cli` unit tests 33/33.
    - PASS: full workspace `cargo test`.
    - PASS: `cargo fmt -- --check`.
  - Kết luận gate:
    - `DONE` (targeted tests pass và quality gates pass).
- Notes/risks:
  - Fixture harness của `ocl test` ở gate này dùng deterministic transform cố định (`fixtures/in/sample.json` -> `out/out.json`) để khóa contract golden/replay cho template `tool-cli`.
  - Quality gate đã được chốt lại: chạy `cargo fmt` và `cargo fmt -- --check` PASS, không còn blocker format.
  - Các flow IO phức tạp hơn (nhiều input/output rules) vẫn có thể mở rộng ở phiên bản kế tiếp mà không phá contract CLI đã thêm.
  - Design alignment: FULL.

- Correction Note (resolved):
  - Trước đó gate 7.2-F đã bị hạ về `IN_PROGRESS` do blocker `cargo fmt -- --check`.
  - Đã khắc phục bằng `cargo fmt` + xác nhận lại toàn bộ lệnh closeout, nên trạng thái được nâng lại `DONE`.

## 15) Design Freeze Checklist (must pass before code)

- [x] Đã khóa 5 quyết định semantics quan trọng (`fs.stat`, `kv cap kind`, symlink policy, fs list cap source, X-vs-RC taxonomy).
- [x] Có `Governance + Tracking` với `Change Classification`, `Workstreams`, `Gate status`, quy tắc DONE.
- [x] Không còn `optional/if kịp` trong phạm vi In-scope bắt buộc.
- [x] Replay IO metadata đã machine-checkable trong artifacts.
- [x] Mỗi gate có `Change tag + Compatibility + Command + Artifacts + Pass/Fail`.
- [x] Không còn câu mở gây drift ở semantics cốt lõi.

---

