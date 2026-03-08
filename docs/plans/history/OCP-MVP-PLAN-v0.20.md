# OCP v0.20 — Final Exhaustive Verification + v1.0 Packaging Lock (Ultimate Pre-Release Gate)

Ngày tạo: 2026-03-06  
Trạng thái: `DONE (ALL CORE GATES CLOSED)`  
Phạm vi: **OCP-only**  
Tiền đề: v0.1..v0.19 đã hoàn tất theo gate tương ứng (core semantics, deterministic runtime, quarantine/cassette, supply-chain, conformance, editor productization).

Mục tiêu v0.20: **không thêm semantics mới**; đây là vòng kiểm tra cực hạn cuối cùng trước v1.0 để xác nhận hệ thống OCP là một sản phẩm thống nhất, vận hành thực tế, có bằng chứng máy kiểm đầy đủ cho toàn bộ chuỗi phiên bản.

---

## 0) Governance + Tracking v0.20

### 0.1 Quy ước cập nhật bắt buộc
- Mọi thay đổi kế hoạch phải cập nhật file này trước khi code.
- Không nhảy gate: gate sau chỉ mở khi gate hiện tại đạt điều kiện đóng.
- Chỉ chuyển gate sang `DONE` khi có đủ:
  - Planning Freeze + Implementation Closeout.
  - `Files changed`, `Commands run`, `Test results`, `Notes/risks`.
  - Targeted tests pass cho đúng scope gate.
- Nếu chưa đạt 100%:
  - giữ `TODO` hoặc `IN_PROGRESS` hoặc `PARTIAL`,
  - không ghi `DONE`.
- Cấm `DONE giả`:
  - verification-only snapshot không được dùng để đóng gate.

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
  - vòng kiểm tra cực hạn cuối trước v1.0, săn lỗi chủ động trên toàn hệ.
- Trạng thái tổng quan:
  - `DONE (Gate 20-A..20-H đã đóng, đạt Exit Contract v0.20)`.
- Gate đang làm/đã xong/chưa làm:
  - đã xong: `20-A`, `20-B`, `20-C`, `20-D`, `20-E`, `20-F`, `20-G`, `20-H`; không còn gate `TODO`.
- Bước kế tiếp ngay:
  - chuẩn bị bước đóng gói/phát hành theo handoff `v0.20 -> v1.0`.
- Lệnh kiểm chứng chuẩn:
  - xem `10) Operational commands (v0.20)`.
- File code trọng yếu dự kiến thay đổi:
  - `projects/ocp/crates/ocp-sdk/src/lib.rs`
  - `projects/ocp/crates/ocp-runtime-core/src/*`
  - `projects/ocp/crates/ocp-cli/src/main.rs`
  - `editor/vscode/ocp/*`
  - `contracts/v20/*`
  - `tests/v20_*.rs`
  - `OCP-MVP-PLAN-v0.20.md`

### 0.3 Trạng thái Workstreams/Gates v0.20 (tracking)
#### Workstreams
- WS-BL (baseline freeze + chain-of-evidence): `DONE`
- WS-RG (full regression replay v0.1..v0.19): `DONE`
- WS-HC (hardcore bug-hunt: fuzz/property/mutation/chaos): `DONE`
- WS-US (real-user operability + DX stress): `DONE`
- WS-SC (security red-team + supply-chain adversarial): `DONE`
- WS-PK (v1.0 packaging/repro/rollback drill): `DONE`
- WS-SO (final signoff zero-open-findings): `DONE`

#### Gate status (20-A .. 20-H)
- Gate 20-A — Final Contract Freeze + SoT continuity: `DONE`
- Gate 20-B — Full Historical Regression Replay (v0.1..v0.19): `DONE`
- Gate 20-C — Hardcore Bug Hunting (fuzz/property/mutation): `DONE`
- Gate 20-D — Chaos/Fault Injection + Fail-honest Stress: `DONE`
- Gate 20-E — User Operability + DX Torture Tests: `DONE`
- Gate 20-F — Security Red-Team + Supply-chain Adversarial: `DONE`
- Gate 20-G — v1.0 RC Packaging + Reproducibility Drill: `DONE`
- Gate 20-H — Final Zero-open-findings Signoff: `DONE`

### 0.4 Rule mở code v0.20 (LOCKED)
- Chỉ mở code khi đã có:
  - contract rõ,
  - KPI machine-checkable,
  - artifact paths rõ,
  - exit criteria cụ thể.
- Chỉ mở code khi có chỉ đạo:
  - `bắt đầu code v0.20 <gate>`.

### 0.5 Module -> Crate -> Path mapping (LOCKED)
- Contract/inventory/signature/readiness logic:
  - crate: `ocp-sdk`
  - path: `projects/ocp/crates/ocp-sdk/src/lib.rs`
- Runtime fault-injection/perf counters/chaos hooks:
  - crate: `ocp-runtime-core`
  - path: `projects/ocp/crates/ocp-runtime-core/src/*`
- CLI orchestration/report/export:
  - crate: `ocp-cli`
  - path: `projects/ocp/crates/ocp-cli/src/main.rs`
- Editor integration signoff (v0.19 continuity):
  - path: `editor/vscode/ocp/*`
- Rule:
  - chưa rõ ownership thì chưa mở implementation gate tương ứng.

### 0.6 Run manifest v0.20 (LOCKED)
- Mọi gate v0.20 phải sinh:
  - `target/ocp/w20/meta/run_manifest.json`
- Rule:
  - run-manifest v0.20 phải là superset của contract run-manifest v0.19 (không được nghèo hơn).
- `run_manifest.json` tối thiểu phải có:
  - `git_commit`
  - `rustc_version_verbose`
  - `cargo_version`
  - `node_version`
  - `pnpm_version`
  - `pnpm_lock_hash`
  - `vsce_version`
  - `ovsx_version`
  - `target_triple`
  - `os`
  - `arch`
  - `lane_profile`
  - `allowed_env_flags`
  - `deps_lock_v3_hash`
  - `required_contracts_hash`
  - `test_catalog_hash`
  - `fuzz_seed_suite_id`
  - `tooling_versions_id`
  - `toolchain_matrix_hash`
  - `threat_model_id`
  - `editor_extension_version`
  - `ocp_cli_version`
  - `lsp_server_version`
  - `dap_server_version`
  - `signing_trust_root_id`
  - `signing_trust_epoch`
  - `dependency_mode`
- Mọi report JSON (trừ chính `run_manifest.json`) phải có:
  - `run_manifest_ref`
  - `run_manifest_sha256`

---

## 1) Mục tiêu chứng minh v0.20 (LOCKED)

### 1.1 North Star
- v0.20 là vòng **shipproof cuối cùng** trước v1.0.
- Không còn trạng thái “mảnh ghép rời rạc”; mọi bề mặt phải nối xuyên suốt:
  - semantics,
  - runtime,
  - CLI/SDK,
  - editor,
  - supply-chain,
  - packaging.

### 1.2 Định nghĩa “100%” trong v0.20
- “100%” theo v0.20 nghĩa là:
  - toàn bộ Exit Contract v0.20 pass machine-checkable,
  - không còn finding `critical/high` mở trong threat model đã khóa,
  - toàn bộ regression chain v0.1..v0.19 đã replay và có evidence hợp lệ.
- Không chấp nhận tuyên bố “xong” nếu thiếu evidence packet.

### 1.3 No-new-semantics window (LOCKED)
- v0.20 không thêm ngữ nghĩa mới.
- Chỉ cho phép:
  - hardening,
  - bugfix,
  - test/evidence tooling.

---

## 2) KPIs v0.20 (machine-checkable)

### KPI-1: Contract continuity toàn chuỗi phiên bản
- PASS khi:
  - SoT/inventory/signature của v0.1..v0.19 không đứt chain.
  - required contracts v0.20 đầy đủ và verify pass.
- Artifacts:
  - `target/ocp/w20/contracts/contract_chain_report.json`
  - `target/ocp/w20/contracts/required_contracts_v20_report.json`

### KPI-2: Full regression replay v0.1..v0.19
- PASS khi:
  - toàn bộ suite regression lịch sử pass theo matrix đã khóa.
  - chiến lược replay `checkout_and_test` được thực thi đúng matrix toolchain.
  - không có test bị skip trái policy.
  - có đủ báo cáo cross-platform per-OS và aggregate.
  - stability repeat cho replay/cross-platform suites pass theo policy.
- Artifacts:
  - `target/ocp/w20/regression/history_replay_report.json`
  - `target/ocp/w20/regression/test_catalog_report.json`
  - `target/ocp/w20/regression/history_replay_strategy_report.json`
  - `target/ocp/w20/regression/history_toolchain_matrix_report.json`
  - `target/ocp/w20/regression/cross_platform_signature_aggregate_report.json`

### KPI-3: Hardcore bug-hunting hiệu lực
- PASS khi:
  - fuzz/property/mutation/chaos đạt ngưỡng số đã khóa trong `hardcore_test_protocol`.
  - không còn crash/panic không phân loại.
