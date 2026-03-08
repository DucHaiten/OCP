# OCP MVP PLAN v0.5

Ngày tạo: 2026-02-25  
Mục tiêu: mở rộng OCP thành nền tảng “cosmology + hive” theo hướng governance-first, deterministic-first, signed-supply-chain-first; không nổ scope và không phá các khóa đã chốt ở v0.4.

## Quy ước cập nhật bắt buộc (áp dụng từ v0.5)
- Mọi thay đổi kế hoạch phải cập nhật file này trước khi triển khai.
- Mọi workstream hoàn tất phải ghi implementation evidence ngay sau khi chạy test.
- Không nhảy gate: chỉ mở workstream tiếp theo khi workstream hiện tại đạt `DONE`.
- Mỗi entry bắt buộc có: ngày, phạm vi, file thay đổi, lệnh chạy, kết quả, rủi ro còn lại.

## Quy ước phát hành cứng (Release Discipline, bắt buộc)
- `v0.5` được coi là bản hoàn thiện trước `v1.0`, không phải bản dev nội bộ.
- Định nghĩa `DONE` cho mỗi gate: không còn TODO kỹ thuật mở, không còn blocker mở, không còn “fix later”.
- Không chấp nhận nợ kỹ thuật mang sang gate sau nếu ảnh hưởng correctness/determinism/signed-locked/security.
- Nếu còn mục `PLANNED` trong workstream thì toàn bộ `v0.5` chưa được phép gắn trạng thái `DONE`.
- Mọi warning/review note phải được phân loại rõ:
  - `BLOCK`: bắt buộc xử lý ngay trong gate hiện tại trước khi đóng gate.
  - `NICE`: chỉ được defer nếu không ảnh hưởng release criteria và phải ghi quyết định defer có lý do cụ thể.
- Không dùng “sẽ quay lại sau” làm cơ chế mặc định; phải đóng dứt điểm trong gate hiện tại hoặc hạ scope chính thức bằng văn bản.

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

## Trạng thái workstreams v0.5
- V5-W0 Rebaseline v0.4 green + topology freeze: `DONE` (2026-03-03)
- V5-W1 Cosmos config + lock/sign + universe wiring: `DONE` (2026-03-03)
- V5-W2 Domain isolation + bridge quotas: `DONE` (2026-03-03)
- V5-W3 Shadow execution + compare digest: `DONE` (2026-03-03)
- V5-W4 Hive supervisors + bounded swarm runtime: `DONE` (2026-03-03)
- V5-W5 Parallel views contract + renderer stub: `DONE` (2026-03-03)
- V5-W6 Organ packs install/verify/lock: `DONE` (2026-03-03)
- V5-W7 Conformance v0.5 SoT: `DONE` (2026-03-03)

---

## 0) Mục tiêu v0.5

v0.5 tập trung 3 nâng cấp:
1. Cosmology layer: đa universe + đa domain trong cùng platform, cách ly nhân quả rõ ràng.
2. Shadow execution: chạy song trùng để so sánh/parity/chính sách mà không commit vào truth world.
3. Hive runtime: Overmind -> Cerebrates -> Swarm workers, bounded và deterministic.

v0.5 không có mục tiêu biến OCP thành game engine, browser engine hay HPC engine trong core. OCP là control-plane + proof-plane; “muscle” đi qua organ packs signed.

## 0.1) Kiến trúc 2 tầng v0.5 (Kernel Axis + Foundation Kits)

### 0.1.1 Định nghĩa 2 tầng
1. Tầng 1 — OCP Kernel Axis (trục gốc):
   - là “luật vật lý” của platform.
   - bao gồm locks kế thừa v0.4 và phần v0.5A/v0.5B: universe/domain/shadow/hive/views/locks/audit/deterministic lane.
   - là phần ổn định, ít thay đổi, khó phá.
2. Tầng 2 — OCP Foundation (Kits/Blueprints + Organ-based muscle):
   - là “bộ cơ phổ thông” + “cụm hive mind theo use-case” để dev đa số không cần học sâu cosmology/hive internals vẫn dùng được.
   - Foundation cung cấp:
     - Kits/Blueprints: bộ module OCP + wiring mẫu theo bài toán (`workflow/webapi/bot/agent-swarm/ui-view/...`).
     - Presets: scaffolding dự án + cấu hình cosmos tối thiểu + policy/budget mẫu.
     - Organ packs curated: IO/HTTP/DB/UI/LLM/TTS... pin bằng `organs.lock.v1` trong locked mode.

### 0.1.2 Mục tiêu của 2 tầng (adoption-first nhưng không hạ chuẩn)
1. Dev phổ thông chỉ cần Preset/Kits để chạy, không bắt buộc hiểu sâu universe/domain/shadow/hive.
2. Người dùng nâng cao vẫn leo lên Kernel với cùng semantics, không có “ngôn ngữ thứ hai”.
3. Foundation phải desugar về Kernel primitives (commit-gated, IAM, budgets, audit, deterministic lane).

### 0.1.3 Nguyên tắc “Kernel invariants không thể bị phá”
1. No naked IO: mọi IO đi qua organ packs/capabilities.
2. Commit-gated effects: kits không được tạo đường side-effect lách commit.
3. Budgets in DNA: preset/kit phải khai báo budget/bounds mặc định.
4. Deterministic lane là SoT: mọi kit chính thức phải có đường chạy deterministic (ít nhất smoke + digest stable).
5. IAM/Audit/Replay: mọi kit chạy trên audit chain chuẩn và replay được.

### 0.1.4 Learning ladder
1. L0 (Preset user): `ocp init --preset ...` -> chạy được dự án, chỉnh config cơ bản.
2. L1 (Kit composer): ghép kits, chỉnh wiring, viết module business nhỏ.
3. L2 (Organ user): chọn/install/verify organ packs, pin locks, dùng capability đúng policy.
4. L3 (Advanced cosmology): tuning universe/domain/bridge/shadow.
5. L4 (Kernel contributor): sửa scheduler/locking/ABI/audit.

### 0.1.5 Phạm vi v0.5 cho Foundation (không nổ scope)
1. v0.5 chỉ ship tối thiểu:
   - 2 presets chính thức: `workflow_basic`, `agent_swarm_basic`.
   - 1 kit registry chuẩn dựa trên catalog/deps lock hiện có.
   - wiring contract để kits map vào domain/view/hive (nằm trong `cosmos.toml`, hash vào `cosmos.lock.v1`).
2. Không tạo lock file mới cho kits; kits là packages bình thường trong deps/catalog lock flow.

---

## 1) Decision Locks v0.5 (đóng cứng)

### 1.1 Axis lock kế thừa v0.4 (bắt buộc giữ nguyên)
1. 4-kind everywhere.
2. Commit-gated effects.
3. Budgets in DNA.
4. No naked IO.
5. IAM per package/module.
6. Audit chain + replay.
7. Deterministic lane là SoT.

### 1.2 Scope lock để tránh nổ timeline
1. v0.5 chia thành 2 internal bands:
   - v0.5A: cosmos + universe/domain + bridge + shadow.
   - v0.5B: hive + views + organ packs + conformance mở rộng.
2. Vẫn giữ một tài liệu v0.5 duy nhất, nhưng gate pass theo thứ tự W0 -> W7.

### 1.3 Cosmos lock + lock-order lock
1. File editable: `cosmos.toml` (schema `ocp.cosmos.v1`).
2. File generated: `cosmos.lock.v1` (canonical, signed trong locked mode).
3. Lock sync order bắt buộc:
   - `deps.lock.v2` + `catalog.lock.v2` + `policy.lock.v1` trước.
   - `cosmos lock sync` sau.
4. `cosmos.lock.v1` chỉ hash lock inputs đã tồn tại; không tạo vòng lock ngược.
5. Cosmos scaffold phải có preset để đóng gói complexity:
   - mặc định sinh `universe=default`, `domain=default`, `shadow=off`.
   - `ocp cosmos init --preset default|ci|prod`.

### 1.4 Universe selection lock
1. Mọi lệnh `check/run/test/build/compose/verify` hỗ trợ `--universe <id>`.
2. CI cosmos lane mặc định: `--universe ci_locked`; legacy compatibility lane không ép universe.
3. Universe profile có thể đổi runtime/engine/audit/trace, nhưng không được phá invariants core.
4. Universe là config overlay, không phải runtime fork:
   - chỉ override cấu hình đã có (`runtime_mode`, `engine`, `audit`, `trace`, `policy_profile`, budgets/permissions).
   - cấm thêm universe-specific runtime codepath mới trong core.

### 1.5 IAM precedence lock (bổ sung universe scope)
1. Thứ tự đánh giá:
   - `module@universe` -> `package@universe` -> `module@project` -> `package@project` -> default deny.
2. Trong mỗi scope: `deny > allow > prompt > default deny`.
3. Locked mode dùng exact key match.

### 1.6 Context binding lock (anti replay chéo)
1. Cert/commit binding bắt buộc gồm:
   - `req + args + policy + tick + universe_id + domain_id + shadow_id`.
2. Replay/commit cross-universe, cross-domain, cross-shadow phải fail-hard.

### 1.7 Domain isolation lock
1. Domain là biên giới state + event + mailbox + budget counters.
2. Domain A không được đọc/commit/event-consume của domain B nếu không qua bridge explicit.
3. Bridge keys:
   - `domain.bridge.emit` (commit-gated)
   - `domain.bridge.poll` (observe)

### 1.8 Bridge quota lock
1. Enforce quota ở 2 điểm:
   - admission khi `emit`.
   - dispatch per-tick.
2. Fail codes:
   - `V-DOMAIN-BRIDGE-DENIED`
   - `V-DOMAIN-BRIDGE-CAP`
3. Không retry ngầm.
4. Bridge quota constant-time:
   - admission quota check phải O(1): chỉ counter + queue depth, không scan backlog.
   - dispatch per-tick pop tối đa `max_per_tick`, không duyệt toàn queue.
   - reset quota theo tick phải tick-local (`tick_epoch`, `count`), không clear map tuyến tính.
5. Per-bridge bounded queue:
   - mỗi bridge có queue riêng, bounded bởi `bridge_queue_capacity`.
   - queue full phải fail-honest `V-DOMAIN-BRIDGE-CAP`, không spill sang queue khác.
