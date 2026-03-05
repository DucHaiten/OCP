# OCL v0.13 — Performance via IR + Reproducible Caching (Determinism-preserving)

Ngày tạo: 2026-03-03  
Trạng thái: `DRAFT (LOCK WHEN CODING)`  
Phạm vi: **OCL-only**.  
Tiền đề: v0.11 đã có trace/index/debugger; v0.10 có deps+lock; v0.9 có schema typing; v0.7.x có core runtime; v0.12 có shadow scheduling + reuse.  
Mục tiêu v0.13: tăng hiệu năng thực tế (app/game mượt, build nhanh) mà **không phá determinism**, bằng:
- IR (intermediate representation) ổn định,
- compile cache (module cache),
- execution cache cho pure subgraphs,
- capability result caching hợp lệ (deterministic/cassette-based),
- verifyable caching nhờ signature/audit.

---

## 0) Governance + Tracking v0.13

### 0.1 Quy ước cập nhật bắt buộc
- Mọi thay đổi kế hoạch phải cập nhật file này trước khi code.
- Mọi triển khai xong phải cập nhật log ngay sau khi chạy test.
- Không nhảy gate: gate sau chỉ mở khi gate hiện tại `DONE`.
- Chỉ chuyển gate sang `DONE` khi có đủ:
  - `Planning Freeze` + `Implementation Closeout`
  - `Files changed`, `Commands run`, `Test results`, `Notes/risks`
  - targeted tests pass cho đúng scope gate.
- Nếu chưa đạt 100% scope gate:
  - giữ trạng thái `TODO` hoặc `IN_PROGRESS`, không gán `DONE`.

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
  - Ship IR + reproducible caching để tăng hiệu năng nhưng giữ determinism/signature invariance.
- Trạng thái tổng quan:
  - `DONE`; Gate 13-A, 13-B, 13-C, 13-D, 13-E, 13-F đã hoàn tất code + test theo scope.
- Gate đang làm/đã xong/chưa làm:
  - Đã xong: `13-A`, `13-B`, `13-C`, `13-D`, `13-E`, `13-F`; Chưa làm: không còn.
- Bước kế tiếp ngay:
  - Rà soát liên phiên bản trước khi mở `v0.14` theo handoff.
