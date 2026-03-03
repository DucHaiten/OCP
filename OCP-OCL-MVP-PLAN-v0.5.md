> [!WARNING]
> ARCHIVED PLAN: file nay chi dung cho muc dich lich su/audit.
> Nguon su that active: projects/ocp-ocl/OCL-PLAN.md.

# OCL MVP PLAN v0.5

Ngày tạo: 2026-02-25  
Mục tiêu: mở rộng OCL thành nền tảng “cosmology + hive” theo hướng governance-first, deterministic-first, signed-supply-chain-first; không nổ scope và không phá các khóa đã chốt ở v0.4.

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
- V5-W0 Rebaseline v0.4 green + topology freeze: `IN_PROGRESS` (Core `PASS` + Windows sub-gate `PASS`)
- V5-W1 Cosmos config + lock/sign + universe wiring: `IN_PROGRESS`
- V5-W2 Domain isolation + bridge quotas: `IN_PROGRESS`
- V5-W3 Shadow execution + compare digest: `IN_PROGRESS`
- V5-W4 Hive supervisors + bounded swarm runtime: `IN_PROGRESS`
- V5-W5 Parallel views contract + renderer stub: `IN_PROGRESS`
- V5-W6 Organ packs install/verify/lock: `IN_PROGRESS`
- V5-W7 Conformance v0.5 SoT: `IN_PROGRESS`

---

## 0) Mục tiêu v0.5

v0.5 tập trung 3 nâng cấp:
1. Cosmology layer: đa universe + đa domain trong cùng platform, cách ly nhân quả rõ ràng.
2. Shadow execution: chạy song trùng để so sánh/parity/chính sách mà không commit vào truth world.
3. Hive runtime: Overmind -> Cerebrates -> Swarm workers, bounded và deterministic.

v0.5 không có mục tiêu biến OCL thành game engine, browser engine hay HPC engine trong core. OCL là control-plane + proof-plane; “muscle” đi qua organ packs signed.

## 0.1) Kiến trúc 2 tầng v0.5 (Kernel Axis + Foundation Kits)

### 0.1.1 Định nghĩa 2 tầng
1. Tầng 1 — OCL Kernel Axis (trục gốc):
   - là “luật vật lý” của platform.
   - bao gồm locks kế thừa v0.4 và phần v0.5A/v0.5B: universe/domain/shadow/hive/views/locks/audit/deterministic lane.
   - là phần ổn định, ít thay đổi, khó phá.
2. Tầng 2 — OCL Foundation (Kits/Blueprints + Organ-based muscle):
   - là “bộ cơ phổ thông” + “cụm hive mind theo use-case” để dev đa số không cần học sâu cosmology/hive internals vẫn dùng được.
   - Foundation cung cấp:
     - Kits/Blueprints: bộ module OCL + wiring mẫu theo bài toán (`workflow/webapi/bot/agent-swarm/ui-view/...`).
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
1. L0 (Preset user): `ocl init --preset ...` -> chạy được dự án, chỉnh config cơ bản.
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
1. File editable: `cosmos.toml` (schema `ocl.cosmos.v1`).
2. File generated: `cosmos.lock.v1` (canonical, signed trong locked mode).
3. Lock sync order bắt buộc:
   - `deps.lock.v2` + `catalog.lock.v2` + `policy.lock.v1` trước.
   - `cosmos lock sync` sau.
4. `cosmos.lock.v1` chỉ hash lock inputs đã tồn tại; không tạo vòng lock ngược.
5. Cosmos scaffold phải có preset để đóng gói complexity:
   - mặc định sinh `universe=default`, `domain=default`, `shadow=off`.
   - `ocl cosmos init --preset default|ci|prod`.

### 1.4 Universe selection lock
1. Mọi lệnh `check/run/test/build/compose/verify` hỗ trợ `--universe <id>`.
2. CI mặc định: `--universe ci_locked`.
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
1. SoT pass/fail platform v0.5: `ocl test --conformance --locked --universe ci_locked`.
2. Conformance report always-written.
3. Required digest order-sensitive, không đưa `warnings_count`.
4. Warning denylist preflight fail-hard:
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
Schema: `ocl.cosmos.v1`

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
1. `ocl cosmos init <project> [--preset default|ci|prod] [--json]`
2. `ocl cosmos lock sync <project> [--locked|--unlocked] [--json]`
3. `ocl run ... --universe <id> [--domain <id>]`
4. `ocl run ... --shadow <id> [--shadow-policy forbid_commit|shadow_commit_log]`
5. `ocl organ install <name> <version> [--registry ...] [--json]`
6. `ocl organ verify <project> [--locked|--unlocked] [--json]`
7. `ocl test --conformance --locked --universe ci_locked ...`
8. `ocl init --preset <preset_id> [--locked|--unlocked]`
9. `ocl kit list [--json]`
10. `ocl kit doctor <project> [--json]`