6. Backpressure visibility:
   - mọi bridge backpressure phải ghi audit/trace hash-only counters.
7. Deterministic dispatch order:
   - order stable theo `(tick, bridge_id, bridge_seq)`.
   - `bridge_seq` monotonic per-bridge.
   - multi-bridge tie-break theo key đã khóa ở deterministic scheduler (`domain_rank/...`), không phụ thuộc map iteration.

### 1.9 Shadow execution lock
1. Default `forbid_commit`:
   - mọi commit bị chặn trước adapter/plugin commit path.
   - ghi audit/trace deny event hash-only.
2. `shadow_commit_log`:
   - không apply world effect.
   - chỉ ghi shadow event log + digest.
3. Mismatch digest giữa main/shadow:
   - `V-SHADOW-MISMATCH`
   - `V-SHADOW-UNSUPPORTED`.
4. Shadow digest contract:
   - digest phải hash trên canonical execution transcript (hash-only) gồm tối thiểu:
     - `logical_tick`
     - `event_id`
     - `actor_id/task_id`
     - `args_hash`
     - `decision_outcome`
     - `commit_intents_hash` (empty nếu `forbid_commit`)
5. Shadow digest cấm dùng nguồn nondeterministic:
   - wall-clock, OS timestamp, pid/tid, pointer address, locale/timezone/env text.
   - filesystem/map/set iteration order nếu chưa canonicalize.
6. Logical time + seeded RNG:
   - deterministic/shadow lane chỉ dùng logical tick cho time source.
   - RNG (nếu có) phải seeded deterministic theo run context (`universe/domain/shadow/scenario/tick`) hoặc seed explicit.
7. Unsupported > mismatch:
   - phát hiện nondeterminism không chuẩn hóa được phải trả `V-SHADOW-UNSUPPORTED` trước.
   - `V-SHADOW-MISMATCH` chỉ dùng khi contract inputs hợp lệ nhưng transcript khác.

### 1.10 Hive lock
1. Topology:
   - 1 Overmind per universe-run.
   - 1 Cerebrate per domain.
   - Swarm workers bounded.
2. Caps bắt buộc:
   - `max_swarm_workers`
   - `max_fanout_per_task`
   - `mailbox_max_depth`
   - `max_spawn_per_tick`
   - `max_domains`
   - `max_shadow_worlds`
3. Deterministic tie-break key:
   - `(domain_rank, cerebrate_id, mailbox_seq, event_id, task_id)`.
4. Worker lifecycle reuse:
   - worker structs phải dùng pool/slab reuse, không alloc/free liên tục theo tick.
   - mailbox buffers phải prealloc + recycle, không tăng vô hạn theo thời gian.
   - ephemeral allocations theo tick nên đi qua tick-arena reset-per-tick.
5. Long-run stability:
   - runtime bắt buộc expose counters:
     - `workers_alive`
     - `workers_spawned_total`
     - `workers_reused_total`
     - `mailbox_max_depth_observed`
     - `alloc_events_total` (hoặc proxy tương đương)
   - phải có soak run N ticks trong deterministic lane để chứng minh tăng trưởng memory/counters bounded.
6. Reuse không được phá determinism:
   - worker/buffer reuse không được thay đổi scheduling order key đã khóa.

### 1.11 View lock
1. Truth state và view state tách biệt.
2. View modules không được direct IO.
3. `std.view.render_tree` / `std.view.render_text` là pure observe contract.

### 1.12 Organ pack lock
1. Organ packs là cơ chế mở rộng chính cho UI/graphics/AI bindings.
2. `organs.lock.v1` bắt buộc pin:
   - `pack_id`, `version`, `platform`, `artifact_hash256`, `signer_id`, `abi_version`.
3. Locked mode bắt buộc verify signer/trust trước install/use.
4. Organ packs phải tái sử dụng pipeline extension đã có từ W4/W6:
   - chung trust store/signature verification/hash policy.
   - tránh tạo hệ lock/security song song gây drift cho dev.

### 1.13 CI/SoT lock
1. SoT pass/fail platform v0.5 gồm 2 lane kế thừa + nâng cấp:
   - Lane kế thừa v0.4 (legacy apps): `ocp test --conformance --locked --manifest projects/ocp/conformance/conformance.v1.toml ...`.
   - Lane nâng cấp v0.5 Foundation: `ocp test --conformance --locked --manifest projects/ocp/conformance/conformance.v5.toml ...`.
2. `--universe ci_locked` là bắt buộc cho cosmos-ready lane/manifest; không áp cứng cho toàn bộ legacy scenarios.
3. Conformance report always-written.
4. Required digest order-sensitive, không đưa `warnings_count`.
5. Warning denylist preflight fail-hard:
   - `W-W4-UNSIGNED-BUILD`
   - `W-W8-CATALOG-V1-COMPAT`.

### 1.14 Migration lock
1. Unlocked mode cho phép legacy fallback single-universe.
2. Locked mode:
   - nếu project dùng cosmology thì bắt buộc `cosmos.lock.v1`.
   - thiếu lock -> fail-hard.

### 1.15 Two-tier contract lock (Kernel vs Foundation)
1. Foundation không được định nghĩa semantics mới cho effect/state/IO.
2. Mọi Kit/Blueprint phải biên dịch/desugar về primitives hiện có:
   - effects: commit-gated.
   - observe: pure hoặc hash-only payload theo trace contract.
   - IO: chỉ qua organ packs/capabilities.
3. Kit không được nâng quyền IAM:
   - không tự cấp key.
   - không ghi đè deny.
   - mọi quyền vẫn theo precedence lock 1.5.

### 1.16 Beginner default lock (unlocked ergonomics)
1. Nếu không có cosmology, runtime được fallback trong unlocked:
   - `universe_id = default_unlocked`
   - `domain_id = default`
   - hive chạy single-cerebrate bounded tối thiểu.
2. Fallback chỉ áp dụng unlocked mode; locked mode vẫn yêu cầu đầy đủ lock theo mục 4.
3. Fallback không được phá invariants: commit-gated, budgets, audit.

### 1.17 Foundation governance lock (official kits/presets)
1. Official presets/kits bắt buộc:
   - pin dependencies qua `deps.lock.v2` + `catalog.lock.v2`.
   - nếu dùng organ thì bắt buộc `organs.lock.v1`.
   - pass deterministic conformance ít nhất subset.
2. Community kits được phép tồn tại nhưng:
   - locked mode vẫn enforce signature/trust/locks như thường.
   - không được gắn nhãn official nếu không pass SoT.

### 1.18 Kit wiring lock (wiring nằm trong cosmos và bị ký)
1. Wiring kit -> domain/view/organ requirements phải nằm trong `cosmos.toml`.
2. Wiring phải được canonicalize + hash vào `cosmos.lock.v1`.
3. Locked mode không cho phép wiring ngoài lock (fail-hard).

---

## 2) Schema và file contracts v0.5

### 2.1 `cosmos.toml` (editable)
Path: `<project>/cosmos.toml`  
Schema: `ocp.cosmos.v1`

Nội dung chính:
1. `[[universes]]`: id, runtime_mode, engine, audit, trace, policy_profile_id, optional permission overrides.
2. `[[domains]]`: id, universe, per-domain caps.
3. `[[bridges]]`: from/to/rule/keys/max_per_tick.
4. `[hive]`: overmind/cerebrate/swarm cap defaults.
5. `[[views]]`: view id -> domain -> module binding.
6. Preset defaults khi scaffold:
   - `default`: 1 universe + 1 domain + shadow off.
   - `ci`: deterministic + dual + audit on.
   - `prod`: throughput + bytecode + audit minimal.
7. Foundation sections (optional, additive):
   - `[foundation]`:
     - `preset_id` (optional): `workflow_basic | agent_swarm_basic | ...`
     - `beginner_defaults` (bool, default `true` ở unlocked)
     - `kit_profile_id` (optional)
   - `[[kits]]`:
     - `kit_id`
     - `version_req`
     - `bind_domain`
     - `bind_view` (optional)
     - `required_organs` (optional)
     - `caps_overrides` (optional)
8. Ràng buộc:
   - `[[kits]]` chỉ là wiring/config; code kit là package bình thường trong deps/catalog lock.
   - `required_organs` là khai báo yêu cầu; pin thực thi vẫn thuộc `organs.lock.v1`.

### 2.2 `cosmos.lock.v1` (generated)
1. Canonical LF, stable order, signed trong locked mode.
2. Hash256 inputs:
   - canonical bytes `cosmos.toml`
   - `deps.lock.v2`, `catalog.lock.v2`, `policy.lock.v1`
3. Chứa sorted canonical universe/domain/bridge/view/hive sections.
4. Chứa canonical section cho Foundation/Kits:
   - `[foundation]` sorted keys
   - `[[kits]]` sorted theo `(kit_id, bind_domain, bind_view)`
5. Lock-order giữ nguyên (1.3): deps/catalog/policy trước, sau đó mới `cosmos lock sync`.

### 2.3 `organs.lock.v1` (generated)
1. Pin exact organ artifacts theo platform.
2. Reuse trust model W4/W6.
3. Locked mode fail-hard nếu project dùng organ mà thiếu `organs.lock.v1`.

---

## 3) Runtime model v0.5

### 3.1 Context propagation
Ctx additive:
1. `universe_id`
2. `domain_id`
3. `view_id` (optional)
4. `shadow_id` (optional)

Trace/audit record additive:
1. `universe_id`
2. `domain_id`
3. `view_id`
4. `shadow_id`
5. hash-only fields cho args/payload/cert/event.

### 3.2 Domain isolation
1. Event/actor/task phải gắn domain id.
2. State store phân vùng theo domain.
3. Budget counters phân vùng theo domain.
4. Cross-domain chỉ qua bridge keys.

### 3.3 Bridge behavior
1. `emit` là commit-gated.
2. `poll` là observe.
3. Bridge policy pin trong `cosmos.lock.v1`.
4. Quota counters reset theo tick.

### 3.4 Shadow behavior
1. Default `forbid_commit` chặn commit trước adapter/plugin.
2. `shadow_commit_log` không apply world effects.
3. Shadow digest compare:
   - outcome digest
   - commit digest (hoặc empty)
   - audit chain head
   - trace required digest.

