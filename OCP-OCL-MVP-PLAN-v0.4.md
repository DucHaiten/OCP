# OCL MVP PLAN v0.4

Ngày tạo: 2026-02-25  
Mục tiêu: đưa OCL từ MVP ngôn ngữ thành platform hoàn chỉnh để xây ứng dụng đa dụng (compute + service), có deploy artifact chuẩn, supply-chain an toàn, tooling vận hành thực tế, và mở rộng được qua plugin ABI.

## Quy ước cập nhật bắt buộc (áp dụng từ v0.4)
- Mọi thay đổi kế hoạch phải cập nhật file này trước khi triển khai.
- Mọi gate/workstream hoàn tất phải ghi implementation evidence ngay sau khi chạy test.
- Không nhảy gate: chỉ mở workstream tiếp theo khi workstream hiện tại đạt `DONE`.
- Mỗi entry bắt buộc có: ngày, phạm vi, file thay đổi, lệnh chạy, kết quả, rủi ro còn lại.

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

## Trạng thái workstreams v0.4
- W0 Stabilize baseline + rebaseline OCL-only: `DONE`
- W1 Concurrency/runtime services: `DONE`
- W2 Capability IAM + audit/replay: `DONE`
- W3 IR/bytecode pipeline: `DONE`
- W4 Packaging v2 + remote registry + signatures: `DONE`
- W5 Debugger/profiler/trace: `DONE`
- W6 FFI/plugin ABI: `DONE`
- W7 Stdlib expanded baseline: `DONE`
- W8 Composer v0.4 (cost/incremental): `DONE`
- W9 Platform conformance + 10 demos: `DONE`

## W0 Gate Log (Final Revised)

- Status: `DONE`
- Date: `2026-02-25`
- Scope lock:
  - OCL-only active surface.
  - Legacy root runtime/test/docs/scripts moved to root archive `/_archive/**`.
  - Merge-base guards + archive read-only blocking.
  - Retire legacy lane entrypoints from active CI scripts.
- Planned validation:
  - `tools/guard_project_boundaries.ps1`
  - `tools/guard_archive_readonly.ps1`
  - `cargo check --workspace`
  - `cargo fmt -- --check`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `tools/ci_ocl_lane.ps1`
  - `cargo test -p ocl-cli --test m5_conformance`
  - `tools/ci_w0_entry.ps1`
  - `tools/ci_ocl_quarantine.ps1` (non-blocking)
- Implemented (W0.1 -> W0.12):
  - Moved root legacy runtime/test/docs/scripts/plans/batch files to `/_archive/legacy-root-pre-v0.4/**`.
  - Converted root `Cargo.toml` to virtual workspace OCL-only.
  - Rebuilt `tools/guard_project_boundaries.ps1` with merge-base diff, name-status parsing, case-insensitive normalization, root allowlist, runtime extension deny policy.
  - Added `tools/guard_archive_readonly.ps1` (blocking, dual override contract).
  - Added `tools/ci_w0_entry.ps1` and updated `tools/ci_ocl_lane.ps1`, `tools/ci_ocl_quarantine.ps1`.
  - Updated `docs/PROJECT-BOUNDARY.md`, root `README.md`, `projects/ocp-ocl/README.md`.
- Validation results:
  - `guard_project_boundaries.ps1`: pass.
  - `guard_archive_readonly.ps1`: pass with explicit W0 override (`--allow-archive-changes` + `ALLOW_ARCHIVE_CHANGES=1`).
  - `guard_archive_readonly.ps1` no-override: blocked as expected (`ps_exit=2`).
  - `guard_archive_readonly.ps1` partial override: blocked as expected (`ps_exit=3`).
  - `cargo check --workspace`: pass.
  - `cargo fmt -- --check`: pass.
  - `cargo clippy --workspace --all-targets -- -D warnings`: pass.
  - `tools/ci_ocl_lane.ps1`: pass.
  - `cargo test -p ocl-cli --test m5_conformance`: pass (15/15).
  - `tools/ci_w0_entry.ps1`: pass.
  - `tools/ci_ocl_quarantine.ps1`: pass, report written to `target/ocl/quarantine/report.json`.
  - `cargo metadata --no-deps`: workspace members chi gom `ocl-runtime-core`, `ocl-sdk`, `ocl-cli`.

## W1 Gate Log (Re-locked)

- Status: `DONE`
- Date: `2026-02-25`
- Scope lock:
  - Runtime services tách layer rõ: scheduler/event loop nằm ở `ocl-runtime-rt`, bridge chỉ giữ `observe/commit`.
  - `std.net.poll` bị cấm ở user OCL path.
  - `std.net.reply` commit-gated, side-effect net chỉ phát sinh ở commit.
  - Deterministic ordering + monotonic `event_id`.
  - Single-actor bounded mailbox cho W1, multi-actor defer.
- Implemented (W1.1 -> W1.13):
  - Added crate `projects/ocp-ocl/crates/ocl-runtime-rt` (mio runtime driver, TCP JSONL event source, deterministic ordering, monotonic event id, outbound apply, runtime counters).
  - Extended `ocl-runtime-core` types/validation:
    - `Reason`: `DeadlineExceeded`, `Cancelled`, `Backpressure`
    - `KeyFamily::StdNet`
    - payloads `StdNetEvent`, `StdNetAck`
    - args schemas `std.net.listen|reply|close`
    - checker reject for `std.net.poll` in user code (`E-KEY-FORBIDDEN`)
  - Extended `ocl-sdk` runtime APIs:
    - `prepare_project_runtime(...)`
    - `run_reactor_service(...)`
    - `ReactorServiceOptions`, `ReactorServiceReport`, runtime report writer
    - manifest runtime budget enforcement for `std.net=true` (`mailbox_max_depth`, `io_max_events_per_tick`, `default_deadline_ms`, `actor_max`)
    - added `ReactorRuntimeBridge` (separate from `OclRuntimeBridge`)
  - Extended `ocl-cli` run surface:
    - flags: `--runtime deterministic|throughput`, `--socket-listen`, `--runtime-report`
    - reactor run path wired to sdk runtime service
    - `LocalBridge` std.net capability observe/commit paths and runtime event injection/outbound drain
  - Updated demo contract:
    - `projects/ocp-ocl/apps/mini-server/Ocl.toml` includes `std.net=true` + runtime budget fields
    - `projects/ocp-ocl/apps/mini-server/src/main.ocl` includes `std.net.reply` commit-gated path
  - Added W1 tests:
    - `projects/ocp-ocl/crates/ocl-runtime-core/tests/m1_language.rs`:
      - `w1_stdnet_poll_forbidden_in_user_ocl`
      - `w1_stdnet_event_payload_line_only_contract`
    - `projects/ocp-ocl/crates/ocl-runtime-rt/src/lib.rs`:
      - `w1_event_id_monotonic_deterministic_order`
    - `projects/ocp-ocl/crates/ocl-sdk/tests/w1_layering.rs`:
      - `w1_no_runtime_hooks_added_to_ocl_runtime_bridge_trait`
      - `w1_runtime_loop_lives_in_ocl_runtime_rt_not_bridge`
    - `projects/ocp-ocl/crates/ocl-sdk/tests/w1_runtime.rs`:
      - `w1_cancelled_reason_emitted`
      - `w1_backpressure_reason_emitted`
    - `projects/ocp-ocl/crates/ocl-cli/tests/w1_runtime.rs`:
      - `w1_miniserver_local_tcp_jsonl_real_socket_pass`
      - `w1_miniserver_deterministic_sim_replay_same_hash`
      - `w1_throughput_mode_smoke_pass`
  - Updated `tools/ci_ocl_lane.ps1`:
    - includes `ocl-runtime-rt` in check/test/clippy
    - runs `cargo test -p ocl-cli --test w1_runtime`
    - runs W1 runtime smoke commands with deterministic + throughput mode and report output
