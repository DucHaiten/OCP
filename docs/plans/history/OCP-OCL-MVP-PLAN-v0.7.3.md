# OCL v0.7.3 — Expansion Plan (Consumer-grade UI/Game + Shadow Primitive + Engines/Modules)

Ngày tạo: 2026-03-03  
Trạng thái: `DONE` (2026-03-04)
Phạm vi: **OCL-only**.  
Tiền đề: v0.7.1 đã có core + DX + replay-first + manifest; v0.7.2 đã có tool-grade IO packs (`std.fs/std.kv/std.time`) và fixture IO runner.

Mục tiêu v0.7.3: biến OCL thành nền làm **tool app/game** dễ khoe, dễ dùng, ít code, có “wow” mà ngôn ngữ phổ thông làm được nhưng tốn infrastructure:
- UI theo mô hình **draw-list** (không nhét widget framework vào core),
- game loop primitives (deterministic RNG + state delta),
- **shadow branching primitive** + compare report chuẩn,
- layer “engine/modules” để **lắp** và **tùy biến** mà không phình project,
- thêm lane `quarantine` tối thiểu (tùy chọn) cho những capability không deterministic (chưa cần net; chủ yếu để mở đường).

---

## Quy ước cập nhật bắt buộc (áp dụng từ 2026-03-04)
- Mọi thay đổi kế hoạch phải cập nhật file này trước khi code.
- Mọi triển khai xong phải cập nhật log ngay sau khi chạy test.
- Không nhảy gate: gate sau chỉ mở khi gate hiện tại đạt `DONE`.
- Không đánh dấu `DONE` khi targeted test chưa pass hoặc thiếu evidence.

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
  - `command-1`
- Test results:
  - `PASS`/`FAIL` + phạm vi (`targeted`/`regression`/`full`)
- Notes/risks:

---

## Tiêu chí tài liệu đầy đủ (bắt buộc cho v0.7.3)

### A) Cấu trúc bắt buộc của file plan
- Header + mục tiêu phiên bản.
- Quy ước cập nhật + 2 template (planning/closeout).
- Trạng thái workstreams + gate status.
- Scope lock (`In-scope`/`Out-of-scope`).
- Operational/CI commands.
- Execution gates theo thứ tự triển khai.
- Execution log theo cặp:
  - Planning Freeze.
  - Implementation Closeout.
- Handoff notes sang phiên bản kế tiếp.

### B) Chuẩn evidence cho mỗi gate
- Mỗi gate phải có đủ:
  - `Date`, `Gate/Step`, `Why`, `Scope`, `Expected tests`, `Exit criteria` (planning freeze).
  - `Implemented`, `Files changed`, `Commands run`, `Test results`, `Notes/risks` (implementation closeout).
- `Commands run` chỉ ghi lệnh đã chạy thật.
- `Test results` phải có `PASS/FAIL` rõ ràng theo phạm vi.
- Bắt buộc tách:
  - `Targeted tests (must-pass for gate)`.
  - `Regression tests (supporting only)`.

### C) Chuẩn trạng thái gate (chống DONE giả)
- Chỉ được ghi `DONE` khi đồng thời có:
  - code delta thật đúng scope gate,
  - targeted tests pass,
  - quality gates pass (`fmt/clippy` nếu nằm trong scope đóng gate),
  - evidence closeout đầy đủ.
- Nếu thiếu bất kỳ điều kiện nào: giữ `IN_PROGRESS` hoặc `PARTIAL`.
- `verification-only` phải ghi rõ nhãn verification, không được chuyển `DONE`.

### D) Chuẩn “Nhìn là hiểu” (30-60 giây)
- File phải có khối tóm tắt đầu file gồm:
  - mục tiêu phiên bản,
  - gate đang mở/đã xong/chưa xong,
  - bước kế tiếp ngay lập tức,
  - lệnh kiểm chứng chuẩn,
  - danh sách file code trọng yếu đã đổi.
- Người đọc không mở code vẫn phải hiểu:
  - đang làm gì,
  - đã làm gì,
  - còn thiếu gì để đóng gate.

### E) Chuẩn an toàn tài liệu
- UTF-8 có dấu tiếng Việt, không mojibake.
- Không ghi đè toàn file bằng script tự động khi chỉ vá nội dung nhỏ.
- Ưu tiên vá nhỏ bằng `apply_patch`.
- Khi dọn file:
  - bỏ phần trùng/lệch/legacy,
  - giữ nguyên evidence kỹ thuật quan trọng.

### F) Checklist khóa trước khi đóng gate
- [x] Gate status cập nhật đúng (`TODO/IN_PROGRESS/DONE`).
- [x] Có đủ Planning Freeze + Implementation Closeout.
- [x] `Files changed` khớp code delta thực tế.
- [x] `Commands run` là lệnh đã chạy thật.
- [x] `Test results` có `PASS/FAIL` rõ và đúng phạm vi.
- [x] Không còn mâu thuẫn giữa gate status và closeout.
- [x] Không còn lỗi encoding/hiển thị tiếng Việt trong nội dung file.

---

## 0) Governance + Tracking v0.7.3

### 0.1 Change Classification

| Item | Tag (S/C/I) | Compatibility | Evidence suite | Owner gate |
|---|---|---|---|---|
| `std.ui` draw-list (`frame_info/input/draw/present`) | `S` | `additive` | `tests/std_ui.rs`, `tests/ui_replay.rs` | `7.3-A` |
| `std.game` (`tick_info/rng/state protocol`) | `S` | `additive` | `tests/std_game.rs` | `7.3-B` |
| `std.shadow` (`shadow.run` + `shadow.compare`) | `S` | `additive` | `tests/std_shadow.rs` | `7.3-C` |
| Engine modules (`engine.ui.core`, `engine.game.loop`, `engine.shadow.preview`) | `S` | `additive` | `tests/engine_ui.rs`, `tests/engine_game.rs` | `7.3-D` |
| Templates `mini-game` / `shadow-preview` | `C` | `additive` | `tests/cli_mini_game_e2e.rs`, `tests/cli_shadow_preview_e2e.rs` | `7.3-E` |
| CLI init/run/replay integration v0.7.3 | `C` | `additive` | `tests/cli_mini_game_e2e.rs`, `tests/cli_shadow_preview_e2e.rs` | `7.3-E` |
| Lane `quarantine` gate + audit marker | `S` | `additive` | `tests/quarantine_lane.rs` | `7.3-F` |
| Workspace/module mapping cập nhật theo crate thực tế | `I` | `internal` | review + targeted compile/test | `7.3-D` |

### 0.2 Trạng thái Workstreams/Gates v0.7.3 (TRACKING)

#### Workstreams
- WS-S (packs semantics + deterministic contracts): `DONE` (7.3-A..7.3-F DONE, 2026-03-04)
- WS-C (templates + CLI + replay UX): `DONE` (covered by 7.3-E, 2026-03-04).
- WS-I (engine/module wiring theo workspace thực tế): `DONE` (covered by 7.3-D, 2026-03-04).