### 3.5 Hive behavior
1. Overmind không route semantically theo payload.
2. Cerebrate chỉ quản lý worker/mailbox domain-local.
3. Swarm workers fail-honest khi vượt cap:
   - `SwarmCapExceeded`
   - `FanoutExceeded`
   - `MailboxBackpressure`.

---

## 4) CLI surface v0.5 (locked)

Lệnh mới:
1. `ocp cosmos init <project> [--preset default|ci|prod] [--json]`
2. `ocp cosmos lock sync <project> [--locked|--unlocked] [--json]`
3. `ocp run ... --universe <id> [--domain <id>]`
4. `ocp run ... --shadow <id> [--shadow-policy forbid_commit|shadow_commit_log]`
5. `ocp organ install <name> <version> [--registry ...] [--json]`
6. `ocp organ verify <project> [--locked|--unlocked] [--json]`
7. `ocp test --conformance --locked [--manifest <path>] [--universe <id>] ...`
8. `ocp init --preset <preset_id> [--locked|--unlocked]`
9. `ocp kit list [--json]`
10. `ocp kit doctor <project> [--json]`

Lưu ý scope Foundation:
1. `init/kit` chỉ làm scaffolding + validation, không thay đổi runtime semantics.
2. `ocp init --preset ...` trong locked mode phải chạy lock sync chain (`deps/catalog/policy/cosmos`, và `organs` nếu required).
3. `ocp kit doctor` locked mode fail-hard nếu thiếu lock bắt buộc cho wiring kits/organs.

Locked requirements:
1. `deps.lock.v2`
2. `catalog.lock.v2`
3. `policy.lock.v1`
4. `cosmos.lock.v1`
5. `plugins.lock.v1` nếu dùng plugin
6. `organs.lock.v1` nếu dùng organ packs

---

## 5) Workstreams v0.5

### V5-W0 Rebaseline v0.4 green
1. Confirm W9 conformance v0.4 green.
2. Freeze topology trước v0.5 coding.
3. Hardened Rev2 locks:
   - signed-strict preflight + verify-supply positive/negative evidence.
   - deterministic conformance double-run digest equality.
   - clean policy hai mức (`target/ocp/*` scoped default, `-FullClean` strict).
   - env pin portable + deterministic digest from report JSON.
   - topology freeze guard with before/after hash snapshots.
   - Windows plugin sub-gate: build signed artifact + verify-supply + deterministic run.
4. Exit: baseline report + no open regression.

### V5-W1 Cosmos config + lock/sign + universe wiring
1. Add parser/writer `cosmos.toml` + `cosmos.lock.v1`.
2. Add `ocp cosmos init --preset ...` + `ocp cosmos lock sync`.
3. Wire `--universe` vào check/run/test/build/compose/verify.
4. Enforce universe là overlay config-only (không runtime fork).
5. Exit: locked path fail nếu thiếu cosmos lock.

### V5-W2 Domain isolation + bridges
1. Domain-partition scheduler/mailbox/state/budget.
2. Implement `domain.bridge.emit/poll`.
3. Enforce bridge rules + dual-point quotas.
4. Exit criteria:
   - cross-domain default deny pass.
   - không có đường code scan backlog trong deterministic lane.
   - dispatch order stable với cùng input (`tick/bridge_id/bridge_seq`).
   - queue full/backpressure fail-honest và có audit/trace counters.

### V5-W3 Shadow execution
1. Add `--shadow` + `--shadow-policy`.
2. Implement forbid_commit default (host-side hard stop).
3. Implement shadow digest compare + fail codes.
4. Exit criteria:
   - no side-effect leakage trong `forbid_commit`.
   - main/shadow cùng inputs -> digest match ổn định.
   - nondeterministic source -> `V-SHADOW-UNSUPPORTED` (không spam mismatch).
   - mismatch chỉ phát khi transcript contract hợp lệ nhưng khác digest.

### V5-W4 Hive runtime supervisors
1. Implement Overmind/Cerebrates/swarm bounded.
2. Add deterministic scheduler ordering key lock.
3. Add counters/report for spawn/fanout/backpressure.
4. Add worker/memory reuse path:
   - worker pool/slab reuse
   - mailbox buffer recycle
   - tick-arena cho ephemeral allocations
5. Exit criteria:
   - deterministic lane stable across reruns.
   - soak N ticks pass với counters cho thấy growth bounded/plateau.
   - reuse không làm đổi deterministic ordering.

### Implementation guidance W2-W4 (deterministic-first)
1. W2 bridge quota path:
   - ưu tiên `VecDeque`/ring-buffer prealloc per-bridge.
   - quota reset bằng `(tick_epoch, count)` thay vì clear map.
2. W3 shadow digest path:
   - build transcript canonical host-side (UTF-8, LF-only, stable field order, hash-only payload).
   - đặt nondet sentinel tại adapter/plugin boundary để classify UNSUPPORTED sớm.
3. W4 hive allocator path:
   - worker pool/slab ID-based, không dựa pointer order.
   - tick-arena cho temporary buffers, reset mỗi tick.

### V5-W5 Parallel views contracts
1. Add `std.view.render_tree` + `std.view.render_text`.
2. Wire view modules by cosmos config.
3. Keep view path pure observe/no direct IO.
4. Exit: same truth + different view_id produce expected distinct view outputs.
5. Deliverable Foundation view kit:
   - `std.kit.view_text_basic` dùng `std.view.render_text`.
   - đảm bảo pure observe + hash-only trace khi bật trace.
6. Test wiring `[[kits]]` tạo output khác nhau theo `view_id` trên cùng truth.

### V5-W6 Organ packs
1. Add `organs.lock.v1`.
2. Add `ocp organ install` + `ocp organ verify`.
3. Reuse thống nhất registry/trust/signature/verify từ W4/W6 (không tạo pipeline song song).
4. Exit: locked mode blocks untrusted/unsigned organ usage.
5. Ship 2 official presets:
   - `workflow_basic`: domain default, 1-2 tasks mẫu, bounded fanout/spawn.
   - `agent_swarm_basic`: hive bounded, mailbox limits, deterministic ordering.
6. Presets bắt buộc khai báo:
   - budgets mặc định đủ chặt.
   - IAM/policy mẫu (default deny rõ ràng).
   - nếu cần IO thì chọn organ packs đã pin.
7. Deliver thêm CLI flow:
   - `ocp init --preset ...`
   - `ocp kit doctor ...`

### V5-W7 Conformance v0.5 SoT
1. Extend conformance scenarios for universe/domain/shadow/hive/views/organ.
2. Keep deterministic lane as blocking SoT.
3. Throughput lane smoke-only, non-blocking.
4. Exit: report always written + required digest stable.
5. Bổ sung scenarios Foundation:
   - `v5_foundation_workflow_basic_locked_pass`
   - `v5_foundation_agent_swarm_basic_locked_pass`
   - `v5_foundation_missing_organs_lock_fail_hard_when_required`
   - `v5_foundation_kit_wiring_drift_detected_by_cosmos_lock`

---

## 6) Mandatory tests v0.5 (hard subset)

### Cosmos + universe
1. `v5_cosmos_lock_required_in_locked`
2. `v5_universe_profile_switch_changes_runtime_mode_but_preserves_invariants`

### Domain isolation + bridge
1. `v5_domain_cross_access_denied_by_default`
2. `v5_domain_bridge_allow_with_cap_and_quota`
3. `v5_domain_bridge_quota_exceeded_fail_honest`
4. `v5_cross_domain_state_leak_denied_without_bridge`
5. `v5_w2_bridge_quota_admission_o1_no_scan`
6. `v5_w2_bridge_dispatch_bounded_max_per_tick`
7. `v5_w2_bridge_queue_capacity_backpressure_fail_honest`
8. `v5_w2_bridge_dispatch_order_stable`

### Shadow
1. `v5_shadow_forbid_commit_blocks_effects`
2. `v5_shadow_digest_matches_when_inputs_equal`
3. `v5_shadow_mismatch_detected`
4. `v5_shadow_forbid_commit_blocks_plugin_and_organ_effects`
5. `v5_w3_shadow_digest_contract_excludes_wall_clock`
6. `v5_w3_shadow_rng_seeded_stable`
7. `v5_w3_shadow_unsupported_on_nondet_source`
8. `v5_w3_shadow_mismatch_only_when_contract_valid`

### Hive
1. `v5_swarm_spawn_bounded`
2. `v5_swarm_fanout_bounded`
3. `v5_hive_deterministic_order_stable`
4. `v5_w4_worker_pool_reuse_hit_rate_nonzero`
5. `v5_w4_mailbox_buffer_reuse_no_unbounded_growth`
6. `v5_w4_soak_n_ticks_memory_plateau_or_bounded`
7. `v5_w4_deterministic_order_unchanged_with_reuse`

### Views
1. `v5_view_render_tree_hash_only_trace_no_payload`
2. `v5_view_different_view_id_different_tree_same_truth`

### Organ packs
1. `v5_organ_install_verify_signature`
2. `v5_locked_requires_organs_lock_when_organs_used`

### Conformance
1. `v5_conformance_locked_deterministic_pass`
2. `v5_conformance_report_always_written_on_fail`
3. `v5_required_digest_order_sensitive`
4. `v5_replay_cross_universe_fail_hard`

### Foundation
1. `v5_foundation_workflow_basic_runs_unlocked_beginner_defaults`
2. `v5_foundation_workflow_basic_locked_requires_cosmos_and_catalog_and_deps`
3. `v5_foundation_agent_swarm_basic_bounded_and_deterministic`
4. `v5_foundation_kit_doctor_detects_missing_required_organs_lock`

---

## 7) Validation matrix v0.5