- Validation results:
  - `cargo check --workspace`: pass.
  - `cargo test -p ocl-runtime-core -p ocl-runtime-rt -p ocl-sdk -p ocl-cli`: pass.
  - `cargo clippy -p ocl-runtime-core -p ocl-runtime-rt -p ocl-sdk -p ocl-cli --all-targets -- -D warnings`: pass.
  - `cargo fmt -- --check`: pass.
  - `cargo run -p ocl-cli -- run projects/ocp-ocl/apps/mini-server --locked --reactor --ticks 128 --runtime deterministic --socket-listen 127.0.0.1:19091 --runtime-report target/ocl/w1/miniserver_det.json`: pass.
  - `cargo run -p ocl-cli -- run projects/ocp-ocl/apps/mini-server --locked --reactor --ticks 128 --runtime throughput --socket-listen 127.0.0.1:19092 --runtime-report target/ocl/w1/miniserver_thr.json`: pass.
  - `tools/ci_ocl_lane.ps1`: pass.

## W2 Gate Log (Final Re-locked)

- Status: `DONE`
- Date: `2026-02-25`
- Scope lock:
  - IAM precedence enforced: `module > package > default deny`, and `deny > allow > prompt`.
  - Canonicalization table-driven (`key.aliases.v1`), no arbitrary `:` stripping.
  - `--locked` requires `[permissions.package]` (`V-PERMISSIONS-MISSING`).
  - Audit includes blocked attempts (`deny` / `prompt_reject`) with hash-chain.
  - Replay comparator uses required-field set; optional fields are warn-only.
  - Deterministic JSONL serializer: fixed key order, UTF-8, LF-only.
- Implemented (W2-A -> W2-L):
  - Extended manifest parsing in `ocl-sdk`:
    - parse `[permissions.package]` and `[permissions.module.<module_key>]`.
    - validate locked-mode presence of permissions.
  - Added IAM model/types and evaluation:
    - `PermissionRules`, `ProjectPermissions`, `PermissionDecision`, `PermissionMatchMode`, `PromptPolicy`.
    - `canonical_permission_key(...)` with alias-prefix map:
      - `world.exists: -> world.exists`
      - `world.bounds: -> world.bounds`
      - `render.prims: -> render.prims`
    - `evaluate_permission(...)` with module/package precedence.
  - Added checker/verifier integration:
    - `type_check_project_with_options(...)` used by `check/test/build`.
    - exact-match enforcement for locked mode, prefix optional only in unlocked mode.
  - Added audit/replay runtime integration at sdk orchestration layer:
    - `run_project_with_audit(...)`
    - `run_reactor_service_with_audit(...)`
    - deterministic audit writer + tamper-evident hash-chain.
    - replay comparator with required-field mismatch fail.
  - Added audit verification API:
    - `verify_audit_log(...) -> AuditVerifyReport`.
  - Extended CLI:
    - run flags: `--audit-log`, `--replay-audit`, `--prompt-policy allow|deny`.
    - command: `ocl audit verify <path> [--json]`.
    - exit code mapping:
      - `2` permission violation
      - `3` audit chain invalid
      - `4` replay mismatch / replay mode violation
  - Updated init/app manifests to scaffold `[permissions.package]` baseline.
  - Added W2 tests:
    - `projects/ocp-ocl/crates/ocl-sdk/tests/w2_iam_audit.rs`
      - canonicalization alias/no-strip
      - locked exact vs unlocked prefix
      - module override package
      - locked missing permissions fail
      - audit hash-chain tamper detection
      - replay required-field mismatch fail
  - Updated CI lane (`tools/ci_ocl_lane.ps1`) with W2 gate commands:
    - hello-cli audit run + `ocl audit verify`
    - mini-server deterministic audit + replay run.
- Validation results:
  - `cargo check --workspace`: pass.
  - `cargo test -p ocl-runtime-core -p ocl-runtime-rt -p ocl-sdk -p ocl-cli`: pass.
  - `cargo clippy -p ocl-runtime-core -p ocl-runtime-rt -p ocl-sdk -p ocl-cli --all-targets -- -D warnings`: pass.
  - `cargo fmt -- --check`: pass.
  - `cargo run -p ocl-cli -- check projects/ocp-ocl/apps/hello-cli --locked --json`: pass (`[]`).
  - `cargo run -p ocl-cli -- run projects/ocp-ocl/apps/hello-cli --locked --audit-log target/ocl/w2/hello.audit.jsonl --arg name=demo --arg do_commit=1`: pass.
  - `cargo run -p ocl-cli -- audit verify target/ocl/w2/hello.audit.jsonl --json`: pass (`valid=true`).
  - `cargo run -p ocl-cli -- run projects/ocp-ocl/apps/mini-server --locked --reactor --ticks 64 --runtime deterministic --audit-log target/ocl/w2/mini.audit.jsonl`: pass.
  - `cargo run -p ocl-cli -- run projects/ocp-ocl/apps/mini-server --locked --reactor --ticks 64 --runtime deterministic --replay-audit target/ocl/w2/mini.audit.jsonl`: pass.
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_lane.ps1`: pass.

## W3 Gate Log (Final Re-locked)

- Status: `DONE`
- Date: `2026-02-25`
- Scope lock:
  - Real pipeline wired: `AST -> Typed IR -> Bytecode -> VM`.
  - No grammar changes.
  - No bridge trait expansion.
  - Dual parity compares stable semantic digest, not raw trace lines.
  - VM metering lock kept: `base_step_cap=50000`, `K_loop=8`.
- Implemented:
  - Added runtime-core modules:
    - `projects/ocp-ocl/crates/ocl-runtime-core/src/ir.rs`
    - `projects/ocp-ocl/crates/ocl-runtime-core/src/bytecode.rs`
    - `projects/ocp-ocl/crates/ocl-runtime-core/src/source_map.rs`
    - `projects/ocp-ocl/crates/ocl-runtime-core/src/vm.rs`
  - Upgraded `compile_script` / `execute_compiled` to real compiled path in:
    - `projects/ocp-ocl/crates/ocl-runtime-core/src/lib.rs`
  - Added `RunEngine` wiring and dual parity in sdk/cli:
    - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
    - `projects/ocp-ocl/crates/ocl-cli/src/lib.rs`
  - Added CLI engine surface:
    - `ocl run --engine interpreter|bytecode|dual`
    - `ocl test --engine interpreter|bytecode|dual`
  - Added W3 tests:
    - `projects/ocp-ocl/crates/ocl-runtime-core/tests/w3_ir_vm.rs`
    - `projects/ocp-ocl/crates/ocl-sdk/tests/w3_parity.rs`
    - `projects/ocp-ocl/crates/ocl-cli/tests/w3_engine.rs`
  - Updated lane script with W3 scenarios:
    - `tools/ci_ocl_lane.ps1`
- Validation results:
  - `cargo check --workspace`: pass.
  - `cargo test -p ocl-runtime-core -p ocl-runtime-rt -p ocl-sdk -p ocl-cli`: pass.
  - `cargo clippy -p ocl-runtime-core -p ocl-runtime-rt -p ocl-sdk -p ocl-cli --all-targets -- -D warnings`: pass.
  - `cargo fmt -- --check`: pass.
  - `cargo run -p ocl-cli -- run projects/ocp-ocl/app-ocl --locked --engine bytecode`: pass.
  - `cargo run -p ocl-cli -- run projects/ocp-ocl/app-ocl --locked --engine dual`: pass.
  - `cargo run -p ocl-cli -- test projects/ocp-ocl/app-ocl --locked --engine dual`: pass.
  - `cargo run -p ocl-cli -- run projects/ocp-ocl/apps/mini-server --locked --reactor --ticks 64 --runtime deterministic --engine dual`: pass.
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_lane.ps1`: pass.

