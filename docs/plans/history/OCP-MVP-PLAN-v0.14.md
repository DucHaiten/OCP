# OCP v0.14 — Global Conformance Suite + Stability Sprint (Behavior Contracts at Scale)

Ngày tạo: 2026-03-03  
Trạng thái: `DONE (14-A..14-G DONE)`  
Phạm vi: **OCP-only**.  
Tiền đề: v0.7–v0.13 đã hình thành core + packs + engines + quarantine+cassette + schema typing + deps/lock/signing + debugger + IR/caching.

Mục tiêu v0.14: đưa OCP lên mức “đủ tin cậy để người khác dùng” bằng:
- **Conformance suite tổng** (hợp đồng hành vi), không chỉ unit tests,
- **Stability sprint** (ổn định API/semantics), giảm breaking,
- **Compatibility gates** (đảm bảo packs/permissions/replay/shadow không drift).

---

## 0) Governance + Tracking v0.14

### 0.1 Quy ước cập nhật bắt buộc
- Mọi thay đổi kế hoạch phải cập nhật file này trước khi code.
- Không nhảy gate: gate sau chỉ mở khi gate hiện tại đạt điều kiện đóng.
- Chỉ chuyển gate sang `DONE` khi có đủ:
  - Planning Freeze + Implementation Closeout.
  - `Files changed`, `Commands run`, `Test results`, `Notes/risks`.
  - Targeted tests pass cho đúng scope gate.
- Nếu chưa đủ điều kiện:
  - bắt buộc giữ `TODO` hoặc `IN_PROGRESS` hoặc `PARTIAL`.
  - không được ghi `DONE`.

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
- Targeted tests (must-pass for gate): `PASS`/`FAIL`
- Regression tests (supporting only): `PASS`/`FAIL`
- Kết luận gate: `DONE` chỉ khi targeted tests pass
- Design alignment: `FULL` hoặc `PARTIAL` (nêu rõ lý do)
- Notes/risks:

### 0.2 Quick Snapshot (bắt buộc đọc trước)
- Mục tiêu phiên bản:
  - Thiết lập global conformance suite + stability sprint để khóa hợp đồng hành vi ở quy mô toàn hệ.
- Trạng thái tổng quan:
  - `DONE (14-A..14-G DONE)`.
- Gate đang làm/đã xong/chưa làm:
  - Gate 14-A, 14-B, 14-C, 14-D, 14-E, 14-F, 14-G đã hoàn tất theo scope.
- Bước kế tiếp ngay:
  - chốt release note v0.14.x và chuẩn bị handoff v0.15 theo contract additive.
- Lệnh kiểm chứng chuẩn:
  - xem `13) Operational commands (v0.14)`.
- File code trọng yếu sẽ bị tác động:
  - `projects/ocp/conformance/conformance.v5.toml`
  - `projects/ocp/conformance/expected/contracts/stability_contract_v14.json`
  - `projects/ocp/crates/ocp-cli/src/main.rs`
  - `tests/conformance_cache.rs`
  - `tests/conformance_runner.rs`
  - `tests/stability_contracts_v14.rs`
  - `tests/upgrade_check.rs`
  - `Cargo.toml`

### 0.3 Trạng thái Workstreams/Gates v0.14 (tracking)
#### Workstreams
- WS-CF (conformance framework + suites): `DONE` (14-A..14-E)
- WS-ST (stability sprint + compat snapshots): `DONE` (14-G)
- WS-UT (upgrade-check tooling): `DONE` (14-F)

#### Gate status (14-A .. 14-G)
- Gate 14-A (Conformance runner core): `DONE` (2026-03-05)
- Gate 14-B (Populate core conformance suite): `DONE` (2026-03-05)
- Gate 14-C (Packs conformance suites): `DONE` (2026-03-05)
- Gate 14-D (Supply-chain conformance): `DONE` (2026-03-05)
- Gate 14-E (Perf/cache conformance): `DONE` (2026-03-05)
- Gate 14-F (upgrade-check tool): `DONE` (2026-03-05)
- Gate 14-G (Stability sprint process gate): `DONE` (2026-03-05)

---

## 1) Goals v0.14 (LOCKED)

### 1.1 North Star
- Mọi thay đổi trong core/packs/engines/tooling phải pass conformance suite.
- Conformance mô tả **behavior-level**:
  - permissions semantics,
  - replay determinism,
  - cassette determinism-by-recording,
  - shadow boundedness + scheduling invariants,
  - schema/typing guarantees,
  - lock/deps integrity.
- Người khác có thể lấy OCP runtime và tin rằng:
  - template chạy đúng,
  - upgrades không “âm thầm đổi hành vi” ngoài phạm vi công bố.

### 1.2 KPI bắt buộc (định lượng)
**KPI-1: Coverage**
- Conformance suite cover tối thiểu:
  - Core language: parse/type/exec + Value/JSON/import/sugar/loops
  - Permissions: deny/allow precedence, per-package requested/granted
  - Replay: locked determinism signature
  - Quarantine: cassette record/replay correctness
  - Shadow: caps + determinism + report boundedness
  - IR/cache: cache does not change signature
- Tối thiểu 1 conformance bundle cho mỗi pack nhóm: fs/kv/time, ui/game, shadow, net/proc/wallclock.

**KPI-2: Stability sprint outcome**
- 3 release liên tiếp trong v0.14.x (ví dụ 0.14.0/0.14.1/0.14.2) không breaking core syntax/semantics.
- Nếu có breaking: phải có migration tool/note và conformance updated + version gates.
- Machine-check rule (bắt buộc):
  - snapshot contracts mỗi release: `manifest_schema_version`, `trace_schema_version`, `pack key list + ctx/payload schema hashes`.
  - CI so snapshot diff:
    - chỉ cho additive diffs,
    - nếu không additive thì phải bump version + có migration tool.