Lưu ý scope Foundation:
1. `init/kit` chỉ làm scaffolding + validation, không thay đổi runtime semantics.
2. `ocl init --preset ...` trong locked mode phải chạy lock sync chain (`deps/catalog/policy/cosmos`, và `organs` nếu required).
3. `ocl kit doctor` locked mode fail-hard nếu thiếu lock bắt buộc cho wiring kits/organs.

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
   - clean policy hai mức (`target/ocl/*` scoped default, `-FullClean` strict).
   - env pin portable + deterministic digest from report JSON.
   - topology freeze guard with before/after hash snapshots.
   - Windows plugin sub-gate: build signed artifact + verify-supply + deterministic run.
4. Exit: baseline report + no open regression.

### V5-W1 Cosmos config + lock/sign + universe wiring
1. Add parser/writer `cosmos.toml` + `cosmos.lock.v1`.
2. Add `ocl cosmos init --preset ...` + `ocl cosmos lock sync`.
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
2. Add `ocl organ install` + `ocl organ verify`.
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
   - `ocl init --preset ...`
   - `ocl kit doctor ...`

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
2. `cargo test -p ocl-runtime-core -p ocl-runtime-rt -p ocl-sdk -p ocl-cli`
3. `cargo clippy -p ocl-runtime-core -p ocl-runtime-rt -p ocl-sdk -p ocl-cli --all-targets -- -D warnings`
4. `cargo fmt -- --check`
5. `cargo run -p ocl-cli -- cosmos lock sync <project> --locked --json`
6. `cargo run -p ocl-cli -- test --conformance --locked --universe ci_locked --runtime deterministic --engine dual --json`
7. `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_lane.ps1`
8. `cargo test -p ocl-runtime-rt --test v5_w2_bridge_quota`
9. `cargo test -p ocl-runtime-rt --test v5_w3_shadow_digest`
10. `cargo test -p ocl-runtime-rt --test v5_w4_hive_soak`

### Quarantine (non-blocking)
1. `cargo run -p ocl-cli -- test --conformance --locked --universe ci_locked --runtime throughput --engine bytecode --json`
2. `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_quarantine.ps1`

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
10. `ocl test --conformance --locked --universe ci_locked` là SoT pass/fail.
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
3. Không đóng gói plugin binaries vào `.oclpkg` ở v0.5.
4. Không làm GUI/window/render engine full trong core.
5. Foundation không tạo “ngôn ngữ thứ hai”: không thêm semantics effect/state/IO mới ngoài Kernel.

---

## 10) Gate log v0.5

### V5-W0 Gate Log
- Status: `IN_PROGRESS` (`W0-Core PASS`, `Windows sub-gate PASS`)
- Date: `2026-02-26`
- Scope lock:
  - Chỉ rebaseline v0.4 + hardened freeze, không mở feature v0.5 mới.
  - Signed-strict blocking, deterministic double-run, topology freeze hash snapshot.
  - Verify-supply có positive/negative trust semantics.
  - Report always-written được chứng minh bằng fail-intentional run.
- Implemented:
  - `tools/ci_w0_entry.ps1`:
    - thêm `-FullClean` và clean policy 2 mức.
    - thêm env/culture/encoding pins cho deterministic path.
    - thêm signed preflight (`sign key`, `trust store`, `signer presence`).
    - chạy deterministic conformance 2 lần và so `required_digest`.
    - chạy fail-intentional conformance và assert report tồn tại.
    - chạy verify-supply positive/negative case.
    - chạy Windows plugin sub-gate (build signed -> verify-supply -> run deterministic).
    - ghi `target/ocl/v5/w0/reports/w0_status.json`.
  - `tools/guard_v5_topology_freeze.ps1` (mới):
    - freeze check bằng before/after SHA256 snapshots.
    - guard workspace members/scenario order/plugin platform/lane pins.
    - ghi `target/ocl/v5/w0/reports/topology_freeze_hash_snapshot.json`.
  - `tools/guard_project_boundaries.ps1`:
    - fix xử lý scalar output để tránh lỗi `.Count` khi chỉ có 1 changed path.
- Validation results:
  - Core blocking entry pass:
    - `powershell -ExecutionPolicy Bypass -File tools/ci_w0_entry.ps1 -FullClean -SignerId dev-root-1 -SignKey <path> -TrustStore projects/ocp-ocl/security/trust.store.toml`
  - Deterministic digest stable:
    - run1 = `fb8cc078d8981617e07af69718c3bd9bb1c113f65e77a1d7d3b17cdd48f70adf`
    - run2 = `fb8cc078d8981617e07af69718c3bd9bb1c113f65e77a1d7d3b17cdd48f70adf`
  - Verify-supply:
    - pass case: `valid=true`, signer `dev-root-1`.
    - fail case (untrusted trust store): fail-hard `[V-OCLPKG-SIGNER]`.
  - Topology freeze:
    - status `pass`, drift `[]`.
  - Report-always-written:
    - `conformance.fail.intentional.json` được ghi dù run fail đúng kỳ vọng.
  - Windows plugin sub-gate:
    - build signed artifact pass.
    - plugin verify-supply pass.
    - plugin deterministic run pass.
  - Evidence files:
    - `target/ocl/v5/w0/reports/conformance.det.run1.json`
    - `target/ocl/v5/w0/reports/conformance.det.run2.json`
    - `target/ocl/v5/w0/reports/conformance.fail.intentional.json`
    - `target/ocl/v5/w0/reports/verify_supply.pass.json`
    - `target/ocl/v5/w0/reports/verify_supply.fail.untrusted.json`
    - `target/ocl/v5/w0/reports/plugin.verify_supply.pass.json`
    - `target/ocl/v5/w0/reports/topology_freeze_before.json`
    - `target/ocl/v5/w0/reports/topology_freeze_hash_snapshot.json`
    - `target/ocl/v5/w0/reports/w0_status.json`