## W4 Gate Log (Security-Closed Patch)

- Status: `DONE`
- Date: `2026-02-25`
- Scope lock:
  - `.oclpkg` custom single-file container implemented and wired as default build output.
  - Supply-chain path uses `BLAKE3-256` for security hashing and `ed25519` signatures encoded in base64.
  - `hash64` retained for deterministic indexing only.
  - No serializable model stores private key material.
  - Locked path enforces signature presence for non-builtin deps via `deps.lock.v2`.
  - Offline `.oclpkg` run enforces path safety and sandbox extraction.
- Implemented:
  - Added W4 supply-chain SDK surface in:
    - `projects/ocp-ocl/crates/ocl-sdk/src/w4.rs`
    - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
  - Added new crypto dependencies in:
    - `projects/ocp-ocl/crates/ocl-sdk/Cargo.toml`
  - Added CLI W4 command surface and wiring:
    - `ocl build` default `.oclpkg` (`--source-only` keeps `.oclbundle`)
    - `ocl publish`
    - `ocl fetch`
    - `ocl verify-supply`
    - `ocl run <artifact.oclpkg>`
    - implemented in `projects/ocp-ocl/crates/ocl-cli/src/lib.rs`
  - Added W4 lock sync coverage for active apps by generating `deps.lock.v2`:
    - `projects/ocp-ocl/app-ocl/deps.lock.v2`
    - `projects/ocp-ocl/apps/*/deps.lock.v2`
  - Fixed `registry.index.v2` parser boundary in:
    - `projects/ocp-ocl/crates/ocl-sdk/src/w4.rs`
    - Prevents top-level signature fields from being consumed as trailing `[[artifacts]]` fields.
- Validation results:
  - `cargo check -p ocl-sdk`: pass.
  - `cargo check -p ocl-cli`: pass.
  - `cargo test -p ocl-sdk`: pass.
  - `cargo test -p ocl-cli`: pass (including `m5_conformance`, `w1_runtime`, `w3_engine`).
  - `cargo test -p ocl-cli --test w4_supply -- --nocapture`: pass (`build -> publish -> fetch -> verify-supply -> run offline` locked path).
  - `cargo check --workspace`: pass.
  - `cargo test -p ocl-runtime-core -p ocl-runtime-rt -p ocl-sdk -p ocl-cli`: pass.
  - `cargo clippy -p ocl-runtime-core -p ocl-runtime-rt -p ocl-sdk -p ocl-cli --all-targets -- -D warnings`: pass.
  - `cargo fmt -- --check`: pass.

## W5 Gate Log (Patched Final)

- Status: `DONE`
- Date: `2026-02-25`
- Scope lock:
  - Added `ocl trace run/view` and `ocl profile run/view` for deterministic runtime triage.
  - Deterministic mode is blocking gate; throughput mode is quarantined smoke only.
  - Trace is hash-only and does not expose raw args/payload text.
  - Trace/audit are emitted from shared in-memory runtime events (no raw audit file reparse).
  - Required digest preserves record order by `seq` (no sort-before-hash).
  - `alloc_units_est`/`eval_steps_est` surfaced in execution metrics and profile report.