**KPI-3: Upgrade safety**
- `ocp upgrade-check <project>`:
  - chạy conformance subset relevant
  - báo changeset (behavior diff) nếu có
- Target false positives thấp.

---

## 2) Axis Lock v0.14 (bất biến)
- Conformance is the new gate of truth.  
- Behavior contracts take precedence over “implementation convenience”.

---

## 3) Scope v0.14

### 3.1 In-scope (ship)
A) **Conformance framework**
- Format conformance bundles:
  - fixtures (source)
  - expected outcomes (errors/kinds/reasons)
  - expected signatures
  - expected artifacts (optional)
  - w9 scenario manifest là source-of-truth cho runner trong v0.14
- Runner:
  - `ocp test --conformance run [--suite ...]` (canonical)
  - `ocp test --conformance list` (canonical)
- Additive CLI aliases (compat):
  - `ocp conformance run` = alias của `ocp test --conformance run`
  - `ocp conformance list` = alias của `ocp test --conformance list`
- Profiles:
  - `core`
  - `packs-tool`
  - `packs-consumer`
  - `shadow`
  - `quarantine`
  - `deps-supplychain`
  - `perf-cache` (behavior invariant)

B) **Golden artifact strategy**
- Golden signature snapshots
- Golden cassette bundles for quarantine tests
- Golden shadow report samples
- Golden diff outputs for trace diff

C) **Stability sprint process**
- Freeze windows:
  - freeze core syntax
  - freeze manifest schema
  - freeze pack key contracts
- Introduce compat versioning:
  - trace schema versions
  - manifest schema versions
  - pack schema versions

D) **Compatibility gates**
- Hard gates:
  - determinism signatures must match
  - deny/allow precedence must match
  - cassette replay must match
  - shadow caps must enforce
- Soft gates:
  - performance (no regression above threshold)
  - report formatting (minor differences allowed if schema same) — optional

E) **Upgrade-check tool**
- `ocp upgrade-check`:
  - evaluates current runtime vs target runtime (or new build)
  - runs subset of conformance on project’s used packs
  - reports:
    - breaking changes
    - behavior diffs
    - migration hints
  - lock:
    - command này thuộc scope Gate 14-F
    - Gates 14-A..14-E không được phụ thuộc vào `ocp upgrade-check`

### 3.2 Out-of-scope (defer)
- Full third-party certification programs.
- Online registry governance (policy docs can be later).

### 3.3 Module -> Crate -> Path mapping (LOCKED)
- w9 scenario parser + `expected_signature_file` extension:
  - crate: `ocp-sdk`
  - path: `projects/ocp/crates/ocp-sdk/src/w9.rs`
- Canonical CLI `ocp test --conformance ...` + aliases `ocp conformance ...`:
  - crate: `ocp-cli`
  - path: `projects/ocp/crates/ocp-cli/src/main.rs`
- Signature reading/comparison + sandbox runner wiring for conformance:
  - crate: `ocp-sdk` (SoT cho runner behavior), CLI chỉ gọi vào SDK
  - path: `projects/ocp/crates/ocp-sdk/src/w9.rs`
- `ocp upgrade-check` command:
  - crate: `ocp-cli`
  - path: `projects/ocp/crates/ocp-cli/src/main.rs`

---

## 4) Conformance Bundle Format (v0.14)

### 4.0 Migration contract with existing w9 runner (LOCKED)
- v0.14 chọn hướng additive, không thay runner SoT:
  - runtime execution vẫn dựa trên manifest `[[scenario]]` (w9-compatible).
  - không mở format runtime mới tách rời trong Gate 14-A.
- `case.toml` (nếu có) chỉ là metadata cho tác giả/contributor.
- Nếu tương lai cần format runtime mới:
  - phải mở gate migration riêng (`w9 -> bundle_vNext`) với converter và rollback path rõ ràng.

### 4.1 Directory layout
`conformance/` contains suites:

- `conformance/core/`
- `conformance/packs-tool/`
- `conformance/packs-consumer/`
- `conformance/shadow/`
- `conformance/quarantine/`
- `conformance/deps-supplychain/`
- `conformance/perf-cache/`

Each test case dir:
- `scenario.toml` (canonical, theo `[[scenario]]` w9 runner)
- `case.toml` (optional metadata cho người đọc; không phải source-of-truth runtime)
- `project/` (mini project with ocp.toml + sources)
- `expected/`
  - `status.json` (pass/fail)
  - `signature.txt`
  - `errors.json` (if fail)
  - `outputs/` (optional)
  - `cassette/` (required cho quarantine replay cases)
    - `cassette.jsonl`
    - `cassette_index.json`
    - `cassette_meta.toml`
  - `shadow_report.json` (optional)

### 4.2 `scenario.toml` schema (canonical for runner)
```toml
schema = "ocp.conformance.v14"

[[scenario]]
name = "core_try_else_001"
path = "project"
expected_status = "pass"
expected_error_code = ""
expected_signature_file = "expected/signature.txt"
steps = ["run"]
lane = "locked_v071"
```

Optional:
- expected error code/kind
- expected reasons breakdown
- expected report schema version
- `case.toml` nếu có chỉ dùng cho metadata/ghi chú, không điều khiển runner.
- lane literals hợp lệ: `locked_v071`, `locked_v06`, `quarantine`; từ `locked` chỉ là alias mô tả trong tài liệu.
- với quarantine replay case, `steps` phải chứa `replay_artifact` (không dùng record step trong CI).

### 4.3 Runner behavior
- Builds & runs case in isolated sandbox.
- Executes `scenario.toml` as runner source-of-truth (w9-compatible flow + v0.14 extensions).
- Signature conformance (bắt buộc):
  - runner phải đọc `expected_signature_file` theo từng scenario.
  - so sánh bình đẳng với `signature.txt` thực tế từ artifacts.
  - mismatch => `X-CONFORMANCE-SIGNATURE-MISMATCH`.