#### Gate status (7.3-A .. 7.3-F)
- Gate 7.3-A (std.ui observe/commit + audit + caps): `DONE` (2026-03-04)
- Gate 7.3-B (std.game tick/rng/state protocol): `DONE` (2026-03-04)
- Gate 7.3-C (std.shadow primitive + compare report): `DONE` (2026-03-04)
- Gate 7.3-D (Engine layer ui/game/shadow): `DONE` (2026-03-04)
- Gate 7.3-E (Templates + CLI + E2E replay): `DONE` (2026-03-04)
- Gate 7.3-F (Optional quarantine lane gate): `DONE` (2026-03-04)

#### Quy tắc cập nhật trạng thái (bắt buộc)
- `DONE` chỉ hợp lệ khi:
  - Có code delta đúng phạm vi gate.
  - Có targeted tests đúng phạm vi gate.
  - Có `Implementation Closeout` ghi rõ evidence `PASS/FAIL`.
- Nếu chỉ planning hoặc verification-only thì giữ `TODO`/`IN_PROGRESS`, không được ghi `DONE`.

---

## 1) Goals v0.7.3 (LOCKED)

### 1.1 North Star
- Dev tạo được **app UI** và **mini game** với template, chạy ngay, replay/share seed.
- Shadow preview chạy N nhánh bounded, có report diff/cost/reason, hiển thị được (text hoặc UI).
- Engine/modules giúp dự án không phình file: project chỉ chứa app code, engine nằm trong runtime distribution.
- Vẫn giữ core: permissions/bounded/deterministic (lane locked) + audit/replay.

### 1.2 KPI bắt buộc (định lượng)
**KPI-1: Mini-game template**
- `ocl init mini-game` tạo project **≤ 7 file** (bao gồm 1–2 assets nhỏ nếu cần).
- Script OCL (không tính engine/packs): **≤ 400 LOC**.
- `ocl run` => game chạy (loop + input + draw) và tạo replay artifacts.
- `ocl replay` reproduces signature (lane locked, input recorded).

**KPI-2: Shadow-preview template**
- `ocl init shadow-preview` tạo project **≤ 7 file**.
- Shadow branches: `N <= 8`, `step_cap <= 5000` mỗi nhánh.
- Compare report có:
  - divergence summary (diff keys bounded)
  - cost table (budget/steps)
  - reason breakdown
- Replays deterministically.

**KPI-3: Engine modularity**
- Engine không copy vào project: chỉ reference/import.
- Engine config override qua `ocl.toml` hoặc record config trong OCL.
- Dev thay UI theme/controls/state logic mà không fork engine.

### 1.3 Decision Locks trước khi code (BLOCKER nếu thiếu)
- `engine.ui.run` không dùng callback record mơ hồ. Chốt `App Protocol` theo `entry_module + ctx`.
- `shadow.run` chỉ có 1 mode entry: current entry/module + inject variant qua `ctx.shadow.*`.
- Trong `shadow.run`, commit IO/effect thật bị chặn theo policy shadow isolation.
- `std.game.rng` khóa thuật toán + seed derivation + output range.
- `std.game.tick_info` là facade của `std.time.tick_info` (một nguồn thời gian duy nhất).
- `std.ui.frame_info` trong lane locked lấy từ config/manifest deterministic, không đọc host env trực tiếp.
- Mọi cap bytes dùng canonical value encoding ổn định, không phụ thuộc serializer ngẫu nhiên.
- `shadow.compare` khóa algorithm diff/order/truncation deterministic.
- `std.ui.input` khóa schema event + truncate tail deterministic.
- Lane `quarantine` bị tách cứng khỏi `locked` (không replay lẫn).
- `std.ui` assets policy tái dùng sandbox/symlink rules của `std.fs` v0.7.2.
- Error contract khóa rõ: `RC-*` là reason của Result4; `X-*` chỉ cho exec/internal diagnostics.

---

## 2) Axis Lock v0.7.x (kế thừa, không đổi)
- No naked IO; IO/effect chỉ qua capability.
- 4-kind everywhere.
- Commit-gated effects.
- Boundedness (step cap, enumerate cap, per-call budget).
- Determinism-by-default (lane locked).
- Diagnostics structured.

---

## 3) Scope v0.7.3

### 3.1 In-scope (ship)
A) Packs:
- `std.ui` (draw-list UI):
  - observe: frame_info, input
  - commit: draw, present
  - optional: assets (image/font) minimal, sandboxed
- `std.game` (loop primitives):
  - observe: tick_info, rng(stream,count)
  - commit: state_delta (idempotent)
- `std.shadow` (primitive):
  - shadow.run (N branches, copy-on-write env, bounded)
  - shadow.compare (report format chuẩn)

B) Engine/Modules layer (composed subsystems):
- `engine.ui.core` (frame lifecycle)
- `engine.ui.widgets` (declarative widgets built on draw-list; optional minimal set)
- `engine.game.loop` (update/render contract)
- `engine.shadow.preview` (integrate shadow report into UI/text)

C) Input record/replay for UI/game:
- input events phải record vào audit để replay deterministically.

D) Lane `quarantine` (minimal spec; optional implement v0.7.3)
- chủ yếu để mở đường cho capability nondet trong tương lai
- v0.7.3 không bắt buộc ship net/http; quarantine chỉ cần “gate by env” + audit marker.

E) Templates:
- `mini-game`
- `shadow-preview`
- (optional) `tool-ui` (kết hợp v0.7.2 fs/kv + v0.7.3 ui)

### 3.2 Out-of-scope (defer)
- Network stack (HTTP/TLS) production-grade.
- Audio advanced, physics advanced, shader pipelines.
- Full widget framework (layout engine lớn).
- Marketplace/registry online.

---

## 4) Common Contracts for Consumer Packs

### 4.1 UI model: draw-list only
- OCL không làm widget framework phức tạp; OCL chỉ xuất “draw commands”.
- Layout/widgets (nếu có) nằm ở engine layer (OCL modules), không phải core.
- Host renderer là adapter của `std.ui`, có thể thay backend.

### 4.2 Determinism rules
- UI input phải được record:
  - keystrokes/mouse/text input events
  - frame tick/dt fixed
- draw-list commit deterministic theo state+input.
- RNG chỉ qua `std.game.rng` deterministic (streamed).
- Canonical key order: lexicographic theo UTF-8 bytes.
- Canonical path (khi ghi artifacts/input refs): dùng `/`, strip `./`, path relative theo project root, cấm absolute path, cấm `..`.
- Bytes dùng cho cap checks (`state_delta_max_bytes`, report bytes) = độ dài canonical value encoding; không dùng serializer phụ thuộc implementation.

### 4.3 Boundedness
- draw-list có cap: max commands per frame.
- input events per frame có cap.
- shadow branches có cap N, step cap per branch, budget cap per branch.
- state diff/report bounded: key cap, list cap.

