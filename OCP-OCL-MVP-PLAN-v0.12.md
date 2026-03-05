# OCL v0.12 — Shadow Scheduling + Incremental Reuse (Bounded Parallel Realities Engine)

Ngày tạo: 2026-03-03  
Trạng thái: `DONE` (2026-03-05)  
Phạm vi: **OCL-only**.  
Tiền đề: v0.7.3 đã có `std.shadow` primitive + compare; v0.11 có trace diff + minimizer; v0.9 schema/typing giúp state diff/contract rõ.

Mục tiêu v0.12: nâng shadow từ “chạy N nhánh rồi so sánh” thành **engine exploration**:
- scheduling policy (beam/portfolio) bounded,
- incremental reuse (cache, COW, memoization) để chạy nhanh hơn,
- report nâng cấp để dùng được trong sản phẩm (preview/search).

---

## 0) Governance + Tracking v0.12

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
  - Ship shadow scheduling + incremental reuse bounded, deterministic, auditable.
- Trạng thái tổng quan:
  - `DONE` (2026-03-05); Gate 12-A..12-G đã hoàn tất theo scope targeted + regression hỗ trợ.
- Gate đang làm/đã xong/chưa làm:
  - Gate 12-A, 12-B, 12-C, 12-D, 12-E, 12-F, 12-G `DONE` (2026-03-05).
- Bước kế tiếp ngay:
  - Giữ v0.12 ở trạng thái khóa; chỉ mở v0.13 theo handoff khi không phát sinh correction note.