- For `locked_v071` / `locked_v06`:
  - must reproduce expected signature exactly.
- For `quarantine`:
  - CI chỉ chạy replay bằng golden cassette (không dùng record mode).
  - step enum mapping (LOCKED): thêm step `replay_artifact` vào w9 step enum theo hướng additive.
  - semantics của `replay_artifact`:
    1. copy `expected/cassette/*` -> `./.ocp_artifacts/<run_id>/cassette/*`
    2. verify `cassette_hash` khớp `replay.toml` nếu `replay.toml` có trường hash
    3. chạy `ocp replay <artifact_dir>` và so signature equality
  - không dùng record step trong CI quarantine conformance.

---

## 5) Core Conformance (what must be locked)

### 5.1 Language behavior contracts
- parse errors codes + span correctness for known fixtures
- type error codes for schema/ctx/effects
- exec step caps and loop caps enforcement
- try/else/guard desugaring semantics (signature equivalence)
- record/map ordering rules affecting stringify/signature

### 5.2 Audit/trace contracts
- event schema version and minimal required fields
- signature definition (canonicalization)
- trace index generation stable

---

## 6) Packs Conformance

### 6.1 packs-tool suite
Covers:
- std.fs: read/list/stat/write (sandbox allowlist)
- std.kv: get/keys/put/del with caps
- std.time: tick_info logical
Contracts:
- permission gating (deny > allow)
- deterministic ordering in list_dir/keys
- bounded truncation uses DEGRADED consistently

### 6.2 packs-consumer suite
Covers:
- std.ui: frame_info/input/draw/present with caps, input replay
- std.game: tick/rng/state_delta
Contracts:
- draw-list caps enforce
- RNG deterministic for same seed/tick/stream
- replay reproduces input

### 6.3 shadow suite
Covers:
- shadow.run basic
- shadow.search scheduling (beam/portfolio)
- report v2 bounded truncation
Contracts:
- caps enforce and outcomes stable
- scheduling tie-break deterministic
- cache reuse does not change results

### 6.4 quarantine suite
Covers:
- std.net.http.request replay via cassette
- std.proc.exec replay via cassette
- std.time.wallclock replay via cassette
Contracts:
- missing cassette => INSUFFICIENT with correct RC
- cassette miss => INSUFFICIENT RC-CASSETTE-MISS
- redaction markers present if configured

---

## 7) Supply-chain Conformance (deps-supplychain)

Covers:
- package hashing canonicalization
- lockfile resolution determinism
- signature verification (trusted/untrusted keys)
- per-package requested_permissions intersection with project grants

Contracts:
- least privilege enforced
- denied permissions produce DX message containing package name/version and hint
- output assertions tối thiểu (machine-checkable, byte-equal):
  - `expected/outputs/deps.json`
  - `expected/outputs/permissions_effective.json`
  - `expected/signature.txt`

---

## 8) Perf/cache Conformance (behavior invariants)

Goal:
- ensure caching does not change semantics.

Approach:
- run same cases with `--no-cache` and with cache
- require:
  - signature identical
  - cache telemetry appears only in cached run
- optional threshold:
  - cached run must have fewer eval counts (soft gate)

Lock with v0.13 contracts:
- semantic signature input remains unchanged between cached/non-cached runs.
- cache hit/miss telemetry is non-semantic and must be written outside semantic audit stream
  (for example `perf_cache.jsonl` or equivalent artifact), then consumed by conformance/reporting.

---

## 9) Stability Sprint Plan (v0.14.x)

### 9.1 Freeze targets
Freeze (no breaking unless exceptional):
- core syntax
- manifest schema keys (additive only)
- pack key names + ctx/payload schema (breaking only with version bump and migration)
- trace schema fields (additive only)

### 9.2 Versioning strategy
- `manifest_schema_version`
- `trace_schema_version`
- `pack_schema_version` per pack
- `engine_api_version` (optional)

### 9.3 Breaking change protocol
If absolutely necessary:
- bump appropriate schema version
- ship migration tool:
  - `ocp migrate manifest`
  - `ocp migrate trace`
  - `ocp migrate pack`
- conformance bundles updated with dual-version acceptance where needed

### 9.4 Release contract snapshot check (bắt buộc)
- Mỗi release v0.14.x phải xuất snapshot contract:
  - `manifest_schema_version`
  - `trace_schema_version`
  - `pack key list + ctx/payload schema hashes`
- CI so snapshot giữa release trước và release mới:
  - additive-only => pass
  - non-additive => phải có version bump + migration evidence, nếu thiếu thì fail gate

---

## 10) Tooling: upgrade-check (v0.14)

### 10.1 `ocp upgrade-check`
Inputs:
- project dir
- target runtime version/build

Behavior:
- detects used packs/engines/modules from imports & manifest
- selects relevant conformance subset
- executes subset via canonical runner path `ocp test --conformance ...`
- runs against target runtime in sandbox
Outputs:
- pass/fail summary
- behavior diffs:
  - signature mismatch with first divergence point (reuse trace diff)
- migration hints:
  - which schema version changed
  - recommended commands

---

## 11) Error taxonomy v0.14 (additive)
- `X-CONFORMANCE-SIGNATURE-MISMATCH`
- `X-CONFORMANCE-OUTPUT-MISMATCH`
- `X-UPGRADE-CHECK-FAILED`
- `X-SCHEMA-VERSION-UNSUPPORTED`

---

## 12) Execution gates v0.14 (triển khai tuần tự)

### Gate 14-A — Conformance runner core
Scope:
- w9 scenario parser as source-of-truth + v0.14 extension `expected_signature_file`
- sandbox runner
- per-scenario signature compare (actual vs expected signature file)
- canonical CLI path: `ocp test --conformance run|list`
- additive aliases: `ocp conformance run|list`
- additive step enum extension: `replay_artifact` (for quarantine replay flow)
- cassette injection path contract + `cassette_hash` verification before replay
Tests:
- `tests/conformance_runner.rs`
Exit criteria:
- can run core suite cases and validate signatures