---

## 5) std.ui — Spec (draw-list)

### 5.1 Permission model
Manifest:

```toml
[permissions.std_ui]
enabled = true
max_draw_cmds = 5000
max_input_events = 500
# optional assets (if enabled)
assets_read = ["./assets/**"]
max_asset_bytes = 2097152
```

Rules:
- locked: only deterministic backend allowed
- deny patterns override
- `assets_read` áp dụng đúng policy sandbox của `std.fs` v0.7.2:
  - canonicalize path trước khi check allowlist,
  - cấm `..` segments,
  - cấm symlink traversal,
  - fail-honest nếu path outside sandbox.
- Source-of-truth cho `std.ui.frame_info` trong lane locked:
  - ưu tiên `config` truyền vào `engine.ui.run(entry_module, config)` với các field `{w,h,scale,theme,locale}`,
  - nếu thiếu field thì fallback từ `[engine.ui]` trong `ocl.toml`,
  - không đọc trực tiếp host runtime values trong lane locked.

### 5.2 Keyspace
**Observe**
- `std.ui.frame_info`
  - ctx: `{}`
  - payload: `{ w:Int, h:Int, scale:Int, theme:String, locale:String }`
  - locked lane rule:
    - `theme/locale/w/h/scale` lấy từ config deterministic (`ocl.toml`/engine config),
    - không đọc trực tiếp host runtime values trong lane locked.
- `std.ui.input`
  - ctx: `{ cap:Int }` (must be <= max_input_events)
  - payload: `{ events: List<Record{ t:String, a:Value }>, truncated:Bool }`
  - determinism: events order preserved; cap truncation => `DEGRADED`
  - event schema lock:
    - `t` thuộc enum cố định: `"key" | "mouse" | "text" | "quit"`.
    - `a` là record canonical theo từng `t`.
    - khi vượt cap: truncate tail (giữ prefix theo order), `truncated=true`, trả `DEGRADED`.

**Commit**
- `std.ui.draw`
  - ctx: `{ list: List<DrawCmd>, cap:Int }` (cap must be <= max_draw_cmds)
  - DrawCmd minimal:
    - `{ kind:"rect", x,y,w,h, color:String }`
    - `{ kind:"text", x,y, text:String, size:Int }`
    - `{ kind:"line", x1,y1,x2,y2, color:String }`
    - `{ kind:"image", x,y,w,h, id:String }` (optional assets)
- `std.ui.present`
  - ctx: `{}`

### 5.3 Audit events
- `UiObserve` (frame_info/input; counts; kind/reason)
- `UiCommit` (draw_cmd_count; present; kind/reason)
- Input events must be included in audit payload in canonical form (bounded).

---

## 6) std.game — Spec (loop primitives)

### 6.1 Permission model
Manifest:

```toml
[permissions.std_game]
enabled = true
fixed_dt_ms = 16
rng_streams = ["main", "loot"]
rng_max_count = 1024
state_delta_max_bytes = 65536
```

### 6.2 Keyspace
**Observe**
- `std.game.tick_info`
  - payload: `{ tick:Int, dt_ms:Int }` (dt fixed)
  - source lock: alias/facade từ `std.time.tick_info` (single source of truth).
- `std.game.rng`
  - ctx: `{ stream:String, count:Int }` (count <= rng_max_count; stream must be allowed)
  - payload: `{ values: List<Int> }`
  - determinism: values derived from seed + tick + stream
  - RNG freeze:
    - algorithm: `PCG32` (fixed version v1 for v0.7.3).
    - output range: `Int` trong `[0, 2^31-1]`.
    - base seed: từ replay/config seed locked lane.
    - stream seed: `fnv1a64("pcg32" || stream_name) XOR base_seed`.
    - hash input bytes là UTF-8.
    - concat format cho hash: `"pcg32\\x00" + stream_name` (separator `0x00`) để tránh collision chuỗi.
    - generation order: deterministic theo `(tick, index)`; cùng seed+ctx phải cho cùng output.

**Commit**
- `std.game.state_delta`
  - ctx: `{ idempotency_key:String, delta:Value }`
  - policy: serialized delta bytes <= state_delta_max_bytes
  - determinism: idempotency_key ensures replay safety
  - v0.7.3 lock:
    - normal game loop không phụ thuộc `state_delta` để tiến trạng thái; state chính đi theo payload return `{state, draw, quit?}`,
    - `state_delta` là integration hook tùy chọn cho host sync/debug,
    - trong `shadow.run` luôn deny `state_delta` commit với `INSUFFICIENT + RC-SHADOW-EFFECT-DISALLOWED`.

### 6.3 Audit events
- `GameObserve` (tick_info/rng; stream/count)
- `GameCommit` (state_delta size; idempotency_key hash)

---

## 7) std.shadow — Spec (primitive + report)

### 7.1 Permission model
Manifest:

```toml
[permissions.std_shadow]
enabled = true
max_branches = 8
branch_step_cap = 5000
branch_budget_cap = 200000
max_diff_keys = 2000
max_report_bytes = 262144
```

### 7.2 Primitive: shadow.run
Signature (conceptual):
- input:
  - `variants`: List<Value> (each variant = injected params record)
  - caps: branches/step/budget
- output:
  - `{ branches: List<BranchResult>, truncated:Bool }`

Entry mode lock (single mode, không có phương án thứ hai):
- `shadow.run` luôn chạy current entry/module của program hiện tại.
- Mỗi branch inject context:
  - `ctx.shadow.branch_id`
  - `ctx.shadow.variant`
  - `ctx.shadow.mode = "shadow"`
- Không dùng callback/function reference động trong v0.7.3.

BranchResult:
- `{ id:Int, outcome:Kind, reason_code?:String, signature:String, cost:{steps:Int,budget:Int}, state_summary:Value }`

Rules:
- copy-on-write env/state per branch (no cross-branch mutation)
- deterministic scheduling: run branches in order id=0..N-1
- early abort if global cap reached => truncated + DEFERRED reason
- Shadow isolation (locked):
  - chặn commit effect thật trong shadow path cho các key IO/effect (`std.fs.*`, `std.kv.*`, `std.ui.present`, `std.game.state_delta`, và các effect keys tương đương).
  - khi vi phạm: `INSUFFICIENT` + `RC-SHADOW-EFFECT-DISALLOWED`.
  - branch chỉ đọc các observe deterministic được phép theo permissions/caps.

### 7.3 Report: shadow.compare
Input:
- list `BranchResult` + baseline branch id
Output:
- `{ diff_keys: List<String>, divergence:Value, cost_table:List<Record>, reason_table:List<Record> }`
Bounded:
- diff_keys <= max_diff_keys
- report bytes <= max_report_bytes else DEGRADED
- Compare lock:
  - diff strategy v0.7.3: shallow record diff (depth=1).
  - `diff_keys` luôn sort lexicographic UTF-8.
  - nếu vượt `max_diff_keys`: cắt deterministic + `DEGRADED` + `truncated=true`.
  - nếu report bytes vượt cap: `DEGRADED` + `RC-SHADOW-REPORT-TOO-LARGE`, cắt deterministic theo canonical order.

