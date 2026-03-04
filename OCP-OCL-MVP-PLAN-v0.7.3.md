# OCL v0.7.3 — Expansion Plan (Consumer-grade UI/Game + Shadow Primitive + Engines/Modules)

Ngày tạo: 2026-03-03  
Trạng thái: `DRAFT (LOCK WHEN CODING)`  
Phạm vi: **OCL-only**.  
Tiền đề: v0.7.1 đã có core + DX + replay-first + manifest; v0.7.2 đã có tool-grade IO packs (`std.fs/std.kv/std.time`) và fixture IO runner.

Mục tiêu v0.7.3: biến OCL thành nền làm **tool app/game** dễ khoe, dễ dùng, ít code, có “wow” mà ngôn ngữ phổ thông làm được nhưng tốn infrastructure:
- UI theo mô hình **draw-list** (không nhét widget framework vào core),
- game loop primitives (deterministic RNG + state delta),
- **shadow branching primitive** + compare report chuẩn,
- layer “engine/modules” để **lắp** và **tùy biến** mà không phình project,
- thêm lane `quarantine` tối thiểu (tùy chọn) cho những capability không deterministic (chưa cần net; chủ yếu để mở đường).

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

### 5.2 Keyspace
**Observe**
- `std.ui.frame_info`
  - ctx: `{}`
  - payload: `{ w:Int, h:Int, scale:Int, theme:String, locale:String }`
- `std.ui.input`
  - ctx: `{ cap:Int }` (must be <= max_input_events)
  - payload: `{ events: List<Record{ t:String, a:Value }>, truncated:Bool }`
  - determinism: events order preserved; cap truncation => `DEGRADED`

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
- `std.game.rng`
  - ctx: `{ stream:String, count:Int }` (count <= rng_max_count; stream must be allowed)
  - payload: `{ values: List<Int> }`
  - determinism: values derived from seed + tick + stream

**Commit**
- `std.game.state_delta`
  - ctx: `{ idempotency_key:String, delta:Value }`
  - policy: serialized delta bytes <= state_delta_max_bytes
  - determinism: idempotency_key ensures replay safety

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
  - `entry` module/function reference (string literal) OR `(program_ref, entry)` (v0.7.3 simplest: current program entry + injected params)
  - `variants`: List<Value> (each variant = injected params record)
  - caps: branches/step/budget
- output:
  - `{ branches: List<BranchResult>, truncated:Bool }`

BranchResult:
- `{ id:Int, outcome:Kind, reason_code?:String, signature:String, cost:{steps:Int,budget:Int}, state_summary:Value }`

Rules:
- copy-on-write env/state per branch (no cross-branch mutation)
- deterministic scheduling: run branches in order id=0..N-1
- early abort if global cap reached => truncated + DEFERRED reason

### 7.3 Report: shadow.compare
Input:
- list `BranchResult` + baseline branch id
Output:
- `{ diff_keys: List<String>, divergence:Value, cost_table:List<Record>, reason_table:List<Record> }`
Bounded:
- diff_keys <= max_diff_keys
- report bytes <= max_report_bytes else DEGRADED

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
- init state (theme, viewport)
- per-frame:
  - observe frame_info
  - observe input
  - call app hooks: `on_input`, `on_update`, `on_render`
  - commit draw list + present
Caps:
- enforce max_draw_cmds, max_input_events

Public API:
- `engine.ui.run(app, config)` where:
  - `app` is record of callbacks `{ on_init, on_input, on_update, on_render }`
  - `config` includes caps + theme tokens

### 8.3 engine.ui.widgets (optional minimal set)
- button, list, textbox (pure layout + draw commands)
- state is explicit record to keep replay/shadow diff simple
- no complex retained-mode framework

### 8.4 engine.game.loop
- integrate std_game.tick_info + rng + state_delta
- `engine.game.run(game, config)`:
  - callbacks: `on_init`, `on_tick`, `on_render`
  - state transitions explicit
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
- state_delta commit (idempotency)
Tests:
- `tests/std_game.rs`
Exit criteria:
- rng stable across replay
- state_delta size cap enforced

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

### YYYY-MM-DD — 7.3-X planning
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
- Test results:
- Notes/risks:

---

