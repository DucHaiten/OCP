# OCL v0.18 — v1.0 Shipproof Lock + Migration + Release Packaging (Final Pre-Release Hard Freeze)

Ngày tạo: 2026-03-06  
Trạng thái: `DONE (18-A..18-F DONE)`  
Phạm vi: **OCL-only**  
Tiền đề: v0.1..v0.16 đã audit/productize; v0.17 đã thiết kế adoption hardening (DX/cassette/ecosystem + RL-1..RL-17).

## 0) Mục tiêu v0.18 (LOCKED)
v0.18 **không mở semantics lõi mới**. v0.18 khóa “shipproof” cho v1.0 theo 4 hướng:

1) **Contract Freeze v1.0**: mọi spec mới (cassette v17, pack ABI, supported platform profile, canonical serialization…) trở thành **SoT + signature + inventory completeness**.
2) **Migration/Upgrade path**: cassette/manifest/pack versions có lối đi chính thức, deterministic, fail-honest.
3) **Operability chuẩn**: `ocl perm doctor`/`ocl perm fix --plan|--apply`/`ocl budget analyze`/`ocl cassette ...` thành workflow chuẩn, không còn “kẹt”.
4) **Release Packaging**: artifacts manifest + signing/trust + reproducible build evidence + golden user journeys được chốt thành gate.

---

## 1) Governance + Tracking v0.18

### 1.1 Quy ước cập nhật bắt buộc
- Mọi thay đổi kế hoạch phải cập nhật file này trước khi code.
- v0.18 là **design-first**: chưa mở code implementation khi chưa chốt đủ contract của gate.
- Không nhảy gate: gate sau chỉ mở khi gate trước đạt điều kiện.
- Chỉ chuyển gate sang `DONE` khi có đủ:
  - Planning Freeze + Implementation Closeout
  - `Files changed`, `Commands run`, `Test results`, `Notes/risks`
  - targeted tests pass cho đúng scope gate
- Nếu chưa đạt:
  - giữ `TODO` hoặc `IN_PROGRESS` hoặc `PARTIAL`, không ghi `DONE`.

### 1.2 Quick Snapshot (bắt buộc đọc trước)
- Mục tiêu phiên bản:
  - khóa shipproof pre-release cho v1.0 (contract freeze + migration rehearsal + operability + packaging).
- Trạng thái tổng quan:
  - `DONE (18-A..18-F DONE)`.
- Gate đang làm/đã xong/chưa làm:
  - đã xong: `18-A`, `18-B`, `18-C`, `18-D`, `18-E`, `18-F`.
  - chưa làm: không còn gate mở.
- Bước kế tiếp ngay:
  - chuyển sang quyết định mở nhánh v1.0 release theo điều kiện handoff.
- Lệnh kiểm chứng chuẩn:
  - xem `6) Operational commands (v0.18)`.
- File code trọng yếu đã thay đổi:
  - `projects/ocp-ocl/crates/ocl-cli/src/main.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/src/w18.rs`
  - `projects/ocp-ocl/crates/ocl-runtime-core/src/*`
  - `tests/v18_gate_b_common.rs`
  - `tests/v18_*.rs`
  - `contracts/*`
  - `OCP-OCL-MVP-PLAN-v0.18.md`

### 1.3 Trạng thái Workstreams/Gates v0.18 (tracking)
Workstreams:
- WS-SF (Spec/contract freeze v1.0): `DONE`
- WS-MG (Migration/upgrade rehearsal): `DONE`
- WS-OP (Operability tooling): `DONE`
- WS-RP (Release packaging + reproducible evidence): `DONE`

Gates:
- Gate 18-A — Contract Freeze v1.0 surfaces + SoT signing: `DONE`
- Gate 18-B — Migration & Upgrade Rehearsal (cassette/manifest/packs): `DONE`
- Gate 18-C — Operability Tooling Lock (doctor/fix/budget/cassette): `DONE`
- Gate 18-D — Cassette Operability Proof Pack (quota/dedup/privacy/DoS/boundary): `DONE`
- Gate 18-E — Pack/Connector Shipproof (pack build/sign/verify + laundering/CVE gates): `DONE`
- Gate 18-F — RC Packaging + Final Signoff: `DONE`

### 1.4 Rule mở code v0.18 (LOCKED)
- Chỉ mở code khi đã có:
  - contract rõ,
  - KPI machine-checkable,
  - artifacts paths rõ,
  - exit criteria cụ thể.
- Chỉ mở code khi có chỉ đạo: `bắt đầu code v0.18 <gate>`.

### 1.5 Module -> Crate -> Path mapping (LOCKED)
- CLI workflow/alias/packaging:
  - crate: `ocl-cli`
  - path: `projects/ocp-ocl/crates/ocl-cli/src/main.rs`