### Blocking
1. `cargo check --workspace`
2. `cargo test -p ocp-runtime-core -p ocp-sdk -p ocp-cli`
3. `cargo clippy -p ocp-runtime-core -p ocp-sdk -p ocp-cli --all-targets -- -D warnings`
4. `cargo fmt -- --check`
5. `cargo run -p ocp-cli -- cosmos lock sync <project> --locked --json`
6. `cargo run -p ocp-cli -- test --conformance --locked --runtime deterministic --engine dual --manifest projects/ocp/conformance/conformance.v1.toml --out target/ocp/w9/reports/conformance_report.json --trust-store projects/ocp/security/trust.store.toml --signer-id dev-root-1 --sign-key projects/ocp/security/dev-root-1.signing.key.toml --json`
7. `cargo run -p ocp-cli -- test --conformance --locked --runtime deterministic --engine dual --manifest projects/ocp/conformance/conformance.v5.toml --out target/ocp/w9/reports/conformance_report.v5.json --trust-store projects/ocp/security/trust.store.toml --signer-id dev-root-1 --sign-key projects/ocp/security/dev-root-1.signing.key.toml --json`
8. `powershell -ExecutionPolicy Bypass -File tools/ci_ocp_lane.ps1`
9. `cargo test -p ocp-sdk --test v5_w2_domain_resolution`
10. `cargo test -p ocp-sdk --test v5_w3_shadow`
11. `cargo test -p ocp-sdk --test v5_w4_hive`

### Quarantine (non-blocking)
1. `cargo run -p ocp-cli -- test --conformance --locked --runtime throughput --engine bytecode --manifest projects/ocp/conformance/conformance.v1.toml --out target/ocp/w9/reports/conformance_quarantine.json --trust-store projects/ocp/security/trust.store.toml --signer-id dev-root-1 --sign-key projects/ocp/security/dev-root-1.signing.key.toml --json`
2. `powershell -ExecutionPolicy Bypass -File tools/ci_ocp_quarantine.ps1`

---

## 8) Acceptance criteria v0.5

v0.5 `DONE` khi đồng thời đạt:
1. Universe profiles chạy được qua `--universe`, locked mode enforce signed `cosmos.lock.v1`.
2. Domain isolation + bridge explicit hoạt động đúng, quota bounded, fail-honest.
3. Bridge quotas deterministic lane đạt O(1) admission/dispatch bounded, không scan backlog.
4. Shadow execution chạy đúng `forbid_commit` default và digest compare ổn định.
5. Shadow nondeterminism được phân loại đúng (`V-SHADOW-UNSUPPORTED`) trước mismatch.
6. Hive runtime bounded và deterministic lane không flake.
7. Hive reuse + soak counters chứng minh growth bounded/plateau, không đổi scheduling order.
8. View contracts hoạt động mà không phá `No naked IO`.
9. Organ packs có install/verify/lock theo signed supply-chain.
10. Conformance SoT v0.5 phải pass cả 2 lane:
   - lane kế thừa v0.4 (manifest v1, locked deterministic).
   - lane nâng cấp Foundation v0.5 (manifest v5, locked deterministic).
   - `--universe ci_locked` áp cho cosmos-ready lane/manifest, không ép cứng lên toàn bộ legacy scenarios.
11. W0-W9 regression v0.4 vẫn green.
12. Có tối thiểu 2 official presets (`workflow_basic`, `agent_swarm_basic` hoặc tương đương) chạy được:
   - unlocked beginner defaults (smoke).
   - locked mode (SoT deterministic pass) với locks đầy đủ.
13. Tất cả workstream `V5-W0..V5-W7` đều ở trạng thái `DONE`, không còn `PLANNED/IN PROGRESS/PENDING`.
14. Không còn technical debt mở ảnh hưởng release:
   - không còn `BLOCK` mở trong gate log,
   - không còn TODO “fix later” trong phạm vi v0.5,
   - không còn rủi ro correctness/security/determinism chưa xử lý.

---

## 9) Non-goals v0.5 (de-scope)
1. Không mở full HTTP parser/server stack mới.
2. Không mở grammar spawn syntax mới trong W5 (swarm jobs runtime-managed).
3. Không đóng gói plugin binaries vào `.ocppkg` ở v0.5.
4. Không làm GUI/window/render engine full trong core.
5. Foundation không tạo “ngôn ngữ thứ hai”: không thêm semantics effect/state/IO mới ngoài Kernel.

---

## 10) Gate log v0.5

### V5-W0 Gate Log
- Status: `DONE` (2026-03-03)
- Scope lock:
  - Chỉ rebaseline v0.4 + hardened freeze, không mở feature v0.5 mới.
  - Signed-strict blocking, deterministic double-run, topology freeze hash snapshot.
  - Verify-supply có positive/negative trust semantics.
  - Report always-written được chứng minh bằng fail-intentional run.
- Ghi chú:
  - Chi tiết triển khai/evidence chuẩn được giữ ở 2 entry ngay dưới:
    - `V5-W0 Planning Freeze`
    - `V5-W0 Implementation Closeout`

#### V5-W0 Planning Freeze (2026-03-03)
- Date:
  - 2026-03-03
- Gate/Step:
  - V5-W0
- Why:
  - Đóng lại W0 bằng run thực tế trên baseline hiện tại, bảo đảm checklist W0 chạy end-to-end thay vì chỉ giữ log cũ.
- Scope:
  - Bổ sung script freeze topology (`before/after + drift`).
  - Nâng `ci_w0_entry.ps1` theo flow W0: signed preflight, deterministic double-run, fail-intentional report, verify-supply dương/âm, plugin sub-gate, status report.
  - Chạy full W0 với signer/trust thật trong workspace.
- Expected tests:
  - `powershell -ExecutionPolicy Bypass -File tools/ci_w0_entry.ps1 -FullClean -SignerId dev-root-1 -SignKey projects/ocp/security/dev-root-1.signing.key.toml -TrustStore projects/ocp/security/trust.store.toml`
  - `powershell -ExecutionPolicy Bypass -File tools/guard_v5_topology_freeze.ps1 -Phase Before`
  - `powershell -ExecutionPolicy Bypass -File tools/guard_v5_topology_freeze.ps1 -Phase After`
- Exit criteria:
  - W0 entry pass đầy đủ, có report artifacts trong `target/ocp/v5/w0/reports/*`, digest deterministic trùng nhau, topology freeze `pass` và `drift=[]`.

#### V5-W0 Implementation Closeout (2026-03-03)
- Date:
  - 2026-03-03
- Gate/Step:
  - V5-W0
- Implemented:
  - Thêm `tools/guard_v5_topology_freeze.ps1`:
    - snapshot `before/after`,
    - hash theo nhóm (`workspace_members`, `conformance_scenario_order`, `plugin_platform_pins`, `lane_pins`, `overall`),
    - fail-hard khi có drift.
  - Nâng `tools/ci_w0_entry.ps1`:
    - thêm tham số `-FullClean`, `-SignerId`, `-SignKey`, `-TrustStore`,
    - thêm deterministic env pins,
    - signed preflight (file tồn tại + signer id),
    - deterministic conformance run1/run2 + so digest,
    - fail-intentional conformance với report bắt buộc tồn tại,
    - verify-supply positive + tampered negative,
    - plugin sub-gate (`plugin lock sync` -> `plugin verify` -> build/verify-supply/run),
    - ghi `w0_status.json`.
- Files changed:
  - `tools/ci_w0_entry.ps1`
  - `tools/guard_v5_topology_freeze.ps1`
  - `OCP-MVP-PLAN-v0.5.md`
- Commands run:
  - `powershell -ExecutionPolicy Bypass -File tools/guard_v5_topology_freeze.ps1 -Phase Before`
  - `powershell -ExecutionPolicy Bypass -File tools/guard_v5_topology_freeze.ps1 -Phase After`
  - `powershell -ExecutionPolicy Bypass -File tools/ci_w0_entry.ps1 -FullClean -SignerId dev-root-1 -SignKey projects/ocp/security/dev-root-1.signing.key.toml -TrustStore projects/ocp/security/trust.store.toml` (chạy lại nhiều lần đến khi pass)
- Test results:
  - PASS:
    - full `ci_w0_entry.ps1` pass end-to-end.
    - deterministic digest: run1=`10deff71c24ef729`, run2=`10deff71c24ef729`.
    - topology freeze: `status=pass`, `drift=[]`.
    - plugin sub-gate pass.
    - verify-supply positive pass, tampered negative fail đúng kỳ vọng.
  - FAIL tạm thời đã xử lý:
    - lỗi parse PowerShell trong `ci_w0_entry.ps1` do chuỗi `$Label:` không hợp lệ.
    - fail-intentional ban đầu fail sai loại (manifest/BOM), đã đổi sang manifest hợp lệ ASCII và reset `LASTEXITCODE` cho expected-fail steps.
- Notes/risks:
  - `ci_w0_entry.ps1` hiện chạy `ci_ocp_lane` trong cùng pipeline W0 nên thời gian chạy dài; đổi lại đảm bảo coverage rộng.
  - Negative verify-supply hiện dùng tamper artifact (không dùng trust-store mismatch) vì command `verify-supply` hiện không nhận trust-store input.

### V5-W1 Gate Log
- Status: `DONE` (2026-03-03)
- Scope lock:
  - Ship thật `policy.lock.v1` + `cosmos.lock.v1`.
  - `--universe` đi xuyên suốt CLI (`check/run/test/build/compose/verify` + `test --conformance`).
  - Locked mode + cosmos bắt buộc `--universe` (`V-UNIVERSE-REQUIRED`).
  - Universe chỉ là config overlay (runtime/engine/policy profile), không tạo runtime fork mới.
  - Mismatch locked fail-hard cho `runtime_mode`, `engine`, `policy_profile_id`.
- Ghi chú:
  - Chi tiết triển khai/evidence chuẩn được giữ ở 2 entry ngay dưới:
    - `V5-W1 Planning Freeze`
    - `V5-W1 Implementation Closeout`

#### V5-W1 Planning Freeze (2026-03-03)
- Date:
  - 2026-03-03
- Gate/Step:
  - V5-W1
- Why:
  - Đồng bộ code thực tế với thiết kế W1: có file lock/preset/universe routing thật, tránh trạng thái “log có nhưng code chưa có”.
- Scope:
  - Thêm module W1 trong SDK (`policy lock`, `cosmos init`, `cosmos lock`, `resolve universe`, mismatch guard).
  - Thêm command CLI: `policy lock sync`, `cosmos init`, `cosmos lock sync`.
  - Wire `--universe` vào các command runtime/build/test/compose/verify + conformance.
  - Bổ sung test W1 mới cho SDK và CLI.