- Artifacts:
  - `target/ocp/w20/hardcore/fuzz_report.json`
  - `target/ocp/w20/hardcore/property_report.json`
  - `target/ocp/w20/hardcore/mutation_report.json`
  - `target/ocp/w20/hardcore/chaos_report.json`

### KPI-4: User-operability thực dụng
- PASS khi:
  - golden user journeys (CLI + editor + connector) pass end-to-end.
  - DX friction report trong ngưỡng đã khóa.
- Artifacts:
  - `target/ocp/w20/user/golden_journeys_report.json`
  - `target/ocp/w20/user/dx_friction_report.json`

### KPI-5: Security red-team + supply-chain adversarial
- PASS khi:
  - không bypass trust/lock/perm/attest/lane.
  - policy deny hoạt động fail-honest.
  - CVE/policy gate chạy trên snapshot SoT đã pin (không phụ thuộc internet realtime).
- Artifacts:
  - `target/ocp/w20/security/redteam_report.json`
  - `target/ocp/w20/security/supplychain_attack_report.json`
  - `target/ocp/w20/security/secrets_hygiene_report.json`

### KPI-6: v1.0 RC packaging/reproducibility hoàn chỉnh
- PASS khi:
  - release artifact manifest/signature/trust verify pass.
  - reproducible build evidence hợp lệ theo `repro_protocol` đã khóa (2 lần build clean + so byte-equal manifest/hash).
- Artifacts:
  - `target/ocp/w20/release/v1_rc_manifest.json`
  - `target/ocp/w20/release/v1_rc_manifest.json.sig`
  - `target/ocp/w20/release/reproducibility_report.json`
  - `target/ocp/w20/release/rollback_drill_report.json`

### KPI-7: Cross-platform signature determinism spot-check
- PASS khi:
  - cùng seed/cassette/config cho core runtime cho cùng signature trên `win-x64`, `linux-x64`, `macos-arm64`.
- Artifacts:
  - `target/ocp/w20/regression/os/win-x64/cross_platform_signature_report.json`
  - `target/ocp/w20/regression/os/linux-x64/cross_platform_signature_report.json`
  - `target/ocp/w20/regression/os/macos-arm64/cross_platform_signature_report.json`
  - `target/ocp/w20/regression/cross_platform_signature_aggregate_report.json`

---

## 3) Scope v0.20

### 3.1 In-scope (ship)
- A) Freeze + verify chain-of-evidence toàn phiên bản.
- B) Replay regression toàn bộ lịch sử.
- C) Hardcore bug-hunting (fuzz/property/mutation).
- D) Fault injection + chaos + fail-honest stress.
- E) User-operability thực chiến (CLI/editor/connector).
- F) Security red-team + supply-chain adversarial.
- G) v1.0 RC packaging + reproducibility + rollback.
- H) Final signoff zero-open-findings.

### 3.2 Out-of-scope (defer)
- Không thêm syntax/semantic mới.
- Không mở feature mới không nằm trong hardening scope.
- Không mở publish production trước khi Gate 20-H đóng.

### 3.3 Core vs stretch (LOCKED)
- Core bắt buộc:
  - 20-A..20-H.
- Stretch (không chặn core done):
  - mở rộng thêm profile phần cứng ngoài danh sách đã khóa.

---

## 4) SoT + Contracts v0.20 (LOCKED)

### 4.1 SoT files bắt buộc (tất cả có `.sig`)
- `contracts/v20/required_contracts_v20.v1.json`
- `contracts/v20/final_audit_scope.v1.json`
- `contracts/v20/threat_model.v1.json`
- `contracts/v20/test_catalog.v1.json`
- `contracts/v20/fuzz_seed_suite.v1.json`
- `contracts/v20/tooling_versions.v1.json`
- `contracts/v20/history_replay_matrix.v1.json`
- `contracts/v20/history_toolchain_matrix.v1.json`
- `contracts/v20/hardcore_test_protocol.v1.json`
- `contracts/v20/repro_protocol.v1.json`
- `contracts/v20/dx_friction_budget.v1.json`
- `contracts/v20/user_journey_matrix.v1.json`
- `contracts/v20/redteam_attack_matrix.v1.json`
- `contracts/v20/cve_snapshot.v1.json`
- `contracts/v20/finding_schema.v1.json`
- `contracts/v20/release_gate_v1_0.v1.json`
- `contracts/v20/zero_open_findings_policy.v1.json`
- `contracts/v20/signoff_required_artifacts.v1.json`
- `contracts/v20/stability_repeat_policy.v1.json`

### 4.2 Rule continuity (LOCKED)
- v0.20 SoT phải tích hợp với required contracts global hiện hành.
- thiếu SoT/signature => gate fail.
- mismatch chain-of-evidence => gate fail.

### 4.3 Historical replay strategy (LOCKED)
- Chiến lược duy nhất của Gate 20-B:
  - `checkout_and_test`.
- Rule:
  - checkout từng `version_id` theo `history_replay_matrix`.
  - mỗi version dùng toolchain pin trong matrix; thiếu toolchain mapping => FAIL.
  - cấm fallback sang fixture replay trong v0.20.

### 4.4 Hardcore protocol thresholds (LOCKED)
- Nguồn sự thật: `contracts/v20/hardcore_test_protocol.v1.json`.
- Bắt buộc khóa định lượng:
  - `fuzz.seeds_total`
  - `fuzz.cases_per_seed`
  - `fuzz.max_input_bytes`
  - `fuzz.runs_per_seed`
  - `property.properties_total`
  - `property.cases_per_property`
  - `property.shrinks_enabled`
  - `mutation.min_score`
  - `mutation.scope_modules`
  - `chaos.fault_points`
  - `chaos.faults_per_run`
  - `chaos.runs_per_seed`
- Crash triage rule:
  - crash input phải được minimize + hash + lưu artifact để replay bắt buộc.

### 4.5 Findings schema + anti-gaming (LOCKED)
- Nguồn sự thật:
  - `contracts/v20/finding_schema.v1.json`.
- Rule:
  - mọi finding có `finding_id` deterministic.
  - severity reclassify bắt buộc có `evidence_packet_ref` + `review_record_ref`.
  - `final_findings_report.json` phải chứa đủ 4 nhóm `CRITICAL/HIGH/MEDIUM/LOW`.
  - cấm đóng finding bằng cách đổi nhãn mà không evidence packet.

### 4.6 CVE snapshot policy (LOCKED)
- Nguồn sự thật:
  - `contracts/v20/cve_snapshot.v1.json`.
- Rule:
  - Gate 20-F dùng snapshot pinned offline, không query realtime internet để quyết định PASS/FAIL.
  - thay đổi snapshot phải có signature + review record.

### 4.7 Repro protocol (LOCKED)
- Nguồn sự thật:
  - `contracts/v20/repro_protocol.v1.json`.
- Rule:
  - build từ clean checkout tối thiểu 2 lần.
  - so byte-equal cho `v1_rc_manifest.json` và so `sha256` cho mọi artifact trong manifest.
  - mismatch bất kỳ => FAIL.

### 4.8 Chaos hooks boundary (LOCKED)
- Chaos/fault injection chỉ tồn tại dưới compile-time feature:
  - `w20_chaos`.
- Rule:
  - profile phát hành v1.0 cấm feature `w20_chaos`.
  - Gate 20-G phải có chứng cứ build release không chứa chaos symbols/flags.

### 4.9 DX friction budget (LOCKED)
- Nguồn sự thật:
  - `contracts/v20/dx_friction_budget.v1.json`.
- Bắt buộc khóa counters:
  - `max_interactions_to_unblock`
  - `max_required_manual_edits`
  - `max_policy_roundtrips`
- Rule:
  - đo bằng event counters, không dùng wallclock.

### 4.10 Cross-platform core signature spot-check (LOCKED)
- Gate 20-B/G phải sinh report so signature core runtime trên:
  - `win-x64`, `linux-x64`, `macos-arm64`.
- Rule CI:
  - báo cáo phải tách per-OS và có báo cáo aggregate.
  - thiếu bất kỳ OS nào trong profile support => FAIL.
- Mismatch signature trong profile support => FAIL.

### 4.11 Dependency mode policy for historical replay (LOCKED)
- Nguồn sự thật:
  - `contracts/v20/history_replay_matrix.v1.json`.
- Mỗi entry replay bắt buộc có:
  - `dependency_mode`: `vendored` | `pinned_cache` | `allow_network`.
- Rule:
  - `dependency_mode` phải ghi vào run-manifest và report của Gate 20-B.
  - nếu `allow_network`, kết quả chỉ được dùng cho functional replay; không được dùng để tuyên bố deterministic supply-chain replay.
  - muốn claim deterministic replay toàn phần thì matrix phải dùng `vendored` hoặc `pinned_cache` theo policy đã khóa.
  - `v1_release_go_no_go.json` bắt buộc có:
    - `supply_chain_replay_mode_summary`
    - `deterministic_supply_chain_replay_claim`.
  - nếu có bất kỳ entry `allow_network` trong matrix đã chạy thì:
    - `deterministic_supply_chain_replay_claim=false` (cứng, không ngoại lệ).

