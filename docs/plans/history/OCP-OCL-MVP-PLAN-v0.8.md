# OCL v0.8 — Quarantine + Cassette Record/Replay for Non-deterministic IO (Deterministic-by-Recording)

Ngày tạo: 2026-03-03  
Trạng thái: `DONE` (2026-03-04)  
Phạm vi: **OCL-only**.  
Tiền đề: v0.7.1–v0.7.3 đã có:
- core Value/JSON/import/sugar/loops + audit/signature/replay,
- manifest permissions + lane `locked`,
- packs tool-grade (`std.fs/std.kv/std.time`) và consumer (`std.ui/std.game/std.shadow`) nếu bạn đã làm đến v0.7.3.

Mục tiêu v0.8: mở rộng OCL để làm **app/tool có network / process / wallclock** mà vẫn giữ được “điểm mạnh độc đáo”:
> **Deterministic-by-recording**: IO nondet không bị cấm tuyệt đối, mà đi qua capability boundary để **record thành cassette**, rồi replay ra y hệt.

---

## 0) Governance + Tracking v0.8

### 0.1 Quy ước cập nhật bắt buộc
- Mọi thay đổi kế hoạch phải cập nhật file này trước khi code.
- Mọi triển khai xong phải cập nhật log ngay sau khi chạy test.
- Không nhảy gate: gate sau chỉ mở khi gate hiện tại `DONE`.
- Chỉ chuyển gate sang `DONE` khi có đủ:
  - `Planning Freeze` + `Implementation Closeout`
  - `Files changed`, `Commands run`, `Test results`, `Notes/risks`
  - targeted tests pass cho đúng scope gate.

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
  - Mở lane `quarantine` + cassette record/replay cho IO nondeterministic, giữ deterministic-by-recording.
- Trạng thái tổng quan:
  - `ACTIVE`; Gate `8-A`, `8-B`, `8-C`, `8-D`, `8-E` đều `DONE` (2026-03-04).
- Gate đang làm/đã xong/chưa làm:
  - Toàn bộ `8-A .. 8-E` đã xong; xem `0.3 Trạng thái Workstreams/Gates v0.8`.
- Bước kế tiếp ngay:
  - Chuẩn bị closeout/handoff sang `v0.9` theo mục `16`.