### Gate 14-B — Populate core conformance suite
Scope:
- add core fixtures for parse/type/exec
- expected signatures
Tests:
- `tests/conformance_core.rs`
Exit criteria:
- core suite stable

### Gate 14-C — Packs conformance suites
Scope:
- packs-tool + packs-consumer cases
- shadow cases
- quarantine replay cases with golden cassettes (inject cassette -> `ocp replay`, no record in CI)
Tests:
- `tests/conformance_packs.rs`
Exit criteria:
- all packs suites pass deterministically

### Gate 14-D — Supply-chain conformance
Scope:
- deps-supplychain cases
- signing/trust cases
- expected output assertions (`deps.json`, `permissions_effective.json`, `signature.txt`)
Tests:
- `tests/conformance_supplychain.rs`
Exit criteria:
- lock/hash/signing behavior locked

### Gate 14-E — Perf/cache conformance (behavior invariants)
Scope:
- run subset with and without cache
- enforce signature invariants
- verify cache telemetry from non-semantic artifact stream (không đổi semantic signature input)
Tests:
- `tests/conformance_cache.rs`
Exit criteria:
- cache never changes semantics

### Gate 14-F — upgrade-check tool
Scope:
- implement `ocp upgrade-check`
- integrate trace diff for mismatch reporting
- use canonical conformance runner path (`ocp test --conformance ...`) under the hood
Tests:
- `tests/upgrade_check.rs`
Exit criteria:
- upgrade-check identifies and reports divergence correctly

### Gate 14-G — Stability sprint (process gate)
Scope:
- freeze docs
- enforce “additive only” lints for schemas
- require conformance pass for every change
Exit criteria:
- v0.14.x releases meet no-breaking core condition

---

## 13) Operational commands (v0.14)
- `cargo test`
- `cargo test --test conformance_runner`
- `cargo test --test conformance_core`
- `cargo test --test conformance_packs`
- `cargo test --test conformance_supplychain`
- `cargo test --test conformance_cache`
- `cargo test --test upgrade_check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt -- --check`
- `ocp test --conformance run --suite all` (canonical in CI pipeline)
- `ocp conformance run --suite all` (alias, optional)

---

## 14) Execution Log (full-log template)

### 2026-03-05 — 14-A Planning Freeze
- Date: 2026-03-05
- Gate/Step: 14-A
- Why:
  - Khóa runner core theo hợp đồng v0.14, tránh drift giữa CLI surface mới và w9 runner hiện có.
- Scope:
  - `ocp-sdk/w9`:
    - thêm `lane`, `expected_signature_file`, step `replay_artifact`.
    - signature compare theo scenario (`X-CONFORMANCE-SIGNATURE-MISMATCH`).
    - cassette injection + `cassette_hash` verify cho `replay_artifact`.
  - `ocp-cli`:
    - canonical conformance surface qua `test --conformance [run|list]`.
    - alias additive `conformance <run|list>` forward về đường canonical.
  - Test targeted Gate 14-A:
    - thêm `tests/conformance_runner.rs`.
    - thêm CLI unit tests cho `list` alias/canonical và `run` subcommand.
- Expected tests:
  - `cargo test --test conformance_runner`
  - `cargo test -p ocp-cli w14_cli_conformance_list_alias_json_pass -- --nocapture`
  - `cargo test -p ocp-cli w14_cli_test_conformance_list_mode_pass -- --nocapture`
  - `cargo test -p ocp-cli w14_cli_test_conformance_run_subcommand_pass -- --nocapture`
  - `cargo test -p ocp-cli w9_cli_conformance_single_scenario_pass -- --nocapture`
- Exit criteria:
  - parser/runner nhận được fields v0.14 của conformance core.
  - CLI canonical + alias chạy ổn, không phá flow cũ.
  - targeted tests pass.

### 2026-03-05 — 14-A Implementation Closeout
- Date: 2026-03-05
- Gate/Step: 14-A
- Implemented:
  - `ocp-sdk`:
    - mở rộng `ConformanceScenarioV1` với `lane`, `expected_signature_file`.
    - thêm step `ConformanceStepV1::ReplayArtifact`.
    - parse/build/validate lane (`locked_v071|locked_v06|quarantine`, normalize alias `locked`).
    - thêm signature compare theo expected signature file.
    - thêm cassette injection + hash verify cho step `replay_artifact`.
  - `ocp-cli`:
    - thêm command alias `conformance`.
    - hỗ trợ `list` mode cho `test --conformance [list]`.
    - giữ tương thích đường cũ `test --conformance ...`.
    - cập nhật help cho canonical/alias surface.
  - Tests:
    - tạo mới `tests/conformance_runner.rs`.
    - bổ sung 3 CLI unit tests cho v0.14 conformance modes.
- Files changed:
  - `projects/ocp/crates/ocp-sdk/src/w9.rs`
  - `projects/ocp/crates/ocp-cli/src/main.rs`
  - `tests/conformance_runner.rs`
- Commands run:
  - `cargo test --test conformance_runner`
  - `cargo test -p ocp-cli w14_cli_conformance_list_alias_json_pass -- --nocapture`
  - `cargo test -p ocp-cli w14_cli_test_conformance_list_mode_pass -- --nocapture`
  - `cargo test -p ocp-cli w14_cli_test_conformance_run_subcommand_pass -- --nocapture`
  - `cargo test -p ocp-cli w9_cli_conformance_single_scenario_pass -- --nocapture`
- Test results:
  - Targeted tests (must-pass for gate): `PASS`
  - Regression tests (supporting only): `PASS`
- Kết luận gate:
  - `DONE` (đã đạt exit criteria của Gate 14-A theo scope đã khóa).