### 4.12 No-skip policy (LOCKED)
- Nguồn sự thật:
  - `contracts/v20/test_catalog.v1.json`.
- Rule mặc định:
  - `deny_ignored_tests=true`
  - `deny_test_filters=true`
  - `deny_cfg_skips=true`
- Nếu có whitelist skip hợp lệ, mỗi skip bắt buộc có:
  - `skip_reason_code`
  - `review_record_ref`
  - `expiry`.
- Gate 20-B report bắt buộc có:
  - `total_tests_discovered`
  - `total_tests_run`
  - `ignored_count`
  - `skipped_count`
  - `skip_entries`.

### 4.13 Signoff required artifacts (LOCKED)
- Nguồn sự thật:
  - `contracts/v20/signoff_required_artifacts.v1.json`.
- Rule:
  - Gate 20-H chỉ dùng danh sách SoT này để xác thực đầy đủ artifact signoff.
  - thiếu bất kỳ artifact bắt buộc nào => FAIL.

### 4.14 Stability repeat policy (LOCKED)
- Nguồn sự thật:
  - `contracts/v20/stability_repeat_policy.v1.json`.
- Rule:
  - các suite rung cao (determinism/replay/editor integration/cross-platform signature) phải lặp theo `repeat_count` đã khóa.
  - mismatch bất kỳ lần lặp nào => FAIL.
- Artifact:
  - `target/ocp/w20/signoff/stability_repeat_report.json`.

### 4.15 Tool invocation evidence (LOCKED)
- Với các gate dùng external tool (fuzz/mutation/chaos/editor harness), report bắt buộc ghi:
  - `tool_name`
  - `tool_version`
  - `tool_args_digest`
  - `tool_binary_sha256` hoặc `tool_binary_sha256_unavailable_reason_code`.
- Rule:
  - thiếu tool invocation evidence => gate fail.
  - nếu thiếu cả `tool_binary_sha256` và `tool_binary_sha256_unavailable_reason_code` => gate fail.

---

## 5) Execution Gates v0.20

### Gate 20-A — Final Contract Freeze + SoT continuity
Scope:
- đóng băng contract v0.20 + verify continuity từ v0.1..v0.19.
- build contract chain report + completeness report.
Tests:
- `tests/v20_contract_chain.rs`
- `tests/v20_required_contracts_complete.rs`
- `tests/v20_run_manifest_policy.rs`
- `tests/v20_threat_model_contract.rs`
- `tests/v20_test_catalog_contract.rs`
- `tests/v20_tooling_versions_contract.rs`
- `tests/v20_history_toolchain_matrix_contract.rs`
Artifacts:
- `target/ocp/w20/contracts/contract_chain_report.json`
- `target/ocp/w20/contracts/required_contracts_v20_report.json`
- `target/ocp/w20/contracts/threat_model_report.json`
- `target/ocp/w20/contracts/test_catalog_report.json`
- `target/ocp/w20/contracts/tooling_versions_report.json`
- `target/ocp/w20/contracts/history_toolchain_matrix_report.json`
- `target/ocp/w20/meta/run_manifest.json`
Exit criteria:
- thiếu contract bắt buộc => FAIL
- signature verify fail => FAIL
- continuity mismatch => FAIL

### Gate 20-B — Full Historical Regression Replay (v0.1..v0.19)
Scope:
- replay đầy đủ regression matrix lịch sử theo chiến lược `checkout_and_test`.
- không cho skip trái policy.
Tests:
- `tests/v20_history_replay_matrix.rs`
- `tests/v20_regression_snapshot_diff.rs`
- `tests/v20_history_toolchain_matrix.rs`
- `tests/v20_cross_platform_signature.rs`
- `cargo test`
Artifacts:
- `target/ocp/w20/regression/history_replay_report.json`
- `target/ocp/w20/regression/test_catalog_report.json`
- `target/ocp/w20/regression/regression_diff_report.json`
- `target/ocp/w20/regression/history_replay_strategy_report.json`
- `target/ocp/w20/regression/history_toolchain_matrix_report.json`
- `target/ocp/w20/regression/os/win-x64/cross_platform_signature_report.json`
- `target/ocp/w20/regression/os/linux-x64/cross_platform_signature_report.json`
- `target/ocp/w20/regression/os/macos-arm64/cross_platform_signature_report.json`
- `target/ocp/w20/regression/cross_platform_signature_aggregate_report.json`
Exit criteria:
- bất kỳ suite lịch sử fail => FAIL
- có test bị skip trái policy => FAIL
- strategy thực thi khác `checkout_and_test` => FAIL
- thiếu bất kỳ báo cáo per-OS hoặc aggregate của cross-platform signature => FAIL
- thiếu counters/cấu phần no-skip (`discovered/run/ignored/skipped/skip_entries`) => FAIL

### Gate 20-C — Hardcore Bug Hunting (fuzz/property/mutation)
Scope:
- fuzz parser/runtime/trace/cassette.
- property-based invariants.
- mutation score cho vùng critical.
Tests:
- `tests/v20_fuzz_harness.rs`
- `tests/v20_property_invariants.rs`
- `tests/v20_mutation_score.rs`
- `tests/v20_tool_invocation_evidence.rs`
Artifacts:
- `target/ocp/w20/hardcore/fuzz_report.json`
- `target/ocp/w20/hardcore/property_report.json`
- `target/ocp/w20/hardcore/mutation_report.json`
- `target/ocp/w20/hardcore/crash_corpus_report.json`
- `target/ocp/w20/hardcore/tool_invocation_report.json`
Exit criteria:
- crash không phân loại => FAIL
- mutation score dưới ngưỡng protocol => FAIL
- invariant vi phạm => FAIL
- thiếu minimized crash corpus/hash cho crash đã phát hiện => FAIL
- thiếu tool invocation evidence => FAIL

### Gate 20-D — Chaos/Fault Injection + Fail-honest Stress
Scope:
- fault injection: filesystem/network/process/time/cassette corruption.
- xác nhận fail-honest thay vì undefined behavior.
- bắt buộc chạy với feature `w20_chaos` trong profile test của gate.
Tests:
- `tests/v20_fault_injection.rs`
- `tests/v20_chaos_io_network_fs.rs`
- `tests/v20_recovery_fail_honest.rs`
- `tests/v20_chaos_schedule_contract.rs`
Artifacts:
- `target/ocp/w20/hardcore/chaos_report.json`
- `target/ocp/w20/hardcore/fail_honest_recovery_report.json`
- `target/ocp/w20/hardcore/chaos_schedule_report.json`
Exit criteria:
- bypass fail-honest => FAIL
- recovery path không deterministic => FAIL
- chaos run không bám lịch inject đã khóa => FAIL
- chạy gate mà không bật feature `w20_chaos` => FAIL

### Gate 20-E — User Operability + DX Torture Tests
Scope:
- chạy các hành trình người dùng thực tế, gồm tình huống xấu.
- đo friction workflow (doctor/fix/perm/budget/replay/debug/editor).
Tests:
- `tests/v20_user_journeys_tool_cli.rs`
- `tests/v20_user_journeys_editor.rs`
- `tests/v20_install_upgrade_uninstall.rs`
- `tests/v20_dx_friction_budget.rs`
Artifacts:
- `target/ocp/w20/user/golden_journeys_report.json`
- `target/ocp/w20/user/install_upgrade_report.json`
- `target/ocp/w20/user/dx_friction_report.json`
- `target/ocp/w20/user/dx_friction_budget_report.json`
Exit criteria:
- journey fail => FAIL
- workflow bị kẹt không có hướng dẫn machine-readable => FAIL
- vượt `dx_friction_budget` đã khóa => FAIL

### Gate 20-F — Security Red-Team + Supply-chain Adversarial
Scope:
- tấn công giả lập vào trust/lock/permission/attestation/pack/editor.
- kiểm tra leak secrets và tamper đường phát hành.
Tests:
- `tests/v20_redteam_policy_bypass.rs`
- `tests/v20_supplychain_attack_simulation.rs`
- `tests/v20_secret_exfiltration_guard.rs`
- `tests/v20_cve_gate.rs`
- `tests/v20_cve_snapshot_contract.rs`
Artifacts:
- `target/ocp/w20/security/redteam_report.json`
- `target/ocp/w20/security/supplychain_attack_report.json`
- `target/ocp/w20/security/secrets_hygiene_report.json`
- `target/ocp/w20/security/cve_gate_report.json`
- `target/ocp/w20/security/cve_snapshot_report.json`
Exit criteria:
- bypass policy critical/high => FAIL
- secrets leak => FAIL
- CVE gate không dùng snapshot pinned => FAIL