- Lệnh kiểm chứng chuẩn:
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
- Danh sách file code trọng yếu đã thay đổi:
  - `src/ocp_ocl/exec.rs`
  - `src/ocp_ocl/diag.rs`
  - `src/ocp_ocl/audit.rs`
  - `src/ocp_ocl/registry.rs`
  - `projects/ocp-ocl/crates/ocl-cli/src/main.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
  - `tests/std_proc_exec.rs`
  - `tests/std_net_http.rs`
  - `tests/cli_tool_http_e2e.rs`
  - `tests/cli_tool_proc_e2e.rs`
  - `tests/cli_tool_wallclock_e2e.rs`

### 0.3 Trạng thái Workstreams/Gates v0.8 (TRACKING)

#### Workstreams
- WS-S (quarantine semantics + cassette contracts): `DONE` (2026-03-04)
- WS-C (CLI/templates/e2e replay): `DONE` (2026-03-04)
- WS-I (workspace wiring nội bộ + tích hợp crate): `DONE` (2026-03-04)

#### Gate status (8-A .. 8-E)
- Gate 8-A (Quarantine lane core + cassette plumbing): `DONE` (2026-03-04)
- Gate 8-B (std.time.wallclock with cassette): `DONE` (2026-03-04)
- Gate 8-C (std.proc.exec with cassette): `DONE` (2026-03-04)
- Gate 8-D (std.net.http.request with cassette): `DONE` (2026-03-04)
- Gate 8-E (CLI templates + E2E): `DONE` (2026-03-04)

### 0.4 Scope khóa cho v0.8

#### In-scope bắt buộc
- Lane `quarantine` với env gate + policy manifest.
- Cassette record/replay core (`cassette.jsonl/index/meta`) + replay fail-honest.
- `std.time.wallclock`, `std.proc.exec`, `std.net.http.request` bản tối thiểu, có cassette-backed path.
- Signature/cassette hash deterministic, có bằng chứng replay offline.
- CLI flow `--record/--replay` + templates tool-grade (`tool-http/tool-proc/tool-wallclock`).

#### Out-of-scope / deferred sau v0.8
- Nondet capability ngoài 3 pack tối thiểu (websocket, fs watch, process streaming nâng cao...).
- Cassette quarantine cross-machine orchestration nâng cao (merge/conflict tự động).
- Grammar/language features lớn mới không liên quan trực tiếp v0.8.

---

## 1) Goals v0.8 (LOCKED)

### 1.1 North Star
- Lane `locked` vẫn deterministic tuyệt đối (không net/proc/wallclock thật).
- Lane `quarantine` cho phép IO nondet **nhưng bắt buộc**:
  - được cấp quyền rõ trong `ocl.toml`,
  - record cassette đầy đủ,
  - replay chạy lại đúng (deterministic-by-recording) khi dùng cassette.

### 1.2 KPI bắt buộc (định lượng)
**KPI-1: Cassette correctness**
- Cùng script + cùng cassette => `ocl replay` reproduces:
  - `signature.txt` (cassette-normalized) ổn định
  - output files/state giống (byte-equal nếu trong sandbox)
- Cassette phải đủ để replay **không cần** truy cập net/proc/wallclock.

**KPI-2: Quarantine gate**
- Không set env `OCL_QUARANTINE=1` => mọi capability nondet trả `DEFERRED(RC-QUARANTINE-REQUIRED)` + hint.
- Có env + có permission => mới được chạy.

**KPI-3: Minimal nondet packs**
- Ship tối thiểu 3 nondet packs:
  - `std.net.http` (subset)
  - `std.proc.exec` (subset)
  - `std.time.wallclock` (read-only)
- Mỗi pack có record/replay path (cassette-backed) trong quarantine.

**KPI-4: Supply of evidence**
- Templates:
  - `tool-http` (fetch JSON -> transform -> write output)
  - `tool-proc` (run command -> parse output -> write report)
  - `tool-wallclock` (read time -> format -> write)
- Mỗi template có:
  - `ocl run --record` tạo cassette
  - `ocl replay` chạy offline và signature ổn định

---

## 2) Axis Lock v0.8 (kế thừa, không đổi)
- No naked IO; IO qua capability.
- 4-kind everywhere.
- Commit-gated effects.
- Boundedness (step cap, enumerate cap, budget cap).
- Diagnostics structured.
- Determinism is default; quarantine is explicit and auditable.

---

## 3) Lanes & Determinism Policy (v0.8)

### 3.1 Lane `locked_v071` (default)
- Trong tài liệu này, từ `locked` chỉ là alias mô tả cho `locked_v071`, không phải literal manifest.
- **Cấm** gọi:
  - `std.net.*`, `std.proc.*`, `std.time.wallclock.*`
- Nếu gọi => `DEFERRED(RC-QUARANTINE-REQUIRED)` + hint chuyển qua lane `quarantine` + record.

### 3.2 Lane quarantine (explicit)
- Bật bằng:
  - `ocl.toml: [project] lane="quarantine"`
  - env: `OCL_QUARANTINE=1`
- Canonical lane literals cho v0.8:
  - `locked_v071` (default)
  - `locked_v06` (compat)
  - `quarantine` (nondet-by-recording)
- Quarantine bắt buộc cassette policy:
  - mode `record` hoặc `replay`
  - không có cassette trong replay => fail-honest (`INSUFFICIENT(RC-CASSETTE-MISSING)`)

### 3.3 Signature policy trong quarantine
Vì quarantine có “real world IO”, signature phải được định nghĩa rõ để không làm người dùng nhầm:

- **Run mode record**:
  - signature quarantine = `sha256(canonical_audit_bytes_with_cassette_entry_ids)`.
  - cassette content hash riêng:
    - `entry_hash = sha256(canonical_entry_bytes)`
    - `cassette_hash = sha256(concat(entry_hashes_in_order) + canonical_index + canonical_meta_excluding_created_at)`
- **Run mode replay**:
  - signature phải match record signature khi dùng đúng cassette + đúng lane.

Artifacts:
- `signature.txt`: signature của execution (đã cassette-normalized)
- `cassette_hash.txt`: hash của cassette bundle
- `replay.toml`: bắt buộc có `lane`, `mode`, `cassette_hash`, `hasher_version`

---

## 4) Cassette System (v0.8 core feature)

### 4.1 Cassette là gì
Cassette là một “bundle” ghi lại:
- request parameters (đã canonicalize)
- response/result (đã canonicalize)
- metadata (timestamps, status, exit codes…)
- mapping từ “capability call id” -> “cassette entry id”

Cassette phải:
- deterministic (stable order)
- portable (chạy lại offline)
- bounded (size caps, truncation rules)

### 4.2 Cassette file format (v0.8)
Directory:
- `./.ocl_artifacts/<run_id>/cassette/`
  - `cassette.jsonl`
  - `cassette_index.json`
  - `cassette_meta.toml`

Precedence (khóa cứng):
- Canonical cassette cho replay luôn nằm trong `./.ocl_artifacts/<run_id>/cassette/`.
- `[quarantine].cassette_dir` chỉ là export/import convenience, không phải nguồn mặc định của replay.
- Replay path:
  1) ưu tiên cassette trong artifacts,
  2) chỉ fallback sang `cassette_dir` khi artifacts cassette thiếu,
  3) fallback bắt buộc verify `cassette_hash` trùng `replay.toml`; mismatch => fail-honest.

**cassette.jsonl**: mỗi dòng 1 entry:
- `seq`: int (0-based, canonical; tăng đúng theo thứ tự append vào `cassette.jsonl`)
- `id`: string (stable)
- `cap`: string (e.g. `std.net.http.request`)
- `req`: record (canonical request)
- `res`: record (canonical response)
- `kind`: kind string
- `reason_code` optional
- `hash`: entry hash (canonical bytes)

**cassette_index.json**
- mapping `{ call_id -> cassette_entry_id }`
- optional reverse index by request hash
- `seq` được xem là canonical index khi xử lý tie-break ở fallback replay.

**cassette_meta.toml**
- schema_version
- created_at (wallclock; excluded from signature)
- caps: max_bytes, max_entries
- policy: redaction settings (nếu có)
- hasher_version = `"sha256-v1"`

### 4.3 Cassette canonicalization rules
- Canonical bytes rule:
  - UTF-8,
  - line ending `LF`,
  - không normalize newline ngoài canonical serializer.
- `canonical_entry_bytes` dùng canonical Value encoding đã khóa từ v0.7.1
  (key order lexicographic UTF-8 bytes, UTF-8 strings, không pretty-print).
- Stable map key order.
- Normalize URLs (lowercase scheme/host, default ports removed).
- Normalize headers: lowercase names, sorted.
- Normalize proc env: only allowed keys, sorted.
- Normalize wallclock: store full raw but signature/hash inputs exclude `created_at`.

### 4.4 Cassette redaction (tối thiểu v0.8)
Mục tiêu: tránh ghi secrets.
- `ocl.toml` cho phép cấu hình:
  - redact headers: `Authorization`, `Cookie`
  - redact env keys: `*_TOKEN`, `*_KEY`
- Redaction phải được ghi dấu trong cassette_meta để biết replay có thể thiếu fields.
- Redaction áp dụng **trước** khi tính `req_hash`/`entry_hash`.
- Field bị redact phải thay bằng placeholder cố định `\"<redacted>\"`, không xóa field.
- Audit events không được ghi raw secret; chỉ ghi placeholder hoặc hash đã canonicalize.