### 7.4 Audit events
- `ShadowRun` (N, caps, per-branch outcome/cost/signature)
- `ShadowCompare` (diff_count, report_bytes)

---

## 8) Engine/Modules Layer (không phình project)

### 8.1 Principles
- Engine nằm trong runtime distribution (`std/engine/*`), project chỉ import.
- Engine API là OCL module functions + record config.
- Engine không giấu quyền lực: permissions vẫn khai rõ trong manifest (std_ui/std_game/std_shadow).

### 8.2 engine.ui.core (minimal)
Responsibilities:
- init state (theme, viewport, locale, scale) từ config deterministic
- per-frame:
  - observe frame_info
  - observe input
  - invoke `entry_module` theo ctx protocol chuẩn
  - commit draw list + present
Caps:
- enforce max_draw_cmds, max_input_events

Public API:
- `engine.ui.run(entry_module, config)`.
- Engine mỗi frame gọi `entry_module` với ctx chuẩn hóa:
  - `ctx.phase = "init" | "frame"`
  - `ctx.tick`, `ctx.dt_ms`
  - `ctx.input_events`
  - `ctx.state`
- Program trả `Result4<Value>` với payload chuẩn:
  - `{ state: Value, draw: List<DrawCmd>, quit?: Bool }`.
- Cách này giữ tương thích với bề mặt ngôn ngữ hiện có, không yêu cầu first-class callbacks.

### 8.3 engine.ui.widgets (optional minimal set)
- button, list, textbox (pure layout + draw commands)
- state is explicit record to keep replay/shadow diff simple
- no complex retained-mode framework

### 8.4 engine.game.loop
- integrate `std_game.tick_info` + `std_game.rng` + state protocol theo payload return
- `engine.game.run(entry_module, config)`:
  - mỗi tick gọi `entry_module` với ctx deterministic (`phase/tick/dt/state/input`)
  - state transitions explicit qua payload `{state, draw, quit?}`
- `std.game.state_delta` là integration hook tùy chọn, không phải điều kiện bắt buộc để game loop chạy
- supports “headless mode” for tests (no UI) and UI mode (draw list)

### 8.5 engine.shadow.preview
- convenience wrapper:
  - define variants
  - call shadow.run
  - call shadow.compare
  - render report as text panel (via engine.ui) or output JSON

---

## 9) Input record/replay integration

### 9.1 Input recording
- std.ui.input events must be appended into audit with stable schema.
- during replay:
  - std.ui.input returns recorded events instead of reading device.

### 9.2 Replay determinism contract
- For mini-game and shadow-preview templates:
  - `ocl replay` must reproduce:
    - signature.txt
    - (optional) final frame checksum (if you implement)
- Any nondet backend disallowed in locked lane.

---

## 10) Lane quarantine (minimal, optional v0.7.3)
Purpose:
- future-proof for nondeterministic capabilities (net, wallclock, etc.)

Rules:
- only enabled if `OCL_QUARANTINE=1`
- audit must include `lane="quarantine"`
- replay may be “best-effort” unless capability provides record/replay
- hard boundary with locked lane:
  - lane locked phải reject quarantine-only capabilities.
  - signature input luôn include lane id; artifacts quarantine không được replay như locked.

v0.7.3 minimum:
- implement lane gate + audit marker
- no new nondet packs required

---

## 11) Templates v0.7.3

### 11.1 `mini-game`
- uses `std.ui` + `std.game` + engine.ui/game
- demonstrates:
  - deterministic RNG
  - input record/replay
  - draw list
- includes:
  - seed in UI; “share seed” concept

### 11.2 `shadow-preview`
- uses `std.shadow` + (optional) std.ui
- demonstrates:
  - run N branches of “move choices”
  - compare report
  - show cost + diff summary

### 11.3 (optional) `tool-ui`
- combines v0.7.2 tool IO with v0.7.3 UI
- e.g., JSON viewer/editor with sandbox fs + kv settings

---

## 12) Error taxonomy v0.7.3 (additive)
Prefer Result4 reason codes. Exec errors should be rare.

New RC codes:
- `RC-UI-DISABLED`
- `RC-UI-CAP-EXCEEDED`
- `RC-GAME-DISABLED`
- `RC-GAME-RNG-INVALID-STREAM`
- `RC-SHADOW-DISABLED`
- `RC-SHADOW-CAP-EXCEEDED`
- `RC-SHADOW-REPORT-TOO-LARGE`
- `RC-SHADOW-EFFECT-DISALLOWED`

Policy lock:
- `RC-*` chỉ dùng làm `reason_code` trong `Result4` (`INSUFFICIENT/DEFERRED/DEGRADED` theo contract).
- `X-*` dành cho exec/internal diagnostics (không dùng thay `RC-*` cho lỗi input/policy của packs).

---

## 13) Execution gates v0.7.3 (triển khai tuần tự)

### Gate 7.3-A — std.ui observe/commit + audit + caps
Scope:
- frame_info + input(cap) observe
- draw(list cap) + present commit
- audit events + input record/replay plumbing
Tests:
- `tests/std_ui.rs`
- `tests/ui_replay.rs`
Exit criteria:
- draw list capped, input capped
- replay returns same input events -> signature stable

### Gate 7.3-B — std.game tick/rng/state_delta
Scope:
- tick_info fixed dt
- rng(stream,count) deterministic
- state_delta commit (idempotency, optional integration hook)
Tests:
- `tests/std_game.rs`
Exit criteria:
- rng stable across replay
- state_delta size cap enforced khi state_delta được dùng

### Gate 7.3-C — std.shadow primitive + compare report
Scope:
- shadow.run (N branches, COW env, caps)
- shadow.compare report format bounded
- audit events
Tests:
- `tests/std_shadow.rs`
Exit criteria:
- branch signatures stable
- compare report bounded and deterministic

### Gate 7.3-D — Engine layer (ui core + game loop + shadow preview)
Scope:
- engine.ui.run
- engine.game.run
- engine.shadow.preview wrapper
Tests:
- `tests/engine_ui.rs`
- `tests/engine_game.rs`
Exit criteria:
- engine doesn’t require project boilerplate
- configs override behavior without fork

### Gate 7.3-E — Templates + CLI + E2E replay
Scope:
- `ocl init mini-game`
- `ocl init shadow-preview`
- (optional) `ocl init tool-ui`
- CLI e2e tests: run + replay
Tests:
- `tests/cli_mini_game_e2e.rs`
- `tests/cli_shadow_preview_e2e.rs`
Exit criteria:
- KPI-1, KPI-2, KPI-3 pass