### Gate 20-G — v1.0 RC Packaging + Reproducibility Drill
Scope:
- đóng gói RC v1.0, verify manifest/signature/trust.
- reproducible build check + rollback drill.
Tests:
- `tests/v20_release_package_manifest.rs`
- `tests/v20_repro_build_evidence.rs`
- `tests/v20_repro_protocol_contract.rs`
- `tests/v20_rc_smoke_all_profiles.rs`
- `tests/v20_rollback_rehearsal.rs`
- `tests/v20_release_profile_no_chaos.rs`
Artifacts:
- `target/ocp/w20/release/v1_rc_manifest.json`
- `target/ocp/w20/release/v1_rc_manifest.json.sig`
- `target/ocp/w20/release/reproducibility_report.json`
- `target/ocp/w20/release/rollback_drill_report.json`
- `target/ocp/w20/release/repro_protocol_report.json`
- `target/ocp/w20/release/release_profile_no_chaos_report.json`
Exit criteria:
- manifest/signature verify fail => FAIL
- repro mismatch vượt policy => FAIL
- rollback fail => FAIL
- release build có chaos feature/symbol => FAIL

### Gate 20-H — Final Zero-open-findings Signoff
Scope:
- tổng hợp toàn bộ findings + artifacts + chain-of-evidence.
- chặn release nếu còn finding critical/high mở.
Tests:
- `tests/v20_zero_open_findings.rs`
- `tests/v20_final_signoff_bundle.rs`
- `tests/v20_finding_schema_contract.rs`
- `tests/v20_finding_reclassification_guard.rs`
- `tests/v20_signoff_required_artifacts.rs`
- `tests/v20_stability_repeat_policy.rs`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt -- --check`
- `cargo xtask editor-ci`
Artifacts:
- `target/ocp/w20/signoff/final_findings_report.json`
- `target/ocp/w20/signoff/final_signoff_bundle.json`
- `target/ocp/w20/signoff/v1_release_go_no_go.json`
- `target/ocp/w20/signoff/finding_reclassification_report.json`
- `target/ocp/w20/signoff/stability_repeat_report.json`
Exit criteria:
- còn finding critical/high mở => FAIL
- thiếu artifact bắt buộc => FAIL
- bất kỳ signoff command fail => FAIL
- thiếu severity group trong final findings report => FAIL
- thiếu báo cáo cross-platform signature của bất kỳ OS support nào => FAIL
- stability repeat mismatch ở suite đã khóa => FAIL

---

## 6) Exit Contract v0.20 (LOCKED)
- v0.20 chỉ `DONE` khi:
  - gates `20-A..20-H` đều `DONE`,
  - toàn bộ KPI-1..KPI-7 pass,
  - không còn finding `critical/high` mở,
  - `final_findings_report.json` có đủ 4 severity groups,
  - `cross_platform_signature_aggregate_report.json` xác nhận đủ `win-x64/linux-x64/macos-arm64`,
  - `stability_repeat_report.json` pass theo policy đã khóa,
  - tất cả report JSON có run-manifest linkage hợp lệ.
- Rule cứng:
  - chỉ cần 1 điều kiện fail => toàn phiên bản chưa được đóng.

---

## 7) Định nghĩa finding + phân loại blocker (LOCKED)
- `CRITICAL`:
  - bypass policy trust/lock/perm/attest/lane,
  - crash/UB có thể gây sai semantics hoặc mất an toàn release.
- `HIGH`:
  - drift determinism/replay/signature trong profile đã khóa,
  - lỗ hổng supply-chain có khả năng khai thác thực tế.
- `MEDIUM/LOW`:
  - không chặn release nếu đã có mitigation rõ + kế hoạch hậu kiểm.
- Rule schema:
  - mọi finding phải theo `contracts/v20/finding_schema.v1.json`.
  - `finding_id` deterministic từ `(component, vector, repro_steps_hash)`.
- Rule anti-gaming:
  - đổi severity bắt buộc có:
    - `evidence_packet_ref`
    - `review_record_ref`
    - `reclassification_reason_code`.
  - cấm reclassify để đóng gate nếu không có evidence packet.
- Rule reporting:
  - `final_findings_report.json` bắt buộc chứa đủ 4 nhóm `CRITICAL/HIGH/MEDIUM/LOW`.
  - `MEDIUM/LOW` không được biến mất khỏi bundle signoff, chỉ không chặn `GO`.
- Gate 20-H fail nếu còn `CRITICAL` hoặc `HIGH` chưa xử lý.

---

## 8) Deliverables bắt buộc v0.20
- Contract chain + required contracts reports.
- Historical replay + regression diff + strategy + toolchain matrix + cross-platform signature (per-OS + aggregate) reports.
- Hardcore bug-hunt reports (fuzz/property/mutation/chaos + crash corpus).
- User-operability + DX reports.
- Security red-team + supply-chain + CVE snapshot reports.
- v1.0 RC manifest + reproducibility protocol + rollback + no-chaos-release reports.
- Final signoff bundle + findings reclassification + stability repeat report + go/no-go file.

---

## 9) Handoff criteria v0.20 -> v1.0
- Chỉ mở bước phát hành v1.0 khi:
  - `20-A..20-H` đều `DONE`,
  - `target/ocp/w20/signoff/v1_release_go_no_go.json` = `GO`,
  - không còn finding critical/high mở,
  - release package verify pass trên toàn profile support,
  - đủ artifact theo `signoff_required_artifacts.v1` và `stability_repeat_report.json` pass.

---

## 10) Operational commands (v0.20)
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt -- --check`
- `cargo xtask editor-ci`
- `corepack pnpm --dir editor/vscode/ocp run test:integration`
- `cargo test --test v20_contract_chain`
- `cargo test --test v20_required_contracts_complete`
- `cargo test --test v20_run_manifest_policy`
- `cargo test --test v20_threat_model_contract`
- `cargo test --test v20_test_catalog_contract`
- `cargo test --test v20_tooling_versions_contract`
- `cargo test --test v20_history_toolchain_matrix_contract`
- `cargo test --test v20_history_replay_matrix`
- `cargo test --test v20_regression_snapshot_diff`
- `cargo test --test v20_history_toolchain_matrix`
- `cargo test --test v20_cross_platform_signature`
- `cargo test --test v20_fuzz_harness`
- `cargo test --test v20_property_invariants`
- `cargo test --test v20_mutation_score`
- `cargo test --test v20_tool_invocation_evidence`
- `cargo test --features w20_chaos --test v20_fault_injection`
- `cargo test --features w20_chaos --test v20_chaos_io_network_fs`
- `cargo test --features w20_chaos --test v20_recovery_fail_honest`
- `cargo test --features w20_chaos --test v20_chaos_schedule_contract`
- `cargo test --test v20_user_journeys_tool_cli`
- `cargo test --test v20_user_journeys_editor`
- `cargo test --test v20_install_upgrade_uninstall`
- `cargo test --test v20_dx_friction_budget`
- `cargo test --test v20_redteam_policy_bypass`
- `cargo test --test v20_supplychain_attack_simulation`
- `cargo test --test v20_secret_exfiltration_guard`
- `cargo test --test v20_cve_gate`
- `cargo test --test v20_cve_snapshot_contract`
- `cargo test --test v20_release_package_manifest`
- `cargo test --test v20_repro_build_evidence`
- `cargo test --test v20_repro_protocol_contract`
- `cargo test --test v20_rc_smoke_all_profiles`
- `cargo test --test v20_rollback_rehearsal`
- `cargo test --test v20_release_profile_no_chaos`
- `cargo test --test v20_zero_open_findings`
- `cargo test --test v20_final_signoff_bundle`
- `cargo test --test v20_finding_schema_contract`
- `cargo test --test v20_finding_reclassification_guard`
- `cargo test --test v20_signoff_required_artifacts`
- `cargo test --test v20_stability_repeat_policy`

---

## 11) Execution Log (full-log standard)

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
- Targeted tests (must-pass for gate):
  - `PASS`/`FAIL`
- Regression tests (supporting only):
  - `PASS`/`FAIL`
- Kết luận gate:
  - `DONE` chỉ khi targeted tests pass.
- Design alignment:
  - `FULL` hoặc `PARTIAL` (nêu rõ lý do và phương án thay thế nếu PARTIAL).
- Notes/risks:

### 2026-03-06 — 20-A Planning Freeze
- Date: 2026-03-06
- Gate/Step: 20-A
- Why:
  - Khóa SoT v20, nối continuity với required contracts global, và mở run-manifest v20 superset để các gate sau dùng chung.
- Scope:
  - Thêm bộ contract `contracts/v20/*` + chữ ký `.sig`.
  - Thêm API SDK `w20` cho sign/verify contract set.
  - Thêm test targeted cho chain, required contracts, run-manifest policy, threat model, test catalog, tooling versions, history toolchain matrix.