### 4.5 Cassette cap overflow behavior (LOCKED)
- Khi đang record, nếu append entry mới làm vượt `max_entries` hoặc `max_cassette_bytes`:
  - không thực hiện real IO cho call đó,
  - trả `DEFERRED(RC-CASSETTE-TOO-LARGE)`,
  - ghi audit marker rõ `cassette_cap_hit`,
  - cassette bundle hiện tại vẫn phải hợp lệ và deterministic.
- Không có trạng thái partial entry trong `cassette.jsonl`.

---

## 5) Manifest extensions (v0.8)

### 5.1 project lane + cassette mode
`ocl.toml`:

```toml
[project]
name = "tool-http"
entry = "main.ocl"
lane = "quarantine"

[quarantine]
mode = "record"          # "record" | "replay"
cassette_dir = "./cassette"
max_cassette_bytes = 10485760
max_entries = 2000
redact_headers = ["authorization", "cookie"]
redact_env_patterns = ["*_TOKEN", "*_KEY"]
```

Lane literals:
- `lane = "locked_v071"` (default)
- `lane = "locked_v06"` (compat)
- `lane = "quarantine"` (nondet-by-recording)

### 5.2 permissions for nondet packs
```toml
[permissions.std_net_http]
enabled = true
allow_hosts = ["api.github.com", "example.com"]
allow_methods = ["GET", "POST"]
max_body_bytes = 1048576
timeout_ms = 5000

[permissions.std_proc]
enabled = true
allow_bins = ["git", "python", "node"]
allow_args_glob = ["*"]
timeout_ms = 5000
max_stdout_bytes = 1048576
max_stderr_bytes = 1048576

[permissions.std_time_wallclock]
enabled = true
```

Rule:
- deny patterns vẫn override.
- missing permission => `INSUFFICIENT(RC-*-PERMISSION-DENIED)` + hint.
- Permission denied không dùng `DEFERRED`.

---

## 6) std.net.http — Spec (quarantine-only)

### 6.1 Keyspace
**Observe**
- `std.net.http.request`
  - ctx:
    - `{ method:String, url:String, headers:Map<String,String>?, body:String?, timeout_ms:Int? }`
  - payload:
    - `{ status:Int, headers:Map<String,String>, body:String, truncated:Bool }`
  - boundedness:
    - body cap = min(ctx cap, perm.max_body_bytes)
    - headers cap count/bytes (configurable)
  - 4-kind:
    - OK: success response within caps
    - DEGRADED: truncated body or headers
    - INSUFFICIENT: invalid URL, DNS fail, timeout, host/method denied, permission denied
    - DEFERRED: quarantine required hoặc cap/budget gate

### 6.2 Cassette behavior
- record mode:
  - execute real HTTP request
  - write cassette entry with canonical req + res
  - return res to program
- replay mode:
  - primary: nếu có mapping `call_id -> entry_id` thì dùng mapping.
  - fallback: lookup `(cap, req_hash)` khi thiếu mapping call_id.
  - nếu nhiều match fallback: chọn entry có `seq` nhỏ nhất.
  - nếu không tìm thấy: `INSUFFICIENT(RC-CASSETTE-MISS)`.

### 6.3 Reason codes (http)
- `RC-NET-INVALID-URL`
- `RC-NET-DNS-FAIL`
- `RC-NET-TIMEOUT`
- `RC-NET-TLS-FAIL` (nếu TLS fail)
- `RC-NET-HOST-DENIED`
- `RC-NET-METHOD-DENIED`
- `RC-CASSETTE-MISS`
- `RC-QUARANTINE-REQUIRED`

### 6.4 Audit events
- `NetHttpObserve` (method, host, path_hash, mode=record|replay, kind, reason, cassette_entry_id?)

---

## 7) std.proc.exec — Spec (quarantine-only)

### 7.1 Keyspace
**Observe**
- `std.proc.exec`
  - ctx:
    - `{ bin:String, args:List<String>, env:Map<String,String>?, cwd:String?, timeout_ms:Int? }`
  - payload:
    - `{ exit_code:Int, stdout:String, stderr:String, truncated:Bool }`
  - boundedness:
    - stdout/stderr capped (perm.max_stdout_bytes, perm.max_stderr_bytes)
  - 4-kind:
    - OK: executed
    - DEGRADED: output truncated
    - INSUFFICIENT: bin denied, bin not found, timeout, exec fail (spawn fail)
    - DEFERRED: quarantine required hoặc cap/budget gate
  - Quy tắc nonzero:
    - `exit_code != 0` vẫn là `OK`, payload giữ nguyên `exit_code/stdout/stderr`.

### 7.2 Cassette behavior
- record: run process thật, record ctx normalized + result
- replay:
  - primary: nếu có mapping `call_id -> entry_id` thì dùng mapping.
  - fallback: lookup `(cap, req_hash)` khi thiếu mapping call_id.
  - nếu nhiều match fallback: chọn entry có `seq` nhỏ nhất.
  - nếu không tìm thấy: `INSUFFICIENT(RC-CASSETTE-MISS)`.

### 7.3 Reason codes (proc)
- `RC-PROC-BIN-DENIED`
- `RC-PROC-TIMEOUT`
- `RC-PROC-NOT-FOUND`
- `RC-PROC-EXEC-FAIL`
- `RC-CASSETTE-MISS`
- `RC-QUARANTINE-REQUIRED`

### 7.4 Audit events
- `ProcObserve` (bin, args_hash, mode, kind, reason, cassette_entry_id?)

---

## 8) std.time.wallclock — Spec (quarantine-only)