### Gate 7.3-F — (Optional) quarantine lane gate
Scope:
- implement lane gate + audit marker
Tests:
- `tests/quarantine_lane.rs`
Exit criteria:
- quarantine requires env flag; locked remains strict

---

## 14) CI Commands (v0.7.3)
- `cargo test`
- `cargo test --test std_ui`
- `cargo test --test ui_replay`
- `cargo test --test std_game`
- `cargo test --test std_shadow`
- `cargo test --test engine_ui`
- `cargo test --test engine_game`
- `cargo test --test cli_mini_game_e2e`
- `cargo test --test cli_shadow_preview_e2e`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt -- --check`

---

## 15) Execution Log (template)

### 2026-03-04 — 7.3-A Planning Freeze
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.3-A
- Why:
  - Mở `std.ui` runtime pack đầu tiên của v0.7.3 để khóa hợp đồng UI draw-list bounded + input deterministic/replay trước khi mở game/shadow.
- Scope:
  - Thêm key observe `std.ui.frame_info`, `std.ui.input`.
  - Thêm key commit path `std.ui.draw`, `std.ui.present` theo commit-gated discipline.
  - Enforce caps: `max_draw_cmds`, `max_input_events` và reason code tương ứng.
  - Bổ sung audit events cho UI observe/commit và canonical input recording.
  - Bổ sung permission parse/verify `[permissions.std_ui]` ở SDK.
  - Thêm tests targeted cho gate A.
- Expected tests:
  - `cargo test --test std_ui`
  - `cargo test --test ui_replay`
  - `cargo test --test ocl_manifest_fs_kv_time`
  - `cargo test --test ocl_stdlib`
  - `cargo test`
- Exit criteria:
  - `std.ui.frame_info/input/draw/present` chạy đúng 4-kind/cap/commit policy.
  - Input truncation deterministic + replay signature ổn định.
  - Permission `std_ui` fail-closed khi thiếu/disabled và pass khi enabled.
  - Targeted tests pass trước khi xét regression/full.

### 2026-03-04 — 7.3-A implementation closeout
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.3-A
- Implemented:
  - Runtime `std.ui` cho gate A:
    - `std.ui.frame_info` payload deterministic (`w/h/scale/theme/locale`) và giữ `ctx_tick/frame` để tương thích ngược.
    - `std.ui.input` parse events deterministic, cap truncate tail -> `DEGRADED + RC-UI-CAP-EXCEEDED`.
    - `std.ui.draw`/`std.ui.present` qua commit-gated path với pending metadata.
  - Audit/trace:
    - thêm `TraceEvent::UiObserve` và `TraceEvent::UiCommit`.
    - map canonical payload trong runtime + SDK trace mapping/hash.
  - Commit integration:
    - thêm pending-op `std.ui` vào `ObservationMeta`.
    - commit accepted sẽ emit `UiCommit` (no-effect, audited), không tạo side-effect IO thật.
  - Registry/diagnostics:
    - thêm reason codes: `RC-UI-DISABLED`, `RC-UI-CAP-EXCEEDED`.
    - registry keyspace `std.ui` cho `frame_info/input/draw`.
  - SDK permission gate:
    - parse `[permissions.std_ui]` (`enabled`, `max_draw_cmds`, `max_input_events`, `assets_read`, `max_asset_bytes`).
    - verify fail-closed cho `std.ui.*` khi thiếu block hoặc `enabled=false`.
  - Test gate A:
    - thêm `tests/std_ui.rs` (4 tests).
    - thêm `tests/ui_replay.rs` (3 tests).
    - mở rộng `tests/ocl_manifest_fs_kv_time.rs` thêm 2 tests cho `std_ui`.
- Files changed:
  - `src/ocp_ocl/diag.rs`
  - `src/ocp_ocl/audit.rs`
  - `src/ocp_ocl/registry.rs`
  - `src/ocp_ocl/exec.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
  - `tests/std_ui.rs`
  - `tests/ui_replay.rs`
  - `tests/ocl_manifest_fs_kv_time.rs`
  - `OCP-OCL-MVP-PLAN-v0.7.3.md`
- Commands run:
  - `cargo test --test std_ui`
  - `cargo test --test ui_replay`
  - `cargo test --test ocl_manifest_fs_kv_time`
  - `cargo test --test ocl_stdlib`
  - `cargo test`
  - `cargo fmt -- --check` (chưa pass lần đầu)
  - `cargo fmt`
  - `cargo fmt -- --check` (PASS sau khi format)
- Test results:
  - Targeted tests (must-pass for gate):
    - PASS: `std_ui` 4/4.
    - PASS: `ui_replay` 3/3.
  - Regression tests (supporting only):
    - PASS: `ocl_manifest_fs_kv_time` 7/7.
    - PASS: `ocl_stdlib` 13/13.
    - PASS: full `cargo test` toàn workspace.
    - PASS: `cargo fmt -- --check` (sau khi chạy `cargo fmt`).
  - Kết luận gate:
    - `DONE` (targeted tests pass + regression pass + quality gate pass).
- Notes/risks:
  - `std.ui.draw` hiện nhận draw-list qua ctx string encoding (`list=...`) để giữ tương thích bề mặt ngôn ngữ hiện tại; chưa mở typed ctx AST ở gate này.
  - `UiCommit` là no-effect audited commit (không chạm IO thật), đúng scope gate A.
  - Design alignment: FULL.

### 2026-03-04 — 7.3-B Planning Freeze
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.3-B
- Why:
  - Mở `std.game` primitives để khóa deterministic tick/rng/state protocol trước khi sang shadow/engine.
- Scope:
  - Runtime observe cho `std.game.tick_info` và `std.game.rng`.
  - Runtime commit-gated cho `std.game.state_delta` (idempotency + size cap).
  - Bổ sung trace events cho game observe/commit.
  - Bổ sung reason codes game vào taxonomy runtime.
  - Mở rộng SDK parse/verify `[permissions.std_game]` theo fail-closed.
  - Thêm test targeted `tests/std_game.rs` + test manifest permissions game.
- Expected tests:
  - `cargo test --test std_game`
  - `cargo test --test ocl_manifest_fs_kv_time`
  - `cargo test --test ocl_stdlib`
  - `cargo test`
- Exit criteria:
  - `std.game.tick_info` deterministic theo single time source.
  - `std.game.rng` deterministic theo stream/count/cap và replay-stable.
  - `std.game.state_delta` commit-gated, enforce idempotency và size cap.
  - SDK permissions game fail-closed khi thiếu/disabled.