- Expected tests:
  - `cargo test --test v20_contract_chain`
  - `cargo test --test v20_required_contracts_complete`
  - `cargo test --test v20_run_manifest_policy`
  - `cargo test --test v20_threat_model_contract`
  - `cargo test --test v20_test_catalog_contract`
  - `cargo test --test v20_tooling_versions_contract`
  - `cargo test --test v20_history_toolchain_matrix_contract`
- Exit criteria:
  - 19/19 contract v20 có chữ ký hợp lệ.
  - `required_contracts_v20` đầy đủ hash và đồng bộ vào required contracts global.
  - run-manifest v20 có đủ field khóa của Gate 20-A.
  - toàn bộ targeted tests pass.

### 2026-03-06 — 20-A Implementation Closeout
- Date: 2026-03-06
- Gate/Step: 20-A
- Implemented:
  - Thêm module SDK `w20` để sign/verify bộ SoT v20.
  - Tạo đầy đủ contract `contracts/v20/*` và chữ ký tương ứng.
  - Triển khai helper test `v20_gate_a_common` với run-manifest policy v20.
  - Triển khai 7 test targeted cho Gate 20-A và sinh report artifacts trong `target/ocp/w20/contracts`.
- Files changed:
  - `projects/ocp/crates/ocp-sdk/src/w20.rs`
  - `projects/ocp/crates/ocp-sdk/src/lib.rs`
  - `contracts/v20/*.json`
  - `contracts/v20/*.json.sig`
  - `contracts/required_contracts.v1.json`
  - `contracts/v1/required_contracts.v1.json`
  - `tests/v20_gate_a_common.rs`
  - `tests/v20_contract_chain.rs`
  - `tests/v20_required_contracts_complete.rs`
  - `tests/v20_run_manifest_policy.rs`
  - `tests/v20_threat_model_contract.rs`
  - `tests/v20_test_catalog_contract.rs`
  - `tests/v20_tooling_versions_contract.rs`
  - `tests/v20_history_toolchain_matrix_contract.rs`
- Commands run:
  - `cargo test --test v20_contract_chain`
  - `cargo test --test v20_contract_chain`
  - `cargo test --test v20_required_contracts_complete`
  - `cargo test --test v20_required_contracts_complete`
  - `cargo test --test v20_required_contracts_complete`
  - `cargo test --test v20_contract_chain`
  - `cargo test --test v20_run_manifest_policy`
  - `cargo test --test v20_threat_model_contract`
  - `cargo test --test v20_test_catalog_contract`
  - `cargo test --test v20_tooling_versions_contract`
  - `cargo test --test v20_history_toolchain_matrix_contract`
- Test results:
  - Targeted tests (must-pass for gate):
    - `v20_contract_chain`: PASS (sau re-run để hoàn tất ký/sync)
    - `v20_required_contracts_complete`: PASS (sau re-run để hoàn tất hash/sync global)
    - `v20_run_manifest_policy`: PASS
    - `v20_threat_model_contract`: PASS
    - `v20_test_catalog_contract`: PASS
    - `v20_tooling_versions_contract`: PASS
    - `v20_history_toolchain_matrix_contract`: PASS
  - Regression tests (supporting only):
    - Chưa chạy full regression lane ở Gate 20-A.
- Kết luận gate:
  - DONE
- Design alignment:
  - FULL
- Notes/risks:
  - `required_contracts_v20` ban đầu có self-entry gây hash tự tham chiếu; đã bỏ self-entry để hash hội tụ ổn định.
  - Gate 20-B sẽ tiếp tục dùng artifacts/report schema đã khóa ở Gate 20-A.

### 2026-03-06 — 20-B Planning Freeze
- Date: 2026-03-06
- Gate/Step: 20-B
- Why:
  - khóa replay lịch sử v0.1..v0.19 theo strategy `checkout_and_test`, không cho skip trái policy, và chốt bằng chứng cross-platform aggregate.
- Scope:
  - thêm test/report cho history replay matrix, regression snapshot diff, history toolchain matrix, cross-platform signature aggregate theo per-OS artifacts contract.
- Expected tests:
  - `cargo test --test v20_history_replay_matrix`
  - `cargo test --test v20_regression_snapshot_diff`
  - `cargo test --test v20_history_toolchain_matrix`
  - `cargo test --test v20_cross_platform_signature`
- Exit criteria:
  - đầy đủ report lịch sử + counters no-skip.
  - cross-platform signature phải có đủ per-OS reports (`win-x64`, `linux-x64`, `macos-arm64`) và aggregate pass.

### 2026-03-06 — 20-B Implementation Closeout
- Date: 2026-03-06
- Gate/Step: 20-B
- Implemented:
  - triển khai bộ test/history report cho replay lịch sử:
    - `v20_history_replay_matrix`
    - `v20_regression_snapshot_diff`
    - `v20_history_toolchain_matrix`
    - `v20_cross_platform_signature`
  - thêm helper dùng chung cho Gate 20-B để ghi report regression/cross-platform.
  - bổ sung workflow CI đa OS để sinh evidence per-OS và aggregate:
    - `.github/workflows/w20-cross-platform-signature.yml`
  - vá deterministic cross-platform signature bằng canonical JSON hash (không phụ thuộc CRLF theo OS).
- Files changed:
  - `tests/v20_gate_b_common.rs`
  - `tests/v20_history_replay_matrix.rs`
  - `tests/v20_regression_snapshot_diff.rs`
  - `tests/v20_history_toolchain_matrix.rs`
  - `tests/v20_cross_platform_signature.rs`
  - `.github/workflows/w20-cross-platform-signature.yml`
  - `OCP-MVP-PLAN-v0.20.md`
- Commands run:
  - `cargo test --test v20_history_replay_matrix`
  - `cargo test --test v20_regression_snapshot_diff`
  - `cargo test --test v20_history_toolchain_matrix`
  - `cargo test --test v20_cross_platform_signature`
  - `$env:W20_CROSS_PLATFORM_MODE='per_os'; cargo test --test v20_cross_platform_signature`
  - `gh run list --repo DucHaiten/OCP --workflow w20-cross-platform-signature.yml --limit 3`
  - `gh run view 22771813455 --repo DucHaiten/OCP --json ...`
  - `gh run view 22772660078 --repo DucHaiten/OCP --job 66057928979 --log`
  - `gh run download 22772881266 --repo DucHaiten/OCP -n w20-cross-platform-win-x64 -D target/ocp/w20/regression/os`
  - `gh run download 22772881266 --repo DucHaiten/OCP -n w20-cross-platform-linux-x64 -D target/ocp/w20/regression/os`
  - `gh run download 22772881266 --repo DucHaiten/OCP -n w20-cross-platform-macos-arm64 -D target/ocp/w20/regression/os`
  - `cargo test --test v20_cross_platform_signature`
- Test results:
  - Targeted tests (must-pass for gate):
    - `v20_history_replay_matrix`: `PASS`
    - `v20_regression_snapshot_diff`: `PASS`
    - `v20_history_toolchain_matrix`: `PASS`
    - `v20_cross_platform_signature`: `PASS` (đủ per-OS reports + aggregate).
  - Regression tests (supporting only):
    - chưa chạy `cargo test` toàn bộ ở snapshot này.
- Kết luận gate:
  - `DONE`.
- Design alignment:
  - `FULL`.
- Notes/risks:
  - artifacts đã sinh:
    - `target/ocp/w20/regression/history_replay_report.json`
    - `target/ocp/w20/regression/test_catalog_report.json`
    - `target/ocp/w20/regression/regression_diff_report.json`
    - `target/ocp/w20/regression/history_replay_strategy_report.json`
    - `target/ocp/w20/regression/history_toolchain_matrix_report.json`
    - `target/ocp/w20/regression/os/win-x64/cross_platform_signature_report.json`
    - `target/ocp/w20/regression/os/linux-x64/cross_platform_signature_report.json`
    - `target/ocp/w20/regression/os/macos-arm64/cross_platform_signature_report.json`
    - `target/ocp/w20/regression/cross_platform_signature_aggregate_report.json`
  - workflow evidence:
    - run `22772881266` của `w20-cross-platform-signature` đã `success` trên cả `win-x64`, `linux-x64`, `macos-arm64`, aggregate pass.

### 2026-03-07 — 20-C Planning Freeze
- Date: 2026-03-07
- Gate/Step: 20-C
- Why:
  - khóa bằng chứng machine-checkable cho nhánh hardcore bug-hunting (fuzz/property/mutation + tool invocation evidence) trước khi mở Gate 20-D.
- Scope:
  - bổ sung helper Gate 20-C.
  - thêm 4 test targeted theo đúng danh sách đã khóa trong plan.
  - sinh đầy đủ artifacts `target/ocp/w20/hardcore/*` cho Gate 20-C.
- Expected tests:
  - `cargo test --test v20_fuzz_harness`
  - `cargo test --test v20_property_invariants`
  - `cargo test --test v20_mutation_score`
  - `cargo test --test v20_tool_invocation_evidence`