### 8.1 Keyspace
**Observe**
- `std.time.wallclock.now`
  - ctx: `{}`
  - payload: `{ unix_ms:Int, iso:String }`
  - 4-kind:
    - OK always in record mode
    - replay returns recorded time
    - DEFERRED if quarantine required

### 8.2 Cassette behavior
- record: store observed time into cassette
- replay: return recorded time based on call order (call_id mapping), not request hash (since ctx empty)

### 8.3 Reason codes
- `RC-QUARANTINE-REQUIRED`
- `RC-CASSETTE-MISSING`
- `RC-CASSETTE-MISS`

### 8.4 Audit events
- `WallclockObserve` (mode, cassette_entry_id?)

---

## 9) Replay semantics & call identity

### 9.1 call_id definition
Mỗi capability call trong exec có stable `call_id`:
- `call_id` là `u64` counter monotonic tăng theo thứ tự thực thi,
- bắt đầu từ `0` tại `ProgramStart`,
- ghi vào audit + cassette_index.

### 9.2 replay lookup strategy
- Primary:
  - nếu `cassette_index` có `call_id -> entry_id` thì dùng mapping này.
- Fallback (chỉ khi thiếu call_id mapping):
  - lookup theo `(cap, req_hash)`,
  - nếu nhiều match thì chọn entry có `seq` nhỏ nhất (`seq` là 0-based canonical index trong `cassette.jsonl`).
- For wallclock:
  - luôn dùng call_id mapping (order-dependent, deterministic).

Rule:
- Replay must be fail-honest on miss, not silently call real world.

---

## 10) CLI extensions (v0.8)

### 10.1 Run flags
- `ocl run --record` (forces quarantine.mode=record)
- `ocl run --replay` (forces quarantine.mode=replay)
- `ocl cassette inspect <dir>` (optional)
- `ocl cassette redact <dir> --rules ...` (optional)

Precedence matrix (LOCKED):
- Flags chỉ override `quarantine.mode`, không override `lane`.
- `lane` luôn đọc từ `ocl.toml`.
- Nếu `lane != quarantine` mà dùng `--record` hoặc `--replay` => fail-honest + hint set `lane="quarantine"`.
- Nếu dùng đồng thời `--record` và `--replay` => fail với `CLI-ARG-CONFLICT`.
- Mọi run lane `quarantine` (record hoặc replay) đều bắt buộc `OCL_QUARANTINE=1`.

### 10.2 Template init
- `ocl init tool-http`
- `ocl init tool-proc`
- `ocl init tool-wallclock`

Each template includes:
- `ocl.toml` quarantine + permissions
- `main.ocl`
- `README.md`
- small fixtures (optional)
- instructions how to run record then replay offline

---

## 11) Error taxonomy (v0.8 additive)

### 11.1 New exec errors (avoid if possible)
- `X-QUARANTINE-DISABLED` (only if you decide to hard error; preferred return Result4.DEFERRED with RC)

### 11.2 New RC codes (global)
- `RC-QUARANTINE-REQUIRED`
- `RC-CASSETTE-MISSING`
- `RC-CASSETTE-MISS`
- `RC-CASSETTE-TOO-LARGE`
- Không dùng `RC-LOCKED-DENY` trong v0.8; dùng thống nhất `RC-QUARANTINE-REQUIRED`.
- Permission-denied classes phải surface qua `INSUFFICIENT(RC-*-PERMISSION-DENIED|...-DENIED)`.

---

## 12) Execution gates v0.8 (triển khai tuần tự)

### Gate 8-A — Quarantine lane core + cassette plumbing
Scope:
- lane gate (env + manifest)
- cassette writer/reader
- call_id system
- signature normalization with cassette hashes
Tests:
- `tests/quarantine_gate.rs`
- `tests/cassette_format.rs`
Exit criteria:
- locked denies nondet with correct RC + hint
- quarantine record creates cassette bundle
- replay uses cassette and reproduces signature

### Gate 8-B — std.time.wallclock with cassette
Scope:
- implement wallclock.now (quarantine-only)
- record/replay by call_id
Tests:
- `tests/std_time_wallclock.rs`
Exit criteria:
- record then replay => identical output + signature

### Gate 8-C — std.proc.exec with cassette
Scope:
- proc exec adapter, bounded outputs, record/replay
- permission allow_bins
Tests:
- `tests/std_proc_exec.rs`
Exit criteria:
- replay offline returns same stdout/stderr/exit_code

### Gate 8-D — std.net.http.request with cassette
Scope:
- http request adapter, bounded body, record/replay
- host/method allowlist
Tests:
- `tests/std_net_http.rs` (use local test server or mock; in CI prefer deterministic local server)
Exit criteria:
- replay offline returns same response payload
- deny host => RC-NET-HOST-DENIED + hint

### Gate 8-E — CLI templates + E2E
Scope:
- init templates
- run record and replay flows
Tests:
- `tests/cli_tool_http_e2e.rs`
- `tests/cli_tool_proc_e2e.rs`
- `tests/cli_tool_wallclock_e2e.rs`
Exit criteria:
- KPI suite pass (cassette correctness + quarantine gate + templates)

---