### V5-W1 Gate Log
- Status: `IN_PROGRESS`
- Date: `2026-02-26`
- Scope lock:
  - Ship policy lock thật (`policy.lock.v1`), cosmos lock thật (`cosmos.lock.v1`), universe overlay không tạo runtime fork.
  - Locked + cosmos thiếu `--universe` fail-hard (`V-UNIVERSE-REQUIRED`).
  - Sentinel ctx legacy bắt buộc: `universe_id="__legacy__"`, `domain_id="default"`.
  - Locked mismatch hard-fail chỉ cho `runtime_mode`, `engine`, `policy_profile_id`; `audit/trace` là metadata-only ở W1.
  - `domains/foundation/kits` parse + pin vào lock, chưa tác động runtime behavior.
- Implemented:
  - SDK:
    - thêm `crates/ocl-sdk/src/w1.rs` với parser/writer/sync/verify cho `policy.lock.v1` và `cosmos.lock.v1`.
    - canonical policy bytes từ `Ocl.toml` dùng active profile `[policy].budget_profile` (không phụ thuộc `--universe` ở W1).
    - canonical cosmos bytes có sorting deterministic cho `universes/domains/bridges/foundation/kits`.
    - lock-order enforced: `deps.lock.v2 -> catalog.lock.v2 -> policy.lock.v1 -> cosmos.lock.v1`.
    - `resolve_universe_v1` theo lock mới (locked require universe khi có cosmos, reject universe khi no-cosmos).
  - Runtime core:
    - mở rộng `Ctx` với `universe_id/domain_id/view_id/shadow_id` và default sentinel legacy.
  - CLI:
    - thêm `ocl policy lock sync`, `ocl cosmos init`, `ocl cosmos lock sync`.
    - thêm `--universe` wiring cho `check/run/test/build/compose/verify/test --conformance`.
    - enforce cosmos/policy lock + trust verify trong locked path.
    - overlay runtime/engine từ universe + hard-fail mismatch khi user explicit flag khác profile.
    - `cmd_lock_sync` sync luôn `policy.lock.v1` để giữ backward compatibility với locked flows hiện có.
  - Conformance:
    - `ConformanceRunOptionsV1` có `universe_id`.
    - runner chỉ append `--universe` cho scenario app có `cosmos.toml` để tránh phá legacy scenarios no-cosmos.
  - CI scripts:
    - `tools/ci_ocl_lane.ps1` thêm `cosmos init + cosmos lock sync` cho canary và dùng `--universe ci_locked` ở các bước canary/conformance W9.
    - `tools/ci_ocl_quarantine.ps1` thêm `--universe ci_locked` cho throughput conformance smoke.
  - Test coverage mới:
    - `crates/ocl-sdk/tests/v5_w1_cosmos.rs`
    - `crates/ocl-cli/tests/v5_w1_universe.rs`
    - bổ sung `v5_w1_ctx_legacy_sentinel_non_empty` trong `crates/ocl-runtime-core/tests/m0a_core_smoke.rs`.
  - Lock artifacts:
    - generate `policy.lock.v1` cho `app-ocl` và toàn bộ `apps/*` active để locked lane không fail do missing policy lock.
- Validation results:
  - Pass:
    - `cargo check -p ocl-runtime-core -p ocl-sdk -p ocl-cli`
    - `cargo test -p ocl-runtime-core -p ocl-sdk -p ocl-cli`
    - `cargo clippy -p ocl-runtime-core -p ocl-sdk -p ocl-cli --all-targets -- -D warnings`
    - `cargo fmt -- --check`
  - Pass W1 command chain (app-ocl):
    - `policy lock sync`
    - `cosmos init --preset ci`
    - `cosmos lock sync --locked --signer-id dev-root-1 --sign-key <path> --trust-store <path>`
    - `check/run/test/build/compose/verify --locked --universe ci_locked`
    - `test --conformance --locked --universe ci_locked --runtime deterministic --engine dual`
  - Conformance report:
    - `target/ocl/v5/w1/reports/conformance.json`
    - `required_digest = fb8cc078d8981617e07af69718c3bd9bb1c113f65e77a1d7d3b17cdd48f70adf`
  - Compatibility note:
    - `cosmos.toml/cosmos.lock.v1` không commit mặc định vào `app-ocl`; CI lane tự `cosmos init` khi chạy W1 path để không phá regression legacy no-cosmos tests.