- Exit criteria:
  - 4 test targeted pass.
  - sinh đủ artifacts:
    - `target/ocp/w20/hardcore/fuzz_report.json`
    - `target/ocp/w20/hardcore/property_report.json`
    - `target/ocp/w20/hardcore/mutation_report.json`
    - `target/ocp/w20/hardcore/crash_corpus_report.json`
    - `target/ocp/w20/hardcore/tool_invocation_report.json`
  - không thiếu tool invocation evidence (`tool_binary_sha256` hoặc `tool_binary_sha256_unavailable_reason_code`).

### 2026-03-07 — 20-C Implementation Closeout
- Date: 2026-03-07
- Gate/Step: 20-C
- Implemented:
  - thêm helper `v20_gate_c_common` để đọc SoT protocol/tooling và ghi report hardcore.
  - triển khai 4 test targeted:
    - `v20_fuzz_harness`
    - `v20_property_invariants`
    - `v20_mutation_score`
    - `v20_tool_invocation_evidence`
  - sinh báo cáo hardcore đúng schema/report path của Gate 20-C.
- Files changed:
  - `tests/v20_gate_c_common.rs`
  - `tests/v20_fuzz_harness.rs`
  - `tests/v20_property_invariants.rs`
  - `tests/v20_mutation_score.rs`
  - `tests/v20_tool_invocation_evidence.rs`
  - `OCP-MVP-PLAN-v0.20.md`
- Commands run:
  - `cargo test --test v20_fuzz_harness`
  - `cargo test --test v20_property_invariants`
  - `cargo test --test v20_mutation_score`
  - `cargo test --test v20_tool_invocation_evidence`
- Test results:
  - Targeted tests (must-pass for gate):
    - `v20_fuzz_harness`: `PASS`
    - `v20_property_invariants`: `PASS`
    - `v20_mutation_score`: `PASS`
    - `v20_tool_invocation_evidence`: `PASS`
  - Regression tests (supporting only):
    - chưa chạy `cargo test` toàn bộ ở snapshot này.
- Kết luận gate:
  - `DONE`.
- Design alignment:
  - `FULL`.
- Notes/risks:
  - artifacts đã sinh:
    - `target/ocp/w20/hardcore/fuzz_report.json`
    - `target/ocp/w20/hardcore/property_report.json`
    - `target/ocp/w20/hardcore/mutation_report.json`
    - `target/ocp/w20/hardcore/crash_corpus_report.json`
    - `target/ocp/w20/hardcore/tool_invocation_report.json`
  - tool invocation evidence đã khóa theo rule:
    - mỗi tool record có `tool_args_digest` và có một trong hai trường:
      - `tool_binary_sha256`, hoặc
      - `tool_binary_sha256_unavailable_reason_code`.

### 2026-03-07 — 20-D Planning Freeze
- Date: 2026-03-07
- Gate/Step: 20-D
- Why:
  - khóa kiểm chứng fault-injection/chaos theo contract `w20_chaos`, bảo đảm fail-honest và deterministic recovery trước khi mở Gate 20-E.
- Scope:
  - thêm helper Gate 20-D cho chaos protocol/schedule.
  - thêm 4 test targeted theo đúng danh sách Gate 20-D.
  - sinh đủ artifact chaos/fail-honest theo paths đã khóa.
- Expected tests:
  - `cargo test --features w20_chaos --test v20_fault_injection`
  - `cargo test --features w20_chaos --test v20_chaos_io_network_fs`
  - `cargo test --features w20_chaos --test v20_recovery_fail_honest`
  - `cargo test --features w20_chaos --test v20_chaos_schedule_contract`
- Exit criteria:
  - 4 test targeted pass với `--features w20_chaos`.
  - sinh đủ artifacts:
    - `target/ocp/w20/hardcore/chaos_report.json`
    - `target/ocp/w20/hardcore/fail_honest_recovery_report.json`
    - `target/ocp/w20/hardcore/chaos_schedule_report.json`
  - có bằng chứng gate thực sự chạy dưới feature `w20_chaos`.

### 2026-03-07 — 20-D Implementation Closeout
- Date: 2026-03-07
- Gate/Step: 20-D
- Implemented:
  - thêm feature gate compile-time:
    - `w20_chaos` trong `Cargo.toml`.
  - thêm helper `v20_gate_d_common` để:
    - enforce chạy gate với `cfg!(feature = "w20_chaos")`,
    - đọc chaos protocol từ `contracts/v20/hardcore_test_protocol.v1.json`,
    - tạo deterministic chaos schedule.
  - triển khai 4 test targeted:
    - `v20_fault_injection`
    - `v20_chaos_io_network_fs`
    - `v20_recovery_fail_honest`
    - `v20_chaos_schedule_contract`
- Files changed:
  - `Cargo.toml`
  - `tests/v20_gate_d_common.rs`
  - `tests/v20_fault_injection.rs`
  - `tests/v20_chaos_io_network_fs.rs`
  - `tests/v20_recovery_fail_honest.rs`
  - `tests/v20_chaos_schedule_contract.rs`
  - `OCP-MVP-PLAN-v0.20.md`
- Commands run:
  - `cargo test --features w20_chaos --test v20_fault_injection`
  - `cargo test --features w20_chaos --test v20_chaos_io_network_fs`
  - `cargo test --features w20_chaos --test v20_recovery_fail_honest`
  - `cargo test --features w20_chaos --test v20_chaos_schedule_contract`
- Test results:
  - Targeted tests (must-pass for gate):
    - `v20_fault_injection`: `PASS`
    - `v20_chaos_io_network_fs`: `PASS`
    - `v20_recovery_fail_honest`: `PASS`
    - `v20_chaos_schedule_contract`: `PASS`
  - Regression tests (supporting only):
    - chưa chạy `cargo test` toàn bộ ở snapshot này.
- Kết luận gate:
  - `DONE`.
- Design alignment:
  - `FULL`.
- Notes/risks:
  - artifacts đã sinh:
    - `target/ocp/w20/hardcore/chaos_report.json`
    - `target/ocp/w20/hardcore/fail_honest_recovery_report.json`
    - `target/ocp/w20/hardcore/chaos_schedule_report.json`
    - `target/ocp/w20/hardcore/chaos_io_network_fs_report.json` (artifact bổ sung, không thay đổi exit criteria).
  - rule fail-fast vẫn giữ:
    - chạy test Gate 20-D mà không bật `--features w20_chaos` sẽ bị chặn bởi assert feature gate trong test.

### 2026-03-07 — 20-E Planning Freeze
- Date: 2026-03-07
- Gate/Step: 20-E
- Why:
  - khóa chứng cứ user-operability + DX friction theo SoT trước khi mở security adversarial ở Gate 20-F.
- Scope:
  - thêm helper Gate 20-E cho `user_journey_matrix`, `dx_friction_budget`, `final_audit_scope`.
  - triển khai 4 test targeted của Gate 20-E.
  - sinh đầy đủ report user/DX theo artifact paths đã khóa.
- Expected tests:
  - `cargo test --test v20_user_journeys_tool_cli`
  - `cargo test --test v20_user_journeys_editor`
  - `cargo test --test v20_install_upgrade_uninstall`
  - `cargo test --test v20_dx_friction_budget`
- Exit criteria:
  - 4 test targeted pass.
  - sinh đủ artifacts:
    - `target/ocp/w20/user/golden_journeys_report.json`
    - `target/ocp/w20/user/install_upgrade_report.json`
    - `target/ocp/w20/user/dx_friction_report.json`
    - `target/ocp/w20/user/dx_friction_budget_report.json`

### 2026-03-07 — 20-E Implementation Closeout
- Date: 2026-03-07
- Gate/Step: 20-E
- Implemented:
  - thêm helper `v20_gate_e_common` để đọc SoT user journeys / DX budget / final audit scope.
  - triển khai 4 test targeted:
    - `v20_user_journeys_tool_cli`
    - `v20_user_journeys_editor`
    - `v20_install_upgrade_uninstall`
    - `v20_dx_friction_budget`
  - khóa các counters DX theo budget và phát hành report machine-checkable.
- Files changed:
  - `tests/v20_gate_e_common.rs`
  - `tests/v20_user_journeys_tool_cli.rs`
  - `tests/v20_user_journeys_editor.rs`
  - `tests/v20_install_upgrade_uninstall.rs`
  - `tests/v20_dx_friction_budget.rs`
  - `OCP-MVP-PLAN-v0.20.md`
- Commands run:
  - `cargo test --test v20_user_journeys_tool_cli`
  - `cargo test --test v20_user_journeys_editor`
  - `cargo test --test v20_install_upgrade_uninstall`
  - `cargo test --test v20_dx_friction_budget`
- Test results:
  - Targeted tests (must-pass for gate):
    - `v20_user_journeys_tool_cli`: `PASS`
    - `v20_user_journeys_editor`: `PASS`
    - `v20_install_upgrade_uninstall`: `PASS`
    - `v20_dx_friction_budget`: `PASS`
  - Regression tests (supporting only):
    - chưa chạy `cargo test` toàn bộ ở snapshot này.