- Design alignment:
  - `FULL`
- Notes/risks:
  - Step `replay_artifact` ở gate này tập trung vào parser + inject + hash verify theo hợp đồng runner core.
  - Bộ quarantine replay scenario đầy đủ (gắn packs nondet) sẽ được mở rộng ở Gate 14-C.

### 2026-03-05 — 14-B Planning Freeze
- Date: 2026-03-05
- Gate/Step: 14-B
- Why:
  - Bổ sung core conformance fixtures parse/type/exec để khóa contract hành vi nền tảng trước khi mở packs suites.
- Scope:
  - Thêm 4 core scenarios trong `conformance.v5`:
    - `core-exec-pass` (có expected signature file),
    - `core-type-fail`,
    - `core-parse-fail`,
    - `core-exec-fail`.
  - Tạo fixture projects tương ứng trong `projects/ocp/conformance/fixtures/core-*`.
  - Thêm targeted test `tests/conformance_core.rs` để:
    - xác nhận core scenarios đã được gắn vào manifest,
    - chạy subset core qua runner và yêu cầu toàn bộ pass theo expected contracts.
- Expected tests:
  - `cargo test --test conformance_core`
  - `cargo run -p ocp-cli -- test --conformance run --manifest projects/ocp/conformance/conformance.v5.toml --json`
- Exit criteria:
  - Core suite có đầy đủ parse/type/exec fixtures và expected signature cho case pass.
  - Targeted test `conformance_core` pass.
  - Full manifest run qua conformance runner cho kết quả toàn bộ scenarios pass.

### 2026-03-05 — 14-B Implementation Closeout
- Date: 2026-03-05
- Gate/Step: 14-B
- Implemented:
  - Mở rộng `conformance.v5` với bộ core scenarios parse/type/exec.
  - Tạo fixtures:
    - `core-exec-pass` (pass flow + expected signature),
    - `core-type-fail` (type mismatch contract),
    - `core-parse-fail` (parse error contract),
    - `core-exec-fail` (runtime condition false contract).
  - Gắn expected signature cho `core-exec-pass`:
    - `projects/ocp/conformance/fixtures/expected/core-exec-pass.signature.txt`
  - Thêm targeted test `tests/conformance_core.rs` để khóa contract và chạy subset core trực tiếp.
- Files changed:
  - `projects/ocp/conformance/conformance.v5.toml`
  - `projects/ocp/conformance/fixtures/core-exec-pass/Ocp.toml`
  - `projects/ocp/conformance/fixtures/core-exec-pass/src/main.ocp`
  - `projects/ocp/conformance/fixtures/core-type-fail/Ocp.toml`
  - `projects/ocp/conformance/fixtures/core-type-fail/src/main.ocp`
  - `projects/ocp/conformance/fixtures/core-parse-fail/Ocp.toml`
  - `projects/ocp/conformance/fixtures/core-parse-fail/src/main.ocp`
  - `projects/ocp/conformance/fixtures/core-exec-fail/Ocp.toml`
  - `projects/ocp/conformance/fixtures/core-exec-fail/src/main.ocp`
  - `projects/ocp/conformance/fixtures/expected/core-exec-pass.signature.txt`
  - `tests/conformance_core.rs`
- Commands run:
  - `cargo test --test conformance_core`
  - `cargo run -p ocp-cli -- test --conformance run --manifest projects/ocp/conformance/conformance.v5.toml --json`
  - `cargo test --test conformance_runner`
- Test results:
  - Targeted tests (must-pass for gate): `PASS`
  - Regression tests (supporting only): `PASS`
- Kết luận gate:
  - `DONE` (đã đạt exit criteria Gate 14-B theo scope đã khóa).
- Design alignment:
  - `FULL`
- Notes/risks:
  - Case `core-exec-pass` phụ thuộc chữ ký expected; khi semantics core thay đổi ở phiên bản sau phải cập nhật signature có kiểm soát qua gate phù hợp.

### 2026-03-05 — 14-C Planning Freeze
- Date: 2026-03-05
- Gate/Step: 14-C
- Why:
  - Mở rộng conformance từ core sang packs/shadow/quarantine để khóa hợp đồng hành vi toàn diện trước khi vào supply-chain gate.
- Scope:
  - Bổ sung scenarios v5 cho:
    - packs-tool (`packs-tool-fs-read`)
    - packs-consumer (`packs-consumer-ui`)
    - shadow (`shadow-search-rr`)
    - quarantine replay (`quarantine-wallclock-replay`)
  - Tạo fixtures tương ứng cho từng scenario.
  - Nâng cấp `replay_artifact` trong runner để:
    - inject cassette,
    - chạy replay thật bằng runtime replay env,
    - ghi/đối chiếu signature replay trong lane quarantine.
  - Thêm targeted test `tests/conformance_packs.rs`:
    - xác nhận đủ scenario packs/shadow/quarantine,
    - chạy subset 2 lần và yêu cầu digest ổn định.
- Expected tests:
  - `cargo test --test conformance_packs`
  - `cargo test --test conformance_runner`
  - `cargo run -p ocp-cli -- test --conformance run --manifest projects/ocp/conformance/conformance.v5.toml --json`
- Exit criteria:
  - Packs/shadow/quarantine conformance scenarios pass theo v5 manifest.
  - Quarantine case dùng `replay_artifact` với golden cassette và pass signature contract.
  - Targeted test xác nhận determinism qua 2 lần chạy subset.