- Expected tests:
  - `cargo test -p ocp-sdk --test v5_w1_cosmos`
  - `cargo test -p ocp-cli`
  - `cargo test -p ocp-sdk --test w9_conformance`
  - `cargo clippy -p ocp-sdk -p ocp-cli --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
- Exit criteria:
  - Command chain W1 chạy được end-to-end.
  - Locked cosmos enforce universe/mismatch đúng hành vi thiết kế.
  - Test/format/lint pass sạch trên scope W1.

#### V5-W1 Implementation Closeout (Sentinel completion, 2026-03-03)
- Date:
  - 2026-03-03
- Gate/Step:
  - V5-W1
- Implemented:
  - Mở rộng `TraceEventV1` với `universe_id` và `domain_id` để trace luôn có context sentinel không rỗng.
  - Gắn sentinel mặc định `__legacy__` (universe) và `default` (domain) trong trace mapping runtime path.
  - Giữ tương thích ngược trace file cũ: `decode_trace_line` nhận cả row 11 cột (legacy) và 13 cột (mới).
  - Cập nhật render trace text/json để hiển thị đầy đủ `universe_id/domain_id`.
  - Bổ sung assert trong CLI trace test để khóa contract sentinel không rỗng.
- Files changed:
  - `projects/ocp/crates/ocp-sdk/src/lib.rs`
  - `projects/ocp/crates/ocp-cli/src/main.rs`
  - `OCP-MVP-PLAN-v0.5.md`
- Commands run:
  - `cargo test -p ocp-sdk --test v5_w1_cosmos`
  - `cargo test -p ocp-cli`
  - `cargo test -p ocp-sdk --test w9_conformance`
  - `cargo clippy -p ocp-sdk -p ocp-cli --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
- Test results:
  - PASS:
    - `v5_w1_cosmos`: 3/3 pass.
    - `ocp-cli`: 17/17 pass.
    - `w9_conformance`: 2/2 pass.
    - `clippy` scope W1 pass (`ocp-sdk`, `ocp-cli`).
    - `fmt --check` pass.
- Notes/risks:
  - W1 đã hoàn tất đúng thiết kế hiện tại và có evidence test/lint/format.

### V5-W2 Gate Log
- Status: `DONE` (2026-03-03)
- Scope lock:
  - Domain selection không còn fallback "domain đầu tiên"; non-reactor buộc `default` hoặc fail `V-DOMAIN-REQUIRED`.
  - `domain.bridge.emit` quota/capacity enforcement ở commit path (observe chỉ validate, không TOCTOU).
  - Poll fairness deterministic round-robin theo `start = tick % bridges_count`.
  - Bridge state per-bridge với epoch reset O(1): `tick_epoch + admit_count + dispatch_count + seq + queue`.
  - Sentinel non-empty bắt buộc cho context/audit/trace: `__legacy__/default`.
  - Locked cosmos lock sync fail-hard nếu bridge thiếu `bridge_queue_capacity`.

#### V5-W2 Planning Freeze (2026-03-03)
- Date:
  - 2026-03-03
- Gate/Step:
  - V5-W2
- Why:
  - Kéo W2 từ trạng thái log-only về code thực chạy được: domain selection không fallback domain đầu tiên, bridge planner có quota/capacity/dispatch-start deterministic.
- Scope:
  - Thêm module W2 trong SDK (`w2.rs`) cho domain + bridge runtime planning.
  - Nối `--domain` vào CLI (`run/test/trace run/profile run`) với fail-honest theo domain/universe lock.
  - Bổ sung test SDK/CLI cho W2.
- Expected tests:
  - `cargo test -p ocp-sdk --test v5_w2_domain_resolution`
  - `cargo test -p ocp-cli`
  - `cargo clippy -p ocp-sdk -p ocp-cli --all-targets -- -D warnings`
  - `cargo fmt --all -- --check`
- Exit criteria:
  - Domain selection enforce `default` hoặc yêu cầu explicit `--domain` (không fallback domain đầu tiên).
  - Bridge plan có quota/capacity + dispatch start index deterministic.
  - Locked cosmos lock sync fail-hard nếu bridge thiếu `bridge_queue_capacity`.

#### V5-W2 Implementation Closeout (Runtime Scheduler + Legacy Reactor Fix, 2026-03-03)
- Date:
  - 2026-03-03
- Gate/Step:
  - V5-W2
- Implemented:
  - Bổ sung scheduler domain-aware trong reactor runtime: chia tick theo domain và ghi telemetry bridge (`emit`, `dispatch`, `backpressure`).
  - Mở rộng `ReactorServiceOptions` và `ReactorServiceReport` để mang context universe/domain và bridge counters.
  - Vá đường chạy legacy ở W9/CLI: legacy sentinel (`__legacy__`) không còn bị forward như cờ `--universe/--domain` thật.
- Files changed:
  - `projects/ocp/crates/ocp-sdk/src/lib.rs`
  - `projects/ocp/crates/ocp-sdk/src/w9.rs`
  - `projects/ocp/crates/ocp-cli/src/main.rs`
  - `projects/ocp/crates/ocp-sdk/tests/v5_w2_domain_resolution.rs`
  - `projects/ocp/crates/ocp-sdk/tests/w1_runtime.rs`
  - `projects/ocp/crates/ocp-sdk/tests/w2_audit.rs`
  - `projects/ocp/crates/ocp-sdk/tests/w9_conformance.rs`
  - `OCP-MVP-PLAN-v0.5.md`
- Commands run:
  - `cargo test -p ocp-sdk --test v5_w2_domain_resolution`
  - `cargo test -p ocp-sdk --test w1_runtime`
  - `cargo test -p ocp-sdk --test w2_audit`
  - `cargo test -p ocp-sdk --test w9_conformance`
  - `cargo test -p ocp-cli`
  - `cargo clippy -p ocp-sdk -p ocp-cli --all-targets -- -D warnings`
  - `cargo fmt --all`
  - `cargo fmt --all -- --check`
- Test results:
  - PASS:
    - `v5_w2_domain_resolution`: 6/6 pass.
    - `w1_runtime`: 2/2 pass.
    - `w2_audit`: 2/2 pass.
    - `w9_conformance`: 2/2 pass.
    - `ocp-cli`: 19/19 pass.
    - `clippy` scope W2 pass.
    - `fmt --check` pass.
- Notes/risks:
  - Đã tái hiện và xử lý lỗi fail 3 scenario reactor trong W9 (`V-UNIVERSE-NO-COSMOS` ở legacy).
  - `cargo run -p ocp-cli -- test --conformance ... --locked` trả exit code 10 khi có scenario fail là hành vi fail-hard đúng thiết kế W9.

### V5-W3 Gate Log
- Status: `DONE` (2026-03-03)
- Scope lock:
  - Shadow chỉ cho deterministic lane; throughput + shadow trả `V-SHADOW-UNSUPPORTED`.
  - Reactor shadow replay-only: main capture IO tape canonical, main/shadow compare cùng replay tape.
  - Digest projection bytes exact và order-sensitive; compare fail-hard `V-SHADOW-MISMATCH`.
  - `commit_intent_hash256` canonical theo `effect_key|args_hash256|target_hash256|kind|seq`.
  - `initial_sandbox_hash256` non-reactor dùng zero-hash constant.
  - Transcript/report naming không overwrite (`...main.r{n}.jsonl` / `...shadow.r{n}.jsonl`).

#### V5-W3 Planning Freeze (rebaseline current workspace, 2026-03-03)
- Date:
  - 2026-03-03
- Gate/Step:
  - V5-W3
- Why:
  - Workspace hiện tại chỉ có `ocp-runtime-core`, `ocp-sdk`, `ocp-cli`; không có crate `ocp-runtime-rt`.
  - Cần triển khai W3 theo kiến trúc thực tế: shadow compare ở SDK/CLI, không tạo fork runtime mới.
- Scope:
  - Thêm module W3 trong SDK:
    - `ShadowPolicyV1`, `ShadowOptionsV1`, `InputEnvelopeV1`.
    - transcript canonical + digest compare.
    - wrapper run/project/reactor với artifact writer.
  - Nối CLI cờ:
    - `--shadow <id>`
    - `--shadow-policy forbid_commit|shadow_commit_log`
  - Wire cho các lệnh:
    - `run`, `test`, `trace run`, `profile run`.
  - Exit code mapping:
    - `V-SHADOW-MISMATCH -> 5`
    - `V-SHADOW-UNSUPPORTED -> 13`
- Expected tests:
  - `cargo test -p ocp-sdk --test v5_w3_shadow`
  - `cargo test -p ocp-cli v5_w3_cli`
  - `cargo fmt --all -- --check`
- Exit criteria:
  - Shadow deterministic compare chạy được trong SDK/CLI.
  - Có test pass cho forbid_commit, digest match, mismatch/unsupported.
  - Không regression lane `ocp-sdk` + `ocp-cli`.

#### V5-W3 Implementation Closeout (runtime hard-stop + IO tape, 2026-03-03)
- Date:
  - 2026-03-03
- Gate/Step:
  - V5-W3
- Implemented:
  - Đóng hard-stop `forbid_commit` ở runtime executor path:
    - thêm `CommitPolicyMode::{Normal,ForbidCommit,ShadowCommitLog}` vào `ExecConfig`.
    - `exec_commit` chặn commit theo policy trước effect apply path (`apply_pending_sqlite_write_if_needed`).
  - Nối runtime-core API theo config:
    - thêm `run_compiled_with_config`, `run_source_with_engine_config`, `run_file_with_engine_config`.
    - giữ API cũ làm wrapper backward-compatible với `CommitPolicyMode::Normal`.
  - Nâng W3 shadow lane dùng runtime policy thật:
    - main lane chạy `CommitPolicyMode::Normal`.
    - shadow lane map theo `--shadow-policy`:
      - `forbid_commit -> CommitPolicyMode::ForbidCommit`
      - `shadow_commit_log -> CommitPolicyMode::ShadowCommitLog`
  - Mở đầy đủ IO tape record/replay cho reactor lane (SDK):
    - thêm parser/writer `io_tape` trong `run_reactor_service_with_lock`.
    - thêm record/replay cho trace reactor runner để shadow compare lane tái lập input ổn định.
    - fail-honest nếu tape mismatch hoặc còn unconsumed entries.
  - Sửa canonical transcript W3 cho `forbid_commit`:
    - mask `commit_result` ở transcript field (`decision_outcome`, `args_hash256`) để tránh mismatch giả do policy.
  - Bổ sung test W3:
    - `v5_shadow_forbid_commit_runtime_trace_denied`
    - `v5_w3_reactor_io_tape_record_replay_stable_digest`