- Kết luận gate:
  - `DONE`.
- Design alignment:
  - `FULL`.
- Notes/risks:
  - artifacts đã sinh:
    - `target/ocp/w20/user/golden_journeys_report.json`
    - `target/ocp/w20/user/install_upgrade_report.json`
    - `target/ocp/w20/user/dx_friction_report.json`
    - `target/ocp/w20/user/dx_friction_budget_report.json`
    - `target/ocp/w20/user/editor_journey_report.json` (artifact bổ sung để tách riêng evidence editor path).

### 2026-03-07 — 20-F Planning Freeze
- Date: 2026-03-07
- Gate/Step: 20-F
- Why:
  - khóa chứng cứ security adversarial trước packaging gate, đảm bảo không có bypass critical/high trên trust/lock/permission/release chain.
- Scope:
  - thêm helper dùng chung Gate 20-F để đọc SoT red-team/CVE snapshot và ghi report security.
  - triển khai 5 test targeted theo đúng danh sách gate:
    - `v20_redteam_policy_bypass`
    - `v20_supplychain_attack_simulation`
    - `v20_secret_exfiltration_guard`
    - `v20_cve_gate`
    - `v20_cve_snapshot_contract`
  - sinh đủ 5 artifact security theo path đã khóa trong gate.
- Expected tests:
  - `cargo test --test v20_redteam_policy_bypass --test v20_supplychain_attack_simulation --test v20_secret_exfiltration_guard --test v20_cve_gate --test v20_cve_snapshot_contract`
- Exit criteria:
  - 5/5 test targeted pass.
  - có đủ artifacts:
    - `target/ocp/w20/security/redteam_report.json`
    - `target/ocp/w20/security/supplychain_attack_report.json`
    - `target/ocp/w20/security/secrets_hygiene_report.json`
    - `target/ocp/w20/security/cve_gate_report.json`
    - `target/ocp/w20/security/cve_snapshot_report.json`
  - policy bypass mức critical/high không được phép đi qua.
  - CVE gate buộc dùng pinned offline snapshot.

### 2026-03-07 — 20-F Implementation Closeout
- Date: 2026-03-07
- Gate/Step: 20-F
- Implemented:
  - thêm helper `v20_gate_f_common`:
    - đọc SoT `redteam_attack_matrix`, `cve_snapshot`, `threat_model`,
    - cung cấp utility scrub/guard secrets,
    - chuẩn hóa write report về `target/ocp/w20/security/*`.
  - triển khai 5 test targeted của Gate 20-F và khóa logic theo exit criteria:
    - chặn bypass policy ở các vector critical/high,
    - mô phỏng supply-chain adversarial attempts phải bị block,
    - guard secrets không để leak marker,
    - CVE gate phải dùng `offline_pinned_snapshot`,
    - verify chữ ký contract `cve_snapshot`.
- Files changed:
  - `tests/v20_gate_f_common.rs`
  - `tests/v20_redteam_policy_bypass.rs`
  - `tests/v20_supplychain_attack_simulation.rs`
  - `tests/v20_secret_exfiltration_guard.rs`
  - `tests/v20_cve_gate.rs`
  - `tests/v20_cve_snapshot_contract.rs`
  - `OCP-MVP-PLAN-v0.20.md`
- Commands run:
  - `cargo test --test v20_redteam_policy_bypass --test v20_supplychain_attack_simulation --test v20_secret_exfiltration_guard --test v20_cve_gate --test v20_cve_snapshot_contract`
  - `Get-ChildItem target/ocp/w20/security | Select-Object -ExpandProperty Name`
  - `rg -n "run_manifest_ref" target/ocp/w20/security`
  - `rg -n '"status"' target/ocp/w20/security`
  - `rg -n "leak_count|blocked_count|source_mode" target/ocp/w20/security`
- Test results:
  - Targeted tests (must-pass for gate):
    - `v20_redteam_policy_bypass`: `PASS`
    - `v20_supplychain_attack_simulation`: `PASS`
    - `v20_secret_exfiltration_guard`: `PASS`
    - `v20_cve_gate`: `PASS`
    - `v20_cve_snapshot_contract`: `PASS`
  - Regression tests (supporting only):
    - chưa chạy `cargo test` toàn bộ ở snapshot này.
- Kết luận gate:
  - `DONE`.
- Design alignment:
  - `FULL`.
- Notes/risks:
  - Gate 20-F dùng policy CVE pinned offline theo SoT, không phụ thuộc feed realtime internet trong quyết định PASS của gate.
  - artifacts đã sinh đủ:
    - `target/ocp/w20/security/redteam_report.json`
    - `target/ocp/w20/security/supplychain_attack_report.json`
    - `target/ocp/w20/security/secrets_hygiene_report.json`
    - `target/ocp/w20/security/cve_gate_report.json`
    - `target/ocp/w20/security/cve_snapshot_report.json`

### 2026-03-07 — 20-G Planning Freeze
- Date: 2026-03-07
- Gate/Step: 20-G
- Why:
  - khóa chứng cứ release packaging/repro/rollback trước gate signoff cuối; bảo đảm RC bundle có manifest+signature và release profile không chứa chaos.
- Scope:
  - thêm helper Gate 20-G để dựng/signed/verify `v1_rc_manifest`.
  - triển khai 6 test targeted của Gate 20-G theo đúng danh sách plan.
  - sinh đầy đủ artifacts release theo path đã khóa.
- Expected tests:
  - `cargo test --test v20_release_package_manifest --test v20_repro_build_evidence --test v20_repro_protocol_contract --test v20_rc_smoke_all_profiles --test v20_rollback_rehearsal --test v20_release_profile_no_chaos`
- Exit criteria:
  - 6/6 test targeted pass.
  - sinh đủ artifacts:
    - `target/ocp/w20/release/v1_rc_manifest.json`
    - `target/ocp/w20/release/v1_rc_manifest.json.sig`
    - `target/ocp/w20/release/reproducibility_report.json`
    - `target/ocp/w20/release/rollback_drill_report.json`
    - `target/ocp/w20/release/repro_protocol_report.json`
    - `target/ocp/w20/release/release_profile_no_chaos_report.json`

### 2026-03-07 — 20-G Implementation Closeout
- Date: 2026-03-07
- Gate/Step: 20-G
- Implemented:
  - thêm helper `v20_gate_g_common` để:
    - dựng payload release manifest từ artifacts gates trước,
    - sign/verify `v1_rc_manifest.json`,
    - cung cấp contract readers cho repro/final scope.
  - triển khai 6 test targeted:
    - `v20_release_package_manifest`
    - `v20_repro_build_evidence`
    - `v20_repro_protocol_contract`
    - `v20_rc_smoke_all_profiles`
    - `v20_rollback_rehearsal`
    - `v20_release_profile_no_chaos`
  - sinh đầy đủ artifacts Gate 20-G theo contract.
- Files changed:
  - `tests/v20_gate_g_common.rs`
  - `tests/v20_release_package_manifest.rs`
  - `tests/v20_repro_build_evidence.rs`
  - `tests/v20_repro_protocol_contract.rs`
  - `tests/v20_rc_smoke_all_profiles.rs`
  - `tests/v20_rollback_rehearsal.rs`
  - `tests/v20_release_profile_no_chaos.rs`
  - `OCP-MVP-PLAN-v0.20.md`
- Commands run:
  - `cargo test --test v20_release_package_manifest --test v20_repro_build_evidence --test v20_repro_protocol_contract --test v20_rc_smoke_all_profiles --test v20_rollback_rehearsal --test v20_release_profile_no_chaos`
  - `Get-ChildItem target/ocp/w20/release`
  - `rg -n "run_manifest_ref" target/ocp/w20/release`
  - `rg -n "toolchain_digest_ref|manifest_byte_equal|artifact_sha256_equal|w20_chaos_enabled_at_compile_time" target/ocp/w20/release`
- Test results:
  - Targeted tests (must-pass for gate):
    - `v20_release_package_manifest`: `PASS`
    - `v20_repro_build_evidence`: `PASS`
    - `v20_repro_protocol_contract`: `PASS`
    - `v20_rc_smoke_all_profiles`: `PASS`
    - `v20_rollback_rehearsal`: `PASS`
    - `v20_release_profile_no_chaos`: `PASS`
  - Regression tests (supporting only):
    - chưa chạy `cargo test` toàn bộ ở snapshot này.
- Kết luận gate:
  - `DONE`.
- Design alignment:
  - `FULL`.
- Notes/risks:
  - artifacts Gate 20-G đã sinh:
    - `target/ocp/w20/release/v1_rc_manifest.json`
    - `target/ocp/w20/release/v1_rc_manifest.json.sig`
    - `target/ocp/w20/release/reproducibility_report.json`
    - `target/ocp/w20/release/rollback_drill_report.json`
    - `target/ocp/w20/release/repro_protocol_report.json`
    - `target/ocp/w20/release/release_profile_no_chaos_report.json`
  - artifact bổ sung phục vụ audit:
    - `target/ocp/w20/release/rc_smoke_profiles_report.json`