## 13) Operational commands (v0.8)
- `cargo test`
- `cargo test --test quarantine_gate`
- `cargo test --test cassette_format`
- `cargo test --test std_time_wallclock`
- `cargo test --test std_proc_exec`
- `cargo test --test std_net_http`
- `cargo test --test cli_tool_http_e2e`
- `cargo test --test cli_tool_proc_e2e`
- `cargo test --test cli_tool_wallclock_e2e`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt -- --check`

---

## 14) Execution Log (template)

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
- `cargo test --test <targeted_suite>`
- `cargo test`
- Test results:
- Targeted tests (must-pass for gate):
  - `PASS`/`FAIL`
- Regression tests (supporting only):
  - `PASS`/`FAIL`
- Kết luận gate:
  - `DONE` chỉ khi targeted tests pass.
- Notes/risks:

---

### 2026-03-04 — 8-A Planning Freeze
- Date:
  - 2026-03-04
- Gate/Step:
  - 8-A
- Why:
  - Mở lane `quarantine` bản v0.8 với cassette plumbing cốt lõi, bảo đảm replay fail-honest và signature có cassette hash.
- Scope:
  - Mở rộng `ocl-cli` để:
    - parse/ghi metadata replay v0.8 (`mode`, `cassette_hash`, `hasher_version`),
    - tạo cassette bundle chuẩn trong `.ocl_artifacts/<run_id>/cassette/`,
    - validate cassette bundle khi `ocl replay`,
    - giữ tương thích lane cũ `locked_v06|locked_v071`.
  - Thêm targeted tests cho gate:
    - `tests/quarantine_gate.rs`
    - `tests/cassette_format.rs`
- Expected tests:
  - `cargo test --test quarantine_gate --test cassette_format`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
- Exit criteria:
  - Quarantine lane bắt buộc `OCL_QUARANTINE=1`.
  - Quarantine run phải sinh cassette bundle đầy đủ (`cassette.jsonl`, `cassette_index.json`, `cassette_meta.toml`, `cassette_hash.txt`).
  - Replay quarantine phải fail-honest nếu cassette hash mismatch.
  - Targeted tests của gate pass.

### 2026-03-04 — 8-A Implementation Closeout
- Date:
  - 2026-03-04
- Gate/Step:
  - 8-A
- Implemented:
  - Nâng `projects/ocp-ocl/crates/ocl-cli/src/main.rs`:
    - mở rộng parse `replay.toml` với các field v0.8: `mode`, `cassette_hash`, `hasher_version`,
    - thêm cassette core plumbing:
      - tạo `cassette/` bundle khi lane `quarantine`,
      - sinh `cassette_index.json` (call_id map + `next_call_id`),
      - sinh `cassette_meta.toml` + `cassette_hash.txt`,
      - validate bundle + hash khi replay,
    - signature path cho lane quarantine được gắn với `cassette_hash` (deterministic).
  - Thêm targeted tests:
    - `tests/quarantine_gate.rs`:
      - gate env quarantine + replay metadata + cassette files.
    - `tests/cassette_format.rs`:
      - kiểm tra format/hash cassette và replay fail-honest khi tamper index.
- Files changed:
  - `projects/ocp-ocl/crates/ocl-cli/src/main.rs`
  - `tests/quarantine_gate.rs`
  - `tests/cassette_format.rs`
  - `OCP-OCL-MVP-PLAN-v0.8.md`
- Commands run:
  - `cargo test --test quarantine_gate --test cassette_format`
  - `cargo test`
  - `cargo fmt`
  - `cargo fmt -- --check`
  - `cargo clippy --all-targets -- -D warnings`
- Test results:
  - Targeted tests (must-pass for gate):
    - `PASS`:
      - `quarantine_gate`: 1/1
      - `cassette_format`: 1/1
  - Regression tests (supporting only):
    - `PASS`: full `cargo test` pass toàn bộ suite.
  - Kết luận gate:
    - `DONE` (đủ code delta + targeted tests + regression + lint/fmt).
- Notes/risks:
  - Cassette entries cho nondeterministic packs hiện là plumbing core; record payload thật cho `std.time.wallclock/std.proc.exec/std.net.http` sẽ hoàn tất ở các gate `8-B..8-D`.
  - Design alignment: `FULL` (đúng scope 8-A, không mở vượt scope).

### 2026-03-04 — 8-B Planning Freeze
- Date:
  - 2026-03-04
- Gate/Step:
  - 8-B
- Why:
  - Bổ sung capability nondeterministic đầu tiên cho v0.8 (`std.time.wallclock.now`) để chứng minh cassette record/replay hoạt động end-to-end theo `call_id`.
- Scope:
  - Runtime:
    - Thêm observe key `std.time.wallclock.now` trong runtime core.
    - Chặn lane `locked_v071|locked_v06`, chỉ cho phép lane `quarantine`.
    - Ở record mode: đọc wallclock thật và trả `OK(payload={ unix_ms, iso })`.
    - Ở replay mode: đọc từ cassette theo `call_id`, không được fallback ra world thật.
  - CLI/cassette:
    - Ghi entry cassette cho `std.time.wallclock.now` (id/cap/seq/call_id/kind/req/res/hash).
    - Replay match ưu tiên `call_id -> entry_id`; miss trả fail-honest.
  - Test:
    - Thêm targeted suite `tests/std_time_wallclock.rs`.
- Expected tests:
  - `cargo test --test std_time_wallclock`
  - `cargo test`
  - `cargo fmt -- --check`
  - `cargo clippy --all-targets -- -D warnings`
- Exit criteria:
  - `std.time.wallclock.now` chạy được ở quarantine record/replay.
  - Replay cho wallclock không gọi world thật và giữ deterministic với cùng cassette.
  - Locked lane gọi wallclock trả fail-honest đúng contract.

### 2026-03-04 — 8-B Implementation Closeout
- Date:
  - 2026-03-04
- Gate/Step:
  - 8-B
- Implemented:
  - Runtime core:
    - thêm reason codes quarantine/cassette cho v0.8 (`RC-QUARANTINE-REQUIRED`, `RC-CASSETTE-MISSING`, `RC-CASSETTE-MISS`, `RC-CASSETTE-TOO-LARGE`).
    - thêm observe path cho `std.time.wallclock.now` theo lane/mode:
      - lane khác `quarantine` => `DEFERRED(RC-QUARANTINE-REQUIRED)`,
      - `record` => đọc wallclock thật và trả `OK(payload={unix_ms,iso})`,
      - `replay` => đọc từ cassette theo `call_id`, miss => `INSUFFICIENT(RC-CASSETTE-MISSING|RC-CASSETTE-MISS)`.
    - thêm `call_id` counter cho quarantine observe flow.
  - CLI/cassette:
    - bổ sung env bridge runtime (`OCL_PROJECT_LANE`, `OCL_QUARANTINE_MODE`, cassette paths) cho run/replay.
    - nâng cassette bundle writer để ingest wallclock record temp file và ghi:
      - `cassette.jsonl` entries có `seq,id,cap,call_id,kind,req,res,hash`,
      - `cassette_index.json` mapping `call_id -> entry_id`.
  - Audit/SDK:
    - thêm trace event `WallclockObserve` và canonical payload mapping để signature/replay trace phản ánh giá trị wallclock từ cassette.
  - Tests:
    - thêm suite `tests/std_time_wallclock.rs` (record/replay + lane/env gate + cassette assertions).
- Files changed:
  - `src/ocp_ocl/diag.rs`
  - `src/ocp_ocl/exec.rs`
  - `src/ocp_ocl/audit.rs`
  - `src/ocp_ocl/registry.rs`
  - `projects/ocp-ocl/crates/ocl-cli/src/main.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
  - `tests/std_time_wallclock.rs`
  - `OCP-OCL-MVP-PLAN-v0.8.md`