- Files changed:
  - `src/ocp/budget.rs`
  - `src/ocp/exec.rs`
  - `src/ocp/mod.rs`
  - `projects/ocp/crates/ocp-runtime-core/src/lib.rs`
  - `projects/ocp/crates/ocp-runtime-core/src/vm.rs`
  - `projects/ocp/crates/ocp-sdk/src/lib.rs`
  - `projects/ocp/crates/ocp-sdk/src/w3.rs`
  - `projects/ocp/crates/ocp-sdk/tests/v5_w2_domain_resolution.rs`
  - `projects/ocp/crates/ocp-sdk/tests/v5_w3_shadow.rs`
  - `OCP-MVP-PLAN-v0.5.md`
- Commands run:
  - `cargo test -p ocp-sdk --test v5_w3_shadow`
  - `cargo test -p ocp-sdk`
  - `cargo test -p ocp-cli v5_w3_cli`
  - `cargo fmt --all`
  - `cargo fmt --all -- --check`
  - `cargo test -p ocp-sdk -p ocp-cli`
- Test results:
  - PASS:
    - `v5_w3_shadow`: 6/6 pass.
    - `v5_w3_cli`: 3/3 pass.
    - full lane `ocp-sdk + ocp-cli`: pass toàn bộ suite.
    - `fmt --check`: pass.
  - Ghi nhận trong quá trình triển khai:
    - có 1 lần fail trung gian `v5_shadow_forbid_commit_blocks_effects` do digest lệch sau khi bật runtime hard-stop; đã sửa canonical transcript và re-run PASS.
- Notes/risks:
  - Các blocker W3 đã đóng:
    - `forbid_commit` đã là runtime hard-stop.
    - reactor IO tape record/replay đã có path chạy thật.
  - Chưa mở scope crate/runtime lane mới ngoài workspace hiện tại; giữ đúng kiến trúc thực tế `ocp-runtime-core + ocp-sdk + ocp-cli`.

### V5-W4 Gate Log
- Status: `DONE` (2026-03-03)

#### 2026-03-03 - V5-W4 planning freeze
- Date:
  - 2026-03-03
- Gate/Step:
  - V5-W4
- Why:
  - Đóng W4 theo đúng scope “Hive supervisors + bounded swarm runtime”, và chốt các phần còn dở để không còn trạng thái `IN_PROGRESS`.
- Scope:
  - Hoàn tất wiring `hive_caps` từ cosmos/lock vào runtime reactor.
  - Khóa counters/report W4 (`workers_*`, mailbox depth, fanout/spawn drop, `dispatch_digest256`).
  - Bổ sung test W4 cho lock fail-hard + determinism digest.
  - Vá compile `ocp-cli` do schema `ReactorServiceOptions` thêm field `hive_caps`.
- Expected tests:
  - `cargo test -p ocp-sdk --test v5_w4_hive`
  - `cargo test -p ocp-sdk`
  - `cargo test -p ocp-cli`
- Exit criteria:
  - W4 pass lane SDK + CLI.
  - Có test W4 chuyên biệt, không chỉ pass compile.
  - Gate log cập nhật đủ planning freeze + implementation closeout.

#### 2026-03-03 - V5-W4 implementation closeout
- Date:
  - 2026-03-03
- Gate/Step:
  - V5-W4
- Implemented:
  - Hoàn tất mô hình hive trong cosmos config/lock:
    - `CosmosHiveV1` + parse/encode/validate.
    - hash canonical cosmos bao gồm `hive`.
    - verify lock fail-hard:
      - `V-HIVE-LOCK-MISSING`
      - `V-HIVE-LOCK-MISMATCH`
    - thêm helper `resolve_hive_caps_v1(...)`.
  - Hoàn tất runtime scheduler bounded trong reactor:
    - enforce `max_swarm_workers`, `max_fanout_per_task`, `mailbox_max_depth`, `max_spawn_per_tick`, `max_domains`.
    - thu thập counters và digest dispatch deterministic.
  - Mở rộng report runtime:
    - `workers_alive`
    - `workers_spawned_total`
    - `workers_reused_total`
    - `mailbox_max_depth_observed`
    - `alloc_events_total`
    - `fanout_drop_count`
    - `spawn_drop_count`
    - `dispatch_digest256`
  - Vá tương thích schema `ReactorServiceOptions` ở CLI/SDK test code (`hive_caps: None`).
  - Thêm test W4 mới:
    - determinism của `dispatch_digest256`
    - fail-hard khi thiếu dòng `hive=` trong `cosmos.lock.v1`
    - fail-hard khi `hive` trong lock lệch với `cosmos.toml`
- Files changed:
  - `projects/ocp/crates/ocp-sdk/src/w1.rs`
  - `projects/ocp/crates/ocp-sdk/src/lib.rs`
  - `projects/ocp/crates/ocp-sdk/src/w9.rs`
  - `projects/ocp/crates/ocp-sdk/tests/w1_runtime.rs`
  - `projects/ocp/crates/ocp-sdk/tests/w2_audit.rs`
  - `projects/ocp/crates/ocp-sdk/tests/v5_w2_domain_resolution.rs`
  - `projects/ocp/crates/ocp-sdk/tests/v5_w3_shadow.rs`
  - `projects/ocp/crates/ocp-sdk/tests/v5_w4_hive.rs`
  - `projects/ocp/crates/ocp-cli/src/main.rs`
  - `OCP-MVP-PLAN-v0.5.md`
- Commands run:
  - `cargo test -p ocp-sdk --test v5_w4_hive`
  - `cargo test -p ocp-sdk`
  - `cargo test -p ocp-cli`
- Test results:
  - PASS:
    - `v5_w4_hive`: 3/3 pass.
    - `ocp-sdk`: full package tests pass.
    - `ocp-cli`: 22/22 pass.
- Notes/risks:
  - W4 đóng theo scope hiện tại trong workspace thực tế (`ocp-sdk` + `ocp-cli`), không khai báo lane/crate ngoài phạm vi đã chạy.
  - Nếu cần hard-gate thêm cho W4 (clippy/fmt/lane script), mở ở W5/W7 để không chồng scope trong closeout này.

### V5-W5 Gate Log
- Status: `DONE` (2026-03-03)
- Scope lock:
  - Parallel views contract only: `std.view.render_tree` + `std.view.render_text`.
  - View path là pure observe, không direct IO và không mở grammar mới.
  - Preflight CLI theo cosmos wiring với mã lỗi `V-VIEW-*` fail-hard.
- Ghi chú:
  - Chi tiết triển khai/evidence chuẩn của W5 được giữ ở 2 entry ngay dưới:
    - `2026-03-03 - V5-W5 planning freeze`
    - `2026-03-03 - V5-W5 implementation closeout`

#### 2026-03-03 - V5-W5 planning freeze
- Date:
  - 2026-03-03
- Gate/Step:
  - V5-W5
- Why:
  - Chuyển W5 về đúng format full-log chuẩn của plan: tách rõ planning freeze và implementation closeout, không giữ block log kiểu cũ.
- Scope:
  - Bổ sung observe view keys `std.view.render_text`/`std.view.render_tree` theo hợp đồng pure observe.
  - Bổ sung resolver view theo `cosmos.toml` và wiring Foundation kit theo `[[kits]]`.
  - Mở rộng lock-chain `cosmos.lock.v1` để pin `view`/`kit`.
  - Wire CLI `--view` cho `run/trace run/profile run` với preflight fail-hard `V-VIEW-*`.
  - Bổ sung test SDK/CLI cho view selection và view wiring.
- Expected tests:
  - `cargo test -p ocp-sdk --test v5_w5_views`
  - `cargo test -p ocp-sdk --test v5_w1_cosmos`
  - `cargo test -p ocp-sdk`
  - `cargo test -p ocp-cli`
  - `cargo fmt --all -- --check`
- Exit criteria:
  - W5 pass đầy đủ test SDK/CLI cho contract views.
  - View path giữ pure observe và lock-chain pin `[[view]]`/`[[kits]]` vào `cosmos.lock.v1`.
  - Gate W5 giữ `DONE` với evidence đầy đủ.

#### 2026-03-03 - V5-W5 implementation closeout
- Date:
  - 2026-03-03
- Gate/Step:
  - V5-W5
- Implemented:
  - Runtime view keys:
    - `src/ocp/registry.rs` thêm `ctx_required` cho `std.view.render_text`/`std.view.render_tree`, và khóa `commit_allowed=false`.
    - `src/ocp/exec.rs` thêm deterministic stub cho 2 key view, render theo `truth + view_id`.
  - SDK W5:
    - thêm module `projects/ocp/crates/ocp-sdk/src/w5.rs` với resolver/select/wiring cho `view` và `kit`.
    - export W5 qua `projects/ocp/crates/ocp-sdk/src/lib.rs`.
  - Cosmos lock-chain:
    - `projects/ocp/crates/ocp-sdk/src/w1.rs` mở rộng parse/encode/hash `cosmos.lock.v1` với `view`/`kit`, locked verify fail-hard khi drift.
  - CLI:
    - `projects/ocp/crates/ocp-cli/src/main.rs` thêm `--view` cho `run/trace run/profile run`, preflight `resolve_view_selection_v1(...)`, và inject `OCP_VIEW_ID`.
  - Tests:
    - thêm `projects/ocp/crates/ocp-sdk/tests/v5_w5_views.rs`.
    - bổ sung unit tests CLI W5 trong `projects/ocp/crates/ocp-cli/src/main.rs`.
- Files changed:
  - `src/ocp/registry.rs`
  - `src/ocp/exec.rs`
  - `projects/ocp/crates/ocp-sdk/src/w5.rs`
  - `projects/ocp/crates/ocp-sdk/src/lib.rs`
  - `projects/ocp/crates/ocp-sdk/src/w1.rs`
  - `projects/ocp/crates/ocp-sdk/tests/v5_w5_views.rs`
  - `projects/ocp/crates/ocp-cli/src/main.rs`
  - `OCP-MVP-PLAN-v0.5.md`