- Contract schema + SoT reader/writer + verify helpers:
  - crate: `ocl-sdk`
  - path: `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
- Runtime migration/cassette operability engines:
  - crate: `ocl-runtime-core`
  - path: `projects/ocp-ocl/crates/ocl-runtime-core/src/*`
- Rule:
  - chưa khóa ownership thì không mở implementation gate tương ứng.

### 1.6 Run manifest contract v0.18 (LOCKED)
- Mọi gate phải phát sinh:
  - `target/ocl/w18/meta/run_manifest.json`
- `run_manifest` tối thiểu gồm:
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
- Rule:
  - thiếu `run_manifest` hoặc thiếu field bắt buộc => gate fail.
  - mọi report JSON phải có `run_manifest_ref`.

### 1.7 Scope khóa v0.18
In-scope:
- Contract freeze v1.0 surfaces + SoT/signature/inventory completeness.
- Migration rehearsal deterministic cho cassette/manifest/pack ABI (bao gồm no-op proof).
- Operability canonical workflow + alias contract + negative proofs.
- RC packaging shipproof với manifest signature + RL matrix aggregation.

Out-of-scope:
- Không mở semantics lõi mới.
- Không thêm pack/runtime capability mới ngoài phạm vi shipproof khóa ở v0.18.
- Không mở release production thực trong v0.18 (chỉ rehearsal + signoff).

---

## 2) KPIs v0.18 (machine-checkable)

### KPI-1: v1.0 Contract Surfaces “đóng-kín”
- Có `contract_inventory` và `required_contracts` cho v1.0.
- Mọi contract mới đều có **SoT + signature**.
- PASS khi inventory completeness + signature verify + schema verify đều xanh.

### KPI-2: Migration không đứt
- Gate migration rehearsal luôn chạy cho cassette/manifest/pack ABI (kể cả no-op).
- Mọi `*_upgrade_report.json` của gate này đều bắt buộc có:
  - `no_op` (bool),
  - `input_hash_before`,
  - `input_hash_after`,
  - `sot_ref`,
  - `run_manifest_ref`.
- Nếu không có thay đổi dữ liệu:
  - report tương ứng phải có `no_op=true` và `input_hash_before == input_hash_after`.
- PASS khi upgrade path deterministic và replay/verify vẫn fail-honest theo contract.

### KPI-3: Operability “không kẹt”
- Canonical commands:
  - `ocl perm doctor`
  - `ocl perm fix --plan/--apply`
  - `ocl budget analyze`
  - `ocl cassette ...`
- Alias compatibility (additive, bắt buộc trong v0.18, không được có semantics riêng):
  - `ocl doctor` -> `ocl perm doctor`
  - `ocl fix` -> `ocl perm fix`
- `ocl perm fix --plan/--apply` chỉ tạo patch + records, không bypass strict lane.
- `ocl budget analyze` xuất budget path machine-readable theo semantics edge model.
- PASS khi doctor/fix/analyze hoạt động đúng, alias forward 1-1 đúng contract, và negative proofs không bypass.

### KPI-4: Release packaging chứng minh được
- `release_artifact_manifest.json` (file list + sha256 + signature status) cho v1.0-rc.
- Có tối thiểu 2 golden user journeys.
- PASS khi artifacts verify + reproducible evidence + clippy/fmt/test đều xanh.

---

## 3) v1.0 Contract Surfaces (SoT bắt buộc)

### 3.1 SoT files (LOCKED)
Canonical SoT (bắt buộc tồn tại, tất cả có `.sig`):
- `contracts/required_contracts.v1.json`
- `contracts/contract_inventory_schema.v1.json`
- `contracts/platform/supported_profile.v1.json`
- `contracts/serialization/canonical_serializer.v1.json`
- `contracts/cassette/cassette_storage_v17.v1.json`
- `contracts/packs/pack_abi.v1.json`
- `contracts/packs/connector_set.v1.json`
- `contracts/policy/risk_locks_rl01_rl17.v1.json`
- `contracts/ops/doctor_fix_cases.v1.json`
- `contracts/sot_aliases.v1.json`

Optional mirror (read-only, nếu cần giữ đường dẫn `contracts/v1/...`):
- `contracts/v1/required_contracts.v1.json`
- `contracts/v1/contract_inventory_schema.v1.json`

### 3.2 SoT canonical root + alias/compat policy (LOCKED)
- Canonical SoT root cho v1.0 là `contracts/` (root hiện hành của code/test).
- Reader luôn resolve về canonical path.
- Writer chỉ được ghi canonical path; ghi trực tiếp vào mirror/alias path => fail-honest.
- Nếu tồn tại cả canonical + mirror thì bytes canonical phải identical:
  - mismatch => gate fail.

### 3.3 Signing contract v1 (LOCKED)
- Signature file naming convention (duy nhất, áp dụng cho SoT và release manifest):
  - `sig_path = <contract_path>.sig`
  - ví dụ: `contracts/required_contracts.v1.json.sig`
- Canonical bytes for signing:
  - JSON: UTF-8, no BOM, LF newline, canonical key order, no trailing whitespace.
- Hash:
  - `sha256-v1` trên canonical bytes.
- Signature format:
  - `sig.v1` gồm tối thiểu: `algorithm`, `pubkey_id`, `trust_epoch`, `signature`.
- Trust root for verify:
  - verify `.sig` dùng trust store canonical tại `trust.toml` ở project root đang được evaluate (schema v0.15 line).
  - không được resolve trust store từ alias path hoặc environment override ngoài allow-list.
  - thiếu `pubkey_id` hoặc `trust_epoch` không hợp lệ trong trust store => deny-by-default.
- Verify policy:
  - `locked_v071`: verify bắt buộc pass, fail => fail-honest.
  - `locked_v06`: cho phép compat warn theo policy có audit marker.
  - `quarantine`: có thể allow theo policy, nhưng bắt buộc audit marker.

### 3.4 Rule (LOCKED)
- Thiếu SoT + signature => contract chưa được coi là “shipproof”.
- Mọi report gate phải có `run_manifest_ref` deny-by-default như v0.16.

### 3.5 RL aggregation contract (LOCKED)
- Source-of-truth tổng hợp RL-1..RL-17:
  - `target/ocl/w18/rc/rl_matrix_report.json`
- Schema tối thiểu:
  - `schema`, `run_manifest_ref`, `entries[]`
  - mỗi `entry` gồm: `rl_id`, `status`, `evidence_ref`.
- Rule:
  - thiếu bất kỳ `RL-1..RL-17` => FAIL
  - có bất kỳ `status != PASS` => FAIL

---

## 4) Execution Gates v0.18

### Gate 18-A — Contract Freeze v1.0 surfaces + SoT signing
**Scope**
- Chốt SoT files ở mục 3, ký số và đưa vào inventory.
- Tạo inventory + completeness report.
- Verify alias contract canonical<->mirror theo `contracts/sot_aliases.v1.json`.

**Tests (dự kiến)**
- `tests/v18_contract_inventory_complete.rs`
- `tests/v18_sot_signature_verify.rs`
- `tests/v18_sot_alias_contract.rs`
- `tests/v18_platform_profile_contract.rs`
- `tests/v18_canonical_serializer_contract.rs`

**Artifacts**
- `target/ocl/w18/contracts/contract_inventory.json`
- `target/ocl/w18/contracts/contract_inventory_completeness_report.json`
- `target/ocl/w18/contracts/sot_signature_report.json`
- `target/ocl/w18/contracts/sot_alias_report.json`
- `target/ocl/w18/meta/run_manifest.json`

**Exit criteria**
- thiếu contract_id bắt buộc => FAIL
- SoT signature verify fail => FAIL
- canonical/mirror mismatch theo alias policy => FAIL
- schema mismatch => FAIL

---

### Gate 18-B — Migration & Upgrade Rehearsal (cassette/manifest/packs)
**Scope**
- Cassette upgrade rehearsal: v0.8 bundle → v17 storage (luôn chạy, kể cả no-op).
- Manifest/permission/approval schema upgrade rehearsal (luôn chạy, kể cả no-op).
- Pack ABI versioning + migration + strict-lane deny downgrade.
- No-op proof bắt buộc:
  - `no_op=true`,
  - `input_hash_before == input_hash_after`,
  - có `sot_ref` + `run_manifest_ref`.

**Tests (dự kiến)**
- `tests/v18_cassette_upgrade_v08_to_v17.rs`
- `tests/v18_manifest_schema_upgrade.rs`
- `tests/v18_pack_abi_upgrade.rs`
- `tests/v18_downgrade_attempts_negative.rs`
- `tests/v18_migration_noop_proof.rs`

**Artifacts**
- `target/ocl/w18/migration/cassette_upgrade_report.json`
- `target/ocl/w18/migration/manifest_upgrade_report.json`
- `target/ocl/w18/migration/pack_abi_upgrade_report.json`
- `target/ocl/w18/migration/migration_noop_report.json`

**Per-report schema lock (bắt buộc cho từng `*_upgrade_report.json`)**
- `no_op` (bool)
- `input_hash_before` (sha256-v1)
- `input_hash_after` (sha256-v1)
- `sot_ref`
- `run_manifest_ref`

**Exit criteria**
- upgrade plan không deterministic => FAIL
- apply làm mất replay correctness => FAIL
- downgrade attempt không fail-honest => FAIL
- report thiếu trường bắt buộc theo per-report schema lock => FAIL
- case no-op thiếu proof deterministic => FAIL

---

### Gate 18-C — Operability Tooling Lock (doctor/fix/budget/cassette)
**Scope**
- Canonical:
  - `ocl perm doctor`
  - `ocl perm fix --plan/--apply`
  - `ocl budget analyze`
  - `ocl cassette ...`
- Alias compatibility bắt buộc (additive, không thêm semantics mới):
  - `ocl doctor` -> `ocl perm doctor`
  - `ocl fix` -> `ocl perm fix`
- `ocl perm doctor` sinh report: permissions, budget, cassette hygiene, pack trust, platform profile, reproducible env checks.
- `ocl perm fix --plan/--apply` chỉ tạo patch + records, không bypass strict lane.
- `ocl budget analyze` bắt buộc budget path machine-readable theo edge model.
- khóa SoT case-suite cho doctor/fix:
  - `contracts/ops/doctor_fix_cases.v1.json` + `.sig`.

**Tests (dự kiến)**
- `tests/v18_doctor_all.rs`
- `tests/v18_fix_plan_apply.rs`
- `tests/v18_fix_strict_lane_negative.rs`
- `tests/v18_budget_analyze_path.rs`
- `tests/v18_perm_rubberstamp_guard.rs`
- `tests/v18_cli_alias_contract.rs`

**Artifacts**
- `target/ocl/w18/ops/doctor_report.json`
- `target/ocl/w18/ops/fix_plan_report.json`
- `target/ocl/w18/ops/budget_analyze_report.json`
- `target/ocl/w18/ops/doctor_fix_cases_report.json`

**Exit criteria**
- fix apply mà không tạo record/approval đúng luật => FAIL
- strict lane có bypass => FAIL
- budget INSUFFICIENT không có path schema => FAIL
- alias không forward 1-1 về canonical command => FAIL
- output digest lệch SoT case-suite mà không có migration note => FAIL

---

### Gate 18-D — Cassette Operability Proof Pack (quota/dedup/privacy/DoS/boundary)
**Scope**
- Cassette chunk/dedup/quota/prune/gc chốt contract v17.
- Privacy: redaction/no-secrets gate.
- DoS: caps cho parser/decode/adapters.
- FS boundary: symlink/traversal/hardlink negative proofs.

**Tests (dự kiến)**
- `tests/v18_cassette_quota_enforcement.rs`
- `tests/v18_cassette_dedup_integrity.rs`
- `tests/v18_cassette_prune_plan_apply.rs`
- `tests/v18_privacy_no_secrets.rs`
- `tests/v18_dos_caps.rs`
- `tests/v18_fs_boundary_negative.rs`

**Artifacts**
- `target/ocl/w18/cassette/cassette_operability_report.json`
- `target/ocl/w18/security/privacy_hygiene_report.json`
- `target/ocl/w18/security/dos_caps_report.json`
- `target/ocl/w18/security/fs_boundary_report.json`

**Exit criteria**
- prune/GC làm replay “tự chạy IO thật” => FAIL
- missing chunk mà không fail-honest => FAIL
- secrets leak => FAIL
- boundary escape => FAIL

---

### Gate 18-E — Pack/Connector Shipproof (pack build/sign/verify + laundering/CVE gates)
**Scope**
- `ocl pack build/sign/verify` theo pack ABI.
- strict lane: chỉ trusted+signed+attested packs.
- capability laundering defense theo edge/call-site.
- connector baseline tối thiểu (LOCKED theo `contracts/packs/connector_set.v1.json`):
  - `std.http.client`
  - `std.db.sql` (subset đã khóa)

**Tests (dự kiến)**
- `tests/v18_pack_build_sign_verify.rs`
- `tests/v18_pack_trust_lane_policy.rs`
- `tests/v18_capability_laundering_negative.rs`
- `tests/v18_connector_http_smoke.rs`
- `tests/v18_connector_db_smoke.rs`
- `tests/v18_adapter_cve_policy_gate.rs`

**Artifacts**
- `target/ocl/w18/packs/pack_shipproof_report.json`
- `target/ocl/w18/packs/connector_baseline_report.json`

**Exit criteria**
- pack không signed mà vẫn chạy strict => FAIL
- laundering bypass => FAIL
- connector thiếu permission schema hooks => FAIL
- connector set runtime khác SoT `connector_set.v1.json` => FAIL

---

### Gate 18-F — RC Packaging + Final Signoff
**Scope**
- `release_artifact_manifest.json` cho v1.0-rc:
  - file list + sha256 + signature status + toolchain digest ref.
- manifest phải tự ký và verify:
  - `release_artifact_manifest.sig`.
- 2 golden user journeys tối thiểu:
  1) “tool-cli”: init → lock → perm → build/attest → run/replay → dbg
  2) “connector”: init → add connector pack → perm/budget → run + replay
- wording guard: không claim vượt supported profile + performance positioning.
- tổng hợp RL matrix bắt buộc:
  - `rl_matrix_report.json` gồm `RL-1..RL-17`.

**Tests (dự kiến)**
- `tests/v18_release_artifact_manifest.rs`
- `tests/v18_release_manifest_signature.rs`
- `tests/v18_golden_journey_tool_cli.rs`
- `tests/v18_golden_journey_connector.rs`
- `tests/v18_release_positioning_guard.rs`
- `tests/v18_rl_matrix_aggregate.rs`
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt -- --check`

**Artifacts**
- `target/ocl/w18/rc/release_artifact_manifest.json`
- `target/ocl/w18/rc/release_artifact_manifest.sig`
- `target/ocl/w18/rc/golden_journey_tool_cli_report.json`
- `target/ocl/w18/rc/golden_journey_connector_report.json`
- `target/ocl/w18/rc/rl_matrix_report.json`
- `target/ocl/w18/rc/release_readiness_report.json`

**Exit criteria**
- manifest thiếu artifact bắt buộc => FAIL
- manifest signature verify fail => FAIL
- hash verify fail => FAIL
- golden journey fail => FAIL
- release wording guard fail => FAIL
- thiếu RL entry hoặc có RL fail trong `rl_matrix_report.json` => FAIL

---

## 5) Exit Contract v0.18 (LOCKED)
v0.18 chỉ `DONE` khi:
- Gates `18-A..18-F` đều `DONE`
- không còn finding critical mở
- RL-1..RL-17 đều `PASS` và được tổng hợp vào:
  - `target/ocl/w18/rc/rl_matrix_report.json`
  - `target/ocl/w18/rc/release_readiness_report.json`
- `target/ocl/w18/rc/release_artifact_manifest.sig` verify pass.
- mọi report JSON có `run_manifest_ref` hợp lệ.

---

## 6) Operational commands (v0.18)
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt -- --check`
- `cargo test --test v18_contract_inventory_complete`
- `cargo test --test v18_sot_signature_verify`
- `cargo test --test v18_sot_alias_contract`
- `cargo test --test v18_platform_profile_contract`
- `cargo test --test v18_canonical_serializer_contract`
- `cargo test --test v18_cassette_upgrade_v08_to_v17`
- `cargo test --test v18_manifest_schema_upgrade`
- `cargo test --test v18_pack_abi_upgrade`
- `cargo test --test v18_downgrade_attempts_negative`
- `cargo test --test v18_migration_noop_proof`
- `cargo test --test v18_doctor_all`
- `cargo test --test v18_fix_plan_apply`
- `cargo test --test v18_fix_strict_lane_negative`
- `cargo test --test v18_budget_analyze_path`
- `cargo test --test v18_perm_rubberstamp_guard`
- `cargo test --test v18_cli_alias_contract`
- `cargo test --test v18_cassette_quota_enforcement`
- `cargo test --test v18_cassette_dedup_integrity`
- `cargo test --test v18_cassette_prune_plan_apply`
- `cargo test --test v18_privacy_no_secrets`
- `cargo test --test v18_dos_caps`
- `cargo test --test v18_fs_boundary_negative`
- `cargo test --test v18_pack_build_sign_verify`
- `cargo test --test v18_pack_trust_lane_policy`
- `cargo test --test v18_capability_laundering_negative`
- `cargo test --test v18_connector_http_smoke`
- `cargo test --test v18_connector_db_smoke`
- `cargo test --test v18_adapter_cve_policy_gate`
- `cargo test --test v18_release_artifact_manifest`
- `cargo test --test v18_release_manifest_signature`
- `cargo test --test v18_golden_journey_tool_cli`
- `cargo test --test v18_golden_journey_connector`
- `cargo test --test v18_release_positioning_guard`
- `cargo test --test v18_rl_matrix_aggregate`

---

## 7) Execution Log (standard)
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

### 2026-03-06 — 18-A Planning Freeze
- Date: 2026-03-06
- Gate/Step: 18-A
- Why:
  - khóa shipproof SoT v1.0 theo canonical root `contracts/` + `.sig` + alias policy để mở code các gate sau mà không drift.
- Scope:
  - thêm API `ocl-sdk` cho verify/sign v18 contract set.
  - chốt canonical SoT files ở `contracts/*` theo danh sách Gate 18-A.
  - thêm tests v18 cho completeness/signature/alias/platform/serializer và xuất artifacts `target/ocl/w18/contracts/*`.
- Expected tests:
  - `cargo test --test v18_contract_inventory_complete`
  - `cargo test --test v18_sot_signature_verify`
  - `cargo test --test v18_sot_alias_contract`
  - `cargo test --test v18_platform_profile_contract`
  - `cargo test --test v18_canonical_serializer_contract`
- Exit criteria:
  - đủ contract bắt buộc trong inventory.
  - verify `.sig` pass cho toàn bộ canonical SoT v18.
  - alias canonical<->mirror không lệch bytes.

### 2026-03-06 — 18-A Implementation Closeout
- Date: 2026-03-06
- Gate/Step: 18-A
- Implemented:
  - thêm module `w18` trong `ocl-sdk`:
    - verify/sign contract signatures v18 (`sig.v1`),
    - verify contract set v18 theo required list,
    - verify SoT alias canonical<->mirror.
  - thêm canonical SoT contracts v18 tại `contracts/*` và sinh `.sig` tương ứng.
  - thêm mirror read-only ở `contracts/v1/*` theo alias policy.
  - thêm test suite Gate 18-A (`v18_*`) + sinh artifacts tại `target/ocl/w18/contracts/*`.
  - tách SoT `required_contracts` của v16 sang `contracts/v16/required_contracts.v1.json` để tránh xung đột với v18.
- Files changed:
  - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/src/w18.rs`
  - `tests/v18_gate_a_common.rs`
  - `tests/v18_contract_inventory_complete.rs`
  - `tests/v18_sot_signature_verify.rs`
  - `tests/v18_sot_alias_contract.rs`
  - `tests/v18_platform_profile_contract.rs`
  - `tests/v18_canonical_serializer_contract.rs`
  - `tests/contract_inventory_complete.rs`
  - `contracts/required_contracts.v1.json`
  - `contracts/contract_inventory_schema.v1.json`
  - `contracts/platform/supported_profile.v1.json`
  - `contracts/serialization/canonical_serializer.v1.json`
  - `contracts/cassette/cassette_storage_v17.v1.json`
  - `contracts/packs/pack_abi.v1.json`
  - `contracts/packs/connector_set.v1.json`
  - `contracts/policy/risk_locks_rl01_rl17.v1.json`
  - `contracts/ops/doctor_fix_cases.v1.json`
  - `contracts/sot_aliases.v1.json`
  - `contracts/v1/required_contracts.v1.json`
  - `contracts/v1/contract_inventory_schema.v1.json`
  - `contracts/v16/required_contracts.v1.json`
  - `.sig` files đi kèm cho canonical SoT v18 trong `contracts/*`
  - `trust.toml`
- Commands run:
  - `cargo test --test v18_contract_inventory_complete`
  - `cargo test --test v18_sot_signature_verify`
  - `cargo test --test v18_sot_alias_contract`
  - `cargo test --test v18_platform_profile_contract`
  - `cargo test --test v18_canonical_serializer_contract`
  - `cargo test --test contract_inventory_complete`
- Test results:
- Targeted tests (must-pass for gate):
  - PASS: `v18_contract_inventory_complete`
  - PASS: `v18_sot_signature_verify`
  - PASS: `v18_sot_alias_contract`
  - PASS: `v18_platform_profile_contract`
  - PASS: `v18_canonical_serializer_contract`
- Regression tests (supporting only):
  - PASS: `contract_inventory_complete` (guard chống regression khi tách SoT v16/v18).
- Kết luận gate:
  - DONE
- Design alignment:
  - FULL
- Notes/risks:
  - `required_contracts.v1.json` đã chuyển ngữ nghĩa sang v18; test v16 được trỏ sang `contracts/v16/required_contracts.v1.json` để giữ backward compatibility.

### 2026-03-06 — 18-B Planning Freeze
- Date: 2026-03-06
- Gate/Step: 18-B
- Why:
  - khóa migration rehearsal theo hướng machine-checkable, bỏ loophole “khi có thay đổi” bằng no-op proof bắt buộc cho từng report.
- Scope:
  - cassette upgrade rehearsal v0.8 -> v17 storage và sinh `cassette_upgrade_report.json`.
  - manifest schema upgrade rehearsal ở chế độ no-op deterministic.
  - pack ABI upgrade rehearsal ở chế độ no-op deterministic.
  - downgrade negative theo lane policy + report.
  - tổng hợp no-op proof cho 3 report upgrade.
- Expected tests:
  - `cargo test --test v18_cassette_upgrade_v08_to_v17`
  - `cargo test --test v18_manifest_schema_upgrade`
  - `cargo test --test v18_pack_abi_upgrade`
  - `cargo test --test v18_downgrade_attempts_negative`
  - `cargo test --test v18_migration_noop_proof`
- Exit criteria:
  - từng `*_upgrade_report.json` có đủ: `no_op`, `input_hash_before`, `input_hash_after`, `sot_ref`, `run_manifest_ref`.
  - downgrade attempt trong `locked_v071` bị chặn fail-honest theo policy.
  - có `migration_noop_report.json` tổng hợp kiểm tra per-report schema lock.

### 2026-03-06 — 18-B Implementation Closeout
- Date: 2026-03-06
- Gate/Step: 18-B
- Implemented:
  - thêm helper chung Gate 18-B để sinh report migration deterministic:
    - `tests/v18_gate_b_common.rs`
  - thêm 5 test đúng scope gate:
    - `v18_cassette_upgrade_v08_to_v17`
    - `v18_manifest_schema_upgrade`
    - `v18_pack_abi_upgrade`
    - `v18_downgrade_attempts_negative`
    - `v18_migration_noop_proof`
  - các test sinh đầy đủ artifacts dưới `target/ocl/w18/migration/*` với schema lock đã chốt.
  - xử lý lệch parser ban đầu của `v18_pack_abi_upgrade` bằng cách đọc JSON contract v18 trực tiếp, không phụ thuộc validator schema w17.
- Files changed:
  - `tests/v18_gate_b_common.rs`
  - `tests/v18_cassette_upgrade_v08_to_v17.rs`
  - `tests/v18_manifest_schema_upgrade.rs`
  - `tests/v18_pack_abi_upgrade.rs`
  - `tests/v18_downgrade_attempts_negative.rs`
  - `tests/v18_migration_noop_proof.rs`
  - `OCP-OCL-MVP-PLAN-v0.18.md`
- Commands run:
  - `cargo test --test v18_cassette_upgrade_v08_to_v17`
  - `cargo test --test v18_manifest_schema_upgrade`
  - `cargo test --test v18_pack_abi_upgrade`
  - `cargo test --test v18_downgrade_attempts_negative`
  - `cargo test --test v18_migration_noop_proof`
  - `cargo test --test cassette_upgrade_v08_to_v17`
  - `cargo test --test downgrade_guard`
- Test results:
- Targeted tests (must-pass for gate):
  - PASS: `v18_cassette_upgrade_v08_to_v17`
  - PASS: `v18_manifest_schema_upgrade`
  - PASS: `v18_pack_abi_upgrade`
  - PASS: `v18_downgrade_attempts_negative`
  - PASS: `v18_migration_noop_proof`
- Regression tests (supporting only):
  - PASS: `cassette_upgrade_v08_to_v17`
  - PASS: `downgrade_guard`
- Kết luận gate:
  - DONE
- Design alignment:
  - FULL
- Notes/risks:
  - `cassette_upgrade_report.json` là migration có thay đổi dữ liệu nên `no_op=false` là hợp lệ theo contract.
  - `manifest_upgrade_report.json` và `pack_abi_upgrade_report.json` hiện ở chế độ no-op deterministic (`no_op=true`) và đã chứng minh `input_hash_before == input_hash_after`.

### 2026-03-06 — 18-C Planning Freeze
- Date: 2026-03-06
- Gate/Step: 18-C
- Why:
  - khóa operability workflow canonical + alias contract theo machine-checkable evidence, tránh drift UX giữa `perm` và alias top-level.
- Scope:
  - thêm alias bắt buộc `ocl doctor` -> `ocl perm doctor` và `ocl fix` -> `ocl perm fix`.
  - thêm test v18 cho doctor/fix/budget/rubberstamp/alias.
  - sinh artifacts `target/ocl/w18/ops/*` đúng schema lock của Gate 18-C.
- Expected tests:
  - `cargo test --test v18_doctor_all`
  - `cargo test --test v18_fix_plan_apply`
  - `cargo test --test v18_fix_strict_lane_negative`
  - `cargo test --test v18_budget_analyze_path`
  - `cargo test --test v18_perm_rubberstamp_guard`
  - `cargo test --test v18_cli_alias_contract`
- Exit criteria:
  - alias `doctor/fix` forward 1-1 về canonical command.
  - strict lane không bị bypass qua `perm fix --apply`.
  - budget analyze có path machine-readable và report v18.

### 2026-03-06 — 18-C Implementation Closeout
- Date: 2026-03-06
- Gate/Step: 18-C
- Implemented:
  - thêm alias CLI top-level:
    - `ocl doctor` forward sang `ocl perm doctor`
    - `ocl fix` forward sang `ocl perm fix`
  - cập nhật help text để hiển thị alias contract rõ ràng.
  - thêm helper test Gate 18-C:
    - `tests/v18_gate_c_common.rs`
  - thêm 6 test theo đúng scope:
    - `v18_doctor_all`
    - `v18_fix_plan_apply`
    - `v18_fix_strict_lane_negative`
    - `v18_budget_analyze_path`
    - `v18_perm_rubberstamp_guard`
    - `v18_cli_alias_contract`
  - sinh artifacts ops:
    - `doctor_report.json`
    - `fix_plan_report.json`
    - `budget_analyze_report.json`
    - `doctor_fix_cases_report.json`
    - `perm_rubberstamp_guard_report.json`
- Files changed:
  - `projects/ocp-ocl/crates/ocl-cli/src/main.rs`
  - `tests/v18_gate_c_common.rs`
  - `tests/v18_doctor_all.rs`
  - `tests/v18_fix_plan_apply.rs`
  - `tests/v18_fix_strict_lane_negative.rs`
  - `tests/v18_budget_analyze_path.rs`
  - `tests/v18_perm_rubberstamp_guard.rs`
  - `tests/v18_cli_alias_contract.rs`
  - `OCP-OCL-MVP-PLAN-v0.18.md`
- Commands run:
  - `cargo test --test v18_doctor_all`
  - `cargo test --test v18_fix_plan_apply`
  - `cargo test --test v18_fix_strict_lane_negative`
  - `cargo test --test v18_budget_analyze_path`
  - `cargo test --test v18_perm_rubberstamp_guard`
  - `cargo test --test v18_cli_alias_contract`
  - `cargo test --test perm_doctor`
  - `cargo test --test perm_fix_apply`
  - `cargo test --test budget_analyze`
  - `cargo test --test perm_rubberstamp_guard`
- Test results:
- Targeted tests (must-pass for gate):
  - PASS: `v18_doctor_all`
  - PASS: `v18_fix_plan_apply`
  - PASS: `v18_fix_strict_lane_negative`
  - PASS: `v18_budget_analyze_path`
  - PASS: `v18_perm_rubberstamp_guard`
  - PASS: `v18_cli_alias_contract`
- Regression tests (supporting only):
  - PASS: `perm_doctor`
  - PASS: `perm_fix_apply`
  - PASS: `budget_analyze`
  - PASS: `perm_rubberstamp_guard`
- Kết luận gate:
  - DONE
- Design alignment:
  - FULL
- Notes/risks:
  - `doctor_fix_cases_report.json` hiện được tổng hợp từ case-suite lock trong test Gate 18-C; nếu thay đổi case suite SoT ở gate sau phải cập nhật test tương ứng.

### 2026-03-06 — 18-D Planning Freeze
- Date: 2026-03-06
- Gate/Step: 18-D
- Why:
  - khóa bằng chứng machine-checkable cho cassette operability (quota/dedup/prune), privacy hygiene, DoS caps, và fs boundary negative proofs trước khi mở Gate 18-E.
- Scope:
  - thêm helper test Gate 18-D để tái sử dụng run manifest + CLI harness + report writer.
  - thêm 6 test targeted đúng hợp đồng đã khóa:
    - `v18_cassette_quota_enforcement`
    - `v18_cassette_dedup_integrity`
    - `v18_cassette_prune_plan_apply`
    - `v18_privacy_no_secrets`
    - `v18_dos_caps`
    - `v18_fs_boundary_negative`
  - sinh artifacts:
    - `target/ocl/w18/cassette/cassette_operability_report.json`
    - `target/ocl/w18/security/privacy_hygiene_report.json`
    - `target/ocl/w18/security/dos_caps_report.json`
    - `target/ocl/w18/security/fs_boundary_report.json`
- Expected tests:
  - `cargo test --test v18_cassette_quota_enforcement`
  - `cargo test --test v18_cassette_dedup_integrity`
  - `cargo test --test v18_cassette_prune_plan_apply`
  - `cargo test --test v18_privacy_no_secrets`
  - `cargo test --test v18_dos_caps`
  - `cargo test --test v18_fs_boundary_negative`
- Exit criteria:
  - prune/GC không được làm replay rơi về IO thật.
  - thiếu block tham chiếu phải fail-honest.
  - privacy leak marker phải bị chặn.
  - fs traversal phải bị chặn bằng reason code đúng contract.

### 2026-03-06 — 18-D Implementation Closeout
- Date: 2026-03-06
- Gate/Step: 18-D
- Implemented:
  - thêm helper `tests/v18_gate_d_common.rs` cho run manifest, cassette fixture, CLI runner, merge report section.
  - thêm 6 test Gate 18-D:
    - `tests/v18_cassette_quota_enforcement.rs`
    - `tests/v18_cassette_dedup_integrity.rs`
    - `tests/v18_cassette_prune_plan_apply.rs`
    - `tests/v18_privacy_no_secrets.rs`
    - `tests/v18_dos_caps.rs`
    - `tests/v18_fs_boundary_negative.rs`
  - mỗi test sinh/ghi report đúng path gate, mọi report đều có `run_manifest_ref`.
- Files changed:
  - `tests/v18_gate_d_common.rs`
  - `tests/v18_cassette_quota_enforcement.rs`
  - `tests/v18_cassette_dedup_integrity.rs`
  - `tests/v18_cassette_prune_plan_apply.rs`
  - `tests/v18_privacy_no_secrets.rs`
  - `tests/v18_dos_caps.rs`
  - `tests/v18_fs_boundary_negative.rs`
  - `OCP-OCL-MVP-PLAN-v0.18.md`
- Commands run:
  - `cargo test --test v18_cassette_quota_enforcement`
  - `cargo test --test v18_cassette_dedup_integrity`
  - `cargo test --test v18_cassette_prune_plan_apply`
  - `cargo test --test v18_privacy_no_secrets`
  - `cargo test --test v18_dos_caps`
  - `cargo test --test v18_fs_boundary_negative`
  - `cargo test --test cassette_chunking`
  - `cargo test --test cassette_prune_policy`
  - `cargo test --test cassette_gc_negative`
  - `cargo test --test privacy_artifact_hygiene`
  - `cargo test --test dos_budget_caps`
  - `cargo test --test fs_boundary_security`
- Test results:
- Targeted tests (must-pass for gate):
  - PASS: `v18_cassette_quota_enforcement`
  - PASS: `v18_cassette_dedup_integrity`
  - PASS: `v18_cassette_prune_plan_apply`
  - PASS: `v18_privacy_no_secrets`
  - PASS: `v18_dos_caps`
  - PASS: `v18_fs_boundary_negative`
- Regression tests (supporting only):
  - PASS: `cassette_chunking`
  - PASS: `cassette_prune_policy`
  - PASS: `cassette_gc_negative`
  - PASS: `privacy_artifact_hygiene`
  - PASS: `dos_budget_caps`
  - PASS: `fs_boundary_security`
- Kết luận gate:
  - DONE
- Design alignment:
  - FULL
- Notes/risks:
  - `cassette_operability_report.json` đang merge theo section (`quota_enforcement`, `dedup_integrity`, `prune_plan_apply`); nếu đổi schema section ở gate sau thì phải cập nhật helper merge tương ứng.

### 2026-03-06 — 18-E Planning Freeze
- Date: 2026-03-06
- Gate/Step: 18-E
- Why:
  - khóa bằng chứng shipproof cho pack/connector trước RC packaging: chain build-sign-verify, trust lane policy strict, laundering negative, connector baseline theo SoT, và CVE policy gate.
- Scope:
  - thêm helper test Gate 18-E cho run manifest + CLI runner + report merge.
  - thêm 6 test targeted theo scope gate:
    - `v18_pack_build_sign_verify`
    - `v18_pack_trust_lane_policy`
    - `v18_capability_laundering_negative`
    - `v18_connector_http_smoke`
    - `v18_connector_db_smoke`
    - `v18_adapter_cve_policy_gate`
  - sinh artifacts:
    - `target/ocl/w18/packs/pack_shipproof_report.json`
    - `target/ocl/w18/packs/connector_baseline_report.json`
- Expected tests:
  - `cargo test --test v18_pack_build_sign_verify`
  - `cargo test --test v18_pack_trust_lane_policy`
  - `cargo test --test v18_capability_laundering_negative`
  - `cargo test --test v18_connector_http_smoke`
  - `cargo test --test v18_connector_db_smoke`
  - `cargo test --test v18_adapter_cve_policy_gate`
- Exit criteria:
  - pack tamper phải bị chặn khi verify.
  - strict lane phải chặn pack thiếu trust chain.
  - laundering edge phải chặn đúng reason code.
  - connector runtime phải khớp `connector_set.v1.json` và policy hooks.
  - CVE strict policy phải block trường hợp critical chưa vá.

### 2026-03-06 — 18-E Implementation Closeout
- Date: 2026-03-06
- Gate/Step: 18-E
- Implemented:
  - thêm helper `tests/v18_gate_e_common.rs` cho run manifest, CLI commands, setup project, merge reports.
  - thêm 6 test Gate 18-E:
    - `tests/v18_pack_build_sign_verify.rs`
    - `tests/v18_pack_trust_lane_policy.rs`
    - `tests/v18_capability_laundering_negative.rs`
    - `tests/v18_connector_http_smoke.rs`
    - `tests/v18_connector_db_smoke.rs`
    - `tests/v18_adapter_cve_policy_gate.rs`
  - report `pack_shipproof_report.json` được tổng hợp theo section:
    - `pack_build_sign_verify`
    - `pack_trust_lane_policy`
    - `capability_laundering_negative`
    - `adapter_cve_policy_gate`
  - report `connector_baseline_report.json` được tổng hợp theo section:
    - `connector_http_smoke`
    - `connector_db_smoke`
- Files changed:
  - `tests/v18_gate_e_common.rs`
  - `tests/v18_pack_build_sign_verify.rs`
  - `tests/v18_pack_trust_lane_policy.rs`
  - `tests/v18_capability_laundering_negative.rs`
  - `tests/v18_connector_http_smoke.rs`
  - `tests/v18_connector_db_smoke.rs`
  - `tests/v18_adapter_cve_policy_gate.rs`
  - `OCP-OCL-MVP-PLAN-v0.18.md`
- Commands run:
  - `cargo test --test v18_pack_build_sign_verify`
  - `cargo test --test v18_pack_trust_lane_policy`
  - `cargo test --test v18_capability_laundering_negative`
  - `cargo test --test v18_connector_http_smoke`
  - `cargo test --test v18_connector_db_smoke`
  - `cargo test --test v18_adapter_cve_policy_gate`
  - `cargo test --test pack_signing`
  - `cargo test --test pack_trust_policy`
  - `cargo test --test capability_laundering`
  - `cargo test --test connector_http`
  - `cargo test --test connector_db`
  - `cargo test --test adapter_cve_policy`
- Test results:
- Targeted tests (must-pass for gate):
  - PASS: `v18_pack_build_sign_verify`
  - PASS: `v18_pack_trust_lane_policy`
  - PASS: `v18_capability_laundering_negative`
  - PASS: `v18_connector_http_smoke`
  - PASS: `v18_connector_db_smoke`
  - PASS: `v18_adapter_cve_policy_gate`
- Regression tests (supporting only):
  - PASS: `pack_signing`
  - PASS: `pack_trust_policy`
  - PASS: `capability_laundering`
  - PASS: `connector_http`
  - PASS: `connector_db`
  - PASS: `adapter_cve_policy`
- Kết luận gate:
  - DONE
- Design alignment:
  - FULL
- Notes/risks:
  - report packs hiện lưu `artifact`/`project_root` từ thư mục tạm trong mỗi test; schema giữ mục đích bằng chứng gate, không dùng làm release manifest.

### 2026-03-06 — 18-F Planning Freeze
- Date: 2026-03-06
- Gate/Step: 18-F
- Why:
  - khóa bằng chứng RC packaging cuối cùng trước handoff v1.0: release manifest self-sign + 2 golden journeys + positioning guard + RL matrix aggregate.
- Scope:
  - thêm helper test chung Gate 18-F để tái sử dụng run manifest, CLI harness, hashing, release manifest build/sign.
  - thêm test suite Gate 18-F:
    - `v18_release_artifact_manifest`
    - `v18_release_manifest_signature`
    - `v18_golden_journey_tool_cli`
    - `v18_golden_journey_connector`
    - `v18_release_positioning_guard`
    - `v18_rl_matrix_aggregate`
  - cập nhật helper tương thích để required artifacts phản ánh đúng outputs của Gate 18-A.
- Expected tests:
  - `cargo test --test v18_release_artifact_manifest`
  - `cargo test --test v18_release_manifest_signature`
  - `cargo test --test v18_golden_journey_tool_cli`
  - `cargo test --test v18_golden_journey_connector`
  - `cargo test --test v18_release_positioning_guard`
  - `cargo test --test v18_rl_matrix_aggregate`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
- Exit criteria:
  - release manifest có đủ file bắt buộc, hash đúng và chữ ký verify pass.
  - cả 2 golden journey pass và tạo report machine-checkable.
  - RL matrix đủ `RL-1..RL-17`, không có entry thiếu.
  - full suite + clippy + fmt check pass.

### 2026-03-06 — 18-F Implementation Closeout
- Date: 2026-03-06
- Gate/Step: 18-F
- Implemented:
  - thêm helper test Gate 18-F:
    - `tests/v18_gate_f_common.rs`
  - thêm 6 test đúng scope Gate 18-F:
    - `tests/v18_release_artifact_manifest.rs`
    - `tests/v18_release_manifest_signature.rs`
    - `tests/v18_golden_journey_tool_cli.rs`
    - `tests/v18_golden_journey_connector.rs`
    - `tests/v18_release_positioning_guard.rs`
    - `tests/v18_rl_matrix_aggregate.rs`
  - sửa helper chung để đảm bảo Gate 18-F dùng đúng required artifact set:
    - `tests/v18_gate_f_common.rs`
    - `tests/v18_rl_matrix_aggregate.rs`
  - xử lý lint ảnh hưởng bởi `-D warnings` khi chạy gate:
    - `tests/v18_gate_c_common.rs`
  - cập nhật trạng thái gate/workstream + checklist và closeout trong:
    - `OCP-OCL-MVP-PLAN-v0.18.md`
- Files changed:
  - `tests/v18_gate_f_common.rs`
  - `tests/v18_release_artifact_manifest.rs`
  - `tests/v18_release_manifest_signature.rs`
  - `tests/v18_golden_journey_tool_cli.rs`
  - `tests/v18_golden_journey_connector.rs`
  - `tests/v18_release_positioning_guard.rs`
  - `tests/v18_rl_matrix_aggregate.rs`
  - `tests/v18_gate_c_common.rs`
  - `OCP-OCL-MVP-PLAN-v0.18.md`
- Commands run:
  - `cargo test --test v18_release_artifact_manifest --test v18_release_manifest_signature --test v18_golden_journey_tool_cli --test v18_golden_journey_connector --test v18_release_positioning_guard --test v18_rl_matrix_aggregate`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
  - `cargo fmt`
  - `cargo fmt -- --check`
- Test results:
- Targeted tests (must-pass for gate):
  - PASS: `v18_release_artifact_manifest`
  - PASS: `v18_release_manifest_signature`
  - PASS: `v18_golden_journey_tool_cli`
  - PASS: `v18_golden_journey_connector`
  - PASS: `v18_release_positioning_guard`
  - PASS: `v18_rl_matrix_aggregate`
- Regression tests (supporting only):
  - PASS: `cargo test`
  - PASS: `cargo clippy --all-targets -- -D warnings`
  - PASS: `cargo fmt -- --check`
- Kết luận gate:
  - DONE
- Design alignment:
  - FULL
- Notes/risks:
  - Release manifest hiện ký theo helper chuẩn v18 (`sign_contract_json_v18`) và xuất đồng thời `.json.sig` + alias `.sig`; consumer mới phải ưu tiên canonical `.json.sig`.
  - `cargo fmt` có chạm nhiều file ngoài scope Gate 18-F do workspace format; cần giữ quy trình review diff trước commit để tránh lẫn logic change không mong muốn.

---

## 8) Checklist khóa trước khi đóng gate
- [x] Gate status đã cập nhật đúng (`TODO/IN_PROGRESS/PARTIAL/DONE`).
- [x] Có đủ planning freeze + implementation closeout cho gate đang đóng.
- [x] `Files changed` khớp code delta thực tế của gate.
- [x] `Commands run` là lệnh đã chạy thật.
- [x] `Targeted tests (must-pass for gate)` đã pass đúng phạm vi.
- [x] `Regression tests (supporting only)` đã ghi rõ và không thay thế targeted tests.
- [x] Mọi output artifacts theo gate đã sinh đúng path đã khóa.
- [x] Mọi report JSON có `run_manifest_ref` hợp lệ.
- [x] Nếu gate `DONE`: không còn placeholder `PASS/FAIL` hoặc marker `FAIL` trong block closeout đó.
- [x] Không có lỗi mã hóa tiếng Việt theo hiển thị IDE.

---

## 9) Handoff v0.18 -> v1.0
- Chỉ mở v1.0 release khi:
  - toàn bộ gate core `18-A..18-F` đã `DONE`
  - `release_readiness_report.json` pass
  - không còn finding critical mở