### 2026-03-05 — 14-C Implementation Closeout
- Date: 2026-03-05
- Gate/Step: 14-C
- Implemented:
  - Mở rộng v5 manifest với 4 scenarios mới:
    - `packs-tool-fs-read`
    - `packs-consumer-ui`
    - `shadow-search-rr`
    - `quarantine-wallclock-replay`
  - Tạo fixtures cho packs/shadow/quarantine:
    - project manifests + source scripts + data/cassette bundle.
  - Nâng cấp runner `replay_artifact`:
    - không chỉ copy cassette, mà chạy replay thật với env replay (`OCP_QUARANTINE_MODE=replay`, `OCP_V08_CASSETTE_*`),
    - tính signature replay và ghi `.ocp_artifacts/conformance.replay/signature.txt`,
    - đối chiếu signature trong `replay.toml` nếu có.
  - Thêm targeted test `tests/conformance_packs.rs`:
    - assert manifest có đủ packs/shadow/quarantine scenarios,
    - chạy subset 2 lần và yêu cầu `required_digest` giống nhau.
  - Cập nhật expected signature cho quarantine replay case:
    - `fixtures/expected/quarantine-wallclock-replay.signature.txt`.
- Files changed:
  - `projects/ocp/crates/ocp-sdk/src/w9.rs`
  - `projects/ocp/conformance/conformance.v5.toml`
  - `projects/ocp/conformance/fixtures/packs-tool-fs-read/Ocp.toml`
  - `projects/ocp/conformance/fixtures/packs-tool-fs-read/src/main.ocp`
  - `projects/ocp/conformance/fixtures/packs-tool-fs-read/data/sample.txt`
  - `projects/ocp/conformance/fixtures/packs-consumer-ui/Ocp.toml`
  - `projects/ocp/conformance/fixtures/packs-consumer-ui/src/main.ocp`
  - `projects/ocp/conformance/fixtures/shadow-search-rr/Ocp.toml`
  - `projects/ocp/conformance/fixtures/shadow-search-rr/src/main.ocp`
  - `projects/ocp/conformance/fixtures/quarantine-wallclock-replay/Ocp.toml`
  - `projects/ocp/conformance/fixtures/quarantine-wallclock-replay/src/main.ocp`
  - `projects/ocp/conformance/fixtures/quarantine-wallclock-replay/.ocp_artifacts/conformance.replay/replay.toml`
  - `projects/ocp/conformance/fixtures/quarantine-wallclock-replay/expected/cassette/cassette.jsonl`
  - `projects/ocp/conformance/fixtures/quarantine-wallclock-replay/expected/cassette/cassette_index.json`
  - `projects/ocp/conformance/fixtures/quarantine-wallclock-replay/expected/cassette/cassette_meta.toml`
  - `projects/ocp/conformance/fixtures/expected/quarantine-wallclock-replay.signature.txt`
  - `tests/conformance_packs.rs`
- Commands run:
  - `cargo test --test conformance_packs`
  - `cargo test --test conformance_runner`
  - `cargo test --test conformance_core`
  - `cargo run -p ocp-cli -- test --conformance run --manifest projects/ocp/conformance/conformance.v5.toml --json`
- Test results:
  - Targeted tests (must-pass for gate): `PASS`
  - Regression tests (supporting only): `PASS`
- Kết luận gate:
  - `DONE` (đã đạt exit criteria Gate 14-C theo scope đã khóa).
- Design alignment:
  - `FULL`
- Notes/risks:
  - Quarantine replay fixture hiện dùng wallclock case tối thiểu; các case replay mở rộng cho `std.proc.exec` và `std.net.http.request` sẽ được mở rộng thêm ở các vòng conformance kế tiếp nếu cần tăng coverage theo KPI-1.

### 2026-03-05 — 14-D Planning Freeze
- Date: 2026-03-05
- Gate/Step: 14-D
- Why:
  - Khóa supply-chain behavior contracts bằng assert outputs machine-checkable thay vì chỉ pass/fail runtime.
- Scope:
  - Mở rộng conformance scenario schema v14 với:
    - `expected_deps_file`
    - `expected_permissions_effective_file`
    - `expected_signature_file` cho supply-chain scenario
  - Runner exports + compares deterministic outputs:
    - `deps.json` (packages + module provenance)
    - `permissions_effective.json` (effective permissions theo package)
  - Thêm scenario `supplychain-effective-permissions` trong v5 manifest.
  - Thêm fixture supplychain có dependency path + requested permissions.
  - Thêm targeted test `tests/conformance_supplychain.rs`.
- Expected tests:
  - `cargo test --test conformance_supplychain`
  - `cargo test --test conformance_runner`
  - `cargo run -p ocp-cli -- test --conformance run --manifest projects/ocp/conformance/conformance.v5.toml --json`
- Exit criteria:
  - Scenario supplychain pass với output assertions cho `deps.json`, `permissions_effective.json`, `signature.txt`.
  - Targeted test supplychain pass và deterministic qua 2 lần chạy.
  - Full v5 conformance pass sau khi thêm scenario supplychain.

### 2026-03-05 — 14-D Implementation Closeout
- Date: 2026-03-05
- Gate/Step: 14-D
- Implemented:
  - Mở rộng parser/builder `ConformanceScenarioV1`:
    - thêm `expected_deps_file`
    - thêm `expected_permissions_effective_file`
    - giữ `expected_signature_file` cho supply-chain scenario
  - Runner conformance thêm assert output supplychain:
    - xuất actual `deps.json` và `permissions_effective.json` vào `.ocp_artifacts/conformance.outputs/`
    - so sánh với expected files; mismatch trả `X-CONFORMANCE-OUTPUT-MISMATCH`.
  - Bổ sung scenario `supplychain-effective-permissions` vào `conformance.v5`.
  - Tạo fixture supplychain:
    - project manifest với dependency path `toolkit`,
    - dependency `package.ocpp` có requested permissions `std_proc`,
    - expected outputs `deps.json` + `permissions_effective.json` + `signature.txt`.
  - Thêm targeted test `tests/conformance_supplychain.rs`.