- Commands run:
  - `cargo test --test std_time_wallclock`
  - `cargo test --test quarantine_gate --test cassette_format --test std_time_wallclock`
  - `cargo test`
  - `cargo fmt`
  - `cargo fmt -- --check`
  - `cargo clippy --all-targets -- -D warnings`
- Test results:
  - Targeted tests (must-pass for gate):
    - `PASS`:
      - `std_time_wallclock`: 1/1
      - `quarantine_gate`: 1/1
      - `cassette_format`: 1/1
  - Regression tests (supporting only):
    - `PASS`: full `cargo test` pass toàn bộ suite.
  - Kết luận gate:
    - `DONE` (đủ code delta + targeted tests + regression + lint/fmt).
- Notes/risks:
  - Wallclock replay hiện phụ thuộc cassette index mapping theo `call_id` (đúng thiết kế v0.8), cần giữ ổn định ordering khi mở rộng thêm nondet packs ở `8-C/8-D`.
  - Design alignment: `FULL` (đúng scope 8-B, không mở vượt scope).

### 2026-03-04 — 8-C Planning Freeze
- Date:
  - 2026-03-04
- Gate/Step:
  - 8-C
- Why:
  - Mở capability nondeterministic thứ hai của v0.8 (`std.proc.exec`) để hoàn thiện trục quarantine record/replay cho process IO, giữ fail-honest và boundedness.
- Scope:
  - Runtime:
    - thêm observe key `std.proc.exec` theo lane/mode quarantine.
    - record mode chạy process thật có cap `stdout/stderr` + timeout.
    - replay mode chỉ đọc cassette theo `call_id`, không fallback ra world thật.
  - Permission contract:
    - bổ sung parse/check `[permissions.std_proc]` (`enabled`, `allow_bins`, caps) trong SDK/CLI path.
    - runtime enforce allowlist `bin` theo config đã map.
  - Cassette:
    - ghi/read entry `std.proc.exec` với `call_id`, `exit_code`, `stdout`, `stderr`, `truncated`.
  - Tests:
    - thêm targeted suite `tests/std_proc_exec.rs`.
- Expected tests:
  - `cargo test --test std_proc_exec`
  - `cargo test`
  - `cargo fmt -- --check`
  - `cargo clippy --all-targets -- -D warnings`
- Exit criteria:
  - `std.proc.exec` chạy được ở quarantine record/replay, replay không gọi world thật.
  - `exit_code != 0` vẫn `OK` theo contract.
  - output truncation trả `DEGRADED` có payload/cờ `truncated` đúng.
  - bin ngoài allowlist bị chặn fail-honest đúng reason code.

### 2026-03-04 — 8-C Implementation Closeout
- Date:
  - 2026-03-04
- Gate/Step:
  - 8-C
- Implemented:
  - Runtime/quarantine:
    - thêm observe key `std.proc.exec` theo lane/mode v0.8 (`record` chạy process thật, `replay` đọc cassette theo `call_id`).
    - enforce fail-honest: lane không phải `quarantine` -> `DEFERRED(RC-QUARANTINE-REQUIRED)`, cassette miss -> `INSUFFICIENT(RC-CASSETTE-MISSING|RC-CASSETTE-MISS)`.
    - enforce contract proc: `exit_code != 0` vẫn `OK`; output vượt cap -> `DEGRADED` + `truncated=true`.
  - Permission/registry:
    - bổ sung reason codes proc trong taxonomy theo spec v0.8 (bin-denied, timeout, not-found, exec-error).
    - bổ sung parse/check `[permissions.std_proc]` ở SDK/CLI và mapping runtime env cho allow bins + caps.
    - cập nhật registry runtime cho `std.proc.exec` (`ctx_required=bin`, commit policy deny).
  - Cassette/audit:
    - thêm trace event `ProcObserve` + canonical payload mapping trong SDK trace.
    - mở rộng cassette bundle writer để ingest proc temp-record và ghi entry `std.proc.exec` có `call_id`, `exit_code`, `stdout/stderr`, `truncated`.
  - Tests:
    - thêm `tests/std_proc_exec.rs`:
      - record/replay pass với nonzero exit + truncation.
      - deny bin ngoài allowlist với reason code đúng.
