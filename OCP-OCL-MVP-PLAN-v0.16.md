# OCL v0.16 — Productization Audit + Unified Readiness (Pre-v1.0 Hard Freeze)

Ngày tạo: 2026-03-06  
Trạng thái: `LOCKED (IMPLEMENTED + VERIFIED, READY FOR v1.0 HANDOFF)`  
Phạm vi: **OCL-only**.  
Tiền đề: v0.1..v0.15 đã hoàn tất theo gate tương ứng (core semantics, packs, quarantine/cassette, schema/typing, lock/trust/permissions, trace/debug, shadow reuse, conformance, LTS rehearsal).

Mục tiêu v0.16: **không thêm tính năng/ngữ nghĩa mới**; thay vào đó triển khai một vòng kiểm tra sản phẩm hóa tổng thể để xác nhận OCP-OCL đã là một hệ thống thống nhất, dùng được thực tế, đủ điều kiện bước sang v1.0.

---

## 0) Governance + Tracking v0.16

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
- Không cho phép “DONE giả” (pass test nền nhưng không có code delta đúng gate).

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
  - Productization audit toàn diện v0.1..v0.15 để khóa “một thể thống nhất” trước v1.0.
- Trạng thái tổng quan:
  - `LOCKED (IMPLEMENTED + VERIFIED, READY FOR v1.0 HANDOFF)`.
- Gate đang làm/đã xong/chưa làm:
  - Gate `16-A`..`16-G` đã hoàn tất.
- Bước kế tiếp ngay:
  - rà checklist khóa đóng phiên bản và chuẩn bị handoff `v0.16 -> v1.0`.
- Lệnh kiểm chứng chuẩn:
  - xem `12) Operational commands (v0.16)`.
- File code trọng yếu đã thay đổi trong v0.16:
  - `tests/v16_gate_a_common.rs`
  - `tests/stability_contracts_v15.rs`
  - `tests/historical_evidence.rs`
  - `tests/historical_evidence_signature.rs`
  - `tests/run_manifest.rs`
  - `tests/run_manifest_allowlist.rs`
  - `tests/contract_inventory_complete.rs`
  - `tests/conformance_runner.rs`
  - `tests/ocl_determinism.rs`
  - `tests/cache_signature_invariance.rs`
  - `tests/cassette_replay_offline_invariance.rs`
  - `tests/lock_migration.rs`
  - `tests/trust_lane_policy.rs`
  - `tests/upgrade_check_negative.rs`
  - `tests/perf_baseline_signature.rs`
  - `tests/perf_baseline_v16.rs`
  - `tests/lts_check.rs`
  - `tests/repo_hygiene.rs`
  - `tests/license_inventory.rs`
  - `tests/lock_sign_negative.rs`
  - `tests/attestation_tamper_negative.rs`
  - `tests/perm_bypass_negative.rs`
  - `tests/trust_policy_negative.rs`
  - `tests/license_policy_negative.rs`
  - `tests/cassette_tamper_negative.rs`
  - `tests/v1_user_journey_smoke.rs`
  - `tests/release_artifact_manifest.rs`
  - `baselines/v015/perf_budget_report.json`
  - `baselines/v015/run_manifest.json`
  - `baselines/v015/baseline_manifest.json`
  - `baselines/v015/baseline_manifest.sig`
  - `projects/ocp-ocl/crates/ocl-sdk/src/w9.rs`
  - `contracts/history/evidence_index.v1.json`
  - `contracts/history/evidence_index.v1.sig`
  - `contracts/required_contracts.v1.json`
  - `projects/ocp-ocl/conformance/expected/contracts/stability_contract_v15.json`
  - `OCP-OCL-MVP-PLAN-v0.16.md`

### 0.3 Trạng thái Workstreams/Gates v0.16 (tracking)
#### Workstreams
- WS-CF (contract freeze + snapshot): `DONE`
- WS-CM (conformance matrix tổng): `DONE`
- WS-DR (determinism/replay soak): `DONE`
- WS-MG (compat/migration rehearsal): `DONE`
- WS-PF (performance/capacity verification): `DONE`
- WS-SC (security/supply-chain verification): `DONE`
- WS-RC (release-candidate dry run + signoff): `DONE`

#### Gate status (16-A .. 16-G)
- Gate 16-A (Contract Freeze + unified snapshot): `DONE`
- Gate 16-B (Unified conformance matrix): `DONE`
- Gate 16-C (Determinism + replay soak): `DONE`
- Gate 16-D (Compatibility + migration rehearsal): `DONE`
- Gate 16-E (Performance + capacity verification): `DONE`
- Gate 16-F (Security + supply-chain verification): `DONE`
- Gate 16-G (RC dry-run + final product signoff): `DONE`

### 0.4 Run manifest bắt buộc (LOCKED)
- Mọi gate v0.16 phải tạo:
  - `target/ocl/w16/meta/run_manifest.json`
- `run_manifest.json` tối thiểu phải có:
  - `git_commit`
  - `rustc_version_verbose`
  - `cargo_version`
  - `target_triple`
  - `os`
  - `arch`
  - `lane_profile`
  - `allowed_env_flags`
  - `deps_lock_v3_hash`
  - `cargo_lock_hash` (nếu có)
- Mọi report JSON của gate phải:
  - có `run_manifest_ref`, hoặc
  - embed đầy đủ object `run_manifest`.
- Ngoại lệ rõ ràng:
  - Các file snapshot/raw dữ liệu không mang tính gate-report (`*_snapshot.json`, `contract_inventory.json`, `sbom.spdx.json`) không bắt buộc nhúng `run_manifest_ref`.
  - Các file snapshot/raw bắt buộc phải được bao phủ bởi một file report cùng gate đã có `run_manifest_ref`.
- Run-manifest policy:
  - `allowed_env_flags` áp dụng `deny-by-default`:
    - env flag ngoài allow-list => fail-honest.
  - `lane_profile` là giá trị suy ra từ runtime config thực tế (derived), không cho phép tự khai báo thủ công trong report.

---

## 1) Goals v0.16 (LOCKED)

### 1.1 North Star
- v0.16 là bước **sản phẩm hóa tổng kiểm**, không phải phát triển tính năng mới.
- Kết quả cần chứng minh:
  - toàn hệ OCP-OCL vận hành nhất quán xuyên suốt các phiên bản,
  - không phân mảnh giữa module/runtime/CLI/SDK,
  - có thể dùng ngay trong quy trình thực tế theo mô hình governed GPL (GGPL) với bằng chứng máy kiểm.

### 1.2 KPI bắt buộc (machine-checkable)
**KPI-1: No-new-semantics window**
- Rule:
  - không thêm key ngôn ngữ mới, không đổi semantics đã khóa.
  - baseline freeze phải bao phủ đầy đủ surface đã ship tới `v0.15` (không chỉ lát cắt `v0.14`).
- Command:
  - `cargo test --test stability_contracts_v15`
- Artifacts:
  - `target/ocl/w16/contracts/contract_freeze_report.json`
  - `target/ocl/w16/history/historical_evidence_report.json`
  - `target/ocl/w16/contracts/contract_inventory_completeness_report.json`
- Pass:
  - không có breaking diff ngoài phạm vi additive đã khai báo.
- Fail:
  - phát hiện semantic drift chưa có migration contract.

**KPI-2: Unified conformance green**
- Command:
  - `cargo test --test conformance_core`
  - `cargo test --test conformance_packs`
  - `cargo test --test conformance_supplychain`
  - `cargo test --test conformance_runner`
  - `cargo test --test conformance_cache`
- Artifacts:
  - `target/ocl/w16/conformance/unified_conformance_report.json`
  - `target/ocl/w16/conformance/unified_conformance_report.txt`