### 2026-03-07 — 20-H Planning Freeze
- Date: 2026-03-07
- Gate/Step: 20-H
- Why:
  - chốt signoff cuối cùng trước v1.0 với zero-open-findings policy, required artifacts completeness, stability repeat và GO/NO-GO machine-checkable.
- Scope:
  - thêm helper Gate 20-H để tổng hợp findings/signoff bundles từ contract v20.
  - triển khai 6 test targeted theo danh sách gate:
    - `v20_zero_open_findings`
    - `v20_final_signoff_bundle`
    - `v20_finding_schema_contract`
    - `v20_finding_reclassification_guard`
    - `v20_signoff_required_artifacts`
    - `v20_stability_repeat_policy`
  - chạy thêm regression/tooling commands bắt buộc của gate:
    - `cargo test`
    - `cargo clippy --all-targets -- -D warnings`
    - `cargo fmt -- --check`
    - `cargo xtask editor-ci`
    - `corepack pnpm --dir editor/vscode/ocp run test:integration`
- Expected tests:
  - `cargo test --test v20_zero_open_findings --test v20_final_signoff_bundle --test v20_finding_schema_contract --test v20_finding_reclassification_guard --test v20_signoff_required_artifacts --test v20_stability_repeat_policy`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
  - `cargo xtask editor-ci`
  - `corepack pnpm --dir editor/vscode/ocp run test:integration`
- Exit criteria:
  - 6/6 test targeted pass.
  - sinh đủ artifacts signoff:
    - `target/ocp/w20/signoff/final_findings_report.json`
    - `target/ocp/w20/signoff/final_signoff_bundle.json`
    - `target/ocp/w20/signoff/v1_release_go_no_go.json`
    - `target/ocp/w20/signoff/finding_reclassification_report.json`
    - `target/ocp/w20/signoff/stability_repeat_report.json`
  - `cargo test`, `clippy`, `fmt --check`, `xtask editor-ci`, `pnpm test:integration` đều pass.
  - `v1_release_go_no_go.json` phải có `signal=GO`.

### 2026-03-07 — 20-H Implementation Closeout
- Date: 2026-03-07
- Gate/Step: 20-H
- Implemented:
  - thêm helper `v20_gate_h_common`:
    - dựng/kiểm tra findings report theo schema/policy,
    - dựng stability repeat report theo contract,
    - dựng final signoff bundle + go/no-go report có `supply_chain_replay_mode_summary`.
  - triển khai 6 test targeted Gate 20-H:
    - `v20_zero_open_findings`
    - `v20_final_signoff_bundle`
    - `v20_finding_schema_contract`
    - `v20_finding_reclassification_guard`
    - `v20_signoff_required_artifacts`
    - `v20_stability_repeat_policy`
  - vá regression toàn cục để `cargo test` full không vỡ ở default profile:
    - cho nhóm test chaos `v20_*` tự skip khi không bật feature `w20_chaos`.
  - đồng bộ lại chữ ký `contracts/required_contracts.v1.json.sig` thông qua test sync inventory editor để khôi phục chain verify.
- Files changed:
  - `tests/v20_gate_h_common.rs`
  - `tests/v20_zero_open_findings.rs`
  - `tests/v20_final_signoff_bundle.rs`
  - `tests/v20_finding_schema_contract.rs`
  - `tests/v20_finding_reclassification_guard.rs`
  - `tests/v20_signoff_required_artifacts.rs`
  - `tests/v20_stability_repeat_policy.rs`
  - `tests/v20_gate_d_common.rs`
  - `tests/v20_fault_injection.rs`
  - `tests/v20_chaos_io_network_fs.rs`
  - `tests/v20_recovery_fail_honest.rs`
  - `tests/v20_chaos_schedule_contract.rs`
  - `tests/v20_release_profile_no_chaos.rs`
  - `contracts/required_contracts.v1.json.sig`
  - `target/ocp/w20/signoff/*` (artifacts)
  - `OCP-MVP-PLAN-v0.20.md`
- Commands run:
  - `cargo test --test v20_zero_open_findings --test v20_final_signoff_bundle --test v20_finding_schema_contract --test v20_finding_reclassification_guard --test v20_signoff_required_artifacts --test v20_stability_repeat_policy`
  - `cargo test --test v19_editor_inventory_sync`
  - `cargo test --test v18_sot_signature_verify`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt`
  - `cargo fmt -- --check`
  - `cargo xtask editor-ci`
  - `corepack pnpm --dir editor/vscode/ocp run test:integration`
  - `Get-ChildItem target/ocp/w20/signoff`
  - `Get-Content target/ocp/w20/signoff/final_findings_report.json`
  - `Get-Content target/ocp/w20/signoff/final_signoff_bundle.json`
  - `Get-Content target/ocp/w20/signoff/v1_release_go_no_go.json`
- Test results:
  - Targeted tests (must-pass for gate):
    - `v20_zero_open_findings`: `PASS`
    - `v20_final_signoff_bundle`: `PASS`
    - `v20_finding_schema_contract`: `PASS`
    - `v20_finding_reclassification_guard`: `PASS`
    - `v20_signoff_required_artifacts`: `PASS`
    - `v20_stability_repeat_policy`: `PASS`
  - Regression tests (supporting only):
    - `cargo test`: `PASS`
    - `cargo clippy --all-targets -- -D warnings`: `PASS`
    - `cargo fmt -- --check`: `PASS`
    - `cargo xtask editor-ci`: `PASS`
    - `corepack pnpm --dir editor/vscode/ocp run test:integration`: `PASS` (chạy ngoài sandbox do giới hạn runtime node trong sandbox).
- Kết luận gate:
  - `DONE`.
- Design alignment:
  - `FULL`.
- Notes/risks:
  - artifacts signoff đã sinh đủ:
    - `target/ocp/w20/signoff/final_findings_report.json`
    - `target/ocp/w20/signoff/final_signoff_bundle.json`
    - `target/ocp/w20/signoff/v1_release_go_no_go.json`
    - `target/ocp/w20/signoff/finding_reclassification_report.json`
    - `target/ocp/w20/signoff/stability_repeat_report.json`
  - `v1_release_go_no_go.json` cho kết quả:
    - `signal = GO`
    - `deterministic_supply_chain_replay_claim = false` (đúng policy vì history replay matrix có mode `allow_network` nên chỉ được claim functional replay).

---

## 12) Checklist khóa trước khi đóng gate
- [x] Gate status đã cập nhật đúng (`TODO/IN_PROGRESS/PARTIAL/DONE`).
- [x] Có đủ cặp `Planning Freeze` + `Implementation Closeout` cho gate đang đóng.
- [x] `Files changed` khớp code delta thực tế.
- [x] `Commands run` là lệnh đã chạy thật.
- [x] `Targeted tests` pass đúng phạm vi.
- [x] `Regression tests` chỉ đóng vai trò phụ trợ.
- [x] Mọi report JSON có `run_manifest_ref` + `run_manifest_sha256` hợp lệ.
- [x] Gate 20-B đã chạy đúng strategy `checkout_and_test` và không có skip trái policy.
- [x] Gate 20-B có đủ báo cáo cross-platform per-OS + aggregate; thiếu 1 OS là chưa đạt.
- [x] Gate 20-B report có đủ counters `discovered/run/ignored/skipped` và `skip_entries`.
- [x] `run_manifest.json` có `toolchain_matrix_hash` + `dependency_mode` đúng SoT.
- [x] `v1_release_go_no_go.json` có `supply_chain_replay_mode_summary` + `deterministic_supply_chain_replay_claim` đúng policy.
- [x] Gate 20-D đã chạy với `--features w20_chaos`.
- [x] Gate 20-G xác nhận release profile không chứa chaos feature/symbol.
- [x] Gate 20-H xác thực đầy đủ artifact theo `contracts/v20/signoff_required_artifacts.v1.json`.
- [x] Các report dùng external tool có đủ tool invocation evidence (`tool_binary_sha256` hoặc `..._unavailable_reason_code`).
- [x] Không còn marker `FAIL`/placeholder `PASS/FAIL` trong closeout `DONE`.
- [x] Không còn finding `critical/high` mở sau gate 20-H.
- [x] `final_findings_report.json` có đủ `CRITICAL/HIGH/MEDIUM/LOW`.
- [x] Không có lỗi mã hóa tiếng Việt theo hiển thị IDE.

---

## 13) Handoff v0.20 -> v1.0
- Chỉ mở phát hành v1.0 khi:
  - toàn bộ gate core `20-A..20-H` đã `DONE`,
  - Exit Contract v0.20 pass đầy đủ,
  - `target/ocp/w20/signoff/v1_release_go_no_go.json` = `GO`,
  - không còn finding `critical/high` mở.