- Files changed:
  - `src/ocp_ocl/diag.rs`
  - `src/ocp_ocl/exec.rs`
  - `src/ocp_ocl/audit.rs`
  - `src/ocp_ocl/registry.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
  - `projects/ocp-ocl/crates/ocl-cli/src/main.rs`
  - `tests/std_proc_exec.rs`
  - `OCP-OCL-MVP-PLAN-v0.8.md`
- Commands run:
  - `cargo test --test std_proc_exec`
  - `cargo fmt -- --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`
- Test results:
  - Targeted tests (must-pass for gate):
    - `PASS`:
      - `std_proc_exec`: 2/2
  - Regression tests (supporting only):
    - `PASS`: full `cargo test` pass toàn bộ suite.
  - Kết luận gate:
    - `DONE` (đủ code delta + targeted tests + regression + lint/fmt).
- Notes/risks:
  - Proc replay đang khóa theo `call_id` mapping; cần giữ ổn định execution ordering khi mở thêm nondet packs (`8-D`, `8-E`).
  - Cấu hình `[permissions.std_proc]` hiện được bridge sang runtime qua env trong CLI; khi mở rộng schema permissions phải cập nhật đồng bộ SDK/CLI/runtime.
  - Design alignment: `FULL` (đúng scope 8-C, không mở vượt scope).

### 2026-03-04 — 8-D Planning Freeze
- Date:
  - 2026-03-04
- Gate/Step:
  - 8-D
- Why:
  - Mở capability nondeterministic thứ ba của v0.8 (`std.net.http.request`) để hoàn thiện bộ tối thiểu net/proc/wallclock cho deterministic-by-recording.
- Scope:
  - Runtime:
    - thêm observe key `std.net.http.request` theo lane/mode quarantine.
    - record mode gọi HTTP thật (subset tối thiểu) với timeout/body cap.
    - replay mode đọc cassette theo `call_id`; chỉ fallback `(cap, req_hash)` khi thiếu mapping.
  - Permission contract:
    - parse/check `[permissions.std_net_http]` (`enabled`, `allow_hosts`, `allow_methods`, `timeout_ms`, `max_body_bytes`) ở SDK/CLI.
    - runtime enforce host/method allowlist theo config đã map.
  - Cassette/audit:
    - ghi/read entry `std.net.http.request` với `call_id`, `method`, `url`, `req_hash`, `status`, `body`, `truncated`.
    - thêm trace event cho net observe để replay/signature phản ánh đúng đường đi.
  - Tests:
    - thêm targeted suite `tests/std_net_http.rs`:
      - record/replay local HTTP server (offline replay phải pass không gọi network).
      - deny host/method ngoài allowlist với reason code đúng.
- Expected tests:
  - `cargo test --test std_net_http`
  - `cargo fmt -- --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`
- Exit criteria:
  - `std.net.http.request` chạy được ở quarantine record/replay; replay không gọi world thật.
  - `host/method` deny trả `INSUFFICIENT(RC-NET-HOST-DENIED|RC-NET-METHOD-DENIED)`.
  - URL invalid/DNS fail/timeout trả reason code đúng theo spec v0.8.
  - có đủ code delta + targeted tests pass + regression/lint/fmt pass mới được `DONE`.

### 2026-03-04 — 8-D Implementation Closeout
- Date:
  - 2026-03-04
- Gate/Step:
  - 8-D
- Implemented:
  - Runtime/quarantine:
    - thêm observe key `std.net.http.request` theo lane/mode v0.8 (`record` + `replay`).
    - enforce contract:
      - lane không phải `quarantine` -> `DEFERRED(RC-QUARANTINE-REQUIRED)`,
      - replay dùng `call_id -> entry_id` làm primary, fallback `req_hash`,
      - miss cassette -> `INSUFFICIENT(RC-CASSETTE-MISSING|RC-CASSETTE-MISS)`.
    - implement reason mapping net theo spec v0.8:
      - invalid-url, dns-error, timeout, tls-error,
      - host-denied, method-denied.
    - body cap bounded + truncated -> `DEGRADED(RC-LIMIT-EXCEEDED)`.
  - Audit/cassette:
    - thêm trace event `NetHttpObserve` và mapping canonical payload.
    - mở rộng runtime record tmp + CLI cassette bundle writer để ghi/read entry `std.net.http.request`:
      - `seq,id,cap,call_id,kind,req(method/url/req_hash),res(status/body_hex/headers_hex/truncated),hash`.
    - thêm deterministic mock adapter `mock.local` cho test record/replay ổn định trong CI (không phụ thuộc network ngoài).
  - Permission contract:
    - SDK parse/check thêm `[permissions.std_net_http]`:
      - `enabled`, `allow_hosts`, `allow_methods`, `timeout_ms`, `max_body_bytes`.
    - CLI bridge env runtime cho std.net.http:
      - `OCL_STD_NET_ALLOW_HOSTS`,
      - `OCL_STD_NET_ALLOW_METHODS`,
      - `OCL_STD_NET_TIMEOUT_MS`,
      - `OCL_STD_NET_MAX_BODY_BYTES`.
  - Tests:
    - thêm `tests/std_net_http.rs`:
      - record/replay mock adapter + truncation,
      - deny host ngoài allowlist,
      - deny method ngoài allowlist.
- Files changed:
  - `src/ocp_ocl/diag.rs`
  - `src/ocp_ocl/audit.rs`
  - `src/ocp_ocl/exec.rs`
  - `src/ocp_ocl/registry.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
  - `projects/ocp-ocl/crates/ocl-cli/src/main.rs`
  - `tests/std_net_http.rs`
  - `OCP-OCL-MVP-PLAN-v0.8.md`
- Commands run:
  - `cargo test --test std_net_http`
  - `cargo fmt`
  - `cargo fmt -- --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`
- Test results:
  - Targeted tests (must-pass for gate):
    - `PASS`:
      - `std_net_http`: 3/3
  - Regression tests (supporting only):
    - `PASS`: full `cargo test` pass toàn bộ suite.
  - Kết luận gate:
    - `DONE` (đủ code delta + targeted tests + regression + lint/fmt).