- Pass:
  - toàn bộ suite bắt buộc pass.
- Fail:
  - có bất kỳ suite fail hoặc thiếu artifact.

**KPI-3: Determinism/replay invariant**
- Command:
  - `cargo test --test ocl_determinism`
  - `cargo test --test quarantine_gate`
  - `cargo test --test ui_replay`
  - `cargo test --test cache_signature_invariance`
  - `cargo test --test cassette_replay_offline_invariance`
- Artifacts:
  - `target/ocl/w16/determinism/determinism_soak_report.json`
  - `target/ocl/w16/determinism/cache_signature_invariance_report.json`
  - `target/ocl/w16/determinism/cassette_offline_invariance_report.json`
- Pass:
  - cùng seed/config => signature ổn định theo protocol soak.
  - cache on/off trong lane locked cho cùng input phải cho cùng signature.
  - quarantine record -> replay offline cho cùng cassette phải cho cùng signature.
- Fail:
  - mismatch signature hoặc replay không fail-honest khi thiếu dữ liệu.

**KPI-4: Compatibility contract giữ đúng**
- Command:
  - `cargo test --test lock_migration`
  - `cargo test --test trust_lane_policy`
  - `cargo test --test upgrade_check`
- Artifacts:
  - `target/ocl/w16/compat/compat_rehearsal_report.json`
- Pass:
  - lane `locked_v06|locked_v071|quarantine` chạy đúng policy đã hứa.
- Fail:
  - có silent break hoặc alias/hint không đúng contract.

**KPI-5: Performance/capacity không hồi quy vượt ngưỡng**
- Command:
  - `cargo test --test compile_cache`
  - `cargo test --test observe_cache`
  - `cargo test --test cli_cache`
- Artifacts:
  - `target/ocl/w16/perf/perf_budget_report.json`
- Pass:
  - chỉ số deterministic counters đạt ngưỡng so với baseline v0.15.
- Fail:
  - regression vượt ngưỡng quy định.

**KPI-6: Security/supply-chain chain-of-trust hoàn chỉnh**
- Command:
  - `cargo test --test trust_policy`
  - `cargo test --test lock_sign`
  - `cargo test --test perm_review`
  - `cargo test --test attestation`
  - `cargo test --test lts_check`
  - `cargo test --test cassette_sign`
  - `cargo test --test repo_hygiene`
  - `cargo test --test lock_sign_negative`
  - `cargo test --test attestation_tamper_negative`
  - `cargo test --test perm_bypass_negative`
  - `cargo test --test trust_policy_negative`
  - `cargo test --test license_policy_negative`
  - `cargo test --test cassette_tamper_negative`
- Artifacts:
  - `target/ocl/w16/security/security_chain_report.json`
  - `target/ocl/w16/security/repo_hygiene_report.json`
  - `target/ocl/w16/security/license_inventory.json`
- Pass:
  - trust/lock/permission/attestation/LTS đều đạt.
- Fail:
  - thiếu một mắt xích hoặc bypass policy.

---

## 2) Axis Lock v0.16 (kế thừa, không đổi)
- No-new-semantics window.
- Determinism + replay vẫn là trục chính.
- Lane literals giữ nguyên:
  - `locked_v071`
  - `locked_v06`
  - `quarantine`
- Lockfile SoT giữ nguyên:
  - `deps.lock.v3` (không đổi SoT).
- Signature/canonicalization/hash policy giữ theo chuẩn đã khóa trước đó.
- Không nới lỏng security policy chỉ để “pass nhanh”.

---

## 3) Scope v0.16

### 3.1 In-scope (ship)
A) Contract freeze snapshot toàn hệ.  
B) Unified conformance matrix (core/packs/supply-chain/cache/quarantine replay).  
C) Determinism + replay soak theo protocol thống nhất.  
D) Compatibility + migration rehearsal từ các lane/lock/CLI contracts.  
E) Performance + capacity verification bằng deterministic counters.  
F) Security + supply-chain verification chain-of-trust.  
G) RC dry-run + final signoff trước v1.0.

### 3.2 Out-of-scope (defer)
- Không thêm pack mới, không thêm cú pháp mới, không mở semantic mới.
- Không mở lại tranh luận thiết kế đã khóa ở v0.1..v0.15 trừ khi phát hiện lỗi thực tế có bằng chứng.
- Không publish production trong v0.16 (chỉ rehearsal/dry-run).

### 3.3 Core vs stretch (LOCKED)
- Core bắt buộc cho `DONE` v0.16:
  - 16-A, 16-B, 16-C, 16-D, 16-E, 16-F, 16-G.
- Stretch (không chặn core done):
  - mở rộng matrix multi-OS sâu hơn nếu hạ tầng sẵn có.

### 3.4 Migration contract from v0.1..v0.15 (LOCKED)
- v0.16 không đổi public semantics; chỉ audit, verify, và chốt bằng chứng.
- Mọi phát hiện lệch phải:
  - ghi finding machine-readable,
  - sửa đúng điểm lệch,
  - rerun gate liên quan trước khi đóng.