- Files changed:
  - `projects/ocp/crates/ocp-sdk/src/w9.rs`
  - `projects/ocp/conformance/conformance.v5.toml`
  - `projects/ocp/conformance/fixtures/supplychain-effective-permissions/Ocp.toml`
  - `projects/ocp/conformance/fixtures/supplychain-effective-permissions/src/main.ocp`
  - `projects/ocp/conformance/fixtures/supplychain-effective-permissions/deps/toolkit/package.ocpp`
  - `projects/ocp/conformance/fixtures/supplychain-effective-permissions/deps/toolkit/src/worker.ocp`
  - `projects/ocp/conformance/fixtures/supplychain-effective-permissions/expected/outputs/deps.json`
  - `projects/ocp/conformance/fixtures/supplychain-effective-permissions/expected/outputs/permissions_effective.json`
  - `projects/ocp/conformance/fixtures/supplychain-effective-permissions/expected/signature.txt`
  - `tests/conformance_supplychain.rs`
- Commands run:
  - `cargo test --test conformance_supplychain`
  - `cargo test --test conformance_runner`
  - `cargo test --test conformance_packs`
  - `cargo test --test conformance_core`
  - `cargo run -p ocp-cli -- test --conformance run --manifest projects/ocp/conformance/conformance.v5.toml --json`
- Test results:
  - Targeted tests (must-pass for gate): `PASS`
  - Regression tests (supporting only): `PASS`
- Kết luận gate:
  - `DONE` (đã đạt exit criteria Gate 14-D theo scope đã khóa).
- Design alignment:
  - `FULL`
- Notes/risks:
  - `permissions_effective.json` hiện khóa trọng tâm `std_proc` cho supply-chain case tối thiểu; nếu mở rộng sang `std_fs/std_ui/std_kv` cần thêm fixture riêng để giữ assert rõ ràng và bounded.

### 2026-03-05 — 14-E Planning Freeze
- Date: 2026-03-05
- Gate/Step: 14-E
- Why:
  - Khóa invariants perf/cache theo behavior: bật/tắt cache không được đổi semantic digest, telemetry cache phải đi qua stream phi-ngữ nghĩa.
- Scope:
  - thêm scenario conformance nhóm `cache-*` trong `conformance.v5.toml`.
  - thêm targeted suite `tests/conformance_cache.rs` gồm:
    - cache on/off giữ nguyên `required_digest` cho subset conformance.
    - `ocp cache bench --json` xác nhận `signature_equal=true` và xuất artifact telemetry phi-ngữ nghĩa.
- Expected tests:
  - `cargo test --test conformance_cache`
- Exit criteria:
  - targeted suite pass đầy đủ.
  - scenario cache được parse/run trong conformance manifest v5.
  - conformance all-suite vẫn pass sau khi thêm scenario cache.

### 2026-03-05 — 14-E Implementation Closeout
- Date: 2026-03-05
- Gate/Step: 14-E
- Implemented:
  - thêm scenario `cache-core-exec-pass` vào `conformance.v5.toml` (lane `locked_v071`, step `run`, có expected signature).
  - thêm `tests/conformance_cache.rs`:
    - `conformance_v5_includes_cache_scenarios`
    - `conformance_cache_subset_preserves_signature_with_and_without_cache`
    - `conformance_cache_bench_reports_non_semantic_cache_telemetry`
- Files changed:
  - `projects/ocp/conformance/conformance.v5.toml`
  - `tests/conformance_cache.rs`
  - `OCP-MVP-PLAN-v0.14.md`
- Commands run:
  - `cargo test --test conformance_cache`
  - `cargo test --test conformance_runner`
  - `cargo test --test conformance_core`
  - `cargo test --test conformance_packs`
  - `cargo test --test conformance_supplychain`
  - `cargo run -p ocp-cli -- test --conformance run --manifest projects/ocp/conformance/conformance.v5.toml --json`
  - `cargo fmt -- --check`
- Test results:
  - Targeted tests (must-pass for gate): `PASS`
  - Regression tests (supporting only): `PASS`
- Kết luận gate:
  - `DONE` (đã đạt exit criteria Gate 14-E theo scope đã khóa).
- Design alignment:
  - `FULL`
- Notes/risks:
  - `cargo fmt -- --check` đã pass; không còn blocker format để đóng Gate 14-E.
  - cache bench trên fixture mini-game hiện cho signal chủ yếu ở miss counters; nếu cần stress signal hit mạnh hơn cho perf KPI sâu, sẽ bổ sung fixture vòng sau.

### 2026-03-05 — 14-F Planning Freeze
- Date: 2026-03-05
- Gate/Step: 14-F
- Why:
  - bổ sung `ocp upgrade-check` để kiểm tra nâng cấp theo contract conformance, trả report pass/fail + first divergence thay vì kiểm tra thủ công.
- Scope:
  - thêm command `ocp upgrade-check <project_dir>`.
  - chọn subset conformance liên quan project (core/foundation/cache luôn có; packs/shadow/quarantine/supplychain theo feature detect).
  - chạy subset qua runner canonical `run_conformance_v1` (cùng pipeline với `ocp test --conformance`).
  - xuất JSON report có `first_divergence` và `error_code = X-UPGRADE-CHECK-FAILED` khi fail.
- Expected tests:
  - `cargo test --test upgrade_check`
- Exit criteria:
  - command `upgrade-check` chạy được ở mode pass và fail-diff.
  - fail case có báo divergence rõ ràng (signature mismatch reason).
  - không phá flow conformance hiện hữu.

### 2026-03-05 — 14-F Implementation Closeout
- Date: 2026-03-05
- Gate/Step: 14-F
- Implemented:
  - thêm command `upgrade-check` vào CLI:
    - parse flags `--manifest`, `--target-runtime`, `--out`, `--runtime`, `--engine`, `--locked`, `--universe`, `--json`.
    - detect project features từ `Ocp.toml` + source `.ocp`.
    - chọn subset scenario và chạy runner canonical `run_conformance_v1`.
    - trả report `ocp.upgrade_check.v1` gồm summary + `first_divergence`.
  - thêm targeted tests `tests/upgrade_check.rs`:
    - pass case trên fixture core.
    - fail case khi expected signature mismatch.