- Lệnh kiểm chứng chuẩn:
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
- Danh sách file code trọng yếu đã thay đổi:
  - `Cargo.lock`
  - `Cargo.toml`
  - `projects/ocp-ocl/crates/ocl-cli/src/main.rs`
  - `projects/ocp-ocl/crates/ocl-runtime-core/src/lib.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
  - `src/ocp_ocl/budget.rs`
  - `src/ocp_ocl/compile_cache.rs`
  - `src/ocp_ocl/exec.rs`
  - `src/ocp_ocl/hir.rs`
  - `src/ocp_ocl/mod.rs`
  - `src/ocp_ocl/registry.rs`
  - `tests/compile_cache.rs`
  - `tests/cli_cache.rs`
  - `tests/exec_cache_pure.rs`
  - `tests/ir_hash.rs`
  - `tests/observe_cache.rs`
  - `tests/ocl_registry.rs`

### 0.3 Trạng thái Workstreams/Gates v0.13 (TRACKING)
#### Workstreams
- WS-IR (HIR canonicalization + hash stability): `DONE` (2026-03-05)
- WS-CC (compile cache + invalidation): `DONE` (2026-03-05)
- WS-EC (exec cache + generation counters): `DONE` (2026-03-05)
- WS-CLI (cache commands + benchmark harness): `DONE` (2026-03-05)

#### Gate status (13-A .. 13-F)
- Gate 13-A (HIR + stable hashing): `DONE` (2026-03-05)
- Gate 13-B (Compile cache): `DONE` (2026-03-05)
- Gate 13-C (Per-run exec cache for pure nodes): `DONE` (2026-03-05)
- Gate 13-D (Capability cacheability framework): `DONE` (2026-03-05)
- Gate 13-E (Incremental UI/game caching): `DONE` (2026-03-05)
- Gate 13-F (CLI cache commands + perf benchmarks): `DONE` (2026-03-05)

### 0.4 Scope khóa cho v0.13
#### In-scope bắt buộc
- IR canonicalization + hash stable.
- Compile cache key/proof đầy đủ theo contract lane/lock/trust.
- Exec cache per-run an toàn với invalidation (`fs_generation`, `kv_generation`).
- Cache telemetry tách riêng `perf_cache.jsonl` (không ảnh hưởng signature semantic).
- CLI `ocl cache stats|clean` + benchmark protocol machine-checkable.

#### Out-of-scope / deferred sau v0.13
- JIT/native codegen.
- Speculative execution và parallel compilation distributed.
- Optimization làm thay đổi semantics hoặc không chứng minh được determinism.

---

## 1) Goals v0.13 (LOCKED)

### 1.1 North Star
- OCL chạy nhanh hơn đáng kể trong các template (tool-ui, mini-game, shadow-preview).
- Build/import nhanh: module compile cache + dependency graph cache.
- Cache không tạo “heisenbugs”: mọi cache entry có proof hash; mismatch => invalidate.
- Audit/trace ghi nhận cache hits/misses để debug.

### 1.2 KPI bắt buộc (định lượng)
**KPI-1: Compile efficiency (deterministic counters)**
- Cold build of medium project: giảm >= 40% `modules_compiled_count` hoặc `typecheck_runs` so với v0.12 baseline (không cache).
- Warm build (no code change): `compile_cache_hit_ratio >= 80%` và `modules_compiled_count <= 20%` so với cold baseline.
- Wallclock build time chỉ là chỉ số tham khảo (informational), không dùng làm tiêu chí pass chính.

**KPI-2: Runtime efficiency (deterministic counters)**
- Mini-game template: `steps_executed_total` (hoặc `node_eval_count`) giảm >= 25% cho frame compute path.
- Shadow-preview: tổng `steps_executed_total` giảm thêm nhờ reuse + IR speedup, trong khi `steps_charged_total` giữ semantics.
- CPU time chỉ là chỉ số tham khảo (informational), không dùng làm tiêu chí pass chính.

**KPI-3: Cache safety**
- Nếu source/module/deps/manifest/caps thay đổi => cache phải tự invalidate (không dùng nhầm).
- Mọi run phải có telemetry `cache_hit`/`cache_miss` theo loại trong `perf_cache.jsonl`.

**KPI-4: Determinism**
- Với lane locked: signature trước và sau bật cache phải **giống hệt**.
- Với quarantine+cassette: signature normalized vẫn giống.

### 1.3 KPI protocol (machine-checkable, LOCKED)
- Dataset cố định:
  - template `shadow-preview` v0.12 đã ship.
  - seed cố định `42`, caps cố định theo template benchmark.
- So sánh bắt buộc:
  - cold-cache run (cache trống) vs warm-cache run (cache đã primed).
  - `--no-cache` run để so semantics/signature.
- Source-of-truth cho pass/fail KPI:
  - counters từ audit/report artifacts (`steps_charged`, `steps_executed`, cache hit/miss counters),
  - không dùng wallclock host làm tiêu chí pass chính.
- Điều kiện pass:
  - KPI-1/2 theo ngưỡng counters deterministic đã khóa,
  - KPI-3 theo invalidation correctness,
  - KPI-4 theo signature equality (`cache on` vs `--no-cache`).

---

## 2) Axis Lock v0.13 (bất biến)
- Determinism is king: cache must not change semantics.
- Cache is an optimization; must be fully disable-able (`--no-cache`) and auditable.
- No caching nondeterministic results unless cassette-backed or explicitly deterministic-class.
- Lane literal khóa cứng:
  - `locked_v071` (default), `locked_v06` (compat), `quarantine`.
  - Trong tài liệu, `locked` chỉ là alias mô tả cho `locked_v071`, không phải literal manifest.
- Hasher policy khóa cứng:
  - Mọi cache key/proof/IR hash dùng `sha256-v1`.
  - Không dùng FNV cho bất kỳ key/proof ảnh hưởng correctness.

---

## 3) Scope v0.13

### 3.1 In-scope (ship)
A) **IR pipeline**
- Parse AST -> Typechecked HIR (high-level IR) -> LIR (lower-level) optional
- IR must be stable and hashable.

B) **Compile cache**
- Cache typecheck + IR results per module
- Keyed by:
  - compiler + OCL version
  - lane literal (`locked_v071`/`locked_v06`/`quarantine`)
  - lock/trust/deps graph hashes
  - module content + source bundle hashes
  - manifest compat flags (affect parsing/typing)
  - schema versions of packs
- Stored in `./.ocl_cache/compile/`

C) **Execution cache (pure subgraphs)**
- Identify pure expressions/subtrees:
  - JSON parse/stringify
  - record/list/map transformations
  - engine widget layout (if pure)
- Cache evaluated results:
  - keyed by (IR node id, input values hash, lane, generation counters if mutable state involved)
- Stored per-run or persistent:
  - persistent only if safe across runs (seed independent).

D) **Capability result caching**
- Locked deterministic keys: allow memoization across ticks/runs (optional strict rules).
- Quarantine: reuse cassette as cache; no additional.
- Keys must declare `cacheability`:
  - `NoCache`
  - `WithinRun` (memo per run)
  - `AcrossRuns` (safe if ctx includes all dependencies, no hidden state)
- Keys must declare `determinism_class`:
  - `Deterministic`
  - `CassetteBased`
  - `NonDeterministic`
- Migration contract (additive):
  - Keys cũ mặc định map sang `Deterministic + NoCache` (fail-safe).
  - Gate 13-D chỉ được `DONE` khi mọi std key có giá trị explicit cho cả `determinism_class` và `cacheability`.

E) **Incremental evaluation**
- For UI/game loop: incremental state update path:
  - if input unchanged, reuse previous computed draw-list or layout (pure cached)
- Must be bounded and invalidation correct.

F) **Audit integration**
- Signature contract (LOCKED):
  - `audit.jsonl` chỉ chứa semantic events và là nguồn băm signature.
  - Cache telemetry tách riêng sang `./.ocl_artifacts/<run_id>/perf_cache.jsonl`.
- Cache telemetry event types (ghi trong `perf_cache.jsonl`):
  - `CacheCompileHit/Miss`
  - `CacheExecHit/Miss`
  - `CacheObserveHit/Miss`
- Privacy:
  - chỉ ghi digest của cache keys, không ghi raw key material.

### 3.2 Out-of-scope (defer)
- JIT or native codegen.
- Advanced escape analysis.
- Parallel compilation (optional later).
- Speculative execution.

### 3.3 Module -> Crate -> Path mapping (LOCKED)
- IR canonicalization + runtime memo/invalidation counters:
  - crate: `ocl-runtime-core`
  - path: `projects/ocp-ocl/crates/ocl-runtime-core/src/*`
- Cache resolver/index/store + registry cacheability metadata wiring:
  - crate: `ocl-sdk`
  - path: `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
- CLI commands `ocl cache stats|clean` + benchmark harness entry:
  - crate: `ocl-cli`
  - path: `projects/ocp-ocl/crates/ocl-cli/src/main.rs`
- Rule:
  - Gate không được mở nếu chưa map rõ module -> crate -> path.

---

## 4) IR Design (v0.13)

### 4.1 HIR (High-level IR)
- Explicit nodes for:
  - literals, record/list/map ops
  - calls (pure vs capability)
  - try/guard desugared
  - loops (bounded constructs)
  - match 4-kind
- Node ids stable within module after canonicalization.

HIR properties:
- Typed: each node has type.
- Has `purity` flag:
  - Pure
  - CapabilityObserve
  - CapabilityCommit
  - Mixed (shouldn’t happen after desugar; enforce)

### 4.2 Canonicalization rules for IR hashing
- stable ordering for record literal fields (sorted by name)
- stable ordering for map keys (string sort)
- normalize whitespace irrelevant
- normalize module ids via lock graph
- canonical bytes:
  - UTF-8
  - line ending `LF` cho text canonicalized bytes
  - stable integer/string encoding theo canonical Value rules đã khóa ở v0.8
- hasher:
  - `ir_hash = sha256-v1(canonical_hir_bytes)`

IR hash không dùng FNV/hasher không versioned cho correctness path.

---

## 5) Compile Cache (v0.13)

### 5.1 Cache key
`compile_cache_key_v1 = sha256(
  compiler_version,
  ocl_version,
  lane_literal,
  lock_hash,
  trust_hash,
  deps_graph_hash,
  manifest_hash,
  schema_versions_of_packs,
  entry_module_hash,
  source_bundle_hash,
  cache_toggle_inputs
)`

Trong đó:
- `lock_hash`: hash lockfile canonical bytes.
- `trust_hash`: hash của trust decisions (`DepsResolved`) đã khóa ở v0.10.
- `cache_toggle_inputs`: danh sách toggle được phép ảnh hưởng cache (xem 5.5).
- Fail-safe: thiếu bất kỳ thành phần nào của key => `cache miss` bắt buộc.
- Canonicalization rule cho các thành phần hash:
  - sort keys lexicographic (UTF-8),
  - UTF-8 encoding,
  - `LF` cho text canonical bytes,
  - không tính comment/whitespace không semantic.

### 5.2 Cache entry
- typecheck output (symbol table summary)
- HIR serialized
- diagnostics (optional)
- export list (for deps)

### 5.3 Invalidation rules
Invalidate if any component of cache_key changes.
Never reuse across different compiler_version.
- Always invalidate across lane/trust/lock changes.
- Không có partial reuse nếu key material không đầy đủ.

### 5.4 CLI controls
- `ocl build` default uses cache
- `ocl build --no-cache`
- `ocl cache clean`
- `ocl cache stats`
- CLI compatibility contract:
  - `ocl cache` là namespace additive, không đổi behavior lệnh cũ.
  - `ocl cache stats` chỉ đọc, không mutate state.
  - `ocl cache clean` yêu cầu `--yes` để tránh xóa nhầm và làm lệch benchmark.

### 5.5 Env toggle policy (LOCKED)
- `locked_v071`:
  - không cho env toggle âm thầm đổi semantics cache.
  - env toggle liên quan cache chỉ hợp lệ nếu nằm trong allow-list và được đưa vào `cache_toggle_inputs`.
- Allow-list v0.13:
  - `OCL_CACHE_DISABLE` (`0|1`)
  - `OCL_CACHE_PROFILE` (`default|strict`)
- Với toggles ngoài allow-list:
  - locked lane: force `no-cache` cho run hiện tại + emit audit warning + ghi marker vào artifacts.
- Legacy shadow toggles từ v0.12 (`OCL_STD_SHADOW_CHECKPOINT_REUSE`, `OCL_STD_SHADOW_MEMOIZE_DETERMINISTIC_OBSERVE`):
  - không được coi là nguồn semantics trong locked lane,
  - chỉ dùng cho benchmark/dev profile khi đã được audit và đưa vào `cache_toggle_inputs`.

---

## 6) Execution Cache for Pure Subgraphs

### 6.1 Purity detection
A node is cacheable if:
- pure, no reads of tick/seed unless included as explicit input
- does not depend on env vars hidden from IR
- does not depend on mutable external state unless generation counter included in key

### 6.2 Cache key
`exec_cache_key_v1 = sha256(
  module_ir_hash,
  node_id,
  input_values_hash,
  seed_if_used,
  tick_if_used,
  fs_generation_if_used,
  kv_generation_if_used,
  lane_literal
)`

Generation counters (runtime state):
- `fs_generation`: tăng khi commit thành công các op `std.fs.write_text|mkdir|remove|rename`.
- `kv_generation`: tăng khi commit thành công các op mutate `std.kv.*`.
- Read/memo keys phụ thuộc FS/KV bắt buộc include generation tương ứng.

### 6.3 Storage policy
- Per-run cache always safe.
- Persistent cache only if:
  - node is marked `DeterministicAcrossRuns`
  - input hash includes everything

Default v0.13:
- persistent only for compile cache
- exec cache is per-run (safer), optional persistent behind flag.
- Với `std.fs.read_text` và các read phụ thuộc state mutable:
  - `WithinRun` chỉ hợp lệ khi key include generation counter,
  - `AcrossRuns` mặc định `NoCache` trừ khi có snapshot pinning contract riêng.

---

## 7) Capability result caching rules

### 7.1 cacheability declaration in registry
Each key has:
- `determinism_class`:
  - `Deterministic`
  - `CassetteBased`
  - `NonDeterministic`
- `cacheability`:
  - `NoCache`
  - WithinRun
  - AcrossRuns (rare, requires strict ctx)

Compatibility/additive rules:
- Key không có declaration explicit => coi là cấu hình lỗi ở gate 13-D (không auto suy diễn cho key mới).
- Với std keys hiện hữu, migration default an toàn: `Deterministic + NoCache`, sau đó mở rộng có chủ đích theo từng key.

### 7.2 Locked keys
- `std.time.tick_info` is deterministic but depends on tick => ctx must include tick or runtime must treat tick as implicit input.
- `std.fs.read_text` depends on filesystem state => AcrossRuns unsafe unless sandbox snapshot pinned; default WithinRun only.
- Nếu dùng `WithinRun` cho `std.fs.read_text`, memo key bắt buộc include `fs_generation`.

### 7.3 Quarantine keys
- `std.net.http.request` uses cassette; treat cassette lookup as cache.
- do not add extra caching layer.
- `CassetteBased` keys trong quarantine không được thêm tầng memo độc lập ngoài cassette lookup.

---

## 8) Incremental UI/game evaluation

### 8.1 Use case
- `engine.ui.widgets` layout is pure given:
  - frame_info
  - widget state
  - input events
If input unchanged and state unchanged:
- reuse previous layout/draw-list to reduce compute.

### 8.2 Invalidation
- hash of (frame_info, relevant widget state, input digest)
- if hash same -> reuse.

### 8.3 Audit
- record `CacheExecHit` với node range summary vào `perf_cache.jsonl` (không đưa vào signature stream).

---

## 9) Determinism Proof & Verification (USP leverage)

### 9.1 Reproducible caching
Because OCL already has:
- module hashes (deps lock)
- signature/audit
v0.13 requires:
- cache entries include:
  - `cache_proof_hash = sha256-v1(cache_key + entry_content_hash)`
- run validates proof before using cache
- if mismatch => miss, rebuild
- Signature invariance contract:
  - signature input chỉ lấy từ semantic `audit.jsonl`,
  - cache telemetry (`perf_cache.jsonl`) không tham gia signature hash.

### 9.2 Debug support
- trace viewer/debugger shows cache hits/misses từ `perf_cache.jsonl` để dev reproduce perf behavior.
- `--no-cache` is guaranteed semantics-equivalent; compare signatures must match.

### 9.3 Trace schema compatibility for cache telemetry
- `perf_cache.jsonl` dùng schema tương thích `trace_schema_version=2` để tái dùng tooling:
  - `i: u64`
  - `t: String` (`CacheCompileHit`, `CacheExecMiss`, ...)
  - `seed: u64`
  - `tick: u64`
  - `call_id: null` (trừ khi có call identity rõ)
  - `span: null|{...}`
  - `data: Value` canonicalized
- Nếu thiếu field v2 bắt buộc => coi là telemetry invalid, fallback miss + warning.

---

## 10) Error taxonomy (v0.13 additive)
- `X-CACHE-CORRUPT` (cache proof mismatch)
- `X-CACHE-UNSUPPORTED-VERSION`
- `X-CACHE-KEY-INCOMPLETE` (missing required key material => force miss)
- `X-CACHE-CONFIG-DISALLOWED` (forbidden cache env/config in locked lane)
- `X-IR-SERIALIZE-FAIL` (should be internal error)

DX:
- on cache error, auto-fallback to miss and rebuild, but report warning in audit.

---

## 11) Execution gates v0.13

### Gate 13-A — HIR + stable hashing
Scope:
- build HIR from typed AST
- canonicalization + IR hash
Tests:
- `tests/ir_hash.rs`
Exit criteria:
- same module content => same IR hash across runs
- minor whitespace changes do not change hash

### Gate 13-B — Compile cache
Scope:
- compile cache store/load
- invalidation by deps/manifest/compiler version
Tests:
- `tests/compile_cache.rs`
Exit criteria:
- warm build hits cache and outputs identical diagnostics/IR

### Gate 13-C — Per-run exec cache for pure nodes
Scope:
- purity analysis
- per-run cache in executor
Tests:
- `tests/exec_cache_pure.rs`
Exit criteria:
- cache hits do not change signature; reduce node eval count

### Gate 13-D — Capability cacheability framework
Scope:
- registry fields determinism_class + cacheability
- within-run memo for allowed keys
Tests:
- `tests/observe_cache.rs`
Exit criteria:
- only keys marked cacheable are memoized
- all std keys have explicit `determinism_class` + `cacheability` values (no implicit gaps)

### Gate 13-E — Incremental UI/game caching
Scope:
- engine.ui/layout caching by digest
Tests:
- `tests/ui_incremental_cache.rs`
Exit criteria:
- repeated frames with same input produce cache hits; signatures unchanged

### Gate 13-F — CLI cache commands + perf benchmarks
Scope:
- `ocl cache stats/clean`
- benchmark harness to measure KPI
Tests:
- `tests/cli_cache.rs`
Exit criteria:
- KPI-1/2/3/4 validated with machine-checkable protocol:
  - fixed dataset/seed/caps,
  - cold vs warm cache runs,
  - no-cache equivalence run,
  - pass/fail derived from semantic audit counters + `perf_cache.jsonl` counters + signature equality.

---

## 12) Operational commands (v0.13)
- `cargo test`
- `cargo test --test ir_hash`
- `cargo test --test compile_cache`
- `cargo test --test exec_cache_pure`
- `cargo test --test observe_cache`
- `cargo test --test ui_incremental_cache` (optional)
- `cargo test --test cli_cache`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt -- --check`

---

## 13) Execution Log (template)

### YYYY-MM-DD — 13-X planning
- Date:
- Gate/Step:
- Why:
- Scope:
- Expected tests:
- Exit criteria:

### YYYY-MM-DD — 13-X implementation closeout
- Date:
- Gate/Step:
- Implemented:
- Files changed:
- Commands run:
- Test results:
  - Targeted tests (must-pass for gate):
    - `Đạt` / `Chưa đạt`
  - Regression tests (supporting only):
    - `Đạt` / `Chưa đạt`
  - Kết luận gate:
    - Chỉ ghi `DONE` khi nhóm targeted đạt đầy đủ.
- Notes/risks:
- Design alignment:
  - `FULL` hoặc `PARTIAL` (nêu rõ bất khả thi + phương án thay thế).

---

### YYYY-MM-DD — 13-A Planning Freeze
- Date:
- Gate/Step: 13-A
- Why:
- Scope:
- Expected tests:
  - `cargo test --test ir_hash`
- Exit criteria:

### YYYY-MM-DD — 13-A Implementation Closeout
- Date:
- Gate/Step: 13-A
- Implemented:
- Files changed:
- Commands run:
  - `cargo test --test ir_hash`
  - `cargo test`
- Test results:
  - Targeted tests (must-pass for gate):
    - `Đạt` / `Chưa đạt`
  - Regression tests (supporting only):
    - `Đạt` / `Chưa đạt`
  - Kết luận gate:
    - Chỉ ghi `DONE` khi targeted đạt đầy đủ.
- Notes/risks:
- Design alignment:
  - `FULL` hoặc `PARTIAL`.

### YYYY-MM-DD — 13-B Planning Freeze
- Date:
- Gate/Step: 13-B
- Why:
- Scope:
- Expected tests:
  - `cargo test --test compile_cache`
- Exit criteria:

### YYYY-MM-DD — 13-B Implementation Closeout
- Date:
- Gate/Step: 13-B
- Implemented:
- Files changed:
- Commands run:
  - `cargo test --test compile_cache`
  - `cargo test`
- Test results:
  - Targeted tests (must-pass for gate):
    - `Đạt` / `Chưa đạt`
  - Regression tests (supporting only):
    - `Đạt` / `Chưa đạt`
  - Kết luận gate:
    - Chỉ ghi `DONE` khi targeted đạt đầy đủ.
- Notes/risks:
- Design alignment:
  - `FULL` hoặc `PARTIAL`.

### YYYY-MM-DD — 13-C Planning Freeze
- Date:
- Gate/Step: 13-C
- Why:
- Scope:
- Expected tests:
  - `cargo test --test exec_cache_pure`
- Exit criteria:

### YYYY-MM-DD — 13-C Implementation Closeout
- Date:
- Gate/Step: 13-C
- Implemented:
- Files changed:
- Commands run:
  - `cargo test --test exec_cache_pure`
  - `cargo test`
- Test results:
  - Targeted tests (must-pass for gate):
    - `Đạt` / `Chưa đạt`
  - Regression tests (supporting only):
    - `Đạt` / `Chưa đạt`
  - Kết luận gate:
    - Chỉ ghi `DONE` khi targeted đạt đầy đủ.
- Notes/risks:
- Design alignment:
  - `FULL` hoặc `PARTIAL`.

### YYYY-MM-DD — 13-D Planning Freeze
- Date:
- Gate/Step: 13-D
- Why:
- Scope:
- Expected tests:
  - `cargo test --test observe_cache`
- Exit criteria:

### YYYY-MM-DD — 13-D Implementation Closeout
- Date:
- Gate/Step: 13-D
- Implemented:
- Files changed:
- Commands run:
  - `cargo test --test observe_cache`
  - `cargo test`
- Test results:
  - Targeted tests (must-pass for gate):
    - `Đạt` / `Chưa đạt`
  - Regression tests (supporting only):
    - `Đạt` / `Chưa đạt`
  - Kết luận gate:
    - Chỉ ghi `DONE` khi targeted đạt đầy đủ.
- Notes/risks:
- Design alignment:
  - `FULL` hoặc `PARTIAL`.

### YYYY-MM-DD — 13-E Planning Freeze
- Date:
- Gate/Step: 13-E
- Why:
- Scope:
- Expected tests:
  - `cargo test --test ui_incremental_cache`
- Exit criteria:

### YYYY-MM-DD — 13-E Implementation Closeout
- Date:
- Gate/Step: 13-E
- Implemented:
- Files changed:
- Commands run:
  - `cargo test --test ui_incremental_cache`
  - `cargo test`
- Test results:
  - Targeted tests (must-pass for gate):
    - `Đạt` / `Chưa đạt`
  - Regression tests (supporting only):
    - `Đạt` / `Chưa đạt`
  - Kết luận gate:
    - Chỉ ghi `DONE` khi targeted đạt đầy đủ.
- Notes/risks:
- Design alignment:
  - `FULL` hoặc `PARTIAL`.

### YYYY-MM-DD — 13-F Planning Freeze
- Date:
- Gate/Step: 13-F
- Why:
- Scope:
- Expected tests:
  - `cargo test --test cli_cache`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
- Exit criteria:

### YYYY-MM-DD — 13-F Implementation Closeout
- Date:
- Gate/Step: 13-F
- Implemented:
- Files changed:
- Commands run:
  - `cargo test --test cli_cache`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
- Test results:
  - Targeted tests (must-pass for gate):
    - `Đạt` / `Chưa đạt`
  - Regression tests (supporting only):
    - `Đạt` / `Chưa đạt`
  - Kết luận gate:
    - Chỉ ghi `DONE` khi targeted đạt đầy đủ.
- Notes/risks:
- Design alignment:
  - `FULL` hoặc `PARTIAL`.

---

### 2026-03-05 — 13-A Planning Freeze
- Date: 2026-03-05
- Gate/Step: 13-A
- Why:
  - Khóa lớp HIR canonical để làm nền cho compile cache ở các gate sau.
- Scope:
  - Thêm module HIR từ AST đã typecheck.
  - Chuẩn hóa canonical bytes cho HIR (không phụ thuộc whitespace, ổn định thứ tự map/record keys).
  - Thêm hash `sha256-v1` cho HIR.
  - Bổ sung targeted tests `tests/ir_hash.rs`.
- Expected tests:
  - `cargo test --test ir_hash`
- Exit criteria:
  - Cùng nội dung module cho cùng `ir_hash`.
  - Khác biệt whitespace không đổi `ir_hash`.
  - Đổi thứ tự key map/record không đổi `ir_hash`.
  - Không làm vỡ regression toàn repo.

### 2026-03-05 — 13-A Implementation Closeout
- Date: 2026-03-05
- Gate/Step: 13-A
- Implemented:
  - Thêm `src/ocp_ocl/hir.rs`:
    - mô hình HIR node có `id`, `type`, `purity`,
    - lowerer từ AST đã typecheck sang HIR,
    - canonical serializer cho HIR,
    - `ir_hash_sha256_v1`.
  - Mở public API qua `src/ocp_ocl/mod.rs`.
  - Thêm targeted test `tests/ir_hash.rs` cho 3 tiêu chí ổn định hash.
  - Bổ sung dependency `sha2` tại `Cargo.toml`.
- Files changed:
  - `Cargo.lock`
  - `Cargo.toml`
  - `src/ocp_ocl/hir.rs`
  - `src/ocp_ocl/mod.rs`
  - `tests/ir_hash.rs`
- Commands run:
  - `cargo test --test ir_hash`
  - `cargo test`
  - `cargo fmt`
  - `cargo fmt -- --check`
  - `cargo clippy --all-targets -- -D warnings`
- Test results:
  - Targeted tests (must-pass for gate):
    - `Đạt` (`tests/ir_hash.rs`: 3/3).
  - Regression tests (supporting only):
    - `Đạt` (`cargo test` toàn repo).
  - Kết luận gate:
    - `DONE` (targeted + regression đạt, format + clippy đạt).
- Notes/risks:
  - Type inference trong HIR đang theo hướng bảo thủ (`Type::Unknown` cho các nhánh không suy diễn chắc chắn), đủ cho scope `13-A`; các tối ưu sâu hơn để Gate 13-B/13-C.
- Design alignment:
  - `FULL`

### 2026-03-05 — 13-B Planning Freeze
- Date: 2026-03-05
- Gate/Step: 13-B
- Why:
  - Bổ sung compile cache có key đầy đủ để tăng tốc build nhưng vẫn giữ invalidation an toàn theo contract v0.13.
- Scope:
  - Thêm module compile cache cho pipeline parse/typecheck/HIR.
  - Khóa cache key theo `compiler/lane/lock/trust/deps/manifest/schema/entry/source/toggles`.
  - Lưu cache artifact gồm canonical HIR bytes + meta.
  - Bổ sung targeted tests `tests/compile_cache.rs` cho warm hit và invalidation.
- Expected tests:
  - `cargo test --test compile_cache`
- Exit criteria:
  - Warm build hit cache và trả về IR artifact tương đương cold build.
  - Đổi `manifest_hash` gây cache miss.
  - Đổi `compiler_version` hoặc `deps_graph_hash` gây cache miss.
  - Không làm vỡ regression toàn repo.

### 2026-03-05 — 13-B Implementation Closeout
- Date: 2026-03-05
- Gate/Step: 13-B
- Implemented:
  - Thêm `src/ocp_ocl/compile_cache.rs`:
    - `CompileCacheKeyInput`, `CompileCacheArtifact`, `CompileCacheError`,
    - `compile_cache_key_sha256_v1`,
    - `compile_with_cache` (read hit -> fallback miss -> build HIR -> write artifact),
    - canonical key material + `sha256-v1`.
  - Mở public API compile cache qua `src/ocp_ocl/mod.rs`.
  - Thêm targeted test `tests/compile_cache.rs`:
    - warm hit trả về IR hash/bytes giống cold compile,
    - invalidation khi đổi `manifest_hash`,
    - invalidation khi đổi `compiler_version` hoặc `deps_graph_hash`.
- Files changed:
  - `src/ocp_ocl/compile_cache.rs`
  - `src/ocp_ocl/mod.rs`
  - `tests/compile_cache.rs`
- Commands run:
  - `cargo test --test compile_cache`
  - `cargo test`
  - `cargo fmt`
  - `cargo fmt -- --check`
  - `cargo clippy --all-targets -- -D warnings`
- Test results:
  - Targeted tests (must-pass for gate):
    - `Đạt` (`tests/compile_cache.rs`: 3/3).
  - Regression tests (supporting only):
    - `Đạt` (`cargo test` toàn repo).
  - Kết luận gate:
    - `DONE` (targeted + regression đạt, format + clippy đạt).
- Notes/risks:
  - Phiên bản đầu của compile cache mới lưu canonical HIR bytes + metadata; decode ngược HIR object chưa cần cho scope `13-B`.
  - Chiến lược “cache read lỗi => xem như miss và rebuild” đã khóa để tránh dùng nhầm artifact lỗi.
- Design alignment:
  - `FULL`

### 2026-03-05 — 13-C Planning Freeze
- Date: 2026-03-05
- Gate/Step: 13-C
- Why:
  - Thêm exec cache per-run cho pure expression subtree để giảm chi phí thực thi mà vẫn giữ semantics deterministic.
- Scope:
  - Bổ sung cache per-run trong executor cho biểu thức pure (không chứa runtime call).
  - Khóa cache key theo fingerprint biểu thức + bindings của identifiers đang được tham chiếu.
  - Cache hit phải charge số bước tương đương miss path để không lệch semantics/signature.
  - Bổ sung thống kê `exec_cache` trong `ExecOutput`.
  - Thêm targeted test `tests/exec_cache_pure.rs`.
- Expected tests:
  - `cargo test --test exec_cache_pure`
- Exit criteria:
  - Có cache hit thực tế trên pure subtree lặp lại.
  - `cache on` và `cache off` cho cùng chương trình có signature và steps tương đương.
  - Node eval executed giảm khi bật cache.
  - Không phát sinh stale value khi binding thay đổi.

### 2026-03-05 — 13-C Implementation Closeout
- Date: 2026-03-05
- Gate/Step: 13-C
- Implemented:
  - Cập nhật `ExecConfig` với cờ `enable_exec_cache` (default bật).
  - Thêm `ExecCacheStats` vào `ExecOutput`.
  - Bổ sung exec cache per-run trong `Executor`:
    - cache key = fingerprint biểu thức + identifier bindings hiện thời,
    - cache chỉ áp dụng cho biểu thức pure (không chứa `Expr::Call`),
    - cache hit trả value từ cache nhưng vẫn charge bước tương đương entry miss trước đó,
    - theo dõi counters `hits/misses/node_evals_charged/node_evals_executed`.
  - Bổ sung helper canonical fingerprint cho expression cache key.
  - Thêm targeted tests trong `tests/exec_cache_pure.rs`:
    - chứng minh hit + giảm executed node eval,
    - chứng minh signature/steps không đổi giữa bật/tắt cache,
    - chứng minh key có theo binding để tránh stale value.
  - Cập nhật initializer `ExecConfig` trong `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs` để tương thích field mới.
- Files changed:
  - `src/ocp_ocl/budget.rs`
  - `src/ocp_ocl/exec.rs`
  - `src/ocp_ocl/mod.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
  - `tests/exec_cache_pure.rs`
- Commands run:
  - `cargo test --test exec_cache_pure`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt`
  - `cargo fmt -- --check`
- Test results:
  - Targeted tests (must-pass for gate):
    - `Đạt` (`tests/exec_cache_pure.rs`: 2/2).
  - Regression tests (supporting only):
    - `Đạt` (`cargo test` toàn repo).
    - `Đạt` (`cargo clippy --all-targets -- -D warnings`).
    - `Đạt` (`cargo fmt -- --check`).
  - Kết luận gate:
    - `DONE` (targeted + regression đạt, không lệch design scope).
- Notes/risks:
  - Cache key hiện dựa trên identifier bindings được tham chiếu trong expression; đủ an toàn cho scope gate C.
  - Chiến lược cache entry chỉ lưu result thành công; nhánh trả lỗi vẫn đi miss path để tránh đổi hành vi runtime.
- Design alignment:
  - `FULL`

### 2026-03-05 — 13-D Planning Freeze
- Date: 2026-03-05
- Gate/Step: 13-D
- Why:
  - Khóa framework `cacheability` theo capability để runtime chỉ memoize đúng các key đủ điều kiện deterministic và tương thích invalidation.
- Scope:
  - Mở rộng registry với metadata `cacheability` + `determinism_class` rõ cho std keys.
  - Thêm API tra cứu `cacheability_for_key` và kiểm tra thiếu metadata explicit cho std keys.
  - Tích hợp observe memo trong executor với policy:
    - chỉ memoize khi `determinism_class=Deterministic` và `cacheability=WithinRun`,
    - memo key có `fs_generation` cho `std.fs.*` và `kv_generation` cho `std.kv.*`.
  - Tăng generation counters khi commit FS/KV thành công để tránh stale read.
  - Thêm targeted tests `tests/observe_cache.rs` và cập nhật `tests/ocl_registry.rs`.
- Expected tests:
  - `cargo test --test observe_cache --test ocl_registry`
- Exit criteria:
  - Registry expose đầy đủ metadata deterministic + cacheability cho std keys.
  - Observe memo hit/miss đúng policy cacheability.
  - Đọc FS sau commit không bị stale do cache.
  - Không làm vỡ regression toàn repo.

### 2026-03-05 — 13-D Implementation Closeout
- Date: 2026-03-05
- Gate/Step: 13-D
- Implemented:
  - Mở rộng `CapabilityRegistry`:
    - thêm enum `Cacheability` (`NoCache`, `WithinRun`, `AcrossRuns`),
    - thêm `DeterminismClass::CassetteBased`,
    - thêm bảng `cacheabilities` và API tra cứu/cập nhật metadata.
  - Trong `v1_baseline()`:
    - gán metadata explicit cho toàn bộ std keys,
    - default std keys: `Deterministic + NoCache`,
    - override cho nhóm cassette (`std.net.http.request`, `std.proc.exec`, `std.time.wallclock.now`) và nhóm within-run (`std.fs.read_text`, `std.fs.list_dir`, `std.fs.stat`, `std.kv.get`, `std.kv.keys`).
  - Thêm kiểm tra `std_keys_missing_cache_metadata()` để gate có thể fail-safe nếu thiếu metadata explicit.
  - Tích hợp observe memoization trong executor:
    - observe path chuyển qua `observe_with_memo(...)`,
    - memo key gồm `key/tier/ctx/budget/lane` + generation phù hợp (`fs_generation`, `kv_generation`),
    - theo dõi `ObserveCacheStats` (`entries`, `hits`, `misses`) trong `ExecOutput`.
  - Bổ sung invalidation:
    - tăng `fs_generation` sau commit FS thành công,
    - tăng `kv_generation` sau commit KV thành công.
  - Thêm targeted tests:
    - `tests/observe_cache.rs`: kiểm tra memoize theo cacheability và invalidation qua `fs_generation`,
    - `tests/ocl_registry.rs`: kiểm tra std keys có metadata explicit + kỳ vọng determinism/cacheability mẫu.
- Files changed:
  - `src/ocp_ocl/registry.rs`
  - `src/ocp_ocl/exec.rs`
  - `src/ocp_ocl/mod.rs`
  - `tests/observe_cache.rs`
  - `tests/ocl_registry.rs`
- Commands run:
  - `cargo test --test observe_cache --test ocl_registry`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt`
  - `cargo fmt -- --check`
- Test results:
  - Targeted tests (must-pass for gate):
    - `Đạt` (`tests/observe_cache.rs`, `tests/ocl_registry.rs`).
  - Regression tests (supporting only):
    - `Đạt` (`cargo test` toàn repo).
    - `Đạt` (`cargo clippy --all-targets -- -D warnings`).
    - `Đạt` (`cargo fmt -- --check`).
  - Kết luận gate:
    - `DONE` (targeted + regression đạt, không lệch scope thiết kế gate 13-D).
- Notes/risks:
  - `AcrossRuns` đã có trong taxonomy để khóa contract v0.13 nhưng chưa mở triển khai dùng thật; scope đó thuộc gate sau.
  - Observe memo hiện chỉ áp dụng cho lớp key deterministic + within-run theo registry policy; key cassette/non-deterministic không đi memo path.
- Design alignment:
  - `FULL`

### 2026-03-05 — 13-E Planning Freeze
- Date: 2026-03-05
- Gate/Step: 13-E
- Why:
  - Bám scope incremental UI/game caching để tận dụng frame repeat trong cùng run, giảm compute nhưng không đổi semantics/signature.
- Scope:
  - Bổ sung metadata explicit cho `engine.ui.run` và `engine.game.run`:
    - `determinism_class=Deterministic`,
    - `cacheability=WithinRun`.
  - Khóa memo key path cho engine incremental:
    - từ `ctx_lit` sinh canonical digest ổn định,
    - memo key include `engine_incremental_digest` cho `engine.ui.run`/`engine.game.run`.
  - Thêm targeted tests `tests/ui_incremental_cache.rs`:
    - frame lặp cùng input phải có cache hit,
    - cache on/off phải cho signature như nhau.
- Expected tests:
  - `cargo test --test ui_incremental_cache`
- Exit criteria:
  - Có cache hit khi quan sát lặp frame/game run cùng input trong cùng run.
  - Kết quả runtime và signature không đổi giữa cache on/off.
  - Không làm vỡ regression toàn repo.

### 2026-03-05 — 13-E Implementation Closeout
- Date: 2026-03-05
- Gate/Step: 13-E
- Implemented:
  - Bổ sung metadata deterministic + cacheability cho engine keys:
    - `engine.ui.run` => `Deterministic + WithinRun`,
    - `engine.game.run` => `Deterministic + WithinRun`.
  - Mở rộng memo key path cho incremental engine caching:
    - thêm `canonical_ctx_digest(...)`,
    - thêm `engine_incremental_digest(...)`,
    - `observe_memo_key(...)` include digest cho `engine.ui.run`/`engine.game.run`.
  - Thêm targeted test suite `tests/ui_incremental_cache.rs`:
    - `ui_incremental_cache_hits_for_repeated_engine_ui_frame`,
    - `game_incremental_cache_keeps_signature_equal_when_cache_disabled`.
- Files changed:
  - `src/ocp_ocl/registry.rs`
  - `src/ocp_ocl/exec.rs`
  - `tests/ui_incremental_cache.rs`
- Commands run:
  - `cargo test --test ui_incremental_cache`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt`
  - `cargo fmt -- --check`
  - `cargo test --test ui_incremental_cache`
- Test results:
  - Targeted tests (must-pass for gate):
    - `Đạt` (`tests/ui_incremental_cache.rs`: 2/2).
  - Regression tests (supporting only):
    - `Đạt` (`cargo test` toàn repo).
    - `Đạt` (`cargo clippy --all-targets -- -D warnings`).
    - `Đạt` (`cargo fmt -- --check`).
  - Kết luận gate:
    - `DONE` (targeted + regression đạt, code delta đúng scope gate 13-E).
- Notes/risks:
  - Incremental digest hiện canonical theo `ctx_lit` key/value, phù hợp scope gate E (within-run reuse).
  - Scope này chưa mở persistent cache cho engine paths; phần đó thuộc gate sau.
- Design alignment:
  - `FULL`

### 2026-03-05 — 13-F Planning Freeze
- Date: 2026-03-05
- Gate/Step: 13-F
- Why:
  - Hoàn thiện CLI cache namespace và benchmark harness machine-checkable để chốt KPI gate cuối của v0.13.
- Scope:
  - Thêm CLI `ocl cache stats`, `ocl cache clean --yes`, `ocl cache bench`.
  - Emit/parse `perf_cache.jsonl` tách khỏi signature stream semantic.
  - Bổ sung report benchmark `target/ocl/v13/cache_benchmark.json`.
  - Bổ sung targeted test `tests/cli_cache.rs`.
- Expected tests:
  - `cargo test --test cli_cache`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
- Exit criteria:
  - `ocl cache stats` trả counters machine-checkable (`--json`).
  - `ocl cache clean` bắt buộc `--yes`, không cho xóa nhầm.
  - `ocl cache bench` sinh report benchmark và xác nhận signature equivalence cache on/off.
  - Targeted + regression pass.

### 2026-03-05 — 13-F Implementation Closeout
- Date: 2026-03-05
- Gate/Step: 13-F
- Implemented:
  - Bổ sung CLI namespace `ocl cache`:
    - `stats`: tổng hợp compile/exec/observe counters từ `.ocl_artifacts/*/perf_cache.jsonl` và compile cache stores.
    - `clean`: yêu cầu `--yes`, dọn `.ocl_artifacts`, `.ocl_cache`, `target/ocl/cache`.
    - `bench`: ép cold-cache trước benchmark, chạy `cold` + `warm` (cache enabled), rồi chạy `no_cache` equivalence để so signature.
  - Mở rộng SDK/runtime summary để mang `exec_cache` và `observe_cache` counters phục vụ benchmark.
  - Tách cache telemetry ra `perf_cache.jsonl` trong artifact emission path.
  - Mở rộng env `OCL_CACHE_DISABLE` để vô hiệu hóa cả observe cache và exec cache trong benchmark no-cache mode.
  - Cập nhật benchmark metric:
    - dùng `exec_node_evals_executed` cho `executed_reduction_bps`,
    - report gồm đủ `cold` / `warm` / `no_cache`.
  - Thêm targeted tests `tests/cli_cache.rs` cho stats/clean/bench.
  - Bổ sung `serde_json` ở `dev-dependencies` để parse JSON report trong test CLI.
- Files changed:
  - `Cargo.toml`
  - `Cargo.lock`
  - `projects/ocp-ocl/crates/ocl-runtime-core/src/lib.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
  - `projects/ocp-ocl/crates/ocl-cli/src/main.rs`
  - `src/ocp_ocl/exec.rs`
  - `tests/cli_cache.rs`