### 2026-03-04 — 7.3-B Implementation Closeout
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.3-B
- Implemented:
  - Hoàn tất runtime `std.game`:
    - `std.game.tick_info` trả payload deterministic `{tick, dt_ms}` theo `fixed_dt_ms`.
    - `std.game.rng` deterministic theo `stream/count/tick`, khóa thuật toán `PCG32`, output range `[0, 2^31-1]`, stream seed theo `fnv1a64("pcg32\\x00"+stream) XOR base_seed`.
    - `std.game.state_delta` commit-gated với `idempotency_key`, `delta_bytes` cap check, tạo `pending_write_id`.
  - Hoàn tất game audit path:
    - `GameObserve` emit cho observe keys `std.game.*`.
    - `GameCommit` emit khi commit `state_delta` được chấp nhận; idempotent apply theo `pending_write_id`.
  - Mở rộng taxonomy/registry:
    - thêm reason code `RC-GAME-DISABLED`, `RC-GAME-RNG-INVALID-STREAM`.
    - mở key contracts `std.game.rng`, `std.game.state_delta` trong registry commit/ctx rules.
  - Mở rộng SDK manifest permissions fail-closed:
    - parse/verify `[permissions.std_game]`.
    - deny rõ `RC-GAME-DISABLED` khi thiếu section hoặc `enabled=false`.
  - Bổ sung test targeted gate B:
    - file mới `tests/std_game.rs` (tick/rng/state_delta + trace/idempotency/cap).
    - mở rộng `tests/ocl_manifest_fs_kv_time.rs` cho permissions `std_game`.
    - cập nhật `tests/ocl_stdlib.rs` để tương thích payload `std.game.tick_info`.
- Files changed:
  - `src/ocp_ocl/diag.rs`
  - `src/ocp_ocl/audit.rs`
  - `src/ocp_ocl/registry.rs`
  - `src/ocp_ocl/exec.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
  - `tests/std_game.rs`
  - `tests/ocl_manifest_fs_kv_time.rs`
  - `tests/ocl_stdlib.rs`
  - `OCP-OCL-MVP-PLAN-v0.7.3.md`
- Commands run:
  - `cargo fmt`
  - `cargo test --test std_game`
  - `cargo test --test ocl_manifest_fs_kv_time`
  - `cargo test --test ocl_stdlib`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
- Test results:
  - Targeted tests (must-pass for gate):
    - `PASS` `std_game` (5/5)
    - `PASS` `ocl_manifest_fs_kv_time` (9/9)
    - `PASS` `ocl_stdlib` (13/13)
  - Regression tests (supporting only):
    - `PASS` full `cargo test` toàn workspace
    - `PASS` `cargo clippy --all-targets -- -D warnings`
    - `PASS` `cargo fmt -- --check`
  - Kết luận gate:
    - `DONE` (targeted tests pass + regression pass + quality gates pass)
- Notes/risks:
  - `std.game.state_delta` ở 7.3-B là no-effect audited commit hook (idempotent trace/apply), chưa ghi state host thật; đúng scope lock “integration hook tùy chọn”.
  - `delta` hiện đi qua `ctx("...")` string path (legacy), typed ctx AST vẫn defer theo roadmap sau.
  - Design alignment: FULL.

### 2026-03-04 — 7.3-C Planning Freeze
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.3-C
- Why:
  - Mở `std.shadow` primitive để khóa nhánh giả lập bounded/deterministic trước khi vào gate engine layer.
- Scope:
  - Runtime observe cho `std.shadow.run` (N branches bounded + deterministic + shadow isolation guard).
  - Runtime observe cho `std.shadow.compare` (diff/report deterministic + cap/truncation).
  - Bổ sung reason codes shadow vào taxonomy runtime.
  - Bổ sung audit events `ShadowRun` và `ShadowCompare`.
  - Mở rộng SDK parse/verify `[permissions.std_shadow]` theo fail-closed.
  - Thêm test targeted `tests/std_shadow.rs` + test manifest permissions shadow.
- Expected tests:
  - `cargo test --test std_shadow`
  - `cargo test --test ocl_manifest_fs_kv_time`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
- Exit criteria:
  - `std.shadow.run` trả branch list deterministic, signature stable cho cùng input.
  - `std.shadow.compare` trả report deterministic, bounded theo `max_diff_keys` và `max_report_bytes`.
  - Shadow isolation guard chặn effect keys theo policy và surfacing `RC-SHADOW-EFFECT-DISALLOWED`.
  - SDK permissions shadow fail-closed khi thiếu/disabled `[permissions.std_shadow]`.

### 2026-03-04 — 7.3-C Implementation Closeout
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.3-C
- Implemented:
  - Hoàn tất runtime `std.shadow`:
    - `std.shadow.run` trả danh sách branch deterministic theo `variants_json`, có caps `max_branches/branch_step_cap/branch_budget_cap`, có `branch_digest`, có cờ `truncated`.
    - `std.shadow.compare` dựng report deterministic (`diff_keys/divergence/cost_table/reason_table`), enforce cap `max_diff_keys` và `max_report_bytes` với truncation deterministic.
    - shadow isolation guard chặn effect keys trong context (`std.fs.*`, `std.kv.*`, `std.ui.present`, `std.game.state_delta`) và surfacing `INSUFFICIENT + RC-SHADOW-EFFECT-DISALLOWED`.
  - Bổ sung taxonomy/audit/registry cho shadow:
    - reason codes mới: `RC-SHADOW-DISABLED`, `RC-SHADOW-CAP-EXCEEDED`, `RC-SHADOW-REPORT-TOO-LARGE`, `RC-SHADOW-EFFECT-DISALLOWED`.
    - trace events mới: `ShadowRun`, `ShadowCompare` (runtime + SDK trace mapping/canonical payload).
    - registry contracts cho `std.shadow.run`/`std.shadow.compare`.
  - Mở rộng SDK permissions fail-closed:
    - parse/verify `[permissions.std_shadow]`.
    - deny rõ `RC-SHADOW-DISABLED` khi thiếu section hoặc `enabled=false`.
  - Bổ sung test targeted gate C:
    - file mới `tests/std_shadow.rs`.
    - mở rộng `tests/ocl_manifest_fs_kv_time.rs` cho permissions `std_shadow`.
- Files changed:
  - `src/ocp_ocl/diag.rs`
  - `src/ocp_ocl/audit.rs`
  - `src/ocp_ocl/registry.rs`
  - `src/ocp_ocl/exec.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
  - `tests/std_shadow.rs`
  - `tests/ocl_manifest_fs_kv_time.rs`
  - `OCP-OCL-MVP-PLAN-v0.7.3.md`
- Commands run:
  - `cargo fmt`
  - `cargo test --test std_shadow`
  - `cargo test --test ocl_manifest_fs_kv_time`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
- Test results:
  - Targeted tests (must-pass for gate):
    - `PASS` `std_shadow` (5/5)
    - `PASS` `ocl_manifest_fs_kv_time` (11/11)
  - Regression tests (supporting only):
    - `PASS` full `cargo test` toàn workspace
    - `PASS` `cargo clippy --all-targets -- -D warnings`
    - `PASS` `cargo fmt -- --check`
  - Kết luận gate:
    - `DONE` (targeted tests pass + regression pass + quality gates pass)
