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
- WS-S (packs semantics + policy + determinism): `IN_PROGRESS` (7.2-A done, 7.2-B..F pending)
- WS-C (CLI template + IO test/replay UX): `TODO`
- WS-I (workspace mapping/internal wiring): `IN_PROGRESS` (7.2-A wiring started)

#### Gate status (7.2-A .. 7.2-F)
- Gate 7.2-A (Registry + manifest extensions for fs/kv/time): `DONE` (2026-03-04)
- Gate 7.2-B (std.time deterministic): `TODO`
- Gate 7.2-C (std.kv store): `TODO`
- Gate 7.2-D (std.fs observe sandbox-first): `TODO`
- Gate 7.2-E (std.fs commit keys): `TODO`
- Gate 7.2-F (CLI template tool-cli + fixture IO runner): `TODO`

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

### YYYY-MM-DD — 7.2-X planning
- Date:
- Gate/Step:
- Why:
- Scope:
- Expected tests:
- Exit criteria:

### YYYY-MM-DD — 7.2-X implementation closeout
- Date:
- Gate/Step:
- Implemented:
- Files changed:
- Commands run:
- Test results:
- Notes/risks:

---

## 15) Design Freeze Checklist (must pass before code)

- [x] Đã khóa 5 quyết định semantics quan trọng (`fs.stat`, `kv cap kind`, symlink policy, fs list cap source, X-vs-RC taxonomy).
- [x] Có `Governance + Tracking` với `Change Classification`, `Workstreams`, `Gate status`, quy tắc DONE.
- [x] Không còn `optional/if kịp` trong phạm vi In-scope bắt buộc.
- [x] Replay IO metadata đã machine-checkable trong artifacts.
- [x] Mỗi gate có `Change tag + Compatibility + Command + Artifacts + Pass/Fail`.
- [x] Không còn câu mở gây drift ở semantics cốt lõi.

---