- Commands run:
  - `cargo test -p ocp-sdk --test v5_w5_views`
  - `cargo test -p ocp-sdk --test v5_w1_cosmos`
  - `cargo test -p ocp-sdk`
  - `cargo test -p ocp-cli`
  - `cargo fmt --all -- --check`
- Test results:
  - PASS:
    - `v5_w5_views`: 5/5.
    - `v5_w1_cosmos`: 3/3.
    - full `ocp-sdk`: pass.
    - full `ocp-cli`: 25/25.
    - `fmt --check`: pass.
- Notes/risks:
  - W5 đã đóng theo scope song song views contract + Foundation view kit wiring.
  - Lock-chain view/kit đã pin vào `cosmos.lock.v1`.

### V5-W6 Gate Log
- Status: `DONE` (2026-03-03)
- Scope lock:
  - Trigger "organs used" chỉ lấy từ `[[kits]].required_organs` đã pin trong `cosmos.lock.v1`
  - Locked enforce `organs.lock.v1`; unlocked warn-only
  - Canonical sign message bao phủ `artifact_rel` + `provides_caps` + `source`
  - Platform lock rõ cho locked/unlocked (`V-ORGAN-PLATFORM-REQUIRED`, `V-ORGAN-PLATFORM-MISMATCH`)
  - Release-grade hardening chỉ bật qua `OCP_RELEASE_GRADE=1`
- Ghi chú:
  - Chi tiết triển khai/evidence chuẩn của W6 được giữ ở 2 entry ngay dưới:
    - `2026-03-03 - V5-W6 planning freeze (completion lane)`
    - `2026-03-03 - V5-W6 implementation closeout (organ packs + preset init)`

#### 2026-03-03 - V5-W6 planning freeze (completion lane)
- Date:
  - 2026-03-03
- Gate/Step:
  - V5-W6
- Why:
  - Chốt W6 theo trạng thái code thực tế: có organ API và CLI command surface nhưng thiếu lane test chuyên biệt W6 + thiếu preset flow `ocp init --preset ...`.
- Scope:
  - Thêm test SDK riêng cho organ packs (`v5_w6_organs`).
  - Thêm test CLI cho organ flow và init preset W6.
  - Hoàn thiện `ocp init --preset workflow_basic|agent_swarm_basic` theo chain lock hiện có trong workspace.
  - Cập nhật help text CLI cho nhóm lệnh organ/kit.
- Expected tests:
  - `cargo fmt --all -- --check`
  - `cargo test -p ocp-sdk --test v5_w6_organs`
  - `cargo test -p ocp-cli w6`
  - `cargo test -p ocp-sdk -p ocp-cli`
  - `cargo clippy -p ocp-sdk -p ocp-cli --all-targets -- -D warnings`
- Exit criteria:
  - W6 có test lane riêng PASS cho organ lock/verify/install/doctor + preset init.
  - CLI help và command usage không lệch thực tế.
  - Trạng thái W6 chuyển `DONE`.

#### 2026-03-03 - V5-W6 implementation closeout (organ packs + preset init)
- Date:
  - 2026-03-03
- Gate/Step:
  - V5-W6
- Implemented:
  - Thêm suite SDK W6:
    - `v5_w6_organ_lock_verify_install_and_doctor_pass`
    - `v5_w6_verify_fails_when_signature_tampered`
    - `v5_w6_locked_requires_organs_lock_when_required`
    - `v5_w6_kit_doctor_detects_missing_required_organs_lock`
  - Hoàn thiện CLI `init` cho W6:
    - hỗ trợ `--preset workflow_basic|agent_swarm_basic`.
    - preset flow tự chạy chain lock hiện có: `deps -> policy -> cosmos` và `organs` nếu kit yêu cầu.
    - hỗ trợ cờ `--locked`, `--registry`, `--signer-id`, `--sign-key`, `--trust-store`, `--json`.
  - Cập nhật help text:
    - thêm usage cho `organ lock sync`, `organ verify`, `organ install`, `kit list`, `kit doctor`.
    - cập nhật usage `init` có preset W6.
  - Thêm CLI tests W6:
    - `v5_w6_cli_organ_lock_verify_install_and_kit_doctor_pass`
    - `v5_w6_cli_locked_missing_organs_lock_fail_hard`
    - `v5_w6_cli_init_preset_workflow_basic_unlocked_pass`
    - `v5_w6_cli_init_preset_agent_swarm_basic_unlocked_pass`
- Files changed:
  - `projects/ocp/crates/ocp-sdk/tests/v5_w6_organs.rs`
  - `projects/ocp/crates/ocp-cli/src/main.rs`
  - `OCP-MVP-PLAN-v0.5.md`
- Commands run:
  - `cargo fmt --all`
  - `cargo fmt --all -- --check`
  - `cargo test -p ocp-sdk --test v5_w6_organs`
  - `cargo test -p ocp-cli w6`
  - `cargo test -p ocp-sdk -p ocp-cli`
  - `cargo clippy -p ocp-sdk -p ocp-cli --all-targets -- -D warnings`
- Test results:
  - PASS:
    - `v5_w6_organs`: 4/4 pass.
    - CLI filter `w6`: 6/6 pass.
    - full lane `ocp-sdk + ocp-cli`: pass toàn bộ suite.
    - `clippy` scope W6 (`ocp-sdk`, `ocp-cli`) pass `-D warnings`.
    - `fmt --check` pass.
- Notes/risks:
  - W6 trong workspace hiện tại không có `catalog.lock` lane riêng; chain lock preset dùng các lock đang tồn tại (`deps/policy/cosmos/organs-if-required`).
  - Với `init --preset ... --locked`, `cosmos lock sync` vẫn cần đủ signer/trust flags theo contract hiện hành.

### V5-W7 Gate Log
- Status: `DONE` (2026-03-03)
- Scope lock:
  - default conformance manifest chuyển sang `conformance.v5.toml`, có rollback env `OCP_CONFORMANCE_DEFAULT=v1|v5`.
  - expected-fail contract pin bằng `expected_status + expected_error_code`.
  - conformance runner luôn chạy trên workspace copy `target/ocp/w9/fixtures/<scenario_id>`, không mutate fixture gốc trong repo.
  - bổ sung no-op organ registry fixture 3 platform (`windows-x64`, `linux-x64`, `macos-arm64`) để verify/install deterministic theo platform.

#### 2026-03-03 - V5-W7 planning freeze
- Date:
  - 2026-03-03
- Gate/Step:
  - V5-W7
- Why:
  - Ở thời điểm mở W7, code thực tế chưa có parser v5/default selector/copy-runner/expected-fail evaluator theo đúng scope đã khóa.
- Scope:
  - Bổ sung parser manifest v5 + scenario fields expected-fail + step-based runner.
  - Bổ sung selector manifest mặc định (`v5` ưu tiên, rollback env `v1|v5`).
  - Bổ sung fixture conformance v5 và organ registry no-op 3 platform.
  - Chạy lane test W9 + lane SDK/CLI + conformance CLI deterministic lặp lại.
- Expected tests:
  - `cargo test -p ocp-sdk --test w9_conformance`
  - `cargo test -p ocp-cli w9`
  - `cargo test -p ocp-sdk -p ocp-cli`
  - `cargo fmt --all -- --check`
  - `cargo clippy -p ocp-sdk -p ocp-cli --all-targets -- -D warnings`
  - `cargo run -p ocp-cli -- test --conformance --manifest projects/ocp/conformance/conformance.v5.toml --locked --runtime deterministic --engine dual --out target/ocp/w9/reports/conformance_report.json --trust-store projects/ocp/security/trust.store.toml --signer-id dev-root-1 --sign-key projects/ocp/security/dev-root-1.signing.key.toml --json`
- Exit criteria:
  - W7 chuyển `DONE` với bằng chứng test PASS + deterministic digest ổn định.

#### 2026-03-03 - V5-W7 implementation closeout
- Date:
  - 2026-03-03
- Gate/Step:
  - V5-W7
- Implemented:
  - SDK conformance (`projects/ocp/crates/ocp-sdk/src/w9.rs`):
    - parser hỗ trợ cả `ocp.conformance.v1` và `ocp.conformance.manifest.v5`.
    - thêm scenario fields: `expected_status`, `expected_error_code`, `steps`, `organ_name`, `organ_version`, `organ_registry`.
    - thêm selector manifest mặc định:
      - `default_conformance_manifest_path`
      - `default_conformance_manifest_path_with_selector`
    - thêm step runner:
      - `kit_doctor`
      - `organ_verify`
      - `organ_install`
    - thêm expected-fail evaluator:
      - `V-W9-EXPECTED-FAIL-MISMATCH`
      - `V-W9-EXPECTED-FAIL-NOT-TRIGGERED`
    - runner sao chép scenario fixture vào `target/ocp/w9/fixtures/...` trước khi chạy.
    - runner sao chép runtime registry vào `target/ocp/w9/registry`.
  - SDK export (`projects/ocp/crates/ocp-sdk/src/lib.rs`):
    - export selector/default API và enum bước/trạng thái expected-fail.
  - CLI conformance (`projects/ocp/crates/ocp-cli/src/main.rs`):
    - bỏ hardcode `conformance.v1.toml`, chuyển sang selector mặc định từ SDK.
  - Conformance data:
    - thêm `projects/ocp/conformance/conformance.v5.toml` (5 scenarios nền + expected-fail).
    - thêm fixtures:
      - `projects/ocp/conformance/fixtures/foundation-workflow-basic/*`
      - `projects/ocp/conformance/fixtures/foundation-agent-swarm-basic/*`
      - `projects/ocp/conformance/fixtures/foundation-missing-organs-lock/*`
      - `projects/ocp/conformance/fixtures/foundation-wiring-drift/*`
  - Organ registry fixtures:
    - thêm `projects/ocp/registry/organs/index.toml`.
    - thêm artifact no-op:
      - `projects/ocp/registry/organs/artifacts/noop.organ/0.1.0/windows-x64/noop.organ.bin`
      - `projects/ocp/registry/organs/artifacts/noop.organ/0.1.0/linux-x64/noop.organ.bin`
      - `projects/ocp/registry/organs/artifacts/noop.organ/0.1.0/macos-arm64/noop.organ.bin`
  - Test updates:
    - cập nhật `projects/ocp/crates/ocp-sdk/tests/w9_conformance.rs`:
      - selector v1/v5
      - parser+run v5 expected-fail contract