- Notes/risks:
  - Record path hiện hỗ trợ cả HTTP thật (host thường) và deterministic mock (`mock.local`) cho CI/test ổn định.
  - Khi mở `8-E`, template `tool-http` cần cấu hình sẵn `[permissions.std_net_http]` để không fail permission ở lane quarantine.
  - Design alignment: `FULL` (đúng scope 8-D, không mở vượt scope).

### 2026-03-04 — 8-E Planning Freeze
- Date:
  - 2026-03-04
- Gate/Step:
  - 8-E
- Why:
  - Hoàn tất lane quarantine ở lớp DX bằng template khởi tạo tool-grade + E2E record/replay, tránh để v0.8 dừng ở mức runtime-only.
- Scope:
  - Mở rộng `ocl init` cho 3 template:
    - `tool-http`
    - `tool-proc`
    - `tool-wallclock`
  - Đảm bảo template chạy được `run`/`replay` trên lane `quarantine` có env gate.
  - Bổ sung targeted E2E tests riêng cho từng template.
  - Không mở thêm capability mới ngoài scope `std.net.http/std.proc/std.time.wallclock`.
- Expected tests:
  - `cargo test --test cli_tool_http_e2e`
  - `cargo test --test cli_tool_proc_e2e`
  - `cargo test --test cli_tool_wallclock_e2e`
  - `cargo test`
- Exit criteria:
  - 3 template mới init/run/replay pass ổn định.
  - Có targeted tests cho từng template và đều pass.
  - Không vi phạm lane gate (`OCL_QUARANTINE=1` vẫn bắt buộc).
  - Có closeout đầy đủ evidence theo mẫu bắt buộc.

### 2026-03-04 — 8-E Implementation Closeout
- Date:
  - 2026-03-04
- Gate/Step:
  - 8-E
- Implemented:
  - CLI template surface:
    - thêm `tool-http`, `tool-proc`, `tool-wallclock` vào `ocl init`.
    - thêm generator cho 3 template với:
      - `Ocl.toml` lane `quarantine`,
      - permissions tối thiểu theo pack,
      - `src/main.ocl`,
      - `README.md`.
  - Runtime/CLI wiring để template chạy thật:
    - bổ sung bridge env `std_fs` từ `[permissions.std_fs]` trong manifest vào runtime env (`OCL_STD_FS_ROOT`, allowlists, max caps).
    - giữ bridge env đã có cho `std_proc`/`std_net_http`.
    - thêm mock deterministic `mock.proc` trong runtime proc observe path để E2E ổn định không phụ thuộc binary host.
  - Tests:
    - giữ `tests/cli_tool_http_e2e.rs` và mở rộng theo flow gate.
    - thêm `tests/cli_tool_proc_e2e.rs`.
    - thêm `tests/cli_tool_wallclock_e2e.rs`.
    - cả 3 test đều verify:
      - init template thành công,
      - run/replay fail khi thiếu `OCL_QUARANTINE=1`,
      - run/replay pass khi có env,
      - cassette có capability entry tương ứng.
- Files changed:
  - `projects/ocp-ocl/crates/ocl-cli/src/main.rs`
  - `src/ocp_ocl/exec.rs`
  - `tests/cli_tool_http_e2e.rs`
  - `tests/cli_tool_proc_e2e.rs`
  - `tests/cli_tool_wallclock_e2e.rs`
  - `OCP-OCL-MVP-PLAN-v0.8.md`
- Commands run:
  - `cargo test --test cli_tool_http_e2e`
  - `cargo test --test cli_tool_proc_e2e`
  - `cargo test --test cli_tool_wallclock_e2e`
  - `cargo test`
  - `cargo fmt -- --check`
  - `cargo clippy --all-targets -- -D warnings`
- Test results:
  - Targeted tests (must-pass for gate):
    - `PASS`:
      - `cli_tool_http_e2e`: 1/1
      - `cli_tool_proc_e2e`: 1/1
      - `cli_tool_wallclock_e2e`: 1/1
  - Regression tests (supporting only):
    - `PASS`:
      - `cargo test`: full suite pass.
      - `cargo fmt -- --check`: pass.
      - `cargo clippy --all-targets -- -D warnings`: pass.
  - Kết luận gate:
    - `DONE` (đủ code delta + targeted tests + regression/hygiene checks).
- Notes/risks:
  - Lỗi ban đầu trong gate: template `tool-http` fail `commit(...)` do thiếu runtime env bridge cho `[permissions.std_fs]`; đã fix bằng `fs_runtime_env_updates_v08` trong CLI run/replay flow.
  - `tool-proc` dùng `mock.proc` deterministic cho E2E để tránh phụ thuộc môi trường host, phù hợp mục tiêu replay ổn định.
  - Design alignment: `FULL` (đúng scope 8-E, không mở vượt scope).

---

## 15) Checklist khóa trước khi đóng gate
- [x] Gate status cập nhật đúng (`TODO/IN_PROGRESS/DONE`).
- [x] Có đủ Planning Freeze + Implementation Closeout.
- [x] `Files changed` khớp code delta thực tế.
- [x] `Commands run` là lệnh đã chạy thật.
- [x] `Test results` có `PASS/FAIL` rõ và đúng phạm vi.
- [x] Không còn mâu thuẫn giữa gate status và closeout.
- [x] Không còn lỗi encoding/hiển thị tiếng Việt trong nội dung file.

---

## 16) Handoff v0.8 -> v0.9 (pre-draft)
- Chỉ mở v0.9 sau khi 8-A..8-E đều `DONE` và KPI v0.8 pass đầy đủ.
- V0.9 kế thừa nguyên xi lane/cassette contracts đã khóa tại v0.8; mọi thay đổi phải qua migration contract rõ.
- Nếu v0.8 còn gate `IN_PROGRESS/PARTIAL`, không mở scope v0.9 ngoài việc xử lý blocker tồn đọng.

---