- Notes/risks:
  - Có 1 lần fail tạm thời ở test `std_shadow_compare_is_bounded_and_deterministic` do expectation chưa ép vượt cap; đã sửa test về cap=1 và rerun pass.
  - `std.shadow.run` hiện là deterministic primitive theo ctx/variants trong runtime hiện tại; engine integration đầy đủ tiếp tục ở gate 7.3-D.
  - Design alignment: FULL.

### 2026-03-04 — 7.3-D Planning Freeze
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.3-D
- Why:
  - Mở engine layer để gom protocol UI/game/shadow thành wrapper runtime thống nhất, giảm boilerplate phía project mà vẫn giữ deterministic + bounded + permission discipline.
- Scope:
  - Runtime wrappers:
    - `engine.ui.run`
    - `engine.game.run`
    - `engine.shadow.preview`
  - Registry contracts cho key engine mới (ctx-required + commit policy `false`).
  - SDK permission-gate cho `engine.*` map về pack gốc:
    - `engine.ui.*` -> `std_ui` phải enabled.
    - `engine.game.*` -> `std_game` phải enabled.
    - `engine.shadow.*` -> `std_shadow` phải enabled.
  - Targeted tests gate D:
    - `tests/engine_ui.rs`
    - `tests/engine_game.rs`
    - mở rộng `tests/ocl_manifest_fs_kv_time.rs` cho deny-path của `engine.*` khi thiếu permissions tương ứng.
- Expected tests:
  - `cargo test --test engine_ui`
  - `cargo test --test engine_game`
  - `cargo test --test ocl_manifest_fs_kv_time`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
- Exit criteria:
  - `engine.ui.run`, `engine.game.run`, `engine.shadow.preview` chạy deterministic theo cùng input.
  - Engine wrappers không yêu cầu project copy engine code vào `src/`.
  - Cấu hình qua ctx/config override hành vi wrapper mà không fork engine internals.
  - Permission fail-closed rõ ràng cho `engine.*` theo pack nền tảng tương ứng.

### 2026-03-04 — 7.3-D Implementation Closeout
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.3-D
- Implemented:
  - Hoàn tất engine layer wrappers trong runtime:
    - `engine.ui.run`: protocol payload deterministic (`entry_module`, `phase`, `tick`, `dt_ms`, `frame_info`, `input_events`, `state`, `draw`, `quit`), cap handling input/draw => `DEGRADED + RC-UI-CAP-EXCEEDED`.
    - `engine.game.run`: game loop payload deterministic (`tick_info` facade + `rng_values` + `state/draw/quit`), enforce stream/count policy; count vượt cap => `DEFERRED + RC-LIMIT-EXCEEDED`.
    - `engine.shadow.preview`: wrapper compose `shadow.run` + `shadow.compare`, giữ deterministic branch ordering/report và enforce shadow isolation (`RC-SHADOW-EFFECT-DISALLOWED`).
  - Cập nhật registry contracts:
    - mở family `engine`.
    - thêm ctx-required + commit policy cho:
      - `engine.ui.run`
      - `engine.game.run`
      - `engine.shadow.preview`
  - Cập nhật SDK permission gate fail-closed cho `engine.*`:
    - `engine.ui.*` yêu cầu `[permissions.std_ui].enabled=true`.
    - `engine.game.*` yêu cầu `[permissions.std_game].enabled=true`.
    - `engine.shadow.*` yêu cầu `[permissions.std_shadow].enabled=true`.
  - Bổ sung targeted tests gate D:
    - file mới `tests/engine_ui.rs` (2 tests).
    - file mới `tests/engine_game.rs` (3 tests, gồm shadow preview wrapper).
    - mở rộng `tests/ocl_manifest_fs_kv_time.rs` thêm 3 tests deny-path cho `engine.*` permissions.