- Implemented:
  - Extended runtime-core execution output:
    - `projects/ocp-ocl/crates/ocl-runtime-core/src/exec.rs`
    - `ExecMetrics { alloc_units_est, eval_steps_est }` wired into `ExecOutput`.
  - Added trace/profile model and APIs in SDK:
    - `TraceEventV1`, `TraceViewOptions`, `TraceViewReport`
    - `ProfileReportV1`, `ProfileKeyCostV1`, `ProfileViewOptions`
    - `build_run_id_deterministic(...)`
    - `run_project_with_trace_engine(...)`
    - `run_reactor_service_with_trace(...)`
    - `build_profile_from_trace(...)`
    - `render_trace_view(...)`
    - `render_profile_view(...)`
    - implemented in `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
  - Added CLI command surface:
    - `ocl trace run`
    - `ocl trace view`
    - `ocl profile run`
    - `ocl profile view`
    - wired in `projects/ocp-ocl/crates/ocl-cli/src/lib.rs`
  - Added W5 test suite:
    - `projects/ocp-ocl/crates/ocl-cli/tests/w5_trace_profile.rs`
    - validates seq monotonic/reset, deterministic run_id, hash-only trace, required digest order sensitivity, and CLI trace/profile E2E.
  - Updated CI lanes:
    - blocking deterministic scenarios in `tools/ci_ocl_lane.ps1`
    - throughput smoke report in `tools/ci_ocl_quarantine.ps1`
- Validation results:
  - `cargo check --workspace`: pass.
  - `cargo test -p ocl-runtime-core -p ocl-runtime-rt -p ocl-sdk -p ocl-cli`: pass.
  - `cargo clippy -p ocl-runtime-core -p ocl-runtime-rt -p ocl-sdk -p ocl-cli --all-targets -- -D warnings`: pass.
  - `cargo fmt -- --check`: pass.
  - `cargo run -p ocl-cli -- trace run projects/ocp-ocl/app-ocl --locked --engine dual --out target/ocl/w5/app.trace.jsonl`: pass.
  - `cargo run -p ocl-cli -- trace view target/ocl/w5/app.trace.jsonl --tail 30 --json`: pass.
  - `cargo run -p ocl-cli -- profile run projects/ocp-ocl/app-ocl --locked --engine dual --out target/ocl/w5/app.profile.json`: pass.
  - `cargo run -p ocl-cli -- profile view target/ocl/w5/app.profile.json --top 10 --json`: pass.
  - `cargo run -p ocl-cli -- trace run projects/ocp-ocl/apps/mini-server --locked --reactor --ticks 64 --runtime deterministic --engine dual --out target/ocl/w5/mini.det.trace.jsonl`: pass.
  - `cargo run -p ocl-cli -- profile run projects/ocp-ocl/apps/mini-server --locked --reactor --ticks 64 --runtime deterministic --engine dual --out target/ocl/w5/mini.det.profile.json`: pass.
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_lane.ps1`: pass.
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_quarantine.ps1`: pass (`target/ocl/quarantine/report.json` written).

## W6 Gate Log (Re-locked Patch)

- Status: `DONE`
- Date: `2026-02-25`
- Scope lock:
  - Plugin ABI v1 chạy out-of-process qua JSONL stdio (UTF-8, LF-only).
  - Không thêm crate workspace mới; chỉ sửa trong `ocl-runtime-core`, `ocl-runtime-rt`, `ocl-sdk`, `ocl-cli`.
  - `OclRuntimeBridge` giữ nguyên shape (không mở rộng hook plugin/runtime).
  - `custom.*` là scope capability plugin duy nhất ở W6.
  - Locked path dùng `plugins.lock.v1` làm source-of-truth; `plugins/index.toml` chỉ dùng cho lock sync.
  - No restart worker trong W6; timeout/crash fail-honest.
- Implemented:
  - Added SDK W6 module:
    - `projects/ocp-ocl/crates/ocl-sdk/src/w6.rs`
    - types: `PluginRegistryIndexV1`, `PluginRegistryEntryV1`, `PluginLockV1`, protocol structs `PluginHelloV1/PluginReqV1/PluginResV1/PluginCommitReqV1`
    - APIs: `sync_plugin_lock_v1`, `verify_plugin_lock_v1`, `verify_plugin_signature`, `canonical_plugin_sign_message`, `resolve_plugin_for_custom_key`, `collect_project_custom_keys`
  - Extended SDK exports + trace schema in:
    - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
    - trace correlation fields for `custom.*`: `plugin_id`, `plugin_req_id`, `plugin_seq`
  - Extended runtime-core reason codes in:
    - `projects/ocp-ocl/crates/ocl-runtime-core/src/lib.rs`
    - `projects/ocp-ocl/crates/ocl-runtime-core/src/exec.rs`
    - added `PluginProtocolError`, `PluginUnavailable`, `PluginTimeout`
  - Added CLI plugin commands and locked enforcement in:
    - `projects/ocp-ocl/crates/ocl-cli/src/lib.rs`
    - `ocl plugin lock sync ...`
    - `ocl plugin verify ...`
    - locked check/run/test/build fail nếu dùng `custom.*` mà lock/verify không hợp lệ
  - Implemented LocalBridge plugin worker manager in:
    - `projects/ocp-ocl/crates/ocl-cli/src/lib.rs`
    - lazy spawn on first allow path
    - protocol `seq + id` (single in-flight in W6)
    - deny path short-circuit không spawn worker
    - commit-gated custom effect với idempotent binding
  - Added W6 demo app + fixture plugin process:
    - `projects/ocp-ocl/apps/plugin-demo/Ocl.toml`
    - `projects/ocp-ocl/apps/plugin-demo/src/main.ocl`
    - `projects/ocp-ocl/apps/plugin-demo/tests/smoke.ocl`
    - `projects/ocp-ocl/apps/plugin-demo/plugins/index.toml`
    - `projects/ocp-ocl/apps/plugin-demo/plugins.lock.v1`
    - `projects/ocp-ocl/apps/plugin-demo/plugins/bin/echo_plugin.ps1`
  - Added W6 tests:
    - `projects/ocp-ocl/crates/ocl-cli/tests/w6_plugin.rs`
    - `projects/ocp-ocl/crates/ocl-sdk/src/w6.rs` unit coverage
  - Updated CI lanes:
    - `tools/ci_ocl_lane.ps1`
    - `tools/ci_ocl_quarantine.ps1`
- Validation results:
  - `cargo check --workspace`: pass.
  - `cargo test -p ocl-runtime-core -p ocl-runtime-rt -p ocl-sdk -p ocl-cli`: pass.
  - `cargo clippy -p ocl-runtime-core -p ocl-runtime-rt -p ocl-sdk -p ocl-cli --all-targets -- -D warnings`: pass.
  - `cargo fmt -- --check`: pass.
  - `cargo test -p ocl-cli --test w6_plugin -- --nocapture`: pass (3/3).
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_lane.ps1`: pass.
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_quarantine.ps1`: pass.

## W7 Gate Log (Final Re-locked Patch)

- Status: `DONE`
- Date: `2026-02-25`
- Scope lock:
  - Core-first stdlib expansion in existing crates only (no new workspace crate).
  - TLS API split only: `std.tls.connect` + `std.tls.handshake` (no `std.tls.http_get`).
  - Deterministic blocking path is fixture-first TLS; local-real TLS is env-gated quarantine.
  - SQLite split contract: `query_int` read-only observe; `exec` commit-gated write with idempotent `pending_write_id`.
  - `std.ui.frame_info` and `std.game.tick_info` are pure observe and return `ctx_tick`.
  - Keep invariants: no naked IO, 4-kind, commit-gated effects, budgets/policy locked enforcement.
- Implemented:
  - Extended runtime-core stdlib surface:
    - `projects/ocp-ocl/crates/ocl-runtime-core/src/bridge.rs`
    - `projects/ocp-ocl/crates/ocl-runtime-core/src/lib.rs`
    - `projects/ocp-ocl/crates/ocl-runtime-core/src/checker.rs`
    - added families `StdTls|StdDb|StdUi|StdGame`, args schema mapping, payload/type mapping, key-family resolution.
  - Extended CLI LocalBridge stdlib handlers:
    - `projects/ocp-ocl/crates/ocl-cli/src/lib.rs`
    - TLS fixture loader + `transcript_hash256` verification
    - env-gated local-real TLS handshake path
    - SQLite `query_int` (SELECT-only) and `exec` staging + commit apply-once idempotency
    - UI/Game headless tick observe path
  - Added W7 dependencies (locked minimal):
    - `projects/ocp-ocl/crates/ocl-cli/Cargo.toml`
    - `rustls`, `rustls-pemfile`, `rusqlite` (`bundled`), `base64`
    - no `hyper/reqwest/diesel/sqlx/openssl`
  - Added W7 demo apps + fixtures:
    - `projects/ocp-ocl/apps/tls-client/**`
    - `projects/ocp-ocl/apps/sqlite-app/**`
    - fixed deterministic fixture hash in `projects/ocp-ocl/apps/tls-client/tests/fixtures/tls/tls_demo_ok.json`
  - Added W7 tests:
    - `projects/ocp-ocl/crates/ocl-cli/tests/w7_stdlib.rs`
    - `projects/ocp-ocl/crates/ocl-cli/tests/w7_tls_local_real.rs` (env-gated quarantine)
    - internal unit tests in `projects/ocp-ocl/crates/ocl-cli/src/lib.rs` for key family, TLS hash mismatch, SQLite no-commit write, UI/Game tick.
  - Updated CI lanes:
    - `tools/ci_ocl_lane.ps1` includes W7 blocking scenarios.
    - `tools/ci_ocl_quarantine.ps1` includes env-gated W7 real TLS smoke.
  - Added post-W7 release hygiene:
    - `tools/ci_ocl_lane.ps1` now defaults to signed/release-grade mode using repo signer/trust baseline.
    - `tools/ci_ocl_release.ps1` added as explicit release entrypoint for signed build + verify-supply + artifact run.
    - unsigned fallback is explicit only (`-AllowUnsignedBuild`) and not used for DONE evidence.
    - security baseline committed:
      - `projects/ocp-ocl/security/dev-root-1.signing.key.toml`
      - `projects/ocp-ocl/security/trust.store.toml`
    - `.gitignore` updated to ignore generated `.oclpkg` directories and explicit `target/ocl/`.
- Validation results:
  - `cargo check --workspace`: pass.
  - `cargo test -p ocl-runtime-core -p ocl-runtime-rt -p ocl-sdk -p ocl-cli`: pass.
  - `cargo clippy -p ocl-runtime-core -p ocl-runtime-rt -p ocl-sdk -p ocl-cli --all-targets -- -D warnings`: pass.
  - `cargo fmt -- --check`: pass.
  - `cargo run -p ocl-cli -- check projects/ocp-ocl/apps/tls-client --locked --json`: pass.
  - `cargo run -p ocl-cli -- run projects/ocp-ocl/apps/tls-client --locked --arg fixture_key=tls_demo_ok`: pass.
  - `cargo run -p ocl-cli -- check projects/ocp-ocl/apps/sqlite-app --locked --json`: pass.
  - `cargo run -p ocl-cli -- run projects/ocp-ocl/apps/sqlite-app --locked`: pass.
  - `cargo run -p ocl-cli -- test projects/ocp-ocl/apps/sqlite-app --locked --engine dual`: pass.
  - `cargo run -p ocl-cli -- build projects/ocp-ocl/apps/sqlite-app --locked --json`: pass.
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_lane.ps1`: pass in signed/release-grade default mode.
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_quarantine.ps1`: pass.
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_release.ps1`: pass with repo security baseline.
  - signed artifact checks observed in lane:
    - `verify-supply --locked` valid for `app-ocl`, `plugin-demo`, `sqlite-app`
    - artifact offline run under `--locked` pass for `app-ocl` and `sqlite-app`

## W8 Gate Log (Final Re-locked)

- Status: `DONE`
- Date: `2026-02-25`
- Scope lock:
  - Composer v2 path implemented with hash256 identity and cache invalidation.
  - Cache path unified to `target/ocl/composer/`.
  - Locked catalog identity uses `catalog.lock.v2` hash256 (`catalog_lock_hash256`) with v1 compat-read warning.
  - Objective weights enforced as integer (`u32`), score evaluated on `u128`.
  - Tie-break and proof identity are deterministic; no silent skip fallback for unsupported paths.
  - Incremental parity anchored on 3 hash256 digests: graph/generated/proof.
- Implemented:
  - Added W8 SDK module:
    - `projects/ocp-ocl/crates/ocl-sdk/src/w8.rs`
    - types: `PhenotypeSpecV2`, objective/envelope/constraints, `ComposerCacheV1`, `AssemblyProofV2`, compose/verify reports
    - APIs: `parse_phenotype_spec_v2`, `parse_phenotype_spec_v1_compat`, `compose_phenotype_v2`, `verify_assembly_v2`, `sync_catalog_lock_v2`
    - canonical hash helpers for phenotype/objective/catalog/proof/generated tree (BLAKE3-256 hex64)
  - Exported W8 surface:
    - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
  - Extended component spec metadata (cost model inputs):
    - `projects/ocp-ocl/crates/ocl-sdk/src/m4.rs`
    - additive parse fields: `estimated_latency_ns`, `estimated_cost_units`, `risk_level`
  - Upgraded CLI compose/verify/catalog-lock to v2 flow:
    - `projects/ocp-ocl/crates/ocl-cli/src/lib.rs`
    - new command: `ocl catalog lock sync <project_dir> [--locked|--unlocked] [--json]`
    - compose JSON schema -> `ocl.compose.v2`
    - verify JSON schema -> `ocl.verify.v2`
    - compose cache-hit now returns full objective + incremental counters (no zero placeholders)
  - Updated composer artifacts and fixtures:
    - `projects/ocp-ocl/app-ocl/phenotype.toml` -> `phenotype.v2`
    - `projects/ocp-ocl/apps/composer-demo/phenotype.toml` -> `phenotype.v2`
    - `projects/ocp-ocl/app-ocl/assembly_proof.toml` -> `assembly.proof.v2`
    - `projects/ocp-ocl/apps/composer-demo/assembly_proof.toml` -> `assembly.proof.v2`
    - generated `catalog.lock.v2` for `app-ocl` and `apps/composer-demo`
  - Added W8 test suites:
    - `projects/ocp-ocl/crates/ocl-sdk/tests/w8_composer.rs` (hash256/cache/catalog/proof parity coverage)
    - `projects/ocp-ocl/crates/ocl-cli/tests/w8_composer.rs` (catalog sync + compose/verify v2 CLI paths)
  - Synced hash64 legacy lock data after component spec additive changes:
    - `projects/ocp-ocl/registry/components/index.toml`
    - `projects/ocp-ocl/app-ocl/catalog.lock`
    - `projects/ocp-ocl/apps/composer-demo/catalog.lock`
    - `projects/ocp-ocl/registry/components/specs/canary_root.toml`
    - `projects/ocp-ocl/registry/components/specs/std_args_provider.toml`
  - CI lane update:
    - `tools/ci_ocl_lane.ps1` now includes `catalog lock sync` before compose for canary and composer-demo.
- Validation results:
  - `cargo check --workspace`: pass.
  - `cargo test -p ocl-runtime-core -p ocl-runtime-rt -p ocl-sdk -p ocl-cli`: pass.
  - `cargo clippy -p ocl-runtime-core -p ocl-runtime-rt -p ocl-sdk -p ocl-cli --all-targets -- -D warnings`: pass.
  - `cargo fmt -- --check`: pass.
  - `cargo run -p ocl-cli -- catalog lock sync projects/ocp-ocl/app-ocl --locked --json`: pass.
  - `cargo run -p ocl-cli -- compose projects/ocp-ocl/app-ocl --phenotype projects/ocp-ocl/app-ocl/phenotype.toml --locked --json` (run twice): pass, stable digest and incremental hit.
  - `cargo run -p ocl-cli -- verify projects/ocp-ocl/app-ocl --phenotype projects/ocp-ocl/app-ocl/phenotype.toml --locked --json`: pass.
  - `cargo run -p ocl-cli -- compose projects/ocp-ocl/apps/composer-demo --phenotype projects/ocp-ocl/apps/composer-demo/phenotype.toml --locked --json`: pass.
  - `cargo run -p ocl-cli -- verify projects/ocp-ocl/apps/composer-demo --phenotype projects/ocp-ocl/apps/composer-demo/phenotype.toml --locked --json`: pass.
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocl_lane.ps1`: pass.

## W9 Gate Log (Final Re-locked Patch 2)

- Status: `DONE`
- Date: `2026-02-25`
- Scope lock:
  - Platform SoT duy nhất: `ocl test --conformance --locked`.
  - Demo #9 dùng `tcp-jsonl-server-real` (scope-safe JSONL/TCP), không mở HTTP parser scope.
  - Sign key không default từ repo path; locked signed flow yêu cầu explicit `--sign-key` hoặc `OCL_SIGN_KEY_PATH`.
  - Runner orchestration đặt ở SDK (`w9.rs`), CLI chỉ làm surface command.
  - Required digest order-sensitive, không phụ thuộc `warnings_count`.
  - Throughput quarantine dùng `--engine bytecode` (không dual parity).
  - Plugin demo pinned platform (`windows-x64`) và locked runtime resolve theo `plugins.lock.v1`.
  - Conformance artifact/cache/report root dưới `target/ocl/w9/*`.
- Implemented:
  - Added SDK W9 surface:
    - `projects/ocp-ocl/crates/ocl-sdk/src/w9.rs`
    - types: `ConformanceManifestV1`, `ConformanceScenarioV1`, `ConformanceRunOptionsV1`, `ConformanceReportV1`
    - APIs: `parse_conformance_manifest_v1`, `run_conformance_v1`, `compute_conformance_required_digest`
  - Exported W9 APIs:
    - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
  - Extended CLI test command:
    - `projects/ocp-ocl/crates/ocl-cli/src/lib.rs`
    - new mode: `ocl test --conformance ...`
    - added `ConformanceOptions` + exit mapping W9 (`9/10/11/12`)
  - Added conformance manifest:
    - `projects/ocp-ocl/conformance/conformance.v1.toml`
    - ordered 10 demos:
      - `hello-cli`, `web-fetch`, `mini-server`, `scheduler`, `composer-demo`, `plugin-demo`, `tls-client`, `sqlite-app`, `tcp-jsonl-server-real`, `ui-demo`
  - Added new demo apps:
    - `projects/ocp-ocl/apps/tcp-jsonl-server-real/**`
    - `projects/ocp-ocl/apps/ui-demo/**`
  - Added W9 tests:
    - `projects/ocp-ocl/crates/ocl-cli/tests/w9_conformance.rs`
  - Updated CI lanes:
    - `tools/ci_ocl_lane.ps1`
      - removed implicit default private sign key path
      - added blocking W9 conformance step
    - `tools/ci_ocl_quarantine.ps1`
      - added throughput conformance smoke (`engine=bytecode`)
  - Updated user guide:
    - `projects/ocp-ocl/docs/OCL-USER-GUIDE.md` (W9 conformance command + 10 demo list)
- Validation results:
  - `cargo check --workspace`: pass.
  - `cargo test -p ocl-cli --test w9_conformance`: pass (4/4).
  - `cargo run -p ocl-cli -- test --conformance --locked --runtime deterministic --engine dual --manifest projects/ocp-ocl/conformance/conformance.v1.toml --out target/ocl/w9/reports/conformance_report.json --trust-store projects/ocp-ocl/security/trust.store.toml --signer-id dev-root-1 --sign-key projects/ocp-ocl/security/dev-root-1.signing.key.toml --json`: pass (10/10).
  - report evidence:
    - `target/ocl/w9/reports/conformance_report.json`
    - `schema=ocl.conformance.v1`, `report_written=true`, `scenarios_failed=0`

---

## 0) Mục tiêu v0.4 (platform-complete)

v0.4 MUST deliver:
1. Language + runtime đủ viết chương trình đa dụng với concurrency chuẩn, cancellation/deadline/backpressure.
2. Capability system hoàn chỉnh (permission/IAM per-package, audit, replay), không có naked IO.
3. Packaging/distribution chuẩn: single-file artifact + remote registry + signatures/attestation + exact pin.
4. Tooling: test framework, debugger/profiler/trace viewer, conformance runner.
5. Composer nâng cấp: cost model + multi-objective + caching + incremental re-assembly.
6. FFI/plugin ABI: third-party native extensions chạy được nhưng vẫn bị sandbox bằng capability contracts.
7. Stdlib mở rộng đủ ship sản phẩm, nhưng vẫn bounded và policy-driven.

Non-goals v0.4:
1. Không làm browser engine/AAA engine trong core.
2. Không làm global solver unbounded.
3. Không cho phép IO direct trong language.

---

## 1) Decision Locks v0.4 (đóng cứng)

### 1.1 Axis lock kế thừa v0.1-v0.3
1. 4-kind everywhere: `OK|DEGRADED|DEFERRED|INSUFFICIENT`.
2. Commit-gated effects: observe không gây side-effect.
3. Budgets in DNA: policy/budget bắt buộc và enforce ở checker + runtime.
4. No naked IO: không syscall trực tiếp từ OCL.
5. Non-smart orchestrator: không semantic-route theo payload.
6. Determinism first-class: có deterministic mode + replay lane.

### 1.2 Concurrency model (LOCK)
1. Hybrid actor + async IO multiplexing.
2. Ngôn ngữ có `async fn` và `await`, runtime thực thi trên actor scheduler.
3. Mỗi actor có mailbox bounded + policy caps.
4. Deterministic mode dùng ordering ổn định (queue order + tie-break cố định), không phụ thuộc timing hệ điều hành.
5. Throughput mode cho production vẫn bounded + audit.

### 1.3 Runtime event model (LOCK)
1. Event chuẩn: `Tick`, `Timer`, `Input`, `IoReady`, `Signal`.
2. `Timer` và `IoReady` luôn đi qua adapters + capability checks.
3. Deterministic gate dùng fixtures/recorded events.

### 1.4 Deterministic commit linearization (LOCK, mới)
1. Commit từ nhiều actor phải được linearize theo khóa ổn định:
   - `logical_time`
   - `actor_id`
   - `task_id`
   - `event_id`
2. Cùng `event_id + cert_hash` commit lặp là idempotent (no-op).
3. Không có implicit reorder theo thread race.

### 1.5 Build artifact (LOCK)
1. Artifact chuẩn: `.oclpkg` single-file.
2. Mặc định `ocl build` xuất `.oclpkg`.
3. `--source-only` xuất `.oclbundle` để compat.
4. `--compiled-only` chỉ dùng nội bộ benchmark.

### 1.6 Reproducibility metadata (LOCK, mới)
1. Reproducible build phải canonical:
   - path order deterministic
   - LF canonical cho text files
   - fixed metadata (`SOURCE_DATE_EPOCH`, canonical timestamps)
2. Same input -> same bundle hash cross-OS.

### 1.7 Registry & supply chain (LOCK)
1. Resolution order:
   - project overlay (pinned khi `--locked`)
   - workspace cache
   - remote registry (signature-verified)
   - builtin stdlib read-only
2. Exact pin bắt buộc trong mọi đường `--locked`: `name+version+source+hash`.
3. Signature bắt buộc cho remote packages và adapter binaries.

### 1.8 Trust model (LOCK, mới)
1. Có trust-root store local.
2. Có key rotation + revocation list.
3. Có timestamp policy cho signature validity.
4. Offline verify policy được định nghĩa rõ (pass/fail conditions).

### 1.9 Permission/IAM (LOCK)
1. Permission theo package/module, không chỉ toàn project.
2. Mỗi capability key phải đi qua policy `allow|deny|prompt`.
3. `prompt` chỉ local dev; CI không cho prompt.

### 1.10 Permission matching semantics (LOCK, mới)
1. `--locked`: key matching **exact**.
2. `--unlocked`/dev: có thể bật prefix policy có cảnh báo.
3. Verifier/runtime phải dùng cùng luật matching để tránh drift.

### 1.11 Audit & replay (LOCK)
1. Audit log bắt buộc cho capability calls + commit outcomes.
2. Replay mode tái hiện deterministic từ audit + fixtures.

### 1.12 Tamper-evident audit chain (LOCK, mới)
1. Mỗi record có `prev_hash` + `record_hash` (hash-chain).
2. Verify audit fail-hard nếu chain bị đứt.

### 1.13 Testing (LOCK)
1. `ocl test` là runner chính thức/source-of-truth.
2. Rust conformance vẫn giữ để kiểm sâu runtime.
3. Có deterministic harness (`--seed`, fixture injection, record/replay).

### 1.14 Composer (LOCK)
1. Compose + verify + proof luôn song hành.
2. Search bounded + deterministic.
3. Multi-objective cost model.
4. Incremental re-assembly dựa trên hashes.

### 1.15 Plugin isolation model (LOCK, mới)
1. Mặc định plugin adapters chạy **out-of-process**.
2. In-process chỉ cho builtin signed adapters.
3. Runtime enforce timeout, cancellation, protocol-level memory guard, permission checks trước khi gọi plugin.

### 1.16 Migration compatibility (LOCK, mới)
1. `deps.lock.v1`, `catalog.lock.v1` read-compat.
2. Writer chuẩn chỉ ghi v2 canonical.
3. Có command migration rõ ràng (`lock sync` tự nâng hoặc subcommand migration tách riêng).

### 1.17 Exit code contract (LOCK, mới)
1. `check/test/build/verify-supply/conformance` có mapping exit code cố định.
2. Report luôn ghi trước khi trả mã lỗi.

---

## 2) Kiến trúc tổng thể v0.4

1. Language layer: syntax/type system/stdlib wrappers.
2. Compiler layer: parse -> typecheck/verifier -> IR -> optimize -> bytecode.
3. Runtime layer: actor scheduler + IO multiplex + capability bridge + audit/replay.
4. Capability layer: key registry + schema registry + ArgsToken/Payload typed + IAM.
5. Packaging layer: lock/signature/registry/cache/distribution.
6. Composer layer: phenotype -> assembly graph -> generated code + proof.
7. Tooling layer: test/trace/profile/debug/conformance.

---

## 3) Language v0.4 (đa dụng nhưng bounded)

### 3.1 Feature set
1. module/package/import
2. struct/enum, exhaustive pattern match
3. Result/Option + `?`
4. List/Map/Set (bounded)
5. generics tối thiểu (bounded implementation strategy)
6. async/await
7. spawn actor / spawn task (bounded)

### 3.2 Boundedness rules
1. call-depth cap
2. loop-step cap
3. mailbox depth cap
4. max tasks cap
5. await-chain cap (nếu bật)
6. budget exceed phải fail-honest (`INSUFFICIENT` + reason code)

---

## 4) Capability contract v0.4

### 4.1 Request/response typed contract
1. `Req = { key, tier_required, ctx, args_token, budgets }`
2. `args_token` giữ canonical schema id + canonical payload + hash.
3. Payload typed schema thật; truy cập qua `res.payload.<field>`.

### 4.2 Side-effect path
1. observe trả plan (`WritePlan`, `NetSendPlan`, `DbTxnPlan`, ...)
2. commit thực thi effect với binding check:
   - `event_id`
   - `cert_hash(req+args+policy+tick)`
   - idempotency

### 4.3 IAM per-package
1. Manifest khai báo capabilities family.
2. Permission section per-package/per-module.
3. Verifier fail-hard nếu gọi key ngoài allow policy.

### 4.4 Permission evaluation order
1. deny thắng allow.
2. allow exact trước allow prefix.
3. default deny khi không match.

### 4.5 Audit log schema v0.4
1. `call_id`, `logical_time`, `actor_id`, `task_id`
2. `key`, `args_hash`, `kind`, `reason`, `cert_hash`
3. `commit_event_id`, `commit_outcome`
4. `prev_hash`, `record_hash`

---

## 5) Concurrency + service semantics

### 5.1 Actor runtime
1. ActorId, typed mailbox, bounded queue.
2. `spawn actor` qua policy caps.
3. `handle(msg)` async path.

### 5.2 IO multiplexing
1. adapters cấp event source (socket/timer/io-ready).
2. `IoReady` delivered to actor/task theo scheduler policy.

### 5.3 Cancellation/deadline/backpressure
1. capability call luôn mang deadline.
2. cancellation token propagate xuyên call chain.
3. mailbox full -> `DEFERRED`/policy-explicit drop.
4. không có drop ngầm.

---

## 6) Compiler / IR / Bytecode

### 6.1 Pipeline
1. AST -> Typed IR
2. bounded optimizations:
   - constant fold
   - dead code elimination
   - small-fn inline (có cap)
3. bytecode VM deterministic + metering hooks

### 6.2 Debug metadata
1. source maps giữ span `file:line:col`.
2. trace/debugger sử dụng source map thống nhất.

---

## 7) Tooling v0.4

### 7.1 Trace
1. `ocl trace run ...` sinh `trace.jsonl`.
2. schema trace có actor/task/event/budget/capability fields.
3. deterministic mode dùng logical timestamps.

### 7.2 Profile
1. cost per key
2. allocations
3. scheduler stats (queue depth, wakeups, stalls)

### 7.3 Debug
1. `ocl trace view` text UI MVP.
2. breakpoint/step là stretch goal nếu không phá timeline.

---

## 8) FFI / Plugin ABI v1

### 8.1 ABI contract
1. `abi_version = 1`
2. entrypoints:
   - `init(registry)`
   - `observe(req)`
   - `commit(event)`
3. plugin phải declare:
   - keys provided
   - schemas
   - side-effect class

### 8.2 Runtime checks
1. permission check trước khi gọi plugin
2. signature check bắt buộc
3. timeout/cancel enforcement
4. protocol-level size caps (request/response)

---

## 9) Stdlib v0.4 baseline

Ship trong v0.4:
1. `std.fs` (read/write/list/stat/delete/rename; delete gated)
2. `std.net` (tcp/udp + http client + http server MVP)
3. `std.tls` (wrapper capability over native adapter)
4. `std.db` (sqlite MVP + generic db interface)
5. `std.ui` (minimal event-loop/window/draw bindings)
6. `std.game` (tick/frame/input/render bindings)
7. `std.crypto` (hash + random deterministic option)

Scope cap:
1. `std.ui` và `std.game` chỉ là minimal bindings trong v0.4.
2. Không thêm runtime semantics mới ngoài reactor/event locks.

---

## 10) Packaging/Registry v0.4

### 10.1 Lockfiles
1. `deps.lock.v2`: exact pin + signer + signature + source URL
2. `catalog.lock.v2`: pin catalog/templates/overlay
3. `policy.lock.v1`: pin policy profiles used by build

### 10.2 Artifact `.oclpkg`
1. `manifest (Ocl.toml)`
2. lockfiles
3. `vendor/` full canonical content
4. `compiled/` bytecode + source maps
5. `assembly_proof` (nếu dùng phenotype)
6. `signatures/` (artifact + lock + proof)

### 10.3 Commands
1. `ocl lock sync`
2. `ocl catalog lock sync`
3. `ocl build` -> `.oclpkg`
4. `ocl publish`
5. `ocl fetch`
6. `ocl verify-supply`

---

## 11) Composer v0.4

### 11.1 Phenotype v2
1. required capability edges
2. performance envelope (latency/cost)
3. risk tolerance
4. policy profile id

### 11.2 Cost model
1. per component:
   - `estimated_latency_ns`
   - `estimated_cost_units`
   - `risk_level`
2. objective:
   - minimize weighted cost/latency/risk
3. bounded search:
   - `max_nodes`, `max_steps`, `max_scan`
4. UNSAT fail-honest có reason

### 11.3 Incremental re-assembly
1. cache keys:
   - phenotype hash
   - catalog hash
   - lock hash
2. chỉ recompute subgraph bị ảnh hưởng
3. giữ stable IDs khi có thể

---

## 12) Testing & CI lanes v0.4

### 12.1 `ocl test` runner
1. snapshot/golden tests
2. deterministic fixtures
3. property tests (bounded)
4. record/replay tests từ audit logs

### 12.2 CI lanes
1. Blocking:
   - deterministic
   - locked
   - supply verify
2. Quarantine non-blocking:
   - real adapter smoke
   - local http/db/tls probe

---

## 13) End-to-end loops

### 13.1 Dev loop
1. `ocl fmt`
2. `ocl check --locked`
3. `ocl test --locked --seed ...`
4. nếu có phenotype: `ocl compose --locked` + `ocl verify --locked`
5. `ocl build --locked` -> `.oclpkg`
6. `ocl run <artifact>`

### 13.2 Deploy loop
1. `.oclpkg` là deploy unit duy nhất.
2. runtime compatibility matrix pinned trong manifest.
3. deploy gate phải pass `verify-supply`.

---

## 14) Workstreams triển khai v0.4

### W0 — Stabilize baseline (entry gate, OCL-only rebaseline)
1. Active surface chi con `projects/ocp-ocl/**`.
2. Legacy root runtime/test/docs/scripts duoc archive vao `/_archive/legacy-root-pre-v0.4/**` va mac dinh read-only.
3. Boundary guard dung merge-base semantics + root allowlist OCL-only.
4. Legacy lane entrypoints bi retire khoi active CI.

### W1 — Concurrency/runtime services
1. actor runtime + IO multiplex + cancel/deadline/backpressure
2. deterministic scheduler mode
3. AC-W1: mini-server local socket chạy thật + deterministic sim pass

### W2 — Capability IAM + audit/replay
1. per-package permissions + verifier
2. audit hash-chain + replay
3. AC-W2: deny policy fail-hard + replay reproducible

### W3 — IR/bytecode pipeline
1. typed IR + bytecode + metering
2. source maps
3. AC-W3: debug span đúng + throughput tăng so với interpreted path

### W4 — Packaging v2 + remote registry + signatures
1. lock v2 + signatures + verify-supply
2. `.oclpkg` single-file
3. AC-W4: build -> publish -> fetch -> verify -> run offline pass

### W5 — Debugger/profiler/trace
1. trace schema + viewer
2. profiler counters
3. AC-W5: triage được vấn đề runtime bằng trace/profile

### W6 — FFI/plugin ABI
1. ABI v1 + loader + signature verify
2. out-of-process plugin default
3. AC-W6: plugin key mới chạy được nhưng vẫn bị IAM enforce

### W7 — Stdlib expanded baseline
1. net/http server + tls wrapper + sqlite + ui/game minimal bindings
2. AC-W7: demo https client + sqlite app pass locked lane

### W8 — Composer v0.4
1. phenotype v2 + cost model + incremental
2. AC-W8: compose chọn provider theo objective; incremental recompose nhanh và proof ổn định

### W9 — Platform conformance + demos
1. 10 demo apps (5 cũ + 5 mới)
2. `ocl test --conformance --locked`
3. AC-W9: toàn bộ pass deterministic + locked lane

---

## 15) Acceptance Criteria v0.4 (DONE gate)

v0.4 chỉ được `DONE` khi đồng thời đạt:
1. Concurrency: actor + IO multiplex + cancellation/deadline/backpressure chạy production và deterministic lane.
2. Security/supply-chain: remote registry + exact pin + signatures + verify-supply pass.
3. Deploy: `.oclpkg` chạy offline, reproducible, có proofs.
4. IAM: per-package permissions enforce fail-hard.
5. Audit/replay: hash-chain verify pass + replay tái hiện deterministic.
6. Performance/tooling: IR/bytecode + metering + source maps + trace/profile usable.
7. Extensibility: plugin ABI v1 hoạt động và không bypass permission.
8. Composer: cost + incremental + proof ổn định.
9. Stdlib baseline: net/tls/sqlite/ui/game minimal binding pass demo.
10. Conformance: `ocl test --conformance --locked` xanh toàn bộ.

---

## 16) Anti-drift checklist (bắt buộc mỗi PR v0.4)

1. Có thêm IO direct path trong language/runtime không? Nếu có -> reject.
2. Có side-effect nào chạy ngoài commit không? Nếu có -> reject.
3. Có path `--locked` nào không enforce exact pin/signature không? Nếu có -> reject.
4. Có thay đổi scheduler làm deterministic order không ổn định không? Nếu có -> reject.
5. Có plugin path bypass IAM/runtime checks không? Nếu có -> reject.
6. Có thay đổi khiến same-input build khác hash không? Nếu có -> reject.