### V5-W2 Gate Log
- Status: `IN_PROGRESS`
- Date: `2026-02-26`
- Scope lock:
  - Domain selection không còn fallback "domain đầu tiên"; non-reactor buộc `default` hoặc fail `V-DOMAIN-REQUIRED`.
  - `domain.bridge.emit` quota/capacity enforcement ở commit path (observe chỉ validate, không TOCTOU).
  - Poll fairness deterministic round-robin theo `start = tick % bridges_count`.
  - Bridge state per-bridge với epoch reset O(1): `tick_epoch + admit_count + dispatch_count + seq + queue`.
  - Sentinel non-empty bắt buộc cho context/audit/trace: `__legacy__/default`.
  - Locked cosmos lock sync fail-hard nếu bridge thiếu `bridge_queue_capacity`.
- Implemented:
  - `ocl-sdk`:
    - thêm `w2.rs` với:
      - `resolve_domain_selection_v1(...)`
      - `resolve_bridge_runtime_plan_v1(...)`
      - `DomainSelectionV1`, `BridgeRuntimeRuleV1`, `BridgeRuntimePlanV1`
    - `w1.rs` mở rộng `CosmosBridgeV1.bridge_queue_capacity`, canonicalization + serialize/parse lock + locked required enforcement.
  - `ocl-runtime-core`:
    - thêm `KeyFamily::DomainBridge` (`manifest_name = "domain.bridge"`).
    - thêm args schema keys: `args.v1.domain.bridge.emit`, `args.v1.domain.bridge.poll`.
    - thêm payloads: `StdBridgeAck`, `StdBridgeEvent`.
    - checker map payload/key family cho `domain.bridge.emit/poll`.
    - thêm reasons: `DomainIsolationDenied`, `DomainBridgeDenied`, `DomainBridgeCap`.
  - `ocl-runtime-rt`:
    - `RuntimeEvent` thêm `domain_id`.
    - `next_tick_event` và `poll_io_events` nhận domain id.
  - `ocl-sdk` runtime orchestration:
    - `ReactorServiceOptions` thêm `domain_ids`.
    - reactor tick emit theo selected domain set (canonical order).
  - `ocl-cli`:
    - thêm `--domain` cho `run/test/trace run/profile run`.
    - `LocalBridge` có scope selection + bridge runtime state.
    - implement `domain.bridge.emit/poll` observe/commit theo lock W2.
    - trace/audit mở rộng `universe_id/domain_id`; trace thêm `bridge_line_hash256/bridge_line_len`.
    - `tools/ci_ocl_lane.ps1` thêm suite W2:
      - `cargo test -p ocl-sdk --test v5_w2_domain_resolution`
      - `cargo test -p ocl-runtime-rt --test v5_w2_bridge_quota`
      - `cargo test -p ocl-cli --test v5_w2_domain_bridge`