- Files changed:
  - `src/ocp_ocl/exec.rs`
  - `src/ocp_ocl/registry.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
  - `tests/engine_ui.rs`
  - `tests/engine_game.rs`
  - `tests/ocl_manifest_fs_kv_time.rs`
  - `OCP-OCL-MVP-PLAN-v0.7.3.md`
- Commands run:
  - `cargo fmt`
  - `cargo test --test engine_ui`
  - `cargo test --test engine_game`
  - `cargo test --test ocl_manifest_fs_kv_time`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
- Test results:
  - Targeted tests (must-pass for gate):
    - `PASS` `engine_ui` (2/2)
    - `PASS` `engine_game` (3/3)
    - `PASS` `ocl_manifest_fs_kv_time` (14/14, gồm 3 test mới cho `engine.*`)
  - Regression tests (supporting only):
    - `PASS` full `cargo test` toàn workspace
    - `PASS` `cargo clippy --all-targets -- -D warnings`
    - `PASS` `cargo fmt -- --check`
  - Kết luận gate:
    - `DONE` (targeted tests pass + regression pass + quality gates pass).
- Notes/risks:
  - Gate D triển khai wrapper runtime `engine.*` additive, không thay đổi behavior key cũ `std.ui/std.game/std.shadow`.
  - Engine layer hiện ở dạng runtime wrapper deterministic; chuẩn module import `std/engine/*` có thể mở rộng sâu hơn ở gate sau nếu cần.
  - Design alignment: FULL.

### 2026-03-04 — 7.3-E Planning Freeze
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.3-E
- Why:
  - Hoàn tất DX của v0.7.3 bằng template CLI dùng ngay cho mini-game/shadow-preview, có run + replay E2E để chứng minh deterministic workflow từ project init.
- Scope:
  - Mở rộng `ocl init --template`:
    - `mini-game`
    - `shadow-preview`
  - Template manifest + source phải bám engine wrappers đã ship ở 7.3-D (`engine.game.run`, `engine.shadow.preview`) và bật permissions tương ứng (`std_ui/std_game/std_shadow`) theo fail-closed contract.
  - Bổ sung test E2E:
    - `tests/cli_mini_game_e2e.rs`
    - `tests/cli_shadow_preview_e2e.rs`
  - Kiểm chứng `ocl run` tạo artifact bundle và `ocl replay` signature match cho cả hai template.
- Expected tests:
  - `cargo test --test cli_mini_game_e2e`
  - `cargo test --test cli_shadow_preview_e2e`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
- Exit criteria:
  - `ocl init <dir> --template mini-game` và `shadow-preview` chạy thành công.
  - Cả hai template chạy `ocl run` thành công, tạo `.ocl_artifacts/<run_id>`.
  - `ocl replay <artifact_dir>` pass signature match cho cả hai template.
  - KPI file count (<=7 file trong project skeleton) đạt cho 2 template.
  - Design alignment: FULL.

### 2026-03-04 — 7.3-E Implementation Closeout
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.3-E
- Implemented:
  - Mở rộng CLI `init --template`:
    - thêm `mini-game`.
    - thêm `shadow-preview`.
    - cập nhật usage/help text cho danh sách template mới.
  - Thêm template generators:
    - `apply_mini_game_template_v073(...)`
    - `apply_shadow_preview_template_v073(...)`
  - Bổ sung auto-sync deps lock sau khi apply template:
    - `sync_deps_lock_v1(root)` được gọi ngay trong flow `ocl init --template ...` để tránh replay fail do lock mismatch.
  - Bổ sung E2E tests đúng scope Gate E:
    - file mới `tests/cli_mini_game_e2e.rs`.
    - file mới `tests/cli_shadow_preview_e2e.rs`.
  - E2E assertions:
    - init thành công.
    - skeleton file count `<=7`.
    - `ocl run` tạo artifact bundle (`audit.jsonl`, `signature.txt`, `replay.toml`).
    - `ocl replay` pass signature match cho artifact vừa tạo.
- Files changed:
  - `projects/ocp-ocl/crates/ocl-cli/src/main.rs`
  - `tests/cli_mini_game_e2e.rs`
  - `tests/cli_shadow_preview_e2e.rs`
  - `OCP-OCL-MVP-PLAN-v0.7.3.md`
- Commands run:
  - `cargo fmt`
  - `cargo test --test cli_mini_game_e2e`
  - `cargo fmt`
  - `cargo test --test cli_mini_game_e2e`
  - `cargo test --test cli_shadow_preview_e2e`
  - `cargo test --test cli_tool_cli_e2e`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
- Test results:
  - Targeted tests (must-pass for gate):
    - `PASS` `cli_mini_game_e2e` (1/1)
    - `PASS` `cli_shadow_preview_e2e` (1/1)
  - Regression tests (supporting only):
    - `PASS` `cli_tool_cli_e2e` (2/2)
    - `PASS` full `cargo test` toàn workspace
    - `PASS` `cargo clippy --all-targets -- -D warnings`
    - `PASS` `cargo fmt -- --check`
  - Temporary failures resolved:
    - Lần chạy đầu `cli_mini_game_e2e` fail do `deps.lock mismatch`.
    - Đã fix bằng auto `sync_deps_lock_v1` trong flow `init --template`, rerun pass.
  - Kết luận gate:
    - `DONE` (targeted tests pass + regression pass + quality gates pass).
- Notes/risks:
  - Không còn blocker mở cho gate 7.3-E.
  - Gate E chỉ mở rộng template/CLI additive; không thay semantics runtime core của std/engine keys.
  - Design alignment: FULL.

### 2026-03-04 — 7.3-F Planning Freeze
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.3-F
- Why:
  - Khóa lane `quarantine` tối thiểu theo spec v0.7.3 để tách cứng khỏi lane locked, có env gate rõ ràng và có lane marker trong audit trước khi mở rộng capability nondeterministic ở các phiên bản sau.
- Scope:
  - Thêm gate runtime/CLI: lane `quarantine` chỉ chạy khi `OCL_QUARANTINE=1`.
  - Replay cho artifact lane `quarantine` cũng phải đi qua env gate tương tự.
  - Thêm audit lane marker machine-readable cho artifact bundle v0.7.1 path.
  - Khóa chữ ký replay theo lane id để artifact `quarantine` không replay như locked.
  - Bổ sung targeted test file mới: `tests/quarantine_lane.rs`.
- Expected tests:
  - `cargo test --test quarantine_lane`
  - `cargo test --test cli_mini_game_e2e --test cli_shadow_preview_e2e`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
- Exit criteria:
  - Lane `quarantine` fail-closed khi thiếu `OCL_QUARANTINE=1`.
  - Lane `quarantine` pass khi có env gate và vẫn tạo đầy đủ artifacts.
  - `audit.jsonl` có lane marker cho run tương ứng.
  - Replay signature gắn lane id, không cho replay chéo lane.
  - Design alignment: FULL.

### 2026-03-04 — 7.3-F Implementation Closeout
- Date:
  - 2026-03-04
- Gate/Step:
  - 7.3-F
- Implemented:
  - Hoàn tất lane gate `quarantine` theo scope tối thiểu v0.7.3:
    - parse lane canonical (`locked_v06|locked_v071|quarantine`) và reject lane không hợp lệ.
    - enforce `quarantine` chỉ chạy khi `OCL_QUARANTINE=1` cho cả `ocl run` (project path) và `ocl replay`.
  - Cập nhật lock mode theo lane cho project-run path:
    - `locked_v071` => locked path.
    - `locked_v06` => legacy unlocked path.
    - `quarantine` => non-locked path và bắt buộc env gate.
  - Bổ sung lane marker machine-readable vào artifact audit:
    - thêm event dòng đầu `{\"t\":\"Lane\", ... \"lane\":\"<lane>\"}` trong `audit.jsonl`.
  - Khóa replay signature theo lane id:
    - signature bundle/replay dùng hash `lane + payload`, không còn chỉ digest thuần của trace.
    - đảm bảo artifact `quarantine` không replay như lane locked.
  - Bổ sung targeted tests gate F:
    - file mới `tests/quarantine_lane.rs` (2 test):
      - quarantine require env + audit marker + replay gate.
      - locked lane không cần env quarantine.
- Files changed:
  - `projects/ocp-ocl/crates/ocl-cli/src/main.rs`
  - `tests/quarantine_lane.rs`
  - `OCP-OCL-MVP-PLAN-v0.7.3.md`
- Commands run:
  - `cargo test --test quarantine_lane`
  - `cargo test --test cli_mini_game_e2e --test cli_shadow_preview_e2e`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check` (chưa pass lần đầu)
  - `cargo fmt`
  - `cargo fmt -- --check` (PASS sau format)
- Test results:
  - Targeted tests (must-pass for gate):
    - `PASS` `quarantine_lane` (2/2).
  - Regression tests (supporting only):
    - `PASS` `cli_mini_game_e2e` (1/1).
    - `PASS` `cli_shadow_preview_e2e` (1/1).
    - `PASS` full `cargo test` toàn workspace.
    - `PASS` `cargo clippy --all-targets -- -D warnings`.
    - `PASS` `cargo fmt -- --check` (sau khi chạy `cargo fmt`).
  - Kết luận gate:
    - `DONE` (targeted tests pass + regression pass + quality gates pass).
- Notes/risks:
  - Rule lane `quarantine` hiện là gate tối thiểu (env + marker + signature boundary), chưa mở capability nondeterministic mới trong gate này.
  - Lane không hợp lệ giờ fail-fast bằng lỗi `V-LANE-INVALID`, tránh drift âm thầm.
  - Design alignment: FULL.

### YYYY-MM-DD — 7.3-X Planning Freeze
- Date:
- Gate/Step:
- Why:
- Scope:
- Expected tests:
- Exit criteria:

### YYYY-MM-DD — 7.3-X implementation closeout
- Date:
- Gate/Step:
- Implemented:
- Files changed:
- Commands run:
  - `cargo test --test <targeted_suite>`
  - `cargo test`
- Test results:
  - `PASS`/`FAIL`
- Notes/risks:

---