- Commands run:
  - `cargo test --test cli_cache`
  - `cargo fmt`
  - `cargo fmt -- --check`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo test`
- Test results:
  - Targeted tests (must-pass for gate):
    - `Đạt` (`tests/cli_cache.rs`: 4/4).
  - Regression tests (supporting only):
    - `Đạt` (`cargo test` toàn repo).
    - `Đạt` (`cargo clippy --all-targets -- -D warnings`).
    - `Đạt` (`cargo fmt -- --check`).
  - Kết luận gate:
    - `DONE` (code delta đúng scope 13-F, targeted + regression đạt).
- Notes/risks:
  - Correction Note:
    - Trước đó `13-F` từng bị chốt sớm khi benchmark chưa phản ánh đủ protocol no-cache/cold-warm.
    - Đã hạ về `IN_PROGRESS`, vá code theo contract, rerun full suite và mới nâng lại `DONE`.
  - Bộ đo benchmark dùng deterministic counters + signature equivalence; wallclock chỉ phục vụ tham khảo.
- Design alignment:
  - `FULL`

## 14) Checklist khóa trước khi đóng gate
- [x] Gate status đã cập nhật đúng (`TODO/IN_PROGRESS/DONE`).
- [x] Có đủ Planning Freeze + Implementation Closeout.
- [x] `Files changed` khớp code delta thực tế.
- [x] `Commands run` là lệnh đã chạy thật.
- [x] `Test results` có trạng thái rõ và đúng phạm vi.
- [x] Nếu gate `DONE`: không còn marker mâu thuẫn trong closeout.
- [x] `Design alignment` đã ghi `FULL` hoặc `PARTIAL` đúng thực tế.
- [x] Không còn lỗi tiếng Việt/mã hóa trong nội dung file.

---

## 15) Handoff v0.13 -> v0.14 (pre-draft)
- Chỉ mở v0.14 khi 13-A..13-F đều `DONE` và KPI v0.13 pass đầy đủ.
- V0.14 kế thừa nguyên xi contracts đã khóa ở v0.13:
  - cache key completeness (`lane/lock/trust/toggle inputs`)
  - invalidation correctness (`fs_generation`, `kv_generation`)
  - signature invariance (semantic audit vs perf telemetry tách riêng)
  - hasher `sha256-v1` + canonical encoding rules.
- Nếu v0.13 còn gate `IN_PROGRESS/PARTIAL`, không mở scope v0.14 ngoài xử lý blocker tồn đọng.

---