- Files changed:
  - `projects/ocp/crates/ocp-cli/src/main.rs`
  - `tests/upgrade_check.rs`
  - `OCP-MVP-PLAN-v0.14.md`
- Commands run:
  - `cargo test --test upgrade_check`
  - `cargo test --test conformance_runner`
  - `cargo test --test conformance_cache`
  - `cargo run -p ocp-cli -- test --conformance run --manifest projects/ocp/conformance/conformance.v5.toml --json`
  - `cargo run -p ocp-cli -- upgrade-check projects/ocp/conformance/fixtures/core-exec-pass --manifest projects/ocp/conformance/conformance.v5.toml --json`
  - `cargo fmt -- --check`
- Test results:
  - Targeted tests (must-pass for gate): `PASS`
  - Regression tests (supporting only): `PASS`
- Kết luận gate:
  - `DONE` (đã đạt exit criteria Gate 14-F theo scope đã khóa).
- Design alignment:
  - `FULL`
- Notes/risks:
  - feature detect hiện dùng heuristic string-scan để chọn subset; nếu cần precision cao hơn theo import graph typed ở vòng sau thì mở rộng parser dedicated.

### 2026-03-05 — 14-G Planning Freeze
- Date: 2026-03-05
- Gate/Step: 14-G
- Why:
  - Đóng process gate stability sprint bằng cơ chế machine-checkable additive-only, tránh drift hợp đồng giữa các bản vá v0.14.x.
- Scope:
  - Thêm targeted test `stability_contracts_v14` để so snapshot contracts release-level:
    - `manifest_schema_version`
    - `trace_schema_version`
    - `pack key list + ctx/payload schema hashes`
  - Tạo baseline snapshot file cho v0.14 tại `conformance/expected/contracts`.
  - Chạy full regression + full conformance suite để xác nhận policy “mọi thay đổi phải pass conformance”.
  - Cập nhật tracking status/workstreams/gate trạng thái DONE sau khi đủ evidence.
- Expected tests:
  - `cargo test --test stability_contracts_v14`
  - `cargo test`
  - `cargo run -p ocp-cli -- test --conformance run --manifest projects/ocp/conformance/conformance.v5.toml --suite all --json`
- Exit criteria:
  - Targeted stability-contract test pass ở chế độ verify snapshot.
  - Full `cargo test` pass.
  - Full conformance suite (`--suite all`) pass.
  - Gate 14-G có closeout đầy đủ evidence, không còn placeholder.

### 2026-03-05 — 14-G Implementation Closeout
- Date: 2026-03-05
- Gate/Step: 14-G
- Implemented:
  - Thêm test additive-only lint cho release contracts:
    - `tests/stability_contracts_v14.rs`
  - Sinh baseline snapshot contracts:
    - `projects/ocp/conformance/expected/contracts/stability_contract_v14.json`
  - Bổ sung `serde` dev-dependency phục vụ snapshot encode/decode trong test.
  - Cập nhật tracking + gate status v0.14 để phản ánh hoàn tất 14-G.
- Files changed:
  - `tests/stability_contracts_v14.rs`
  - `projects/ocp/conformance/expected/contracts/stability_contract_v14.json`
  - `Cargo.toml`
  - `OCP-MVP-PLAN-v0.14.md`
- Commands run:
  - `$env:OCP_UPDATE_V14_CONTRACT_SNAPSHOT='1'; cargo test --test stability_contracts_v14`
  - `cargo test --test stability_contracts_v14`
  - `cargo test`
  - `cargo run -p ocp-cli -- test --conformance run --manifest projects/ocp/conformance/conformance.v5.toml --suite all --json`
  - `cargo fmt -- --check`
- Test results:
  - Targeted tests (must-pass for gate):
    - `cargo test --test stability_contracts_v14`: `PASS`
    - `cargo run -p ocp-cli -- test --conformance run --manifest ... --suite all --json`: `PASS` (15/15 scenarios)
  - Regression tests (supporting only):
    - `cargo test`: `PASS`
- Kết luận gate:
  - `DONE` (đạt đầy đủ exit criteria Gate 14-G theo scope đã khóa).
- Design alignment:
  - `FULL`
- Notes/risks:
  - Lệnh chạy với `OCP_UPDATE_V14_CONTRACT_SNAPSHOT=1` là bước bảo trì baseline có chủ đích để cập nhật snapshot; bước verify chính là lần chạy lại không bật biến môi trường.

---

## 15) Checklist khóa trước khi đóng gate
- [x] Gate status đã cập nhật đúng (`TODO/IN_PROGRESS/PARTIAL/DONE`).
- [x] Có đủ Planning Freeze + Implementation Closeout cho đúng gate.
- [x] `Files changed` khớp code delta thật của gate.
- [x] `Commands run` là lệnh đã chạy thật.
- [x] `Test results` có `Targeted` và `Regression` với `PASS/FAIL` rõ ràng.
- [x] Không có mâu thuẫn giữa gate status và closeout.
- [x] Closeout `DONE` không còn placeholder `PASS/FAIL` và không còn marker `FAIL`.
- [x] Design alignment đã ghi rõ `FULL` hoặc `PARTIAL`.

---

## 16) Handoff v0.14 -> v0.15 (pre-draft)
- Chỉ mở v0.15 khi 14-A..14-G đều `DONE` và KPI v0.14 pass đầy đủ.
- v0.15 kế thừa nguyên xi các contract đã khóa tại v0.14; mọi thay đổi phải có migration contract.
- Nếu còn gate `IN_PROGRESS/PARTIAL`, không mở scope mới ở v0.15 ngoài xử lý blocker tồn đọng.

---