### 3.5 Module -> Crate -> Path mapping (LOCKED)
- Contract freeze/snapshot/model diff:
  - crate: `ocl-sdk`
  - path: `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
- Runtime deterministic counters/replay invariants/perf hooks:
  - crate: `ocl-runtime-core`
  - path: `projects/ocp-ocl/crates/ocl-runtime-core/src/*`
- Orchestration/report CLI cho product-check:
  - crate: `ocl-cli`
  - path: `projects/ocp-ocl/crates/ocl-cli/src/main.rs`
- Rule:
  - chưa khóa rõ ownership thì chưa mở gate implementation tương ứng.

---

## 4) Unified Contract Freeze (v0.16)

### 4.1 Contract surfaces (LOCKED)
- Language/runtime semantics contracts.
- Trace/audit schema contracts.
- Artifact layout contracts.
- Manifest/permissions schema contracts.
- Error taxonomy + alias contracts.
- Lane/lock/trust/approval contracts.

### 4.2 Snapshot outputs
- `target/ocl/w16/contracts/language_contract_snapshot.json`
- `target/ocl/w16/contracts/runtime_contract_snapshot.json`
- `target/ocl/w16/contracts/trace_contract_snapshot.json`
- `target/ocl/w16/contracts/supplychain_contract_snapshot.json`
- `target/ocl/w16/contracts/contract_inventory.json`
- `target/ocl/w16/contracts/contract_freeze_report.json`

### 4.3 Contract Inventory (LOCKED)
- Mỗi contract phải có:
  - `contract_id`
  - `version`
  - `schema_hash`
  - `producer` (crate/path)
  - `consumer` (runtime/cli/sdk)
  - `allowed_diffs_v016`
- Inventory bắt buộc bao phủ tối thiểu:
  - trace schema contract (v0.11 line),
  - package/lock/trust contracts (v0.10-v0.15 line),
  - cassette/replay contracts (v0.8 line),
  - attestation/permission contracts (v0.15 line).

### 4.4 Freeze policy
- Không cho phép silent change.
- Mọi thay đổi additive phải có migration note + test evidence.
- Breaking change trong v0.16 bị coi là blocker.

### 4.5 Historical evidence verification (LOCKED)
- v0.16 phải có chain-of-evidence machine-checkable cho v0.1..v0.15:
  - `target/ocl/w16/history/historical_evidence_report.json`
- Historical SoT (nguồn chân lý) bắt buộc:
  - `contracts/history/evidence_index.v1.json`
  - `contracts/history/evidence_index.v1.sig`
- Mỗi entry phiên bản tối thiểu gồm:
  - `version_id`
  - `commit_or_tag`
  - `evidence_files`
  - `sha256`
  - `schema_version`
  - `verification` (`PASS`/`FAIL`)
- Rule:
  - thiếu bất kỳ entry nào trong dải `v0.1..v0.15` => gate fail.
  - checksum/schema mismatch => gate fail.
  - report không khớp SoT index => gate fail.
  - chữ ký SoT không verify được => gate fail.

### 4.6 Contract inventory completeness enforcement (LOCKED)
- Required-contracts SoT:
  - `contracts/required_contracts.v1.json`
- Output bắt buộc:
  - `target/ocl/w16/contracts/contract_inventory_completeness_report.json`
- Rule:
  - thiếu `contract_id` bắt buộc => gate fail.
  - duplicate `contract_id` => gate fail.
  - `version/schema_hash` mismatch với required set => gate fail.

---

## 5) Unified Conformance Matrix (v0.16)

### 5.1 Matrix dimensions
- Lane:
  - `locked_v071`, `locked_v06`, `quarantine`.
- Engine:
  - `interpreter`, `bytecode`, `dual` (khi áp dụng).
- Domain:
  - core semantics
  - packs
  - supply-chain
  - cache/perf non-semantic telemetry
  - quarantine replay with golden cassette

### 5.2 Source-of-truth
- Giữ canonical flow conformance từ v0.14.
- Không tạo runner song song gây phân mảnh.

### 5.3 Outputs
- `target/ocl/w16/conformance/unified_conformance_report.json`
- `target/ocl/w16/conformance/unified_conformance_report.txt`

---

## 6) Determinism + Replay Soak (v0.16)

### 6.1 Soak protocol (LOCKED)
- Cùng seed/config, chạy lặp N lần theo profile đã khóa.
- So signature, trace digest, replay digest.
- Quarantine chỉ dùng replay với golden cassette (không record trong CI soak).
- Invariant bắt buộc:
  - cache on/off signature equality (locked lane),
  - cassette record->replay offline signature equality (quarantine lane).

### 6.2 Failure handling
- Mismatch bất kỳ => gate fail ngay.
- Không chấp nhận “flaky pass”.

### 6.3 Outputs
- `target/ocl/w16/determinism/determinism_soak_report.json`
- `target/ocl/w16/determinism/replay_invariant_report.json`
- `target/ocl/w16/determinism/cache_signature_invariance_report.json`
- `target/ocl/w16/determinism/cassette_offline_invariance_report.json`

---

## 7) Compatibility + Migration Rehearsal (v0.16)

### 7.1 Compatibility surfaces
- Lane behavior (`locked_v06`, `locked_v071`, `quarantine`).
- Lock migration behavior (`deps.lock.v2` -> `deps.lock.v3`).
- Alias + hint policy (không silent break).

### 7.2 Rules
- `locked_v071` strict contracts không bị nới lỏng.
- Compat path chỉ cho mục tiêu tương thích, không được kéo lùi hợp đồng mới.

### 7.3 Outputs
- `target/ocl/w16/compat/compat_rehearsal_report.json`
- `target/ocl/w16/compat/migration_diff_report.json`

---

## 8) Performance + Capacity Verification (v0.16)

### 8.1 Measurement policy (LOCKED)
- PASS/FAIL dựa trên deterministic counters từ audit/trace/report.
- Wallclock time chỉ để tham khảo, không làm tiêu chí pass chính.

### 8.2 Required metrics
- compile cache:
  - hit/miss ratio
  - modules compiled count
- runtime observe/exec cache:
  - hits/misses
  - steps_charged vs steps_executed
- shadow reuse:
  - reduction theo protocol đã khóa.

### 8.3 Outputs
- `target/ocl/w16/perf/perf_budget_report.json`
- `target/ocl/w16/perf/perf_trend_vs_v015.json`
- `target/ocl/w16/perf/perf_baseline_verify_report.json`

### 8.4 Perf baseline contract (LOCKED)
- `baseline_ref`: `baselines/v015/perf_budget_report.json`
- Baseline SoT bundle (đã chốt, không tự tạo tại chỗ):
  - `baselines/v015/perf_budget_report.json`
  - `baselines/v015/run_manifest.json`
  - `baselines/v015/baseline_manifest.json`
  - `baselines/v015/baseline_manifest.sig`
- Baseline policy:
  - tách riêng `cold_cache` và `warm_cache`, không trộn.
  - PASS/FAIL dựa trên deterministic counters, không dựa wallclock.
  - baseline phải verify qua manifest + signature trước khi so ngưỡng.
  - baseline lấy sai nguồn SoT hoặc không verify được => gate fail.
- Regression thresholds (v0.16 default):
  - `steps_executed_total` không tăng quá `+10%` so với baseline cùng chế độ cache.
  - `modules_compiled_count` không tăng quá `+10%`.
  - `compile_cache_hit_ratio` không giảm quá `10` điểm phần trăm tuyệt đối.
    - ví dụ baseline `70%` thì ngưỡng thấp nhất hợp lệ là `60%`.

---

## 9) Security + Supply-chain Verification (v0.16)

### 9.1 Required chain
- Trust policy strict enforcement.
- Signed lockfile verification.
- Permission snapshot/diff/approval enforcement.
- Build attestation + reproducibility verification.
- LTS strict profile check.

### 9.2 Policy boundary
- Trust/lock/policy deny ở `locked_v071` phải fail-honest.
- Không downgrade strict->warn trong `locked_v071`.

### 9.3 Outputs
- `target/ocl/w16/security/security_chain_report.json`
- `target/ocl/w16/security/lts_strict_report.json`
- `target/ocl/w16/security/license_inventory.json`
- `target/ocl/w16/security/sbom.spdx.json` (optional, nếu pipeline hỗ trợ)

### 9.4 Repo hygiene preflight (LOCKED)
- Bắt buộc thêm preflight trước signoff:
  - repo clean sau lane chạy chuẩn (`git status --porcelain` rỗng).
  - không track build/generated artifacts trái policy.
  - không lộ secret/private key trong release path.
- Output:
  - `target/ocl/w16/security/repo_hygiene_report.json`

### 9.5 Negative proofs (LOCKED)
- Mục tiêu:
  - chứng minh guardrail có hiệu lực fail-honest, không chỉ happy-path.
- Bộ negative proofs tối thiểu:
  - tamper lockfile signature => fail (`lock_sign_negative`)
  - tamper attestation => fail (`attestation_tamper_negative`)
  - bypass permission approval => fail (`perm_bypass_negative`)
  - trust artifact trái policy lane => fail (`trust_policy_negative`)
  - dependency license bị deny => fail (`license_policy_negative`)
  - tamper cassette signature/hash => fail (`cassette_tamper_negative`)
  - upgrade-check alias/hint sai contract => fail (`upgrade_check_negative`)

---

## 10) Release Candidate Rehearsal (v0.16)

### 10.1 RC dry-run scope
- version bump rehearsal (dry-run),
- build artifacts rehearsal,
- verify hash/signature nội bộ,
- publish staging rehearsal (nếu có pipeline staging).

### 10.2 Golden user journey (LOCKED)
- RC phải chạy được chuỗi thao tác người dùng mới trên clean checkout:
  1. `ocl init <template>`
  2. `ocl lock sync` + `ocl lock verify`
  3. `ocl perm snapshot` + `ocl perm diff` + `ocl perm approve` (khi có diff)
  4. `ocl build --attest`
  5. `ocl verify --attest`
  6. `ocl run` + `ocl replay`
  7. `ocl dbg` smoke script (không IO mới)
- Tất cả bước phải có artifact evidence, không chỉ log text.

### 10.3 Hard boundary
- Không publish production ở v0.16.
- Chỉ xác nhận quy trình “có thể lặp lại ổn định” cho v1.0.

### 10.4 Outputs
- `target/ocl/w16/rc/rc_dryrun_report.json`
- `target/ocl/w16/rc/release_readiness_checklist.json`
- `target/ocl/w16/rc/golden_user_journey_report.json`
- `target/ocl/w16/rc/release_artifact_manifest.json`

---

## 11) Execution gates v0.16 (triển khai tuần tự)

### Gate 16-A — Contract Freeze + unified snapshot
Scope:
- tạo snapshot contract toàn hệ v0.15-line và report diff.
- phát hành `contract_inventory.json` với `contract_id/version/schema_hash`.
- verify historical chain-of-evidence cho dải `v0.1..v0.15`.
- khóa contract `run_manifest` và liên kết vào toàn bộ report JSON.
- verify chữ ký của historical SoT index.
- chạy completeness enforcement cho required contract IDs.
Tests:
- `tests/stability_contracts_v15.rs`
- `tests/historical_evidence.rs`
- `tests/historical_evidence_signature.rs`
- `tests/run_manifest.rs`
- `tests/run_manifest_allowlist.rs`
- `tests/contract_inventory_complete.rs`
Exit criteria:
- không có breaking diff chưa khai báo.
- historical evidence report đầy đủ và hợp lệ cho toàn bộ v0.1..v0.15.
- historical SoT signature verify pass.
- contract inventory completeness report pass.

### Gate 16-B — Unified conformance matrix
Scope:
- chạy full matrix core/packs/supply-chain/cache/quarantine replay.
Tests:
- `tests/conformance_core.rs`
- `tests/conformance_packs.rs`
- `tests/conformance_supplychain.rs`
- `tests/conformance_runner.rs`
- `tests/conformance_cache.rs`
Exit criteria:
- toàn bộ suite bắt buộc xanh.

### Gate 16-C — Determinism + replay soak
Scope:
- chạy soak N-run + replay invariants.
Tests:
- `tests/ocl_determinism.rs`
- `tests/quarantine_gate.rs`
- `tests/ui_replay.rs`
- `tests/cache_signature_invariance.rs`
- `tests/cassette_replay_offline_invariance.rs`
Exit criteria:
- signature/replay invariant không lệch theo protocol.

### Gate 16-D — Compatibility + migration rehearsal
Scope:
- xác nhận lane compat + migration lock + upgrade-check.
Tests:
- `tests/lock_migration.rs`
- `tests/trust_lane_policy.rs`
- `tests/upgrade_check.rs`
- `tests/upgrade_check_negative.rs`
Exit criteria:
- không có silent break; alias/hint đúng contract.

### Gate 16-E — Performance + capacity verification
Scope:
- tổng hợp và kiểm tra ngưỡng counters deterministic theo baseline contract.
Tests:
- `tests/compile_cache.rs`
- `tests/observe_cache.rs`
- `tests/cli_cache.rs`
- `tests/perf_baseline_v16.rs`
- `tests/perf_baseline_signature.rs`
Exit criteria:
- không hồi quy vượt ngưỡng đã khóa.
- baseline SoT manifest/signature verify pass.

### Gate 16-F — Security + supply-chain verification
Scope:
- verify full chain trust/lock/perm/attest/lts.
- chạy repo hygiene preflight trước signoff.
- tạo license inventory và kiểm tra policy tương thích license.
- chạy bộ negative proofs cho trust/lock/perm/attest/license/cassette.
Tests:
- `tests/trust_policy.rs`
- `tests/lock_sign.rs`
- `tests/perm_review.rs`
- `tests/attestation.rs`
- `tests/lts_check.rs`
- `tests/cassette_sign.rs`
- `tests/repo_hygiene.rs`
- `tests/license_inventory.rs`
- `tests/lock_sign_negative.rs`
- `tests/attestation_tamper_negative.rs`
- `tests/perm_bypass_negative.rs`
- `tests/trust_policy_negative.rs`
- `tests/license_policy_negative.rs`
- `tests/cassette_tamper_negative.rs`
Exit criteria:
- chain-of-trust đủ mắt xích, fail-honest khi vi phạm.
- license inventory report hợp lệ, không có dependency vi phạm policy đã khóa.
- toàn bộ negative proofs bắt buộc pass.

### Gate 16-G — RC dry-run + final product signoff
Scope:
- chạy release rehearsal + golden user journey + tổng signoff v0.16.
- tạo `release_artifact_manifest.json` (file list + sha256 + signature status).
Tests:
- `cargo test --test v1_user_journey_smoke`
- `cargo test --test release_artifact_manifest`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt -- --check`
Exit criteria:
- đạt Exit Contract v0.16 đầy đủ.
- release artifact manifest đầy đủ và verify pass trong RC dry-run.

### Exit Contract v0.16 (machine-checkable)
- No-new-semantics window:
  - PASS khi không có semantic/pack/runtime contract mới ngoài scope bugfix/hardening.
- Contract freeze:
  - PASS khi snapshot contract ổn định và parser/check schema pass.
  - PASS khi historical evidence chain `v0.1..v0.15` verify đầy đủ.
  - PASS khi historical SoT signature verify pass và contract inventory completeness pass.
- Conformance green:
  - PASS khi matrix suite bắt buộc đều pass.
- Determinism invariant:
  - PASS khi soak/replay invariant ổn định theo protocol.
- Compatibility rehearsal:
  - PASS khi lane/lock/alias contracts không vỡ.
  - PASS khi negative proof `upgrade_check_negative` pass.
- Security/supply-chain chain:
  - PASS khi trust/lock/perm/attest/lts strict đều pass.
  - PASS khi full negative-proof set pass (lock/attest/perm/trust/license/cassette).
  - PASS khi `repo_hygiene_report.json` đạt chuẩn (repo sạch, không artifacts cấm, không lộ secret/key).
- Release rehearsal dry-run:
  - PASS khi pipeline rehearsal đủ artifact/evidence.
  - PASS khi mọi file gate-report JSON có `run_manifest_ref`/`run_manifest` hợp lệ.
  - PASS khi baseline perf SoT manifest/signature verify pass.
- Final closure rule:
  - v0.16 chỉ `DONE` khi toàn bộ gate core `16-A..16-G` đạt `DONE` và không còn finding critical mở.

---

## 12) Operational commands (v0.16)
- `cargo test`
- `cargo test --test stability_contracts_v15`
- `cargo test --test historical_evidence`
- `cargo test --test historical_evidence_signature`
- `cargo test --test run_manifest`
- `cargo test --test run_manifest_allowlist`
- `cargo test --test contract_inventory_complete`
- `cargo test --test conformance_core`
- `cargo test --test conformance_packs`
- `cargo test --test conformance_supplychain`
- `cargo test --test conformance_runner`
- `cargo test --test conformance_cache`
- `cargo test --test ocl_determinism`
- `cargo test --test quarantine_gate`
- `cargo test --test ui_replay`
- `cargo test --test cache_signature_invariance`
- `cargo test --test cassette_replay_offline_invariance`
- `cargo test --test lock_migration`
- `cargo test --test trust_lane_policy`
- `cargo test --test upgrade_check`
- `cargo test --test upgrade_check_negative`
- `cargo test --test compile_cache`
- `cargo test --test observe_cache`
- `cargo test --test cli_cache`
- `cargo test --test perf_baseline_v16`
- `cargo test --test perf_baseline_signature`
- `cargo test --test trust_policy`
- `cargo test --test lock_sign`
- `cargo test --test perm_review`
- `cargo test --test attestation`
- `cargo test --test lts_check`
- `cargo test --test cassette_sign`
- `cargo test --test repo_hygiene`
- `cargo test --test license_inventory`
- `cargo test --test lock_sign_negative`
- `cargo test --test attestation_tamper_negative`
- `cargo test --test perm_bypass_negative`
- `cargo test --test trust_policy_negative`
- `cargo test --test license_policy_negative`
- `cargo test --test cassette_tamper_negative`
- `cargo test --test v1_user_journey_smoke`
- `cargo test --test release_artifact_manifest`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt -- --check`

---

## 13) Execution Log (full-log standard)

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

### 2026-03-06 — 16-A Planning Freeze
- Date: 2026-03-06
- Gate/Step: 16-A
- Why:
  - Chốt contract freeze v0.15-line bằng snapshot machine-checkable, khóa historical evidence chain, run-manifest và contract inventory completeness trước khi mở 16-B.
- Scope:
  - Thêm bộ test/bằng chứng cho:
    - `stability_contracts_v15`
    - `historical_evidence` + `historical_evidence_signature`
    - `run_manifest` + `run_manifest_allowlist`
    - `contract_inventory_complete`
  - Tạo SoT files:
    - `projects/ocp-ocl/conformance/expected/contracts/stability_contract_v15.json`
    - `contracts/history/evidence_index.v1.json`
    - `contracts/history/evidence_index.v1.sig`
    - `contracts/required_contracts.v1.json`
  - Sinh report artifacts Gate 16-A trong `target/ocl/w16/contracts|history|meta`.
- Expected tests:
  - `cargo test --test stability_contracts_v15 -- --nocapture`
  - `cargo test --test historical_evidence -- --nocapture`
  - `cargo test --test historical_evidence_signature -- --nocapture`
  - `cargo test --test run_manifest -- --nocapture`
  - `cargo test --test run_manifest_allowlist -- --nocapture`
  - `cargo test --test contract_inventory_complete -- --nocapture`
  - `cargo test --test stability_contracts_v15 --test historical_evidence --test historical_evidence_signature --test run_manifest --test run_manifest_allowlist --test contract_inventory_complete`
  - `cargo test`
- Exit criteria:
  - Targeted tests của 16-A pass đầy đủ.
  - SoT files được chốt và verify thành công.
  - Artifacts Gate 16-A được tạo đúng đường dẫn.

### 2026-03-06 — 16-A Implementation Closeout
- Date: 2026-03-06
- Gate/Step: 16-A
- Implemented:
  - Thêm helper chuẩn hóa Gate 16-A (`tests/v16_gate_a_common.rs`) gồm hash/canonical JSON, snapshot v15, inventory v16, run-manifest builder, allowlist enforcement và history signature.
  - Thêm `tests/stability_contracts_v15.rs` để khóa additive-only contract freeze v15-line và sinh:
    - `target/ocl/w16/contracts/language_contract_snapshot.json`
    - `target/ocl/w16/contracts/runtime_contract_snapshot.json`
    - `target/ocl/w16/contracts/trace_contract_snapshot.json`
    - `target/ocl/w16/contracts/supplychain_contract_snapshot.json`
    - `target/ocl/w16/contracts/contract_freeze_report.json`
  - Thêm `tests/historical_evidence.rs` và `tests/historical_evidence_signature.rs` để verify chain `v0.1..v0.15` + chữ ký SoT, sinh:
    - `target/ocl/w16/history/historical_evidence_report.json`
  - Thêm `tests/run_manifest.rs` và `tests/run_manifest_allowlist.rs` để kiểm tra trường bắt buộc + deny-by-default allowlist, sinh:
    - `target/ocl/w16/meta/run_manifest.json`
  - Thêm `tests/contract_inventory_complete.rs` để verify required-contracts SoT, sinh:
    - `target/ocl/w16/contracts/contract_inventory.json`
    - `target/ocl/w16/contracts/contract_inventory_completeness_report.json`
  - Chốt SoT files:
    - `projects/ocp-ocl/conformance/expected/contracts/stability_contract_v15.json`
    - `contracts/history/evidence_index.v1.json`
    - `contracts/history/evidence_index.v1.sig`
    - `contracts/required_contracts.v1.json`
- Files changed:
  - `tests/v16_gate_a_common.rs`
  - `tests/stability_contracts_v15.rs`
  - `tests/historical_evidence.rs`
  - `tests/historical_evidence_signature.rs`
  - `tests/run_manifest.rs`
  - `tests/run_manifest_allowlist.rs`
  - `tests/contract_inventory_complete.rs`
  - `projects/ocp-ocl/conformance/expected/contracts/stability_contract_v15.json`
  - `contracts/history/evidence_index.v1.json`
  - `contracts/history/evidence_index.v1.sig`
  - `contracts/required_contracts.v1.json`
  - `OCP-OCL-MVP-PLAN-v0.16.md`
- Commands run:
  - `cargo test --test stability_contracts_v15 -- --nocapture`
  - `cargo test --test stability_contracts_v15 -- --nocapture`
  - `cargo test --test historical_evidence -- --nocapture`
  - `cargo test --test historical_evidence -- --nocapture`
  - `cargo test --test historical_evidence_signature -- --nocapture`
  - `cargo test --test run_manifest -- --nocapture`
  - `cargo test --test run_manifest_allowlist -- --nocapture`
  - `cargo test --test contract_inventory_complete -- --nocapture`
  - `cargo test --test contract_inventory_complete -- --nocapture`
  - `cargo test --test stability_contracts_v15 --test historical_evidence --test historical_evidence_signature --test run_manifest --test run_manifest_allowlist --test contract_inventory_complete`
  - `cargo test`
- Test results:
  - Targeted tests (must-pass for gate):
    - PASS: `stability_contracts_v15`
    - PASS: `historical_evidence`
    - PASS: `historical_evidence_signature`
    - PASS: `run_manifest`
    - PASS: `run_manifest_allowlist`
    - PASS: `contract_inventory_complete`
  - Regression tests (supporting only):
    - PASS: `cargo test` toàn repo
- Kết luận gate:
  - `DONE`
- Design alignment:
  - `FULL`
- Notes/risks:
  - Các test snapshot/SoT có cơ chế bootstrap khi file còn thiếu (khởi tạo rồi yêu cầu chạy lại). Đây là hành vi có chủ đích để tránh “tự pass ở lần tạo đầu”.
  - Còn warning `dead_code` trong test-helper do mỗi integration test biên dịch module riêng; không ảnh hưởng kết quả pass/fail Gate 16-A.

### 2026-03-06 — 16-B Planning Freeze
- Date: 2026-03-06
- Gate/Step: 16-B
- Why:
  - Đóng unified conformance matrix v0.16 theo đúng scope core/packs/supplychain/cache/quarantine replay và sinh report machine-checkable có liên kết run_manifest.
- Scope:
  - Thêm code delta thật tại `tests/conformance_runner.rs` để:
    - chạy full conformance.v5 2 lần kiểm tra digest ổn định,
    - tổng hợp suite-level reports (core/packs/quarantine/supplychain/cache),
    - xuất `target/ocl/w16/conformance/unified_conformance_report.json`,
    - xuất `target/ocl/w16/conformance/unified_conformance_report.txt`,
    - nhúng `run_manifest_ref` và `run_manifest`.
- Expected tests:
  - `cargo test --test conformance_core`
  - `cargo test --test conformance_packs`
  - `cargo test --test conformance_supplychain`
  - `cargo test --test conformance_runner`
  - `cargo test --test conformance_cache`
  - `cargo test`
- Exit criteria:
  - Targeted tests pass toàn bộ.
  - Unified report json/txt được sinh đúng đường dẫn v0.16.
  - Không có drift digest giữa hai lần full run.

### 2026-03-06 — 16-B Implementation Closeout
- Date: 2026-03-06
- Gate/Step: 16-B
- Implemented:
  - Thêm test `v16_unified_conformance_matrix_generates_report_artifacts` trong `tests/conformance_runner.rs`.
  - Test mới thực hiện:
    - parse full `conformance.v5`,
    - xác thực đủ domain bắt buộc (`core/packs/quarantine_replay/supplychain/cache`),
    - chạy full matrix hai lần và assert `required_digest` ổn định,
    - chạy từng suite con để ghi thống kê tổng hợp,
    - sinh `target/ocl/w16/conformance/unified_conformance_report.json`,
    - sinh `target/ocl/w16/conformance/unified_conformance_report.txt`,
    - gắn `run_manifest_ref` và `run_manifest` vào báo cáo JSON.
- Files changed:
  - `tests/conformance_runner.rs`
  - `OCP-OCL-MVP-PLAN-v0.16.md`
- Commands run:
  - `cargo test --test conformance_core --test conformance_packs --test conformance_supplychain --test conformance_runner --test conformance_cache`
  - `cargo test`
- Test results:
  - Targeted tests (must-pass for gate):
    - PASS: `conformance_core`
    - PASS: `conformance_packs`
    - PASS: `conformance_supplychain`
    - PASS: `conformance_runner`
    - PASS: `conformance_cache`
  - Regression tests (supporting only):
    - PASS: `cargo test` toàn repo
- Kết luận gate:
  - `DONE`
- Design alignment:
  - `FULL`
- Notes/risks:
  - Unified matrix hiện chạy trên engine mặc định `dual` theo profile conformance; mở rộng coverage engine-specific sâu hơn sẽ xử lý ở gate nâng cao nếu cần.
  - Artifact JSON đã có `run_manifest_ref` và embed `run_manifest`, tương thích quy tắc run-manifest v0.16.

### 2026-03-06 — 16-C Planning Freeze
- Date: 2026-03-06
- Gate/Step: 16-C
- Why:
  - Khóa determinism/replay invariants theo chuẩn machine-checkable trước khi mở compatibility rehearsal.
- Scope:
  - Bổ sung code delta thật cho `16-C`:
    - thêm soak report v0.16 từ `tests/ocl_determinism.rs`,
    - thêm targeted test mới `tests/cache_signature_invariance.rs`,
    - thêm targeted test mới `tests/cassette_replay_offline_invariance.rs`,
    - sinh artifacts determinism tại `target/ocl/w16/determinism/*`,
    - xử lý ổn định regression song song ở conformance runner (`projects/ocp-ocl/crates/ocl-sdk/src/w9.rs`) để full `cargo test` chạy nhất quán.
- Expected tests:
  - `cargo test --test ocl_determinism`
  - `cargo test --test quarantine_gate`
  - `cargo test --test ui_replay`
  - `cargo test --test cache_signature_invariance`
  - `cargo test --test cassette_replay_offline_invariance`
  - `cargo test`
- Exit criteria:
  - Targeted tests pass đầy đủ cho soak/cache/cassette invariance.
  - Sinh đủ 4 artifact bắt buộc:
    - `determinism_soak_report.json`
    - `replay_invariant_report.json`
    - `cache_signature_invariance_report.json`
    - `cassette_offline_invariance_report.json`
  - Full regression `cargo test` pass ổn định sau xử lý race.

### 2026-03-06 — 16-C Implementation Closeout
- Date: 2026-03-06
- Gate/Step: 16-C
- Implemented:
  - Mở rộng `tests/ocl_determinism.rs` với test soak `v16_determinism_soak_signature_and_trace_stable`, sinh `target/ocl/w16/determinism/determinism_soak_report.json`.
  - Thêm `tests/cache_signature_invariance.rs` để kiểm chứng cache on/off trong lane locked vẫn giữ signature và trace bất biến, sinh `target/ocl/w16/determinism/cache_signature_invariance_report.json`.
  - Thêm `tests/cassette_replay_offline_invariance.rs` để kiểm chứng quarantine record -> replay offline giữ signature bất biến, sinh:
    - `target/ocl/w16/determinism/cassette_offline_invariance_report.json`
    - `target/ocl/w16/determinism/replay_invariant_report.json`
  - Vá race song song trong conformance runtime workspace tại `projects/ocp-ocl/crates/ocl-sdk/src/w9.rs` bằng runtime root tách theo process id (`target/ocl/w9/runtime/<pid>`), loại bỏ đụng độ copy/delete khi full suite chạy đồng thời.
- Files changed:
  - `tests/ocl_determinism.rs`
  - `tests/cache_signature_invariance.rs`
  - `tests/cassette_replay_offline_invariance.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/src/w9.rs`
  - `OCP-OCL-MVP-PLAN-v0.16.md`
- Commands run:
  - `cargo test --test ocl_determinism --test quarantine_gate --test ui_replay --test cache_signature_invariance --test cassette_replay_offline_invariance`
  - `cargo test --test ocl_determinism --test quarantine_gate --test ui_replay --test cache_signature_invariance --test cassette_replay_offline_invariance`
  - `cargo test`
  - `cargo test`
  - `cargo test --test lts_check`
  - `cargo test`
- Test results:
  - Targeted tests (must-pass for gate):
    - PASS: `ocl_determinism`
    - PASS: `quarantine_gate`
    - PASS: `ui_replay`
    - PASS: `cache_signature_invariance`
    - PASS: `cassette_replay_offline_invariance`
  - Regression tests (supporting only):
    - PASS: `cargo test` toàn repo
- Kết luận gate:
  - `DONE`
- Design alignment:
  - `FULL`
- Notes/risks:
  - Trong lần regression đầu có xuất hiện lỗi IO do đụng độ workspace tạm của conformance khi chạy song song; đã xử lý bằng runtime root theo process và kiểm tra lại bằng full `cargo test`.
  - Artifacts determinism v0.16 đã được sinh đầy đủ tại `target/ocl/w16/determinism/`.

### 2026-03-06 — 16-D Planning Freeze
- Date: 2026-03-06
- Gate/Step: 16-D
- Why:
  - Khóa cứng compatibility + migration rehearsal theo contract v0.15-line trước khi mở Gate 16-E.
- Scope:
  - Bổ sung negative proof `upgrade_check_negative` cho contract runtime/engine hint.
  - Sinh artifact machine-checkable cho gate:
    - `target/ocl/w16/compat/compat_rehearsal_report.json`
    - `target/ocl/w16/compat/migration_diff_report.json`
  - Giữ nguyên semantics đã khóa; chỉ bổ sung evidence và guardrail test.
- Expected tests:
  - `cargo test --test lock_migration`
  - `cargo test --test trust_lane_policy`
  - `cargo test --test upgrade_check`
  - `cargo test --test upgrade_check_negative`
- Exit criteria:
  - Toàn bộ targeted suites pass.
  - Hai artifact report compat/migration được tạo đúng path và có `run_manifest_ref`.
  - Không có silent break với lane/lock/upgrade-check contracts.

### 2026-03-06 — 16-D Implementation Closeout
- Date: 2026-03-06
- Gate/Step: 16-D
- Implemented:
  - Thêm suite `tests/upgrade_check_negative.rs` để khóa negative proofs:
    - invalid runtime mode phải fail và trả hint deterministic|throughput.
    - invalid engine mode phải fail và trả hint interpreter|bytecode|dual.
  - Mở rộng `tests/lock_migration.rs` với test tạo `migration_diff_report.json`.
  - Mở rộng `tests/trust_lane_policy.rs` với test tạo `compat_rehearsal_report.json`.
- Files changed:
  - `tests/lock_migration.rs`
  - `tests/trust_lane_policy.rs`
  - `tests/upgrade_check_negative.rs`
  - `OCP-OCL-MVP-PLAN-v0.16.md`
- Commands run:
  - `cargo test --test lock_migration`
  - `cargo test --test trust_lane_policy`
  - `cargo test --test upgrade_check`
  - `cargo test --test upgrade_check_negative`
  - `cargo test`
- Test results:
  - Targeted tests (must-pass for gate):
    - PASS: `lock_migration` (3 tests)
    - PASS: `trust_lane_policy` (4 tests)
    - PASS: `upgrade_check` (2 tests)
    - PASS: `upgrade_check_negative` (2 tests)
  - Regression tests (supporting only):
    - PASS: `cargo test` toàn repo
- Kết luận gate:
  - `DONE`
- Design alignment:
  - `FULL`
- Notes/risks:
  - Report `migration_diff_report.json` có chứa đường dẫn tạm cục bộ để audit context; không dùng làm baseline freeze input.
  - Artifact gate đã sinh đầy đủ dưới `target/ocl/w16/compat/`.

### 2026-03-06 — 16-E Planning Freeze
- Date: 2026-03-06
- Gate/Step: 16-E
- Why:
  - Khóa cứng baseline perf SoT v0.15 và xác nhận ngưỡng regression bằng deterministic counters trước khi vào gate security.
- Scope:
  - Bổ sung baseline bundle ký số tại `baselines/v015/*`.
  - Thêm test verify baseline manifest/signature và test threshold compare v0.16 vs v0.15 baseline.
  - Sinh artifacts:
    - `target/ocl/w16/perf/perf_budget_report.json`
    - `target/ocl/w16/perf/perf_trend_vs_v015.json`
    - `target/ocl/w16/perf/perf_baseline_verify_report.json`
- Expected tests:
  - `cargo test --test compile_cache`
  - `cargo test --test observe_cache`
  - `cargo test --test cli_cache`
  - `cargo test --test perf_baseline_v16`
  - `cargo test --test perf_baseline_signature`
- Exit criteria:
  - Tất cả targeted tests pass.
  - Baseline bundle SoT verify thành công (hash + signature).
  - Report perf v0.16 có `run_manifest_ref` và pass threshold contract.

### 2026-03-06 — 16-E Implementation Closeout
- Date: 2026-03-06
- Gate/Step: 16-E
- Implemented:
  - Thêm baseline SoT bundle cho perf:
    - `baselines/v015/perf_budget_report.json`
    - `baselines/v015/run_manifest.json`
    - `baselines/v015/baseline_manifest.json`
    - `baselines/v015/baseline_manifest.sig`
  - Thêm test mới:
    - `tests/perf_baseline_signature.rs`: verify hash/signature baseline bundle và ghi `perf_baseline_verify_report.json`.
    - `tests/perf_baseline_v16.rs`: chạy benchmark deterministic counters, so với baseline v0.15 theo ngưỡng khóa, ghi `perf_budget_report.json` + `perf_trend_vs_v015.json`.
- Files changed:
  - `baselines/v015/perf_budget_report.json`
  - `baselines/v015/run_manifest.json`
  - `baselines/v015/baseline_manifest.json`
  - `baselines/v015/baseline_manifest.sig`
  - `tests/perf_baseline_signature.rs`
  - `tests/perf_baseline_v16.rs`
  - `OCP-OCL-MVP-PLAN-v0.16.md`
- Commands run:
  - `cargo test --test compile_cache`
  - `cargo test --test observe_cache`
  - `cargo test --test cli_cache`
  - `cargo test --test perf_baseline_signature`
  - `cargo test --test perf_baseline_v16`
  - `cargo test`
- Test results:
  - Targeted tests (must-pass for gate):
    - PASS: `compile_cache` (3 tests)
    - PASS: `observe_cache` (2 tests)
    - PASS: `cli_cache` (4 tests)
    - PASS: `perf_baseline_signature` (1 test)
    - PASS: `perf_baseline_v16` (1 test)
  - Regression tests (supporting only):
    - PASS: `cargo test` toàn repo
- Kết luận gate:
  - `DONE`
- Design alignment:
  - `FULL`
- Notes/risks:
  - Baseline v0.15 đang pin theo fixture ổn định `projects/ocp-ocl/apps/hello-cli` với mode `interpreter`; nếu thay fixture baseline trong tương lai phải cập nhật đồng bộ bundle + signature.
  - Ngưỡng hiện dùng deterministic counters, không dùng wallclock làm tiêu chí đạt.

### 2026-03-06 — 16-F Planning Freeze
- Date: 2026-03-06
- Gate/Step: 16-F
- Why:
  - Hoàn tất chain-of-trust productization trước RC gate, bổ sung negative proofs để chứng minh fail-honest thay vì chỉ happy-path.
- Scope:
  - Bổ sung test repo hygiene + license inventory machine-checkable.
  - Bổ sung các negative proofs cho lock signing, attestation, permission approval, trust policy, license policy, cassette tamper.
  - Ghi đầy đủ output bảo mật bắt buộc cho `target/ocl/w16/security/*`.
- Expected tests:
  - `cargo test --test trust_policy`
  - `cargo test --test lock_sign`
  - `cargo test --test perm_review`
  - `cargo test --test attestation`
  - `cargo test --test lts_check`
  - `cargo test --test cassette_sign`
  - `cargo test --test repo_hygiene`
  - `cargo test --test license_inventory`
  - `cargo test --test lock_sign_negative`
  - `cargo test --test attestation_tamper_negative`
  - `cargo test --test perm_bypass_negative`
  - `cargo test --test trust_policy_negative`
  - `cargo test --test license_policy_negative`
  - `cargo test --test cassette_tamper_negative`
  - `cargo test`
- Exit criteria:
  - Tất cả targeted tests pass.
  - Có đủ `security_chain_report.json`, `lts_strict_report.json`, `repo_hygiene_report.json`, `license_inventory.json`.
  - Gate đạt `Design alignment: FULL`.

### 2026-03-06 — 16-F Implementation Closeout
- Date: 2026-03-06
- Gate/Step: 16-F
- Implemented:
  - Bổ sung test `repo_hygiene` với deny-by-default và allowlist tường minh cho 2 fixture path hợp lệ.
  - Bổ sung test `license_inventory` tạo `license_inventory.json` + `sbom.spdx.json` và kiểm policy deny-list.
  - Bổ sung 6 negative tests: lock sign tamper, attestation tamper, permission bypass, trust policy negative, license policy negative, cassette tamper.
  - Mở rộng `lts_check` để phát sinh `security_chain_report.json` và `lts_strict_report.json` theo chuẩn report Gate 16-F.
  - Sửa fixture `attestation_tamper_negative` để đáp ứng precondition locked lane (`[permissions.package]`).
- Files changed:
  - `tests/lts_check.rs`
  - `tests/repo_hygiene.rs`
  - `tests/license_inventory.rs`
  - `tests/lock_sign_negative.rs`
  - `tests/attestation_tamper_negative.rs`
  - `tests/perm_bypass_negative.rs`
  - `tests/trust_policy_negative.rs`
  - `tests/license_policy_negative.rs`
  - `tests/cassette_tamper_negative.rs`
  - `OCP-OCL-MVP-PLAN-v0.16.md`
- Commands run:
  - `cargo test --test trust_policy`
  - `cargo test --test lock_sign`
  - `cargo test --test perm_review`
  - `cargo test --test attestation`
  - `cargo test --test lts_check`
  - `cargo test --test cassette_sign`
  - `cargo test --test repo_hygiene`
  - `cargo test --test license_inventory`
  - `cargo test --test lock_sign_negative`
  - `cargo test --test attestation_tamper_negative`
  - `cargo test --test perm_bypass_negative`
  - `cargo test --test trust_policy_negative`
  - `cargo test --test license_policy_negative`
  - `cargo test --test cassette_tamper_negative`
  - `cargo test`
- Test results:
  - Targeted tests (must-pass for gate):
    - PASS: `trust_policy` (3 tests)
    - PASS: `lock_sign` (3 tests)
    - PASS: `perm_review` (1 test)
    - PASS: `attestation` (2 tests)
    - PASS: `lts_check` (2 tests)
    - PASS: `cassette_sign` (4 tests)
    - PASS: `repo_hygiene` (1 test)
    - PASS: `license_inventory` (1 test)
    - PASS: `lock_sign_negative` (1 test)
    - PASS: `attestation_tamper_negative` (1 test)
    - PASS: `perm_bypass_negative` (1 test)
    - PASS: `trust_policy_negative` (1 test)
    - PASS: `license_policy_negative` (1 test)
    - PASS: `cassette_tamper_negative` (1 test)
  - Regression tests (supporting only):
    - PASS: `cargo test` toàn repo
- Kết luận gate:
  - `DONE`
- Design alignment:
  - `FULL`
- Notes/risks:
  - `repo_hygiene` đang có allowlist cố định cho 2 fixture path đã được track hợp lệ; mọi path artifact/secret mới ngoài allowlist sẽ bị chặn.
  - `license_inventory` hiện parse offline từ `Cargo.lock`; nếu nâng cấp policy license sâu hơn cần bổ sung metadata nguồn license nhưng vẫn phải giữ deterministic/offline cho CI.

### 2026-03-06 — 16-G Planning Freeze
- Date: 2026-03-06
- Gate/Step: 16-G
- Why:
  - Hoàn tất RC dry-run và signoff machine-checkable cho toàn bộ v0.16 trước handoff v1.0.
- Scope:
  - Bổ sung smoke test user journey end-to-end theo luồng CLI chuẩn (`init -> lock -> perm -> attest -> run/replay -> dbg`).
  - Bổ sung release artifact manifest có hash + signature status và checklist readiness.
  - Đảm bảo các report RC bắt buộc được tạo dưới `target/ocl/w16/rc/`.
  - Đóng gate chỉ sau khi pass đầy đủ: targeted tests + full test + clippy + fmt check.
- Expected tests:
  - `cargo test --test v1_user_journey_smoke`
  - `cargo test --test release_artifact_manifest`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
- Exit criteria:
  - 2 targeted tests pass.
  - `cargo test` pass toàn repo.
  - `cargo clippy --all-targets -- -D warnings` pass.
  - `cargo fmt -- --check` pass.
  - Có đủ:
    - `target/ocl/w16/rc/rc_dryrun_report.json`
    - `target/ocl/w16/rc/release_readiness_checklist.json`
    - `target/ocl/w16/rc/golden_user_journey_report.json`
    - `target/ocl/w16/rc/release_artifact_manifest.json`

### 2026-03-06 — 16-G Implementation Closeout
- Date: 2026-03-06
- Gate/Step: 16-G
- Implemented:
  - Thêm test `v1_user_journey_smoke` chạy đầy đủ chuỗi RC user journey:
    - `ocl init --template tool-cli`
    - `ocl lock sync/sign/verify`
    - `ocl perm snapshot/diff/approve`
    - `ocl build --attest` + `ocl verify --attest`
    - `ocl run` + `ocl replay`
    - `ocl dbg --script` smoke
  - Thêm test `release_artifact_manifest`:
    - tổng hợp artifact list bắt buộc cho RC,
    - tính `sha256` cho từng artifact,
    - ghi `signature_status`,
    - verify lại hash ngay trong test,
    - ghi checklist readiness.
  - Sửa lỗi chất lượng để pass chuẩn gate:
    - vá `clippy` (`useless_vec`, `unnecessary_unwrap`) trong các test liên quan.
    - chạy `cargo fmt` để đưa workspace về chuẩn format trước `fmt --check`.
- Files changed:
  - `tests/v1_user_journey_smoke.rs`
  - `tests/release_artifact_manifest.rs`
  - `tests/run_manifest.rs`
  - `tests/run_manifest_allowlist.rs`
  - `tests/ocl_determinism.rs`
  - `tests/conformance_runner.rs`
  - `OCP-OCL-MVP-PLAN-v0.16.md`
- Commands run:
  - `cargo test --test v1_user_journey_smoke`
  - `cargo test --test release_artifact_manifest`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt`
  - `cargo fmt -- --check`
  - `cargo test --test v1_user_journey_smoke`
  - `cargo test --test release_artifact_manifest`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
  - `cargo test`
- Test results:
  - Targeted tests (must-pass for gate):
    - PASS: `v1_user_journey_smoke` (1 test)
    - PASS: `release_artifact_manifest` (1 test)
  - Regression tests (supporting only):
    - PASS: `cargo test` toàn repo
    - PASS: `cargo clippy --all-targets -- -D warnings`
    - PASS: `cargo fmt -- --check`
- Kết luận gate:
  - `DONE`
- Design alignment:
  - `FULL`
- Notes/risks:
  - `release_artifact_manifest` hiện đánh dấu `signature_status` theo sự tồn tại sidecar `.sig`; nếu policy RC yêu cầu ký toàn bộ artifact thì cần mở rộng bước ký trong pipeline v1.0.
  - User journey smoke chạy trong `target/tests/v1_user_journey_smoke/*`; là đường chạy deterministic nội bộ, không thay thế kiểm thử phát hành đa nền tảng.

---

## 14) Checklist khóa trước khi đóng gate
- [x] Gate status đã cập nhật đúng (`TODO/IN_PROGRESS/PARTIAL/DONE`).
- [x] Có đủ cặp `Planning Freeze` + `Implementation Closeout` cho gate đang đóng.
- [x] `Files changed` khớp code delta thực tế của gate.
- [x] `Commands run` là lệnh đã chạy thật, không ghi lệnh dự kiến.
- [x] `Targeted tests (must-pass for gate)` đã pass cho đúng phạm vi thay đổi.
- [x] `Regression tests (supporting only)` đã ghi rõ phạm vi và kết quả.
- [x] Historical evidence chain `v0.1..v0.15` đã verify và có report machine-checkable.
- [x] Historical SoT index (`evidence_index.v1`) đã verify chữ ký thành công.
- [x] Contract inventory completeness report pass với required-contracts SoT.
- [x] Mọi file gate-report JSON đã có `run_manifest_ref`/`run_manifest` hợp lệ.
- [x] Run-manifest allowlist enforcement đã pass (deny-by-default với env flag lạ).
- [x] Gate 16-F có `license_inventory.json` hợp lệ theo policy đã khóa.
- [x] Baseline perf SoT manifest/signature đã verify trước khi so ngưỡng.
- [x] Bộ negative proofs bắt buộc đã pass (lock/attest/perm/trust/license/cassette/upgrade).
- [x] Gate 16-G có `release_artifact_manifest.json` và verify pass.
- [x] Nếu chưa đạt 100%: giữ `IN_PROGRESS/PARTIAL`, không ghi `DONE`.
- [x] Không còn marker `FAIL`/placeholder `PASS/FAIL` trong closeout đã đánh dấu `DONE`.
- [x] Không có lỗi mã hóa tiếng Việt trong nội dung file theo hiển thị IDE.
- [x] Đã ghi `Design alignment: FULL` hoặc `Design alignment: PARTIAL` kèm lý do bất khả thi.

## 15) Handoff v0.16 -> v1.0
- Chỉ mở v1.0 khi:
  - toàn bộ gate core `16-A..16-G` đã `DONE`,
  - Exit Contract v0.16 đạt đầy đủ,
  - không còn finding critical mở.
- Snapshot hợp đồng trước v1.0 phải được chốt và lưu:
  - lane policy,
  - signature canonicalization,
  - trace/audit schema,
  - artifact layout,
  - lock/trust/permission/attestation policy.
- Nếu còn gate `IN_PROGRESS/PARTIAL`, không được đóng v0.16.

---