- Validation results:
  - Pass:
    - `cargo check --workspace`
    - `cargo test -p ocl-runtime-core -p ocl-runtime-rt -p ocl-sdk -p ocl-cli`
    - `cargo clippy -p ocl-runtime-core -p ocl-runtime-rt -p ocl-sdk -p ocl-cli --all-targets -- -D warnings`
    - `cargo fmt -- --check`
    - `cargo test -p ocl-sdk --test v5_w2_domain_resolution`
    - `cargo test -p ocl-runtime-rt --test v5_w2_bridge_quota`
    - `cargo test -p ocl-cli --test v5_w2_domain_bridge`
    - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_lane.ps1` (with `OCL_SIGN_KEY_PATH` set)
  - Added test files:
    - `projects/ocp-ocl/crates/ocl-sdk/tests/v5_w2_domain_resolution.rs`
    - `projects/ocp-ocl/crates/ocl-runtime-rt/tests/v5_w2_bridge_quota.rs`
    - `projects/ocp-ocl/crates/ocl-cli/tests/v5_w2_domain_bridge.rs`
  - Added/validated W2-specific tests:
    - `v5_domain_bridge_allow_with_cap_and_quota`
    - `v5_domain_bridge_quota_exceeded_fail_honest`
    - `v5_cross_domain_state_leak_denied_without_bridge`
    - `v5_w2_emit_quota_enforced_at_commit_only`
    - `v5_w2_poll_round_robin_no_starvation_over_n_ticks`
    - `v5_w2_bridge_quota_reset_o1_epoch`
    - `v5_w2_bridge_quota_admission_o1_no_scan`
    - `v5_w2_bridge_dispatch_bounded_max_per_tick`
    - `v5_w2_bridge_dispatch_order_stable`
    - `v5_w2_locked_bridge_queue_capacity_required`
    - `v5_w2_trace_audit_universe_domain_non_empty`
    - `v5_w2_bridge_trace_hash_only_line_not_raw`
    - `v5_w2_replay_cross_domain_fail_hard`

### V5-W3 Gate Log
- Status: `IN_PROGRESS`
- Date: `2026-02-26`
- Scope lock:
  - Shadow chỉ cho deterministic lane; throughput + shadow trả `V-SHADOW-UNSUPPORTED`.
  - Reactor shadow replay-only: main capture IO tape canonical, main/shadow compare cùng replay tape.
  - Digest projection bytes exact và order-sensitive; compare fail-hard `V-SHADOW-MISMATCH`.
  - `commit_intent_hash256` canonical theo `effect_key|args_hash256|target_hash256|kind|seq`.
  - `initial_sandbox_hash256` non-reactor dùng zero-hash constant.
  - Transcript/report naming không overwrite (`...main.r{n}.jsonl` / `...shadow.r{n}.jsonl`).
- Implemented:
  - Added SDK W3 module API/types/writers:
    - `ShadowPolicyV1`, `ShadowOptionsV1`, `InputEnvelopeV1`, `ShadowTranscriptEventV1`, `ShadowCompareReportV1`
    - `build_commit_intent_hash256`, `build_shadow_required_digest`, IO tape record/replay helpers
    - `run_project_with_shadow_compare`, `run_reactor_service_with_shadow_compare`
  - Added CLI flags wiring:
    - `--shadow <id>`
    - `--shadow-policy forbid_commit|shadow_commit_log`
    - wired for `run`, `test`, `trace run`, `profile run`
  - Added exit mapping:
    - `V-SHADOW-MISMATCH -> 5`
    - `V-SHADOW-UNSUPPORTED -> 13`
  - Added reactor IO tape integration into runtime service options:
    - `io_tape_record_path`
    - `io_tape_replay_path`
  - Added test suites:
    - `projects/ocp-ocl/crates/ocl-sdk/tests/v5_w3_shadow.rs`
    - `projects/ocp-ocl/crates/ocl-runtime-rt/tests/v5_w3_shadow_digest.rs`
    - `projects/ocp-ocl/crates/ocl-cli/tests/v5_w3_shadow_cli.rs`
  - Updated CI scripts:
    - `tools/ci_ocl_lane.ps1` add blocking W3 shadow suite
    - `tools/ci_ocl_quarantine.ps1` add throughput shadow expected-unsupported check
- Validation results:
  - `cargo check --workspace` -> PASS
  - `cargo fmt -- --check` -> PASS
  - `cargo clippy -p ocl-runtime-core -p ocl-runtime-rt -p ocl-sdk -p ocl-cli --all-targets -- -D warnings` -> PASS
  - `cargo test -p ocl-runtime-core -p ocl-runtime-rt -p ocl-sdk -p ocl-cli` -> PASS
  - `cargo test -p ocl-sdk --test v5_w3_shadow` -> PASS
  - `cargo test -p ocl-runtime-rt --test v5_w3_shadow_digest` -> PASS
  - `cargo test -p ocl-cli --test v5_w3_shadow_cli` -> PASS
  - `cargo run -p ocl-cli -- run projects/ocp-ocl/app-ocl --locked --runtime deterministic --engine dual --shadow parity` -> PASS
  - `cargo run -p ocl-cli -- run projects/ocp-ocl/apps/mini-server --locked --reactor --ticks 64 --runtime deterministic --engine dual --shadow parity` -> PASS
  - `cargo run -p ocl-cli -- test projects/ocp-ocl/app-ocl --locked --engine dual --shadow parity` -> PASS
  - `cargo run -p ocl-cli -- run projects/ocp-ocl/app-ocl --locked --runtime throughput --engine bytecode --shadow parity` -> expected `V-SHADOW-UNSUPPORTED` (exit 13)
  - Evidence artifacts:
    - `target/ocl/v5/w3/reports/shadow_compare.run.json`
    - `target/ocl/v5/w3/reports/shadow_compare.reactor.json`
    - `target/ocl/v5/w3/reports/*.main.r*.jsonl`
    - `target/ocl/v5/w3/reports/*.shadow.r*.jsonl`
    - `target/ocl/v5/w3/reports/replay/*.io.jsonl`

### V5-W4 Gate Log
- Status: `IN_PROGRESS`
- Date: `2026-02-26`
- Scope lock:
  - Internal scheduler only, không thêm grammar/key mới.
  - `hive` resolved luôn được ghi vào `cosmos.lock.v1`.
  - Locked runtime fail-hard `V-HIVE-LOCK-MISSING` nếu lock cũ thiếu `hive`, kèm hướng dẫn rerun `ocl cosmos lock sync --locked`.
  - Fanout truncation deterministic giữ first-K theo order ổn định, drop phần dư và tăng counter.
  - Bounded counters + runtime report additive fields cho hive lifecycle/reuse.
- Implemented:
  - Added hive config model and lock migration wiring:
    - `projects/ocp-ocl/crates/ocl-sdk/src/w1.rs`
      - `CosmosHiveV1`
      - `CosmosSpecV1.hive` (optional)
      - `CosmosLockV1.hive` (resolved in sync flow)
      - canonical cosmos hash now includes resolved hive section
      - strict verify emits `V-HIVE-LOCK-MISSING` for old lock files
      - `cosmos init` now writes explicit `[hive]` defaults
  - Added W4 hive helper module:
    - `projects/ocp-ocl/crates/ocl-sdk/src/v5_w4_hive.rs`
      - `resolve_hive_caps_v1`
      - `verify_hive_lock_presence_v1`
      - `build_hive_required_digest_v1`
  - Added runtime hive scheduler internals:
    - `projects/ocp-ocl/crates/ocl-runtime-rt/src/hive.rs`
      - `HiveCaps`, `HiveCounters`, `HiveScheduler`
      - deterministic task/dispatch ordering
      - bounded spawn/fanout/mailbox admission
      - worker pool reuse + mailbox capacity observation + alloc event counters
    - exported via `projects/ocp-ocl/crates/ocl-runtime-rt/src/lib.rs`
  - Wired scheduler into reactor services:
    - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
      - `ReactorServiceOptions.hive_caps`
      - `ReactorServiceReport` additive hive counters + `dispatch_digest256`
      - runtime report JSON includes hive fields
  - CLI runtime wiring:
    - `projects/ocp-ocl/crates/ocl-cli/src/lib.rs`
      - resolve hive caps from cosmos/lock in runtime commands
      - pass caps into reactor options for run/test/trace/profile reactor flows
  - Runtime-core reason taxonomy additive:
    - `projects/ocp-ocl/crates/ocl-runtime-core/src/lib.rs`
    - `projects/ocp-ocl/crates/ocl-runtime-core/src/exec.rs`
    - added `SwarmCapExceeded`, `FanoutExceeded` reason mapping
  - Added W4 tests:
    - `projects/ocp-ocl/crates/ocl-sdk/tests/v5_w4_hive_lock.rs`
    - `projects/ocp-ocl/crates/ocl-runtime-rt/tests/v5_w4_hive_scheduler.rs`
    - `projects/ocp-ocl/crates/ocl-cli/tests/v5_w4_hive_runtime.rs`
  - CI updates:
    - `tools/ci_ocl_lane.ps1` adds blocking W4 suite + blocking reactor report command
    - `tools/ci_ocl_quarantine.ps1` adds long-soak W4 quarantine command
- Validation results:
  - `cargo check -p ocl-runtime-core -p ocl-runtime-rt -p ocl-sdk -p ocl-cli` -> PASS
  - `cargo test -p ocl-sdk --test v5_w1_cosmos --test v5_w4_hive_lock --test v5_w2_domain_resolution --test v5_w3_shadow` -> PASS
  - `cargo test -p ocl-runtime-rt --test v5_w2_bridge_quota --test v5_w3_shadow_digest --test v5_w4_hive_scheduler` -> PASS
  - `cargo test -p ocl-cli --test w1_runtime --test v5_w2_domain_bridge --test v5_w3_shadow_cli --test v5_w4_hive_runtime` -> PASS
  - `cargo clippy -p ocl-runtime-core -p ocl-runtime-rt -p ocl-sdk -p ocl-cli --all-targets -- -D warnings` -> PASS
  - `cargo fmt -- --check` -> PASS

### V5-W5 Gate Log
- Status: `IN_PROGRESS`
- Date: `2026-02-26`
- Scope lock:
  - Parallel views contract only: `std.view.render_tree` + `std.view.render_text`
  - Deterministic view selection + preflight (`V-VIEW-*`)
  - No grammar change, no new workspace crate, no `OclRuntimeBridge` trait change
  - Trace/audit view payload stays hash-only (`payload_hash256`, `payload_len`)
- Implemented:
  - Runtime-core key mapping:
    - `projects/ocp-ocl/crates/ocl-runtime-core/src/bridge.rs`
    - `projects/ocp-ocl/crates/ocl-runtime-core/src/lib.rs`
    - `projects/ocp-ocl/crates/ocl-runtime-core/src/checker.rs`
    - added `KeyFamily::StdView`, args schemas, checker payload/key-family mapping
  - Cosmos schema/lock + selection helpers:
    - `projects/ocp-ocl/crates/ocl-sdk/src/w1.rs`
      - added `[[views]]` parse/validate/sort/canonical/lock roundtrip
    - `projects/ocp-ocl/crates/ocl-sdk/src/w5.rs`
      - `ViewSelectionV1`, `resolve_view_selection_v1`, `verify_view_wiring_v1`
    - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
      - `TraceEventV1.payload_len` + serialization/parser wiring
  - CLI/runtime wiring:
    - `projects/ocp-ocl/crates/ocl-cli/src/lib.rs`
      - `RunOptions.view_id`
      - `--view` parse for `run/test/trace run/profile run`
      - preflight locks:
        - `V-VIEW-REQUIRED`
        - `V-VIEW-DOMAIN-AMBIG`
        - `V-VIEW-NOT-FOUND`
        - `V-VIEW-DOMAIN-MISMATCH`
        - `V-VIEW-NO-COSMOS`
      - `make_context()` now propagates `Ctx.view_id`
      - LocalBridge support for `std.view.render_text` / `std.view.render_tree`
      - tick rule: non-reactor `tick=0`, reactor uses logical tick
      - trace payload hygiene wired through hash/len-only fields
  - New app/demo:
    - `projects/ocp-ocl/apps/view-demo/Ocl.toml`
    - `projects/ocp-ocl/apps/view-demo/cosmos.toml`
    - `projects/ocp-ocl/apps/view-demo/src/main.ocl`
    - `projects/ocp-ocl/apps/view-demo/tests/smoke.ocl`
    - `projects/ocp-ocl/apps/view-demo/deps.lock`
    - `projects/ocp-ocl/apps/view-demo/deps.lock.v2`
    - `projects/ocp-ocl/apps/view-demo/policy.lock.v1`
  - Tests:
    - `projects/ocp-ocl/crates/ocl-sdk/tests/v5_w5_views.rs`
    - `projects/ocp-ocl/crates/ocl-cli/tests/v5_w5_view_runtime.rs`
  - CI updates:
    - `tools/ci_ocl_lane.ps1` adds W5 view tests + view-demo blocking commands
    - `tools/ci_ocl_quarantine.ps1` adds view throughput smoke
- Validation results:
  - `cargo check --workspace` -> PASS
  - `cargo test -p ocl-sdk --test v5_w5_views` -> PASS
  - `cargo test -p ocl-cli --test v5_w5_view_runtime` -> PASS
  - `cargo test -p ocl-cli --test w5_trace_profile` -> PASS
  - `cargo test -p ocl-sdk -p ocl-cli` -> PASS
  - `cargo clippy -p ocl-runtime-core -p ocl-sdk -p ocl-cli --all-targets -- -D warnings` -> PASS
  - `cargo fmt -- --check` -> PASS

### V5-W6 Gate Log
- Status: `IN_PROGRESS`
- Date: 2026-02-26
- Scope lock:
  - Trigger "organs used" chỉ lấy từ `[[kits]].required_organs` đã pin trong `cosmos.lock.v1`
  - Locked enforce `organs.lock.v1`; unlocked warn-only
  - Canonical sign message bao phủ `artifact_rel` + `provides_caps` + `source`
  - Platform lock rõ cho locked/unlocked (`V-ORGAN-PLATFORM-REQUIRED`, `V-ORGAN-PLATFORM-MISMATCH`)
  - Release-grade hardening chỉ bật qua `OCL_RELEASE_GRADE=1`
- Implemented:
  - SDK organ pipeline mới:
    - `projects/ocp-ocl/crates/ocl-sdk/src/v5_w6_organs.rs`
    - exported via `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
    - APIs: `collect_required_organs_from_cosmos_v1`, `sync_organs_lock_v1`, `verify_organs_lock_v1`, `install_organs_v1`, `resolve_platform_tag_v1`, `canonical_organ_sign_message`, `verify_organ_entry_signature`, `run_kit_doctor_v1`
  - Cosmos canonicalization chặt hơn:
    - `projects/ocp-ocl/crates/ocl-sdk/src/w1.rs`
    - `required_organs` sort + dedup deterministic trong lock
  - CLI commands mới:
    - `ocl organ install`, `ocl organ verify`, `ocl kit list`, `ocl kit doctor`
    - `ocl init --preset workflow_basic|agent_swarm_basic` (locked chain: deps -> catalog -> policy -> cosmos -> organs)
    - wired in `projects/ocp-ocl/crates/ocl-cli/src/lib.rs`
  - Registry indexes:
    - `projects/ocp-ocl/registry/organs/index.toml`
    - `projects/ocp-ocl/registry/kits/index.toml`
  - Tests mới:
    - `projects/ocp-ocl/crates/ocl-sdk/tests/v5_w6_organs.rs`
    - `projects/ocp-ocl/crates/ocl-cli/tests/v5_w6_organs_cli.rs`
  - CI updates:
    - `tools/ci_ocl_lane.ps1` thêm W6 suites + lock-order guard cho `view-demo` (`catalog lock sync` trước `cosmos lock sync`)
    - `tools/ci_ocl_quarantine.ps1` view throughput smoke dùng `--universe ci_locked` để khớp W1 locked rule
  - Project lock hygiene:
    - added `projects/ocp-ocl/apps/view-demo/catalog.lock.v2` để giữ lock chain đầy đủ trong lane
- Validation results:
  - `cargo check --workspace` -> PASS
  - `cargo test -p ocl-runtime-core -p ocl-runtime-rt -p ocl-sdk -p ocl-cli` -> PASS
  - `cargo clippy -p ocl-runtime-core -p ocl-runtime-rt -p ocl-sdk -p ocl-cli --all-targets -- -D warnings` -> PASS
  - `cargo fmt -- --check` -> PASS
  - `cargo test -p ocl-sdk --test v5_w6_organs` -> PASS
  - `cargo test -p ocl-cli --test v5_w6_organs_cli` -> PASS
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_lane.ps1 -SignerId dev-root-1 -SignKey projects/ocp-ocl/security/dev-root-1.signing.key.toml -TrustStore projects/ocp-ocl/security/trust.store.toml` -> PASS
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_quarantine.ps1` -> PASS (non-blocking report mode)

### V5-W7 Gate Log
- Status: `IN_PROGRESS`
- Date: `2026-02-26`
- Scope lock:
  - default conformance manifest chuyển sang `conformance.v5.toml`, có rollback env `OCL_CONFORMANCE_DEFAULT=v1|v5`.
  - expected-fail contract pin theo `expected_status + expected_error_code` và stage pin qua single-step scenarios.
  - runner luôn chạy trên workspace copy `target/ocl/w9/fixtures/<scenario_id>` (không mutate fixtures trong repo).
  - error-code extraction JSON-first (`--json` khi command hỗ trợ), text regex chỉ fallback.
  - bổ sung no-op organ fixtures 3 platform (`windows-x64`, `linux-x64`, `macos-arm64`) cho verify/install deterministic.
- Implemented:
  - SDK conformance (`projects/ocp-ocl/crates/ocl-sdk/src/w9.rs`):
    - parser hỗ trợ `ocl.conformance.v1` và `ocl.conformance.manifest.v5`.
    - thêm fields scenario: `expected_status`, `expected_error_code`.
    - default manifest selector theo env + fallback v5 (`default_conformance_manifest_path`).
    - structured command outcome + JSON-first error extraction.
    - thêm steps: `kit_doctor`, `organ_verify`, `organ_install`.
    - expected-fail evaluator: pass only khi đúng stage + đúng code; fail codes `V-W9-EXPECTED-FAIL-MISMATCH` / `V-W9-EXPECTED-FAIL-NOT-TRIGGERED`.
    - copy scenario fixture sang `target/ocl/w9/fixtures/...` trước khi chạy.
    - copy runtime registry vào `target/ocl/w9/registry` để giữ registry resolution ổn định cho copied fixtures.
  - Organ doctor classification (`projects/ocp-ocl/crates/ocl-sdk/src/v5_w6_organs.rs`):
    - normalize drift lỗi wiring sang `V-KIT-DOCTOR-WIRING`.
  - CLI workspace root resolution fix (`projects/ocp-ocl/crates/ocl-cli/src/lib.rs`):
    - `find_workspace_root` chọn ancestor cao nhất có `Cargo.toml` để tránh sai root khi chạy từ crate subdir.
  - Conformance manifests/fixtures:
    - thêm `projects/ocp-ocl/conformance/conformance.v5.toml` (14 scenarios, gồm foundation positive + negative expected-fail).
    - thêm fixtures:
      - `projects/ocp-ocl/conformance/fixtures/foundation-workflow-basic`
      - `projects/ocp-ocl/conformance/fixtures/foundation-agent-swarm-basic`
      - `projects/ocp-ocl/conformance/fixtures/foundation-missing-organs-lock`
      - `projects/ocp-ocl/conformance/fixtures/foundation-wiring-drift`
  - Organ registry fixtures:
    - update `projects/ocp-ocl/registry/organs/index.toml` với `noop.organ@0.1.0` cho 3 platform.
    - add artifacts:
      - `projects/ocp-ocl/registry/organs/artifacts/noop.organ/0.1.0/windows-x64/noop.organ.bin`
      - `projects/ocp-ocl/registry/organs/artifacts/noop.organ/0.1.0/linux-x64/noop.organ.bin`
      - `projects/ocp-ocl/registry/organs/artifacts/noop.organ/0.1.0/macos-arm64/noop.organ.bin`
  - Tests + CI/docs:
    - cập nhật `projects/ocp-ocl/crates/ocl-cli/tests/w9_conformance.rs` cho v5 parser/default/expected-fail/copy semantics.
    - update `tools/ci_ocl_lane.ps1` và `tools/ci_ocl_quarantine.ps1` dùng manifest v5.
    - update `projects/ocp-ocl/docs/OCL-USER-GUIDE.md` cho default v5 + rollback env.
- Validation results:
  - `cargo fmt -- --check` -> PASS
  - `cargo check -p ocl-runtime-core -p ocl-runtime-rt -p ocl-sdk -p ocl-cli` -> PASS
  - `cargo test -p ocl-cli --test w9_conformance` -> PASS
  - `cargo test -p ocl-sdk --test v5_w6_organs` -> PASS
  - `cargo test -p ocl-runtime-core -p ocl-runtime-rt -p ocl-sdk -p ocl-cli` -> PASS
  - `cargo clippy -p ocl-runtime-core -p ocl-runtime-rt -p ocl-sdk -p ocl-cli --all-targets -- -D warnings` -> PASS
  - `cargo run -p ocl-cli -- test --conformance --locked --universe ci_locked --runtime deterministic --engine dual --manifest projects/ocp-ocl/conformance/conformance.v5.toml --out target/ocl/w9/reports/conformance_report.json --trust-store projects/ocp-ocl/security/trust.store.toml --signer-id dev-root-1 --sign-key projects/ocp-ocl/security/dev-root-1.signing.key.toml --json` -> PASS
  - repeated deterministic run:
    - digest run1 = `68a7328a048d23c62466cf4fe845e01d9f4ceee8402b84c1b3d4411de3eaf561`
    - digest run2 = `68a7328a048d23c62466cf4fe845e01d9f4ceee8402b84c1b3d4411de3eaf561`