- Files changed:
  - `projects/ocp/crates/ocp-sdk/src/w9.rs`
  - `projects/ocp/crates/ocp-sdk/src/lib.rs`
  - `projects/ocp/crates/ocp-cli/src/main.rs`
  - `projects/ocp/crates/ocp-sdk/tests/w9_conformance.rs`
  - `projects/ocp/conformance/conformance.v5.toml`
  - `projects/ocp/conformance/fixtures/foundation-workflow-basic/Ocp.toml`
  - `projects/ocp/conformance/fixtures/foundation-workflow-basic/src/main.ocp`
  - `projects/ocp/conformance/fixtures/foundation-workflow-basic/tests/smoke.ocp`
  - `projects/ocp/conformance/fixtures/foundation-agent-swarm-basic/Ocp.toml`
  - `projects/ocp/conformance/fixtures/foundation-agent-swarm-basic/src/main.ocp`
  - `projects/ocp/conformance/fixtures/foundation-agent-swarm-basic/tests/smoke.ocp`
  - `projects/ocp/conformance/fixtures/foundation-missing-organs-lock/Ocp.toml`
  - `projects/ocp/conformance/fixtures/foundation-missing-organs-lock/src/main.ocp`
  - `projects/ocp/conformance/fixtures/foundation-missing-organs-lock/cosmos.toml`
  - `projects/ocp/conformance/fixtures/foundation-wiring-drift/Ocp.toml`
  - `projects/ocp/conformance/fixtures/foundation-wiring-drift/src/main.ocp`
  - `projects/ocp/conformance/fixtures/foundation-wiring-drift/cosmos.toml`
  - `projects/ocp/registry/organs/index.toml`
  - `projects/ocp/registry/organs/artifacts/noop.organ/0.1.0/windows-x64/noop.organ.bin`
  - `projects/ocp/registry/organs/artifacts/noop.organ/0.1.0/linux-x64/noop.organ.bin`
  - `projects/ocp/registry/organs/artifacts/noop.organ/0.1.0/macos-arm64/noop.organ.bin`
  - `OCP-MVP-PLAN-v0.5.md`
- Commands run:
  - `cargo test -p ocp-sdk --test w9_conformance`
  - `cargo test -p ocp-cli w9`
  - `cargo run -p ocp-cli -- test --conformance --manifest projects/ocp/conformance/conformance.v5.toml --locked --runtime deterministic --engine dual --out target/ocp/w9/reports/conformance_report.json --trust-store projects/ocp/security/trust.store.toml --signer-id dev-root-1 --sign-key projects/ocp/security/dev-root-1.signing.key.toml --json`
  - `cargo test -p ocp-sdk -p ocp-cli`
  - `cargo fmt --all`
  - `cargo fmt --all -- --check`
  - `cargo clippy -p ocp-sdk -p ocp-cli --all-targets -- -D warnings`
- Test results:
  - PASS:
    - `w9_conformance` (SDK): 4/4.
    - CLI filter `w9`: 2/2.
    - conformance CLI v5: 5/5 scenario pass, `required_digest=9e97cc112c567170`.
    - full lane `ocp-sdk + ocp-cli`: pass toàn bộ suite.
    - `fmt --check`: pass.
    - `clippy -D warnings` (scope `ocp-sdk`, `ocp-cli`): pass.
  - Determinism:
    - chạy lặp lại cùng command conformance v5 cho cùng `run_id` và `required_digest`:
      - `run_id=w9-9976f76339dea293`
      - `required_digest=9e97cc112c567170`
- Notes/risks:
  - Nếu ép `--universe ci_locked` lên lane legacy (manifest v1) sẽ fail các scenario không có `cosmos.toml` với `V-UNIVERSE-NO-COSMOS`; đây là hành vi đúng theo contract.
  - Manifest v5 hiện là baseline Foundation 5 scenarios; nếu muốn hard-gate toàn bộ theo universe locked đồng nhất, cần tách manifest riêng cho lane `--universe`.

### 2026-03-04 - V0.6 audit v0.5 consistency re-verify (planning freeze)
- Date:
  - 2026-03-04
- Gate/Step:
  - v0.5 re-verify audit
- Why:
  - Xác nhận toàn bộ trạng thái `DONE` của v0.5 là done thực thi trên workspace hiện tại, không chỉ dựa log lịch sử.
  - Chốt các drift trong matrix lệnh để tránh báo `DONE` giả ở lần audit sau.
- Scope:
  - Rerun matrix v0.5 theo command hiện hành trong workspace.
  - Ghi rõ command nào cũ/lệch và command thay thế đã PASS.
  - Cập nhật thủ công section `Validation matrix v0.5`.
- Expected tests:
  - `cargo check --workspace`
  - `cargo test -p ocp-runtime-core -p ocp-sdk -p ocp-cli`
  - `cargo clippy -p ocp-runtime-core -p ocp-sdk -p ocp-cli --all-targets -- -D warnings`
  - `cargo fmt -- --check`
  - `cargo test -p ocp-sdk --test v5_w2_domain_resolution`
  - `cargo test -p ocp-sdk --test v5_w3_shadow`
  - `cargo test -p ocp-sdk --test v5_w4_hive`
  - `cargo run -p ocp-cli -- test --conformance --locked --runtime deterministic --engine dual --manifest projects/ocp/conformance/conformance.v1.toml --out target/ocp/w9/reports/conformance_report.json --trust-store projects/ocp/security/trust.store.toml --signer-id dev-root-1 --sign-key projects/ocp/security/dev-root-1.signing.key.toml --json`
  - `cargo run -p ocp-cli -- test --conformance --locked --runtime deterministic --engine dual --manifest projects/ocp/conformance/conformance.v5.toml --out target/ocp/w9/reports/conformance_report.v5.json --trust-store projects/ocp/security/trust.store.toml --signer-id dev-root-1 --sign-key projects/ocp/security/dev-root-1.signing.key.toml --json`
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocp_lane.ps1`
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocp_quarantine.ps1`
- Exit criteria:
  - Toàn bộ matrix hiện hành PASS.
  - Drift command cũ được ghi nhận rõ và đã thay bằng command chuẩn trong chính file v0.5.

### 2026-03-04 - V0.6 audit v0.5 consistency re-verify (implementation closeout)
- Date:
  - 2026-03-04
- Gate/Step:
  - v0.5 re-verify audit
- Implemented:
  - Cập nhật section `7) Validation matrix v0.5` theo workspace hiện tại:
    - bỏ package không tồn tại `ocp-runtime-rt`.
    - thay test W2/W3/W4 từ package cũ sang suite hiện hành trong `ocp-sdk`.
    - thay command conformance/quarantine sang bản locked đầy đủ signer/trust/manifest.
  - Rerun full matrix v0.5 ở mode audit và đối chiếu với lane.
- Files changed:
  - `OCP-MVP-PLAN-v0.5.md`
- Commands run:
  - `cargo check --workspace`
  - `cargo test -p ocp-runtime-core -p ocp-runtime-rt -p ocp-sdk -p ocp-cli` (drift reproduction)
  - `cargo clippy -p ocp-runtime-core -p ocp-runtime-rt -p ocp-sdk -p ocp-cli --all-targets -- -D warnings` (drift reproduction)
  - `cargo test -p ocp-runtime-core -p ocp-sdk -p ocp-cli`
  - `cargo clippy -p ocp-runtime-core -p ocp-sdk -p ocp-cli --all-targets -- -D warnings`
  - `cargo fmt -- --check`
  - `cargo test -p ocp-sdk --test v5_w2_domain_resolution`
  - `cargo test -p ocp-sdk --test v5_w3_shadow`
  - `cargo test -p ocp-sdk --test v5_w4_hive`
  - `cargo run -p ocp-cli -- test --conformance --locked --universe ci_locked --runtime deterministic --engine dual --json` (drift reproduction)
  - `cargo run -p ocp-cli -- test --conformance --locked --universe ci_locked --runtime throughput --engine bytecode --json` (drift reproduction)
  - `cargo run -p ocp-cli -- test --conformance --locked --runtime deterministic --engine dual --manifest projects/ocp/conformance/conformance.v1.toml --out target/ocp/w9/reports/conformance_report.json --trust-store projects/ocp/security/trust.store.toml --signer-id dev-root-1 --sign-key projects/ocp/security/dev-root-1.signing.key.toml --json`
  - `cargo run -p ocp-cli -- test --conformance --locked --runtime deterministic --engine dual --manifest projects/ocp/conformance/conformance.v5.toml --out target/ocp/w9/reports/conformance_report.v5.json --trust-store projects/ocp/security/trust.store.toml --signer-id dev-root-1 --sign-key projects/ocp/security/dev-root-1.signing.key.toml --json`
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocp_lane.ps1`
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocp_quarantine.ps1`
- Test results:
  - PASS:
    - `cargo check --workspace`
    - `cargo test -p ocp-runtime-core -p ocp-sdk -p ocp-cli` pass toàn bộ suite.
    - `cargo clippy -p ocp-runtime-core -p ocp-sdk -p ocp-cli --all-targets -- -D warnings` pass.
    - `cargo fmt -- --check` pass.
    - `v5_w2_domain_resolution` 6/6 pass.
    - `v5_w3_shadow` 6/6 pass.
    - `v5_w4_hive` 3/3 pass.
    - conformance deterministic locked với manifest v1 pass `10/10`, `required_digest=10deff71c24ef729`.
    - conformance deterministic locked với manifest v5 pass `5/5`, `required_digest=9e97cc112c567170`.
    - `tools/ci_ocp_lane.ps1` pass end-to-end.
    - `tools/ci_ocp_quarantine.ps1` pass (có conformance throughput/bytecode pass).
  - FAIL có chủ đích để xác nhận drift:
    - `cargo test/clippy` có `-p ocp-runtime-rt` fail do package không tồn tại.
    - command conformance locked trong matrix cũ fail `W9-CONFORMANCE-SIGN-REQUIRED` vì thiếu signer/trust.
- Notes/risks:
  - Matrix cũ trong v0.5 có drift lịch sử; đã cập nhật sang command đang chạy thật trên workspace.
  - Không có thay đổi code runtime/SDK/CLI trong đợt audit này; chỉ cập nhật tài liệu và bằng chứng re-verify.