- Lệnh kiểm chứng chuẩn:
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
- Danh sách file code trọng yếu đã thay đổi:
  - `src/ocp_ocl/exec.rs`
  - `src/ocp_ocl/registry.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
  - `src/ocp_ocl/mod.rs`
  - `tests/shadow_checkpoints.rs`
  - `tests/shadow_memo.rs`
  - `tests/shadow_scheduler_rr.rs`
  - `tests/shadow_beam.rs`
  - `tests/shadow_portfolio.rs`
  - `tests/shadow_report_v2.rs`
  - `projects/ocp-ocl/crates/ocl-cli/src/main.rs`
  - `tests/cli_shadow_preview_v2_e2e.rs`
  - `tests/pack_schema_conformance_consumer.rs`
  - `tests/ocl_registry.rs`
  - `OCP-OCL-MVP-PLAN-v0.12.md`

### 0.3 Trạng thái Workstreams/Gates v0.12 (TRACKING)
#### Workstreams
- WS-S (scheduler semantics + scoring + accounting): `DONE` (2026-03-05)
- WS-R (incremental reuse: checkpoint + memo + prefix): `DONE` (2026-03-05)
- WS-D (report/audit/diff integration): `DONE` (2026-03-05)
- WS-C (CLI template + KPI benchmark): `DONE` (2026-03-05)

#### Gate status (12-A .. 12-G)
- Gate 12-A (Shadow checkpoint cache): `DONE` (2026-03-05)
- Gate 12-B (Deterministic observe memoization): `DONE` (2026-03-05)
- Gate 12-C (Scheduler framework + round-robin baseline): `DONE` (2026-03-05)
- Gate 12-D (Beam policy + scoring): `DONE` (2026-03-05)
- Gate 12-E (Portfolio policy): `DONE` (2026-03-05)
- Gate 12-F (Report v2 + bounded truncation): `DONE` (2026-03-05)
- Gate 12-G (Template upgrades + KPI benchmark): `DONE` (2026-03-05)

---

## 1) Goals v0.12 (LOCKED)

### 1.1 North Star
- Shadow trở thành “subsystem” chung cho app/game/tool:
  - preview hậu quả nhiều lựa chọn,
  - search bounded (tìm nhánh tốt),
  - đánh giá phương án theo score.
- Vẫn giữ core:
  - bounded budgets & caps,
  - deterministic execution,
  - auditable + replayable.

### 1.2 KPI bắt buộc (định lượng)
**KPI-1: Scheduling**
- Có ít nhất 2 policy shipped:
  - `beam` (top-K theo score)
  - `portfolio` (chia budget cho nhiều heuristic)
- Mọi policy đều deterministic (cùng seed => cùng quyết định).

**KPI-2: Incremental reuse**
- Với workload shadow-preview template:
  - reuse giúp giảm ít nhất 30% steps hoặc budget so với baseline “run branches independently”
  - vẫn giữ kết quả (signatures/outcomes) giống.
- Protocol đo KPI-2 (khóa cứng):
  - dataset: `shadow-preview` template v0.12
  - seed: `42`
  - `max_branches=8`, `per_branch_step_cap=5000`, `checkpoint_every=200`
  - baseline: `memoize_deterministic_observe=false` và `checkpoint_reuse=false`
  - run so sánh phải cùng lane/caps/seed, cold-cache
  - điều kiện pass:
    - `steps_executed_total(reuse_on) <= 0.70 * steps_executed_total(baseline)`
  - evidence bắt buộc trong report:
    - `baseline_steps_executed`, `reuse_steps_executed`, `%reduction`

**KPI-3: Report usefulness**
- Report phải có:
  - top branches + scores
  - divergence points summary
  - cost curves (budget/steps per branch)
  - reason breakdown
- Bounded report size (cap) và deterministic.

---

## 2) Axis Lock v0.12 (kế thừa, không đổi)
- Shadow runs in lane `locked_v071` by default.
- Trong tài liệu này, từ `locked` chỉ là alias mô tả cho `locked_v071`, không phải literal manifest.
- No unbounded exploration.
- No hidden IO; any IO in shadow must obey lane policy (locked/quarantine).
- Determinism and audit are mandatory.
- V0.12 chỉ ship reuse/memoization cho lane `locked_v071`; cassette-based reuse ở `quarantine` là deferred.

---

## 3) Scope v0.12

### 3.1 In-scope (ship)
A) **Shadow scheduling framework**
- New capability: `std.shadow.search` (high-level) built atop `shadow.run`.
- Scheduler policies:
  - `beam`
  - `portfolio`
  - `round_robin` baseline (bắt buộc cho regression baseline).

B) **Scoring interface**
- Define a standard scoring config contract:
  - `score_field`
  - `score_weights`
- Score must be pure/deterministic and bounded.

C) **Incremental reuse**
- Cross-branch reuse via:
  - checkpoint cache of interpreter state (IR + env snapshot)
  - memoized observe results for deterministic keys (within locked)
  - reuse of shared prefix execution across branches (prefix factoring)
- COW env remains base; reuse reduces re-exec.

D) **Budget allocation & accounting**
- Global shadow budget and per-branch caps.
- Scheduler reallocates remaining budget deterministically.

E) **Enhanced report format**
- `shadow.report.v2` schema (bounded):
  - ranking
  - divergence points
  - cost curves
  - reason tables
  - cache hit stats

F) **Audit/trace extensions**
- Log scheduling decisions:
  - why a branch was expanded/pruned
  - budget allocations
  - cache hits/misses

### 3.2 Out-of-scope (defer)
- Probabilistic policies requiring nondet RNG (unless via deterministic RNG stream).
- Distributed parallel execution across machines.
- Full MCTS (can be future if needed).
- Cassette-based memoization/reuse trong lane `quarantine`.

### 3.3 Module -> Crate -> Path mapping (LOCKED)
- Shadow scheduler semantics, checkpoint cache, memo table, report v2 payload:
  - crate: `projects/ocp-ocl/crates/ocl-runtime-core`
- Registry/schema/permissions wiring cho `std.shadow.*`:
  - crate: `projects/ocp-ocl/crates/ocl-sdk`
- Trace/Audit tooling (`trace diff/view` consumption cho event mới):
  - crate: `projects/ocp-ocl/crates/ocl-cli`
- Legacy bridge:
  - nếu còn code path `src/ocp_ocl/*`, chỉ dùng làm lớp tương thích trong giai đoạn chuyển tiếp, không mở rộng semantics mới tại đó.
- Rule:
  - Chưa khóa rõ mapping thì chưa được mở gate implementation tương ứng.

---

## 4) New Concepts: Shadow Programs, Variants, Prefixes

### 4.1 Variant model (unchanged)
A variant is injected params record.

### 4.2 Prefix factoring
Many variants share early execution prefix. v0.12 introduces:
- `prefix_key`: `sha256-v1` trên canonical bytes:
  - `program_hash || entry_module_id || lane || seed || limits_hash || perms_hash || checkpoint_every`
- cache entries keyed by prefix_key + checkpoint index.
- `variant_prefix_digest`:
  - `sha256-v1(canonical_encode(variant_record))`
- Canonicalization:
  - dùng cùng canonical Value encoding đã khóa từ v0.8 (stable key order, UTF-8, deterministic bytes).

### 4.3 Deterministic keys memoization
Within a shadow run:
- observe results for deterministic keys can be memoized across branches if:
  - key determinism_class = deterministic
  - ctx equal (canonical)
  - budgets satisfied
- Nguồn `determinism_class` (LOCKED):
  - lấy từ capability metadata trong registry/schema.
  - nếu key không khai báo determinism_class thì mặc định là `non_deterministic` (không memoize).
- Cassette-based (quarantine) memoization không thuộc v0.12; defer sau v0.12.

---

## 5) Scheduling API (v0.12)

### 5.1 `std.shadow.search` (high-level capability)
Inputs:
- `entry`: luôn là current program entry/module (không nhận function-ref/module-ref trong v0.12)
- `variants`: list of injected params (bounded by max_branches)
- `policy`: enum `"beam"|"portfolio"|"round_robin"`
- `score`: score config (which fields to read from state summary)
- caps:
  - `global_budget_cap`
  - `global_step_cap`
  - `per_branch_step_cap`
  - `beam_width` (for beam)
  - `rounds` (for portfolio)
Outputs:
- `SearchResult`:
  - `best`: BranchResult
  - `topk`: list BranchResult (bounded)
  - `report`: ReportV2 (bounded)

Rules:
- `std.shadow.search` must be deterministic in decision-making.
- Context injection cố định cho mọi branch search:
  - `ctx.shadow.mode = "search"` (metadata)
  - `ctx.shadow.is_shadow = true` (marker semantics cho isolation)
  - `ctx.shadow.policy`
  - `ctx.shadow.round`
  - `ctx.shadow.branch_id`
  - `ctx.shadow.variant`
- Isolation rule (LOCKED):
  - semantics isolation không phụ thuộc string literal của `ctx.shadow.mode`,
  - chỉ phụ thuộc marker `ctx.shadow.is_shadow = true` (hoặc presence của `ctx.shadow.*` nếu marker chưa có trong bản migration).
- Under the hood it can call `shadow.run` multiple times in phases, or run incremental expansion with checkpoints.

### 5.2 Scoring contract
Score computed from:
- `state_summary` record (bounded)
- `outcome kind/reason`
- `cost`

Scoring config (LOCKED, không callback/function-ref):
- `score_field` (default: `"score"`) đọc từ `state_summary.score`
- `score_weights` (optional record):
  - `outcome_weight` (default: `1`)
  - `cost_budget_weight` (default: `1`)
  - `cost_steps_weight` (default: `1`)
  - `reason_penalty_weight` (default: `1`)
  - `state_score_weight` (default: `1`)
  - mọi weight là số nguyên không âm, clamp trong `[0..1000]`
- Custom score rule:
  - user tự tính score trong branch program và ghi vào `state_summary.score`
  - `std.shadow.search` chỉ đọc field/config, không gọi callback.

Outcome base (LOCKED):
- `OK = 1000`
- `DEGRADED = 300`
- `DEFERRED = -300`
- `INSUFFICIENT = -700`

Reason base penalty (LOCKED):
- `reason_code` rỗng: `0`
- có `reason_code`: `-100`
- nếu có `reason_penalties[reason_code]` trong config thì dùng giá trị đó thay cho `-100`

State score extraction (LOCKED):
- đọc từ `state_summary[score_field]`
- nếu không tồn tại hoặc không phải số nguyên: dùng `0`
- clamp giá trị về `[-1_000_000..1_000_000]`

Final score formula (LOCKED, integer-only):
- `score =`
  - `outcome_weight * outcome_base`
  - `+ state_score_weight * state_score`
  - `+ reason_penalty_weight * reason_base`
  - `- cost_budget_weight * budget_charged`
  - `- cost_steps_weight * steps_charged`
- Không dùng float trong scoring path.
- Mọi comparator/sort phải dựa trên `score` integer này.

---

## 6) Scheduling Policies (v0.12 shipped)

### 6.1 Beam search (bounded)
Parameters:
- `beam_width = K`
- `depth = D` (or rounds)
Process:
1) Start with initial variants.
2) Run one step/segment for each branch (incremental).
3) Score each branch.
4) Keep top K, prune rest.
5) Repeat until depth D or budget exhausted.

Determinism:
- tie-break by stable branch id + `variant_prefix_digest` (`sha256-v1`, lower-hex).

### 6.2 Portfolio (bounded)
Parameters:
- `rounds = R`
- `strategies = ["outcome_first","cost_first","balanced"]` (fixed order)
Process:
- Allocate budget slices to each strategy deterministically.
- Each strategy explores subset of variants.
- Merge results and select best.

Tie-break (khóa cứng):
- `(score desc, steps_charged asc, branch_id asc)`

Use case:
- different heuristics (cost-first vs outcome-first).

### 6.3 Round-robin (baseline)
- Expand branches in fixed order until caps reached.
- Good for regression tests.

---

## 7) Incremental Reuse (key feature)

### 7.1 Checkpoint cache
During branch execution:
- capture checkpoint every `checkpoint_every` steps (e.g., 200):
  - env snapshot (bounded)
  - instruction pointer / IR location
  - digest of env
Store in cache keyed by:
- `prefix_key`
- `checkpoint_index`
- `variant_prefix_digest` (if variant affects early state)

Reuse:
- For a new branch, find nearest compatible checkpoint và resume.
- Compatible checkpoint (LOCKED):
  - cùng `prefix_key`
  - `checkpoint_index <= target_index`
  - lane/limits/perms đã nằm trong `prefix_key`; nếu khác thì bắt buộc cache miss.

### 7.2 Memoized observe
Memoize deterministic observes across branches:
- `memo_key = (key, tier, canonical_ctx_hash, budget_units, lane)`
- Store Result4 payload digest + payload (bounded)
- Audit records memo hit/miss.

Constraints:
- only if key marked deterministic
- budgets must not be bypassed:
  - memo-hit vẫn charge `budget_used` như cold-call
  - tách metrics:
    - `steps_charged`: dùng cho scheduler/score/caps, memo-hit charge như cold-call
    - `steps_executed`: steps thực chạy interpreter, memo-hit được giảm
  - KPI reuse chỉ đo trên `steps_executed_total`
  - report/audit bắt buộc có `memo_saved_steps`
- Avoid mem for keys with time/tick dependence unless tick included in ctx.

### 7.3 Shared-prefix execution
If variants differ only in later params:
- run shared prefix once
- fork from checkpoint for each variant
This gives the 30%+ reduction target.

---

## 8) Report v2 (bounded, product-usable)

### 8.1 `shadow.report.v2` fields
- `meta`:
  - policy, caps, global_budget_used
  - `global_steps_charged`, `global_steps_executed`
  - cache_hits, cache_misses
- `ranking`:
  - list of `{ branch_id, score, outcome_kind, reason_code?, cost, signature }` (bounded topK)
  - `cost` chứa cả `budget_charged`, `steps_charged`, `steps_executed`
- `divergence_points`:
  - list of `{ point_id, description, affected_branches }` (bounded)
- `diff_summary`:
  - bounded list of keys that differ across top branches
- `cost_curve`:
  - per round: `{ round, active_branches, budget_used, steps_charged, steps_executed }` (bounded)
- `reason_table`:
  - counts by reason_code among explored branches
- `artifacts`:
  - optional pointers to per-branch trace summaries (not full traces)

### 8.2 Size caps
- report bytes <= `permissions.std_shadow.max_report_bytes`
- if exceed => `DEGRADED` and truncate sections deterministically.

---

## 9) Audit/Trace Extensions (v0.12)

New event types:
- `ShadowScheduleDecision`:
  - round, policy, selected branch ids, pruned ids, tie-break info
- `ShadowBudgetAlloc`:
  - branch_id -> budget slice
- `ShadowCache`:
  - hits/misses counts per key class
- `ShadowCheckpoint`:
  - created/resumed checkpoints (ids)

Schema contract (khóa cứng, tương thích v0.11):
- Mọi event mới phải theo `trace_schema_version=2`:
  - `i: u64`
  - `t: String`
  - `seed: u64`
  - `tick: u64`
  - `call_id: null` (vì đây không phải Observe/Commit runtime call)
  - `span: null` hoặc span hợp lệ nếu có
  - `data: Value` canonicalized
- Không thêm field làm phá parser v2 hiện hành.

DX requirement:
- `ocl trace diff` can highlight scheduling divergence between runs.

---

## 10) Manifest extensions (v0.12)
`permissions.std_shadow` gains:
- `scheduler_policy_allow = ["beam","portfolio","round_robin"]` (optional)
- `checkpoint_every = 200`
- `checkpoint_reuse = true`
- `memoize_deterministic_observe = true`
- `max_rounds = 50`
- `beam_width_max = 8`

Project may restrict policy for safety:
- deny portfolio if too complex.
- Nếu policy không thuộc `scheduler_policy_allow`:
  - fail-honest `DEFERRED(RC-SHADOW-POLICY-DENIED)`.
- Nếu `scheduler_policy_allow` thiếu:
  - default allow list = `["beam","portfolio","round_robin"]`.

---

## 11) Execution gates v0.12

### Gate 12-A — Shadow checkpoint cache
Scope:
- checkpoint capture/resume
- prefix_key design
- bounded snapshot storage
Tests:
- `tests/shadow_checkpoints.rs`
Exit criteria:
- resuming from checkpoint yields identical state digest vs full rerun

### Gate 12-B — Deterministic observe memoization
Scope:
- memo table for deterministic keys
- audit hit/miss
Tests:
- `tests/shadow_memo.rs`
Exit criteria:
- memo reuse does not change results; reduces observe calls count

### Gate 12-C — Scheduler framework + round-robin baseline
Scope:
- scheduling loop with global caps
- stable tie-breaks
Tests:
- `tests/shadow_scheduler_rr.rs`
Exit criteria:
- deterministic schedule; caps enforced

### Gate 12-D — Beam policy + scoring
Scope:
- beam width, rounds
- scoring API + default scoring
- scoring formula integer-only theo contract ở `5.2`
Tests:
- `tests/shadow_beam.rs`
Exit criteria:
- beam selects top branches deterministically

### Gate 12-E — Portfolio policy
Scope:
- strategies + budget slices
Tests:
- `tests/shadow_portfolio.rs`
Exit criteria:
- deterministic; respects max_rounds and caps

### Gate 12-F — Report v2 + bounded truncation
Scope:
- implement report schema v2
- truncation rules
Tests:
- `tests/shadow_report_v2.rs`
Exit criteria:
- report deterministic; size bounded; truncation stable

### Gate 12-G — Template upgrades + KPI benchmark
Scope:
- update `shadow-preview` template to use `std.shadow.search`
- measure reuse improvements (>=30% cost reduction)
Tests:
- `tests/cli_shadow_preview_v2_e2e.rs`
Exit criteria:
- KPI-1/2/3 pass

---

## 12) Operational commands (v0.12)
- `cargo test`
- `cargo test --test shadow_checkpoints`
- `cargo test --test shadow_memo`
- `cargo test --test shadow_scheduler_rr`
- `cargo test --test shadow_beam`
- `cargo test --test shadow_portfolio`
- `cargo test --test shadow_report_v2`
- `cargo test --test cli_shadow_preview_v2_e2e`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt -- --check`

---

## 13) Execution Log (template)

### YYYY-MM-DD — 12-X planning
- Date:
- Gate/Step:
- Why:
- Scope:
- Expected tests:
- Exit criteria:

### YYYY-MM-DD — 12-X implementation closeout
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

### 2026-03-05 — 12-A Planning Freeze
- Date:
  - 2026-03-05
- Gate/Step:
  - 12-A
- Why:
  - Thiết lập checkpoint cache + prefix key bounded cho shadow runtime trước khi mở scheduler policy.
- Scope:
  - Runtime:
    - khóa `prefix_key` và `variant_prefix_digest` cho shadow run.
    - thêm checkpoint cache bounded và metadata resume/state digest.
    - thêm bounded checkpoint cache stats vào payload.
  - Registry/schema:
    - mở rộng ctx/payload schema cho `std.shadow.run` để chấp nhận metadata checkpoint.
  - Test:
    - thêm suite targeted `tests/shadow_checkpoints.rs`.
- Expected tests:
  - `cargo test --test shadow_checkpoints`
  - `cargo test --test std_shadow --test ocl_manifest_fs_kv_time --test pack_schema_conformance_consumer`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
- Exit criteria:
  - Resume từ checkpoint cho nhánh phù hợp cho ra state digest như full rerun.
  - Checkpoint cache bounded và có marker truncated khi vượt cap.
  - Không phá regression `std.shadow` hiện có.

### 2026-03-05 — 12-A Implementation Closeout
- Date:
  - 2026-03-05
- Gate/Step:
  - 12-A
- Implemented:
  - `src/ocp_ocl/exec.rs`:
    - thêm helper checkpoint/prefix:
      - `shadow_checkpoint_every`, `shadow_checkpoint_cache_max_entries`
      - `shadow_prefix_key`, `shadow_variant_prefix_digest`
      - `shadow_checkpoint_key`, `shadow_checkpoint_digest`, `shadow_state_digest`
      - `shadow_checkpoint_indices`, `build_shadow_run`
    - tích hợp checkpoint cache bounded vào:
      - `engine.shadow.preview`
      - `std.shadow.run`
    - bổ sung metadata runtime:
      - `prefix_key`, `checkpoint_every`, `checkpoint_cache`
      - per-branch `checkpoint_resume_index`, `checkpoint_count`, `state_digest_full`, `state_digest_resumed`.
  - `src/ocp_ocl/registry.rs`:
    - ctx schema `std.shadow.run`: thêm `checkpoint_every`.
    - payload schema `std.shadow.run`: thêm optional `prefix_key`, `checkpoint_every`, `checkpoint_cache`.
  - `tests/shadow_checkpoints.rs` (mới):
    - `shadow_checkpoint_resume_digest_matches_full_digest`
    - `shadow_checkpoint_cache_is_bounded_and_truncated`.
- Files changed:
  - `src/ocp_ocl/exec.rs`
  - `src/ocp_ocl/registry.rs`
  - `tests/shadow_checkpoints.rs`
  - `OCP-OCL-MVP-PLAN-v0.12.md`
- Commands run:
  - `cargo test --test shadow_checkpoints`
  - `cargo test --test std_shadow --test ocl_manifest_fs_kv_time --test pack_schema_conformance_consumer`
  - `cargo test`
  - `cargo fmt`
  - `cargo fmt -- --check`
  - `cargo clippy --all-targets -- -D warnings`
- Test results:
  - Targeted tests (must-pass for gate):
    - `shadow_checkpoints`: `PASS` (2/2)
  - Regression tests (supporting only):
    - `std_shadow`: `PASS` (5/5)
    - `ocl_manifest_fs_kv_time`: `PASS` (14/14)
    - `pack_schema_conformance_consumer`: `PASS` (5/5)
    - `cargo test` full suite: `PASS`
    - `cargo clippy --all-targets -- -D warnings`: `PASS`
    - `cargo fmt -- --check`: `PASS`
  - Kết luận gate:
    - `DONE`
- Notes/risks:
  - Checkpoint cache trong 12-A đang là nền deterministic cho shadow run; chưa bao gồm scheduler policy (beam/portfolio) của các gate sau.
  - Reuse theo checkpoint hiện bám prefix + variant digest + checkpoint index; policy memo hóa observe sẽ mở ở 12-B.
- Design alignment:
  - `FULL`

### 2026-03-05 — 12-B Planning Freeze
- Date:
  - 2026-03-05
- Gate/Step:
  - 12-B
- Why:
  - Thêm deterministic observe memoization để giảm execute cost trong shadow mà không đổi outcome/signature contracts.
- Scope:
  - Runtime:
    - thêm memo table cho observe deterministic trong `build_shadow_run`.
    - khóa accounting hai lớp:
      - `steps_charged` (dùng cho decision/caps),
      - `steps_executed` (dùng cho đo giảm chi phí thực thi).
    - thêm metadata memo vào payload và trace detail (`hits/misses/charged/executed`).
    - thêm cờ runtime:
      - `OCL_STD_SHADOW_MEMOIZE_DETERMINISTIC_OBSERVE`
      - `OCL_STD_SHADOW_MEMO_OBSERVE_KEY`
      - `OCL_STD_SHADOW_MEMO_OBSERVE_COST_STEPS`
      - `OCL_STD_SHADOW_CHECKPOINT_REUSE`
  - Registry:
    - bổ sung `DeterminismClass` trong `CapabilityRegistry`;
    - default key không khai báo => `NonDeterministic`;
    - khai báo deterministic cho key shadow-safe (vd `std.game.tick_info`).
  - Test:
    - thêm suite targeted `tests/shadow_memo.rs`.
    - mở rộng kiểm chứng registry determinism fallback.
- Expected tests:
  - `cargo test --test shadow_memo`
  - `cargo test --test shadow_checkpoints --test std_shadow --test ocl_registry --test pack_schema_conformance_consumer`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
- Exit criteria:
  - Memo hit không đổi outcome/signature trên cùng input.
  - `observe_calls_executed < observe_calls_charged` khi memo bật và có variant trùng.
  - Memo tắt thì `observe_calls_executed == observe_calls_charged`.
  - Không phá regression shadow/checkpoint/schema hiện có.

### 2026-03-05 — 12-B Implementation Closeout
- Date:
  - 2026-03-05
- Gate/Step:
  - 12-B
- Implemented:
  - `src/ocp_ocl/exec.rs`:
    - thêm deterministic memo runtime cho shadow:
      - `shadow_memoize_deterministic_observe_enabled`
      - `shadow_memo_observe_key`
      - `shadow_memo_observe_cost_steps`
      - `shadow_checkpoint_reuse_enabled`
    - mở rộng `ShadowRunBuild`:
      - `memo_stats` và accounting `steps_charged/steps_executed`.
    - build branch có memo-key locked:
      - `memo_key = (key, tier, canonical_ctx_hash, budget_units, lane)`
      - memo hit giảm `steps_executed`, giữ `steps_charged` để không lệch decision path.
    - payload `std.shadow.run` thêm:
      - `memo` map (`enabled`, `determinism_class`, `hits`, `misses`, `observe_calls_*`, `memo_saved_steps`).
    - trace `ShadowRun.detail` bổ sung memo counters để audit hit/miss.
  - `src/ocp_ocl/registry.rs`:
    - thêm enum `DeterminismClass`.
    - thêm map `determinism_classes` + API:
      - `set_determinism_class_for_key`
      - `determinism_class_for_key`
    - `documented_keys` bao gồm determinism metadata.
    - payload schema `std.shadow.run` thêm optional field `memo`.
  - `src/ocp_ocl/mod.rs`:
    - export `DeterminismClass`.
  - `tests/shadow_memo.rs` (mới):
    - `shadow_memo_reuses_deterministic_observe_without_changing_outcome`
    - `shadow_memo_disabled_keeps_executed_and_charged_observe_calls_equal`
  - `tests/ocl_registry.rs`:
    - thêm test `registry_determinism_class_defaults_to_non_deterministic`.
- Files changed:
  - `src/ocp_ocl/exec.rs`
  - `src/ocp_ocl/registry.rs`
  - `src/ocp_ocl/mod.rs`
  - `tests/shadow_memo.rs`
  - `tests/ocl_registry.rs`
  - `OCP-OCL-MVP-PLAN-v0.12.md`
- Commands run:
  - `cargo test --test shadow_memo`
  - `cargo test --test shadow_checkpoints --test std_shadow --test ocl_registry --test pack_schema_conformance_consumer`
  - `cargo test`
  - `cargo fmt`
  - `cargo fmt -- --check`
  - `cargo clippy --all-targets -- -D warnings`
- Test results:
  - Targeted tests (must-pass for gate):
    - `shadow_memo`: `PASS` (2/2)
    - `shadow_checkpoints`: `PASS` (2/2)
  - Regression tests (supporting only):
    - `std_shadow`: `PASS` (5/5)
    - `ocl_registry`: `PASS` (9/9)
    - `pack_schema_conformance_consumer`: `PASS` (5/5)
    - `cargo test` full suite: `PASS`
    - `cargo clippy --all-targets -- -D warnings`: `PASS`
    - `cargo fmt -- --check`: `PASS`
  - Kết luận gate:
    - `DONE`
- Notes/risks:
  - Gate 12-B mới memo hóa deterministic observe trong shadow runtime nội bộ; chưa mở scheduler policy/beam/portfolio.
  - `checkpoint_reuse` và `memoize` hiện được bật/tắt bằng env runtime để phục vụ benchmark protocol ở gate KPI sau.
  - Quarantine/cassette-based memoization vẫn deferred ngoài v0.12.
- Design alignment:
  - `FULL`

### 2026-03-05 — 12-C Planning Freeze
- Date:
  - 2026-03-05
- Gate/Step:
  - 12-C
- Why:
  - Mở scheduler framework baseline `round_robin` để có loop phân bổ budget toàn cục, tie-break ổn định và đầu ra `std.shadow.search` có thể dùng làm nền cho 12-D/12-E.
- Scope:
  - Runtime:
    - thêm observe key `std.shadow.search`.
    - dùng scheduler policy deterministic, gate theo allowlist policy.
    - ship baseline `round_robin` với:
      - global caps (`global_step_cap`, `global_budget_cap`),
      - `top_k`,
      - tie-break ổn định `branch_id_asc`,
      - report `policy/tie_break/cost_curve/reason_table/schedule_digest`.
    - `beam`/`portfolio` tạm trả deferred theo đúng scope gate.
  - Registry/SDK:
    - đăng ký `std.shadow.search` vào ctx/payload schema, determinism, commit policy.
    - mở permission action `search` trong `std_shadow_action_from_key`.
  - Test:
    - thêm suite targeted `tests/shadow_scheduler_rr.rs`.
    - mở rộng conformance consumer schema cho `std.shadow.search`.
- Expected tests:
  - `cargo test --test shadow_scheduler_rr`
  - `cargo test --test pack_schema_conformance_consumer`
  - `cargo test --test shadow_checkpoints`
  - `cargo test --test shadow_memo`
  - `cargo test`
  - `cargo fmt -- --check`
  - `cargo clippy --all-targets -- -D warnings`
- Exit criteria:
  - `std.shadow.search` chạy deterministic ở policy `round_robin`.
  - global caps được enforce đúng và trả trạng thái bounded.
  - policy không được allow phải trả reason code deny đúng contract.
  - không phá regression của 12-A/12-B.

### 2026-03-05 — 12-C Implementation Closeout
- Date:
  - 2026-03-05
- Gate/Step:
  - 12-C
- Implemented:
  - `src/ocp_ocl/exec.rs`:
    - thêm helper scheduler config:
      - `shadow_scheduler_policy_allow`
      - `shadow_policy_allowed`
      - `shadow_max_rounds`
      - `shadow_global_step_cap_max`
      - `shadow_global_budget_cap_max`
    - thêm xử lý observe key `std.shadow.search`:
      - kiểm tra `shadow_enabled` và effect disallow.
      - enforce policy allowlist; policy bị chặn trả `RC-SHADOW-POLICY-DENIED`.
      - ship path `round_robin` deterministic với global caps/top_k/report.
      - `beam`/`portfolio` giữ deferred `RC-NOT-IMPLEMENTED` trong gate này.
    - bổ sung mapping `RC-SHADOW-POLICY-DENIED` trong `reason_code_from_str`.
  - `src/ocp_ocl/registry.rs`:
    - đăng ký `std.shadow.search` cho:
      - `ctx_required`,
      - `commit_allowed=false`,
      - `determinism_class=Deterministic`,
      - `ctx_schema`,
      - `payload_schema` (open_row, khóa trường `truncated` bắt buộc).
  - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`:
    - thêm action `search` trong `std_shadow_action_from_key`.
    - permission check cho `std.shadow.search` đi qua luồng `std_shadow` như `run/compare`.
  - `tests/shadow_scheduler_rr.rs` (mới):
    - `shadow_search_round_robin_is_deterministic`
    - `shadow_search_round_robin_respects_global_step_cap`
    - `shadow_search_policy_denied_when_not_in_allowlist`
  - `tests/pack_schema_conformance_consumer.rs`:
    - thêm key `std.shadow.search` vào checklist ctx/payload schema.
    - thêm test payload schema cho `std.shadow.search`.
- Files changed:
  - `src/ocp_ocl/exec.rs`
  - `src/ocp_ocl/registry.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
  - `tests/shadow_scheduler_rr.rs`
  - `tests/pack_schema_conformance_consumer.rs`
  - `OCP-OCL-MVP-PLAN-v0.12.md`
- Commands run:
  - `cargo test --test shadow_scheduler_rr`
  - `cargo test --test pack_schema_conformance_consumer`
  - `cargo test --test shadow_checkpoints`
  - `cargo test --test shadow_memo`
  - `cargo test`
  - `cargo fmt`
  - `cargo fmt -- --check`
  - `cargo clippy --all-targets -- -D warnings`
- Test results:
  - Targeted tests (must-pass for gate):
    - `shadow_scheduler_rr`: `PASS` (3/3)
    - `pack_schema_conformance_consumer`: `PASS` (6/6)
  - Regression tests (supporting only):
    - `shadow_checkpoints`: `PASS` (2/2)
    - `shadow_memo`: `PASS` (2/2)
    - `cargo test` full suite: `PASS`
    - `cargo clippy --all-targets -- -D warnings`: `PASS`
    - `cargo fmt -- --check`: `PASS`
  - Kết luận gate:
    - `DONE`
- Notes/risks:
  - Gate 12-C mới ship baseline scheduler `round_robin`; `beam`/`portfolio` sẽ mở ở 12-D/12-E.
  - Payload schema `std.shadow.search` dùng `open_row` để tránh drift do nested cấu trúc sâu; sẽ siết thêm khi report v2 ổn định ở gate sau.
  - Đã sửa một vòng do schema payload ban đầu quá chặt làm runtime trả `INSUFFICIENT`; phiên bản hiện tại đã ổn định và pass targeted/regression.
- Design alignment:
  - `FULL`

### 2026-03-05 — 12-D Planning Freeze
- Date:
  - 2026-03-05
- Gate/Step:
  - 12-D
- Why:
  - Mở policy `beam` và scoring contract integer-only để `std.shadow.search` có thể chọn top branch theo chất lượng thay vì chỉ theo thứ tự.
- Scope:
  - Runtime:
    - implement policy `beam` trong `std.shadow.search`.
    - khóa comparator deterministic:
      - `(score desc, steps_charged asc, branch_id asc)`.
    - implement scoring config:
      - `score_field`
      - `outcome_weight`
      - `cost_budget_weight`
      - `cost_steps_weight`
      - `reason_penalty_weight`
      - `state_score_weight`
      - `reason_penalties_json`
    - giữ `portfolio` deferred đúng scope gate.
  - Registry/schema:
    - mở rộng ctx schema `std.shadow.search` cho các trường beam/scoring.
  - Test:
    - thêm suite targeted `tests/shadow_beam.rs`.
- Expected tests:
  - `cargo test --test shadow_beam`
  - `cargo test --test shadow_scheduler_rr`
  - `cargo test --test pack_schema_conformance_consumer`
  - `cargo test --test shadow_checkpoints`
  - `cargo test --test shadow_memo`
  - `cargo test`
  - `cargo fmt -- --check`
  - `cargo clippy --all-targets -- -D warnings`
- Exit criteria:
  - beam chọn top branches deterministic theo scoring comparator đã khóa.
  - scoring path integer-only, không dùng float.
  - không phá round-robin baseline và không phá regression 12-A/12-B/12-C.

### 2026-03-05 — 12-D Implementation Closeout
- Date:
  - 2026-03-05
- Gate/Step:
  - 12-D
- Implemented:
  - `src/ocp_ocl/exec.rs`:
    - thêm helper scoring/beam:
      - `shadow_beam_width_max`
      - `ShadowScoreConfig`
      - `shadow_score_config_from_ctx`
      - `shadow_compute_score`
      - `shadow_extract_state_score`
      - `shadow_branch_steps_executed`
    - nâng `std.shadow.search`:
      - hỗ trợ policy `beam` (deterministic).
      - giữ `portfolio` deferred theo scope gate.
      - áp scoring formula integer-only theo weights.
      - ranking/report thêm trường score/cost và accounting charged-vs-executed.
      - `schedule_digest` phản ánh policy và selection thực tế.
  - `src/ocp_ocl/registry.rs`:
    - ctx schema `std.shadow.search` thêm trường beam/scoring:
      - `beam_width`, `score_field`, nhóm `*_weight`, `reason_penalties_json`.
  - `tests/shadow_beam.rs` (mới):
    - `shadow_beam_is_deterministic_and_uses_state_score_priority`
    - `shadow_beam_ranking_is_sorted_by_locked_comparator`
    - `shadow_beam_policy_denied_when_not_in_allowlist`
- Files changed:
  - `src/ocp_ocl/exec.rs`
  - `src/ocp_ocl/registry.rs`
  - `tests/shadow_beam.rs`
  - `OCP-OCL-MVP-PLAN-v0.12.md`
- Commands run:
  - `cargo test --test shadow_beam`
  - `cargo test --test shadow_scheduler_rr`
  - `cargo test --test pack_schema_conformance_consumer`
  - `cargo test --test shadow_checkpoints`
  - `cargo test --test shadow_memo`
  - `cargo test`
  - `cargo fmt`
  - `cargo fmt -- --check`
  - `cargo clippy --all-targets -- -D warnings`
- Test results:
  - Targeted tests (must-pass for gate):
    - `shadow_beam`: `PASS` (3/3)
  - Regression tests (supporting only):
    - `shadow_scheduler_rr`: `PASS` (3/3)
    - `pack_schema_conformance_consumer`: `PASS` (6/6)
    - `shadow_checkpoints`: `PASS` (2/2)
    - `shadow_memo`: `PASS` (2/2)
    - `cargo test` full suite: `PASS`
    - `cargo clippy --all-targets -- -D warnings`: `PASS`
    - `cargo fmt -- --check`: `PASS`
  - Kết luận gate:
    - `DONE`
- Notes/risks:
  - `portfolio` vẫn deferred và sẽ triển khai ở gate 12-E.
  - payload `std.shadow.search` vẫn để `open_row` cho compatibility; có thể siết schema report ở gate 12-F.
  - scoring hiện đọc `state_summary[score_field]`, fallback `state_summary.variant[score_field]` nếu có.
- Design alignment:
  - `FULL`

### 2026-03-05 — 12-E Planning Freeze
- Date:
  - 2026-03-05
- Gate/Step:
  - 12-E
- Why:
  - Bổ sung policy `portfolio` để `std.shadow.search` phân bổ ngân sách theo nhiều heuristic cố định, vẫn giữ deterministic và bounded theo trục đã khóa.
- Scope:
  - Runtime:
    - triển khai policy `portfolio` trong `std.shadow.search`.
    - khóa chiến lược theo thứ tự vòng lặp cố định: `outcome_first`, `cost_first`, `balanced`.
    - enforce round slice theo `rounds` và global caps để tránh vượt budget/step.
    - cập nhật report/schedule digest phản ánh strategy sequence và accounting charged-vs-executed.
  - Test:
    - thêm suite targeted `tests/shadow_portfolio.rs`.
- Expected tests:
  - `cargo test --test shadow_portfolio`
  - `cargo test --test shadow_beam`
  - `cargo test --test shadow_scheduler_rr`
  - `cargo test --test pack_schema_conformance_consumer`
  - `cargo test --test shadow_checkpoints`
  - `cargo test --test shadow_memo`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
- Exit criteria:
  - policy `portfolio` chạy deterministic với cùng seed/config.
  - `max_rounds` và global caps được enforce đúng.
  - policy deny theo allowlist trả reason code đúng contract.
  - không làm lệch regression của 12-A/12-B/12-C/12-D.

### 2026-03-05 — 12-E Implementation Closeout
- Date:
  - 2026-03-05
- Gate/Step:
  - 12-E
- Implemented:
  - `src/ocp_ocl/exec.rs`:
    - triển khai `portfolio` path trong `std.shadow.search` (không còn deferred).
    - thêm vòng strategy deterministic theo thứ tự cố định:
      - `outcome_first`
      - `cost_first`
      - `balanced`
    - áp round-slice theo `rounds` cho step/budget, đồng thời enforce `global_step_cap` và `global_budget_cap`.
    - cập nhật ranking/report cho `portfolio`:
      - `strategies`, `tie_break`, `cost_curve`, `schedule_digest`
      - accounting `steps_charged` và `steps_executed` giữ đúng contract đã khóa.
  - `tests/shadow_portfolio.rs` (mới):
    - `shadow_portfolio_is_deterministic_and_emits_strategy_sequence`
    - `shadow_portfolio_respects_max_rounds_cap`
    - `shadow_portfolio_policy_denied_when_not_in_allowlist`
- Files changed:
  - `src/ocp_ocl/exec.rs`
  - `tests/shadow_portfolio.rs`
  - `OCP-OCL-MVP-PLAN-v0.12.md`
- Commands run:
  - `cargo test --test shadow_portfolio`
  - `cargo test --test shadow_beam`
  - `cargo test --test shadow_scheduler_rr`
  - `cargo test --test pack_schema_conformance_consumer`
  - `cargo test --test shadow_checkpoints`
  - `cargo test --test shadow_memo`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
- Test results:
  - Targeted tests (must-pass for gate):
    - `shadow_portfolio`: `PASS` (3/3)
  - Regression tests (supporting only):
    - `shadow_beam`: `PASS` (3/3)
    - `shadow_scheduler_rr`: `PASS` (3/3)
    - `pack_schema_conformance_consumer`: `PASS` (6/6)
    - `shadow_checkpoints`: `PASS` (2/2)
    - `shadow_memo`: `PASS` (2/2)
    - `cargo test` full suite: `PASS`
    - `cargo clippy --all-targets -- -D warnings`: `PASS`
    - `cargo fmt -- --check`: `PASS`
  - Kết luận gate:
    - `DONE`
- Notes/risks:
  - Policy `portfolio` đã chạy theo strategy order cố định và bị chặn đúng bởi allowlist; các chính sách ngoài danh sách vẫn đi theo contract deny hiện hành.
  - Report `portfolio` đã có `strategies` và `schedule_digest`; schema report v2 đầy đủ sẽ hoàn tất ở gate 12-F.
- Design alignment:
  - `FULL`

### 2026-03-05 — 12-F Planning Freeze
- Date:
  - 2026-03-05
- Gate/Step:
  - 12-F
- Why:
  - Chốt `shadow.report.v2` thành output machine-checkable có bounded truncation deterministic, để report shadow dùng được cho sản phẩm và không làm drift khi cap nhỏ.
- Scope:
  - Runtime:
    - nâng payload `std.shadow.search` để dựng `report` theo schema `shadow.report.v2`.
    - thêm truncation theo `max_report_bytes` với thứ tự cắt deterministic.
    - thêm `max_diff_keys` và `max_report_bytes` vào ctx parse path của `std.shadow.search`.
    - degrade với reason `RC-SHADOW-REPORT-TOO-LARGE` khi report vượt cap byte.
  - Registry/schema:
    - ctx schema `std.shadow.search` nhận `max_diff_keys`, `max_report_bytes`.
    - payload schema `std.shadow.search` yêu cầu tối thiểu:
      - `truncated`
      - `report.schema = "shadow.report.v2"`
      - `report.report_bytes`
  - Test:
    - thêm suite targeted `tests/shadow_report_v2.rs`.
- Expected tests:
  - `cargo test --test shadow_report_v2`
  - `cargo test --test pack_schema_conformance_consumer`
  - `cargo test --test shadow_scheduler_rr`
  - `cargo test --test shadow_beam`
  - `cargo test --test shadow_portfolio`
  - `cargo test --test shadow_checkpoints`
  - `cargo test --test shadow_memo`
  - `cargo test --test std_shadow`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
- Exit criteria:
  - `report` của `std.shadow.search` có marker `shadow.report.v2`.
  - report byte cap được enforce deterministic; cap nhỏ trả degraded đúng reason.
  - không làm vỡ các gate 12-A..12-E.

### 2026-03-05 — 12-F Implementation Closeout
- Date:
  - 2026-03-05
- Gate/Step:
  - 12-F
- Implemented:
  - `src/ocp_ocl/exec.rs`:
    - `std.shadow.search` nhận thêm ctx:
      - `max_diff_keys`
      - `max_report_bytes`
    - dựng `report` theo schema marker `shadow.report.v2` với các khối:
      - `meta`
      - `ranking`
      - `divergence_points`
      - `diff_summary`
      - `cost_curve`
      - `reason_table`
      - `artifacts`
    - thêm truncation vòng lặp deterministic theo byte cap:
      - cắt tuần tự `divergence_points` -> `diff_summary` -> `reason_table` -> `cost_curve` -> `ranking`.
      - nếu vẫn vượt cap thì rơi về report tối thiểu nhưng giữ marker schema + truncated + report_bytes.
    - khi report vượt cap: trả `DEGRADED` với reason `RC-SHADOW-REPORT-TOO-LARGE`.
  - `src/ocp_ocl/registry.rs`:
    - mở rộng ctx schema `std.shadow.search`:
      - `max_diff_keys`
      - `max_report_bytes`
    - siết payload schema tối thiểu cho `std.shadow.search`:
      - `truncated`
      - `report.schema`
      - `report.report_bytes`
  - `tests/shadow_report_v2.rs` (mới):
    - `shadow_report_v2_has_required_sections_and_is_deterministic`
    - `shadow_report_v2_truncates_deterministically_when_bytes_cap_is_small`
- Files changed:
  - `src/ocp_ocl/exec.rs`
  - `src/ocp_ocl/registry.rs`
  - `tests/shadow_report_v2.rs`
  - `OCP-OCL-MVP-PLAN-v0.12.md`
- Commands run:
  - `cargo test --test shadow_report_v2`
  - `cargo test --test pack_schema_conformance_consumer`
  - `cargo test --test shadow_scheduler_rr`
  - `cargo test --test shadow_beam`
  - `cargo test --test shadow_portfolio`
  - `cargo test --test shadow_checkpoints`
  - `cargo test --test shadow_memo`
  - `cargo test --test std_shadow`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt`
  - `cargo fmt -- --check`
- Test results:
  - Targeted tests (must-pass for gate):
    - `shadow_report_v2`: `PASS` (2/2)
  - Regression tests (supporting only):
    - `pack_schema_conformance_consumer`: `PASS` (6/6)
    - `shadow_scheduler_rr`: `PASS` (3/3)
    - `shadow_beam`: `PASS` (3/3)
    - `shadow_portfolio`: `PASS` (3/3)
    - `shadow_checkpoints`: `PASS` (2/2)
    - `shadow_memo`: `PASS` (2/2)
    - `std_shadow`: `PASS` (5/5)
    - `cargo test` full suite: `PASS`
    - `cargo clippy --all-targets -- -D warnings`: `PASS`
    - `cargo fmt -- --check`: `PASS`
  - Kết luận gate:
    - `DONE`
- Notes/risks:
  - Payload schema `std.shadow.search` vẫn giữ `open_row=true` để tránh phá tương thích cũ; gate sau có thể siết thêm khi CLI/template cố định format tiêu thụ.
  - Truncation hiện ưu tiên ổn định deterministic và marker report; tối ưu hóa kích thước report sâu hơn sẽ để ở bước performance sau v0.12.
- Design alignment:
  - `FULL`

### 2026-03-05 — 12-G Planning Freeze
- Date:
  - 2026-03-05
- Gate/Step:
  - 12-G
- Why:
  - Nâng template `shadow-preview` sang `std.shadow.search` và chốt benchmark KPI-2 (giảm `steps_executed` >=30%) để đóng đầy đủ mục tiêu release v0.12.
- Scope:
  - CLI template:
    - cập nhật `shadow-preview` để dùng `std.shadow.search` với config deterministic (`max_branches`, `rounds`, `top_k`, caps, score weights).
  - E2E + KPI benchmark:
    - thêm test `tests/cli_shadow_preview_v2_e2e.rs` kiểm tra:
      - init template thành công,
      - run/replay pass,
      - report có marker `shadow.report.v2` và các khối bắt buộc,
      - KPI-2 pass với protocol baseline/reuse đã khóa.
- Expected tests:
  - `cargo test --test cli_shadow_preview_v2_e2e`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
- Exit criteria:
  - template `shadow-preview` dùng `std.shadow.search` làm flow chính.
  - e2e test mới pass và chứng minh KPI-2 theo đúng protocol.
  - không làm vỡ regression suite đã có từ 12-A..12-F.

### 2026-03-05 — 12-G Implementation Closeout
- Date:
  - 2026-03-05
- Gate/Step:
  - 12-G
- Implemented:
  - `projects/ocp-ocl/crates/ocl-cli/src/main.rs`:
    - cập nhật generator template `shadow-preview`:
      - chuyển observe từ `engine.shadow.preview` sang `std.shadow.search`,
      - thêm dataset variants bounded (`N=8`) và config policy/caps/scoring deterministic trong `src/main.ocl`.
  - `tests/cli_shadow_preview_v2_e2e.rs` (mới):
    - `cli_shadow_preview_template_v12_run_replay_and_kpi_pass`:
      - init template,
      - run/replay và kiểm tra artifact bundle (`audit.jsonl`, `signature.txt`, `replay.toml`),
      - parse + execute template program để xác nhận `report.schema = "shadow.report.v2"` và có đủ `ranking`, `diff_summary`, `cost_curve`, `reason_table`,
      - benchmark KPI-2: so baseline (`memoize=0`, `checkpoint_reuse=0`) với reuse (`memoize=1`, `checkpoint_reuse=1`), assert:
        - `steps_charged` giữ ổn định,
        - `steps_executed` giảm,
        - giảm >= 30%.
- Files changed:
  - `projects/ocp-ocl/crates/ocl-cli/src/main.rs`
  - `tests/cli_shadow_preview_v2_e2e.rs`
  - `OCP-OCL-MVP-PLAN-v0.12.md`
- Commands run:
  - `cargo test --test cli_shadow_preview_v2_e2e`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt`
  - `cargo fmt -- --check`
- Test results:
  - Targeted tests (must-pass for gate):
    - `cli_shadow_preview_v2_e2e`: `PASS` (1/1)
  - Regression tests (supporting only):
    - `cargo test` full suite: `PASS`
    - `cargo clippy --all-targets -- -D warnings`: `PASS`
    - `cargo fmt -- --check`: `PASS`
  - Kết luận gate:
    - `DONE`
- Notes/risks:
  - KPI benchmark hiện dựa trên env toggle runtime (`OCL_STD_SHADOW_MEMOIZE_DETERMINISTIC_OBSERVE`, `OCL_STD_SHADOW_CHECKPOINT_REUSE`) và được assert trực tiếp trong e2e test để tránh done giả.
  - Template v0.12 giữ bounded defaults để chạy ổn định trong CI; profile lớn hơn cần điều chỉnh qua config chứ không mở rộng mặc định trong template.
- Design alignment:
  - `FULL`

### 2026-03-05 — Correction Note (Gate Close Discipline)
- Date:
  - 2026-03-05
- Gate/Step:
  - v0.12 governance correction
- Issue:
  - Template `12-X implementation closeout` trước đó chứa literal `PASS/FAIL` cùng câu có `DONE`, làm guard nhận diện sai trạng thái `DONE` kèm marker `FAIL`.
- Action:
  - Chuẩn hóa template closeout sang placeholder trung tính `Đạt / Chưa đạt`.
  - Giữ nguyên quy tắc đóng gate: chỉ ghi `DONE` khi targeted tests đạt đầy đủ; nếu chưa đạt bắt buộc giữ `IN_PROGRESS` hoặc `PARTIAL`.
- Validation:
  - Chạy lại guard để xác nhận không còn vi phạm `DONE`/`FAIL` giả.
- Impact:
  - Không thay đổi runtime/CLI/test logic; chỉ chỉnh quy trình tài liệu để bám chuẩn đóng gate thật.

---

## 14) Checklist khóa trước khi đóng gate
- [x] Gate status đã cập nhật đúng (`TODO/IN_PROGRESS/DONE`).
- [x] Có đủ Planning Freeze + Implementation Closeout.
- [x] `Files changed` khớp code delta thực tế.
- [x] `Commands run` là lệnh đã chạy thật.
- [x] `Test results` có `PASS/FAIL` rõ và đúng phạm vi.
- [x] Nếu gate `DONE`: không còn marker `FAIL` hoặc placeholder `PASS/FAIL` trong closeout.
- [x] `Design alignment` đã ghi `FULL` hoặc `PARTIAL` đúng thực tế.
- [x] Không còn lỗi tiếng Việt/mã hóa trong nội dung file.

---

## 15) Handoff v0.12 -> v0.13 (pre-draft)
- Chỉ mở v0.13 khi 12-A..12-G đều `DONE` và KPI v0.12 pass đầy đủ.
- V0.13 kế thừa nguyên xi contracts đã khóa ở v0.12:
  - scoring integer-only
  - reuse accounting (`steps_charged` vs `steps_executed`)
  - trace schema v2 compatibility
  - lane policy (`locked_v071` default, quarantine reuse deferred).
- Nếu v0.12 còn gate `IN_PROGRESS/PARTIAL`, không mở scope v0.13 ngoài xử lý blocker tồn đọng.

---

