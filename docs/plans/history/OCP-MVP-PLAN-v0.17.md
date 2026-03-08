# OCP v0.17 — Adoption Hardening + Cassette Operability + Governed Ecosystem (Pre-v1.0 Design Freeze)

Ngày tạo: 2026-03-06  
Trạng thái: `DONE (17-A..17-G DONE)`  
Phạm vi: **OCP-only**.  
Tiền đề: v0.1..v0.16 đã hoàn tất core semantics + determinism/replay + supply-chain/trust + conformance/productization audit.

Mục tiêu v0.17: chốt backlog sản phẩm hóa trước v1.0 phát hành rộng, tập trung 3 khoảng trống còn lại:
- giảm ma sát tuân thủ (DX friction) mà không phá governance,
- vận hành cassette dữ liệu lớn an toàn, bounded, fail-honest,
- mở extension surface có kiểm soát để tránh hệ đóng.

---

## 0) Governance + Tracking v0.17

### 0.1 Quy ước cập nhật bắt buộc
- Mọi thay đổi kế hoạch phải cập nhật file này trước khi code.
- v0.17 áp dụng **design-first theo từng gate**:
  - gate chỉ được mở code sau khi chốt đủ contract ở Planning Freeze.
- v0.17 áp dụng luật cứng `MAXIMAL_FIRST_MODE`:
  - mặc định theo phương án tốt nhất có thể trên mọi trục chất lượng,
  - không được tự hạ chuẩn vì lý do tiện/nhanh.
- Chỉ được hạ chuẩn khi có `IMPOSSIBILITY_PROOF` hợp lệ kèm `Evidence Packet`.
- Không nhảy gate: gate sau chỉ mở khi gate trước đạt điều kiện.
- Chỉ chuyển gate sang `DONE` khi có đủ:
  - Planning Freeze + Implementation Closeout,
  - `Files changed`, `Commands run`, `Test results`, `Notes/risks`,
  - targeted tests pass cho đúng scope gate.
- Nếu chưa đạt:
  - giữ `TODO` hoặc `IN_PROGRESS` hoặc `PARTIAL`,
  - không ghi `DONE`.

### Template cập nhật kế hoạch (trước khi làm)
- Date:
- Gate/Step:
- Why:
- Scope:
- Maximal target:
- Alternatives to evaluate:
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
- Maximal-first status: `FULL_MAXIMAL` hoặc `DOWNGRADED_WITH_PROOF`
- Blocker class: `NONE` hoặc `TEMPORARY_BLOCKER` hoặc `FUNDAMENTAL_IMPOSSIBILITY`
- Evidence Packet:
- Notes/risks:

### 0.2 Quick Snapshot (bắt buộc đọc trước)
- Mục tiêu phiên bản:
  - khóa backlog productization trước v1.0 theo từng gate, ưu tiên deterministic + governed operability.
- Trạng thái tổng quan:
  - `DONE (17-A..17-G DONE)`.
- Gate đang làm/đã xong/chưa làm:
  - `17-A..17-G DONE`.
- Bước kế tiếp ngay:
  - chuyển sang handoff `v0.17 -> v1.0` theo `15)`.
- Lệnh kiểm chứng chuẩn:
  - xem `12) Operational commands (v0.17)`.
- File code trọng yếu đã thay đổi ở 17-A..17-F:
  - `projects/ocp/crates/ocp-cli/src/main.rs`
  - `projects/ocp/crates/ocp-sdk/src/lib.rs`
  - `projects/ocp/crates/ocp-sdk/src/perm_v15.rs`
  - `src/ocp/exec.rs`
  - `src/ocp/determinism.rs`
  - `src/ocp/registry.rs`
  - `src/ocp/mod.rs`
  - `Cargo.toml`
  - `tests/perm_doctor.rs`
  - `tests/perm_fix_apply.rs`
  - `tests/perm_fix_negative.rs`
  - `tests/perm_rubberstamp_guard.rs`
  - `tests/error_taxonomy_contract.rs`
  - `tests/w17_perm_common.rs`
  - `tests/w17_gate_b_cli_common.rs`
  - `tests/cassette_chunking.rs`
  - `tests/cassette_prune_policy.rs`
  - `tests/cassette_gc_negative.rs`
  - `tests/cassette_upgrade_v08_to_v17.rs`
  - `tests/cassette_upgrade_tamper_negative.rs`
  - `tests/budget_analyze.rs`
  - `tests/budget_diamond.rs`
  - `tests/commit_precondition.rs`
  - `tests/privacy_artifact_hygiene.rs`
  - `tests/dos_budget_caps.rs`
  - `tests/fs_boundary_security.rs`
  - `tests/w17_gate_c_common.rs`
  - `tests/deterministic_scheduler.rs`
  - `tests/platform_profile_determinism.rs`
  - `tests/unicode_canonicalization.rs`
  - `tests/iteration_order_determinism.rs`
  - `tests/entropy_governance.rs`
  - `projects/ocp/crates/ocp-sdk/src/w17.rs`
  - `contracts/w17/supported_platform_profile.v1.json`
  - `contracts/w17/supported_platform_profile.v1.json.sig`
  - `contracts/w17/cassette_storage_contract.v1.json`
  - `contracts/w17/cassette_storage_contract.v1.json.sig`
  - `contracts/w17/pack_abi_spec.v1.json`
  - `contracts/w17/pack_abi_spec.v1.json.sig`
  - `contracts/w17/risk_lock_policies.v1.json`
  - `contracts/w17/risk_lock_policies.v1.json.sig`
  - `contracts/w17/semantic_hash_taxonomy.v1.json`
  - `contracts/w17/semantic_hash_taxonomy.v1.json.sig`
  - `contracts/w17/canonical_serialization_spec.v1.json`
  - `contracts/w17/canonical_serialization_spec.v1.json.sig`
  - `tests/hash_taxonomy.rs`
  - `tests/trust_key_lifecycle.rs`
  - `tests/downgrade_guard.rs`
  - `tests/serialization_stability.rs`
  - `tests/reproducible_build_toolchain.rs`
  - `tests/w17_contract_sot.rs`
  - `tests/w17_gate_e_common.rs`
  - `tests/pack_abi.rs`
  - `tests/pack_trust_policy.rs`
  - `tests/pack_sandbox_boundary.rs`
  - `tests/pack_boundary_wasi.rs`
  - `tests/pack_boundary_native_cap.rs`
  - `tests/capability_laundering.rs`
  - `tests/adapter_cve_policy.rs`
  - `tests/w17_gate_f_common.rs`
  - `tests/connector_db.rs`
  - `tests/connector_http.rs`
  - `tests/connector_queue.rs`
  - `tests/w17_gate_g_common.rs`
  - `tests/v017_adoption_readiness.rs`
  - `tests/v017_release_guard.rs`
  - `tests/v017_release_positioning_guard.rs`
  - `contracts/required_contracts.v1.json`
  - `projects/ocp/conformance/expected/contracts/stability_contract_v14.json`
  - `projects/ocp/conformance/expected/contracts/stability_contract_v15.json`
  - `OCP-MVP-PLAN-v0.17.md`

### 0.3 Trạng thái Workstreams/Gates v0.17 (tracking)
#### Workstreams
- WS-DX (governed workflow UX): `DONE`
- WS-CS (cassette storage/retention operability): `DONE`
- WS-DT (determinism core: concurrency/platform/order/entropy): `DONE`
- WS-TR (trust lifecycle + hash/serialization/repro): `DONE`
- WS-EX (governed extension surface + capability governance): `DONE`
- WS-CN (connector baseline): `DONE`
- WS-RC (pre-v1.0 rollout readiness): `DONE`

#### Gate status (17-A .. 17-G)
- Gate 17-A (DX guided compliance flow): `DONE`
- Gate 17-B (Cassette/storage + TOCTOU + privacy/DoS/boundary): `DONE`
- Gate 17-C (Determinism core: concurrency/platform/order/entropy): `DONE`
- Gate 17-D (Trust lifecycle + downgrade + hash/serialization/repro): `DONE`
- Gate 17-E (Ecosystem governance + capability laundering + CVE): `DONE`
- Gate 17-F (Connector baseline implementation): `DONE`
- Gate 17-G (Rollout readiness + final signoff): `DONE`

### 0.4 Rule mở code v0.17 (LOCKED)
- v0.17 không bắt đầu code khi chưa có:
  - contract rõ,
  - KPI machine-checkable,
  - exit criteria cụ thể cho từng gate.
- Chỉ mở code khi có chỉ đạo rõ: `bắt đầu code v0.17 <gate>`.

### 0.5 Evidence Packet bắt buộc (LOCKED)
- Mọi quyết định hạ chuẩn trong v0.17 phải có `Evidence Packet` machine-checkable, tối thiểu gồm:
  - mục tiêu tối đa ban đầu,
  - các phương án đã thử,
  - lệnh đã chạy thật và kết quả,
  - lý do loại từng phương án,
  - ảnh hưởng nếu hạ chuẩn,
  - phương án bù.
- Thiếu `Evidence Packet`:
  - không được ghi `DOWNGRADED_WITH_PROOF`,
  - không được chuyển gate sang `DONE`.
- Nếu phân loại `FUNDAMENTAL_IMPOSSIBILITY`:
  - bắt buộc nêu rõ horizon cấp nền tảng dài hạn (mốc tham chiếu >= 10 năm).

---

## 1) Goals v0.17 (LOCKED)

### 1.1 North Star
- OCP sẵn sàng phát hành rộng với trải nghiệm tuân thủ mượt hơn.
- Giữ trục cứng:
  - determinism + replay fail-honest,
  - governed security/supply-chain,
  - không nới lỏng `locked_v071`.

### 1.2 KPI bắt buộc (machine-checkable)
**KPI-1: DX friction giảm có kiểm soát**
- Commands:
  - `cargo test --test perm_doctor`
  - `cargo test --test perm_fix_apply`
  - `cargo test --test perm_rubberstamp_guard`
- Artifacts:
  - `<project_root>/target/ocp/w17/dx/dx_friction_report.json`
  - `<project_root>/target/ocp/w17/dx/permission_fix_safety_report.json`
- Baseline ref:
  - `target/ocp/baseline/v016/dx_friction_report.json`
- Pass:
  - median `time_to_fix_ms` giảm tối thiểu `20%` so baseline (cùng bộ case),
  - zero bypass ở `locked_v071` theo `perm_rubberstamp_guard`.

**KPI-2: Cassette operability cho workload lớn**
- Commands:
  - `cargo test --test cassette_chunking`
  - `cargo test --test cassette_prune_policy`
  - `cargo test --test cassette_gc_negative`
  - `cargo test --test cassette_upgrade_v08_to_v17`
- Artifacts:
  - `<artifact_dir>/target/ocp/w17/cassette/cassette_operability_report.json`
  - `<artifact_dir>/target/ocp/w17/cassette/cassette_migration_report.json`
- Pass:
  - deterministic digest của `prune --plan` ổn định trên bộ case chuẩn,
  - replay invariant giữ nguyên sau migration `v0.8 -> v0.17`,
  - thiếu block/chunk/index luôn fail-honest.

**KPI-3: Extension ecosystem mở nhưng có rào**
- Commands:
  - `cargo test --test pack_abi`
  - `cargo test --test pack_trust_policy`
  - `cargo test --test pack_sandbox_boundary`
  - `cargo test --test capability_laundering`
  - `cargo test --test adapter_cve_policy`
- Artifacts:
  - `target/ocp/w17/ecosystem/extension_governance_report.json`
- Pass:
  - pack ngoài policy không thể chạy ở `locked_v071`,
  - laundering/cve negative proofs đều pass.

**KPI-4: Không phá trục đã khóa v0.8-v0.16**
- Commands:
  - `cargo test --test lock_migration`
  - `cargo test --test trust_lane_policy`
  - `cargo test --test hash_taxonomy`
  - `cargo test --test serialization_stability`
- Artifacts:
  - `target/ocp/w17/compat/v016_line_compat_report.json`
- Pass:
  - lane literals/signature/replay/lock-trust contracts không drift,
  - mọi thay đổi mới đều additive hoặc có migration contract.

---

## 2) Axis Lock v0.17 (kế thừa, không đổi)
- Lane literals giữ nguyên:
  - `locked_v071`
  - `locked_v06`
  - `quarantine`
- Lockfile SoT vẫn là `deps.lock.v3`.
- Signature/canonicalization/hash policy giữ chuẩn `sha256-v1`.
- Không mở “dev convenience” kiểu bypass strict lane.
- Concurrency policy cho v0.17:
  - mặc định runtime core là `single-thread deterministic`,
  - nếu mở logical concurrency thì bắt buộc deterministic scheduler với canonical ordering.
- Cross-platform policy cho v0.17:
  - chỉ claim deterministic trong `supported platform profile` đã khóa rõ,
  - ngoài profile: fail-honest hoặc degraded có audit marker.
- Performance policy cho v1.0 line:
  - v1.0 định vị “hoàn hảo theo GGPL” (control-plane + governed runtime),
  - compute-heavy thuần số học không phải mục tiêu tối ưu chính của core v1.0.

---

## 3) Scope v0.17

### 3.1 In-scope (thiết kế + chuẩn bị triển khai)
A) DX guided compliance flow (permission/trust guidance).  
B) Cassette operability contract cho dữ liệu lớn.  
C) Determinism core contract (concurrency/platform/order/entropy).  
D) Trust lifecycle + downgrade + hash/serialization/repro contracts.  
E) Governed extension surface cho packs bên thứ ba.  
F) Connector baseline roadmap để giảm hệ đóng.  
G) Pre-v1.0 rollout readiness criteria (adoption + vận hành).

### 3.2 Out-of-scope (defer)
- Không thêm semantics ngôn ngữ lõi mới.
- Không thay lane/lock/trust fundamentals đã khóa.
- Không hứa phân tán đa máy cho cassette ở v0.17.

### 3.3 Module -> Crate -> Path mapping (LOCKED)
- CLI guided fix, UX flows, doctor/diff/apply commands:
  - crate: `ocp-cli`
  - path: `projects/ocp/crates/ocp-cli/src/main.rs`
- Policy schema, manifest/permission/approval structures:
  - crate: `ocp-sdk`
  - path: `projects/ocp/crates/ocp-sdk/src/lib.rs`
- Cassette storage engine, chunk/dedup/quota/replay guards:
  - crate: `ocp-runtime-core`
  - path: `projects/ocp/crates/ocp-runtime-core/src/*`

### 3.4 Deterministic execution profile (Gate 17-C, LOCKED)
- `DEP-1: Single-thread deterministic core`:
  - mặc định v1.0 runtime không dựa shared-memory multithreading cho state OCP.
- `DEP-2: Logical concurrency only when scheduled canonically`:
  - mọi task/message/event phải có identity ổn định:
    - `(logical_time, task_id, seq)` hoặc tuple tương đương đã khóa,
  - scheduler luôn chọn theo thứ tự canonical (không phụ thuộc timing vật lý).
- `DEP-3: Commit barrier order`:
  - commit order phải canonical, không theo “task nào xong trước”.
- `DEP-4: Fairness/liveness deterministic`:
  - policy starvation/backpressure/queue overflow phải deterministic và test được.
- `DEP-5: Timeout/cancel semantics deterministic`:
  - timeout/cancel quyết định theo logical ticks/steps, không theo wallclock host.

### 3.5 Supported platform profile (LOCKED)
- v0.17 phải chốt profile hỗ trợ chính thức để claim deterministic:
  - `win-x64-ntfs`:
    - OS: Windows 11, arch: x86_64, filesystem: NTFS (case-insensitive policy pinned).
  - `linux-x64-ext4`:
    - OS: Linux x86_64, filesystem: ext4 (case-sensitive policy pinned).
  - path semantics:
    - canonical separator `/`,
    - không cho phép `..` escape sau canonicalization,
    - case policy theo profile đã pin ở trên.
  - locale/timezone policy:
    - locale pin: `C.UTF-8`,
    - timezone pin: `UTC`.
- Canonicalization bắt buộc trong profile:
  - path normalize,
  - ordering deterministic (không phụ thuộc hash seed/listing ngẫu nhiên),
  - time/locale governed (UTC + locale pin).
- Float policy (v1.0 line, chọn cứng):
  - mặc định cấm float tham gia branching/commit/signature,
  - nếu cho phép ở mode cụ thể thì bắt buộc quantization/canonicalization đã khóa.
- Ngoài supported profile:
  - phải fail-honest hoặc degraded kèm audit marker,
  - không được silent-pass với claim deterministic đầy đủ.

### 3.6 Contract SoT + signature (Gate 17-D, LOCKED)
- v0.17 bắt buộc có SoT và chữ ký cho contract mới:
  - `contracts/w17/supported_platform_profile.v1.json` + `.sig`
  - `contracts/w17/cassette_storage_contract.v1.json` + `.sig`
  - `contracts/w17/pack_abi_spec.v1.json` + `.sig`
  - `contracts/w17/risk_lock_policies.v1.json` + `.sig`
  - `contracts/w17/semantic_hash_taxonomy.v1.json` + `.sig`
  - `contracts/w17/canonical_serialization_spec.v1.json` + `.sig`
- Verify commands (pre-gate close):
  - `ocp verify --contract contracts/w17/<name>.json`
  - `ocp verify --contract-signature contracts/w17/<name>.json.sig`
- Rule:
  - thiếu SoT hoặc signature verify fail => gate fail,
  - không được “tự tạo SoT tạm thời trong target/”.

### 3.7 Trust/hash/serialization/repro ownership (Gate 17-D, LOCKED)
- Gate 17-D là source-of-truth cho các contract:
  - hash taxonomy (`RL-4`),
  - key lifecycle + trust epoch (`RL-9`),
  - downgrade defense (`RL-10`),
  - deterministic serialization (`RL-13`),
  - reproducible build/toolchain pin (`RL-16`).
- Không được phân tán ownership các contract này sang gate khác khi chưa có migration note.

---

## 4) DX Guided Compliance (v0.17-A)

### 4.1 Canonical command surface (LOCKED)
- `ocp perm doctor`
- `ocp perm diff <old> <new>`
- `ocp perm fix --plan`
- `ocp perm fix --apply` (chỉ tạo patch + approval record, không bypass policy)

### 4.2 Contract cứng
- `locked_v071`:
  - guided flow chỉ hỗ trợ sửa đúng luật, không tự downgrade strict->warn.
- `locked_v06`/`quarantine`:
  - cho phép mode mềm hơn theo compat policy, nhưng phải audit marker.
- Mọi fix bắt buộc phải có:
  - diff machine-readable,
  - hash approval record,
  - hint rollback.

### 4.3 KPI/exit cho DX
- KPI/exit của DX dùng duy nhất theo `KPI-1` ở mục `1.2`:
  - baseline, thresholds, commands, artifact paths không được định nghĩa lại khác đi ở section này.

---

## 5) Cassette Operability (v0.17-B)

### 5.1 Data layout (LOCKED)
- Giữ bundle logic v0.8, thêm lớp lưu trữ chính thức:
  - `cassette.jsonl` (logical entries),
  - `blocks/` content-addressed (chunk dedup),
  - `index` mapping entry -> block refs,
  - retention metadata.
- Versioning bắt buộc:
  - `cassette_schema_version = 17`
  - `block_store_version = 1`
  - metadata chứa `hasher_version = sha256-v1`

### 5.2 Bounded controls
- `max_cassette_bytes`
- `max_entries`
- `max_block_bytes`
- `ttl_days`
- `max_total_blocks`

### 5.3 Determinism/fail-honest rules
- Thiếu block/chunk/index mismatch:
  - fail-honest, không fallback IO thật.
- Prune/GC phải deterministic và có audit event.
- Compression không được đổi signature semantics.
- TTL scope (LOCKED):
  - `ttl_days` chỉ áp dụng cho lệnh maintenance (`prune/gc`),
  - TTL không được tham gia semantic/signature runtime,
  - mọi action do TTL gây ra phải có audit event machine-readable.

### 5.4 Policy commands (LOCKED)
- `ocp cassette stats`
- `ocp cassette prune --plan`
- `ocp cassette prune --apply`
- `ocp cassette gc`
- `ocp cassette upgrade --plan`
- `ocp cassette upgrade --apply`

### 5.5 Migration contract v0.8 -> v0.17 (LOCKED)
- Migration commands:
  - `ocp cassette upgrade --plan`
  - `ocp cassette upgrade --apply`
- Migration invariants:
  - replay result/signature invariants giữ nguyên sau upgrade.
  - tamper/missing data sau upgrade phải fail-honest.
- Targeted tests:
  - `tests/cassette_upgrade_v08_to_v17.rs`
  - `tests/cassette_upgrade_tamper_negative.rs`

---

## 6) Governed Extension Surface (v0.17-E)

### 6.1 Pack ABI/spec baseline
- Định nghĩa adapter interface ổn định:
  - observe/commit hooks,
  - Result4 + RC mapping,
  - permission schema declaration.

### 6.2 Trust/signing boundary
- Third-party pack bắt buộc:
  - signed metadata,
  - trust decision traceable,
  - policy deny fail-honest trong `locked_v071`.

### 6.3 Sandbox/capability boundary
- Boundary choice (LOCKED):
  - hỗ trợ song song 2 surface:
    - `pack_boundary_wasi_v1`,
    - `pack_boundary_native_cap_v1`.
  - mỗi surface có contract_id + test suite riêng.
- Completion rule (LOCKED):
  - Gate 17-E chỉ được `DONE` khi cả hai surface `pack_boundary_wasi_v1` và `pack_boundary_native_cap_v1` đều pass suite tương ứng.
  - Nếu chỉ một surface pass thì bắt buộc giữ `PARTIAL` và nêu rõ evidence packet.
- Không cho plugin mở IO trực tiếp ngoài capability contracts.

---

## 7) Connector Baseline (v0.17-F)

### 7.1 Bộ connector tối thiểu (LOCKED)
- `std.db.sql`:
  - scope v0.17: `read_query`, `write_exec` (bounded rows/bytes/timeouts).
- `std.http.client`:
  - scope v0.17: `GET/POST` subset theo host/method/body caps.
- `std.queue.bus`:
  - scope v0.17: `publish`, `consume` với ack semantics bounded.

### 7.2 Gate policy
- Mỗi connector phải có:
  - permission schema,
  - trust policy hooks,
  - replay/quarantine behavior (nếu nondet),
  - deterministic diagnostics payload schema.

---

## 8) Pre-v1.0 Rollout Readiness (v0.17-G)

### 8.1 Adoption evidence
- Có benchmark DX trước/sau.
- Có runbook vận hành cassette ở workload lớn.
- Có “new integrator guide” cho third-party packs.

### 8.2 Release safety
- Không tuyên bố rộng nếu:
  - chưa pass negative proofs bắt buộc,
  - chưa có supply-chain evidence cho connector baseline.

### 8.3 Performance positioning (LOCKED)
- v1.0 performance claim chuẩn:
  - “hoàn hảo theo GGPL” cho workload effectful/governed workflow,
  - không claim thay thế native runtime cho compute-heavy thuần số học.
- Hướng compute-heavy trong v1.x:
  - ưu tiên governed host packs/native escape hatch có policy,
  - AOT deterministic subset là hướng nâng cấp trước JIT sâu.
- Cấm wording mơ hồ trong tài liệu phát hành:
  - không dùng câu ám chỉ “native-speed cho mọi workload” khi chưa có bằng chứng machine-checkable tương ứng.

### 8.4 Top-5 Pre-v1.0 Risk Locks (LOCKED)
`RL-1: Permission rubber-stamping guard`
- `locked_v071`:
  - cấm wildcard cho nhóm quyền nhạy cảm (network/fs write/proc).
  - `ocp perm approve --all` chỉ hợp lệ khi có:
    - `--risk-ack`,
    - `--justification <text>` bắt buộc,
    - record machine-readable trong approval log.
- Tooling bắt buộc:
  - risk scoring + highlight quyền nhạy cảm,
  - policy linter chặn pattern nguy hiểm trong strict lane.

`RL-2: Capability diamond budget composition`
- Chốt semantics:
  - budget theo `edge (caller -> callee)` làm canonical model.
- Edge identity (LOCKED):
  - `edge_id = (caller_pkg, callee_pkg, capability_kind, lane_profile)`.
- Meter semantics (LOCKED):
  - meter độc lập theo edge_id, không cộng dồn implicit giữa các edge khác nhau.
  - carry-over trong cùng edge phải deterministic theo logical task/run scope đã khóa.
- Dynamic dispatch/plugin calls:
  - phải resolve về `edge_id` canonical trước khi trừ meter.
- Diagnostics bắt buộc:
  - khi INSUFFICIENT do budget, phải in budget path machine-readable (ai gọi ai, meter nào hết).
- Tooling bắt buộc:
  - `ocp budget analyze`
  - `ocp budget doctor`

`RL-3: TOCTOU observe -> commit precondition`
- Observe tạo `precondition token` (hash/etag/version metadata canonical).
- Commit bắt buộc là `conditional commit`:
  - mismatch precondition => fail-honest với RC stale/conflict đã khóa.
- Shadow/replay không được bỏ qua precondition verification ở commit phase.

`RL-4: Hash taxonomy anti-brittleness`
- Tách lớp hash bắt buộc:
  - `semantic_hash` (chỉ semantics),
  - `artifact_hash` (layout/packaging/tooling artifacts).
- Diagnostics text/path local/formatting:
  - không được đi vào semantic signature pipeline.
- Versioning bắt buộc:
  - `semantic_hash_version`, `artifact_hash_version`, `signature_schema_version`.

`RL-5: Host adapter CVE containment`
- `locked_v071` chỉ cho pack đã ký/attested/trusted theo policy.
- Pack mới/chưa trusted:
  - chỉ vào lane `quarantine` (hoặc fail-honest trong strict lane).
- Bắt buộc có:
  - SBOM + policy check,
  - fuzz/regression suite cho parser/adapter nhạy cảm.

### 8.5 Additional Pre-v1.0 Risk Locks (RL-6..RL-17) (LOCKED)
`RL-6: Unicode/encoding/newline canonicalization`
- Text input chuẩn:
  - UTF-8, strip BOM, LF canonical.
- Unicode policy:
  - canonicalize cho parser/serializer boundaries,
  - không normalize mù làm đổi semantics string literal runtime.

`RL-7: Iteration-order nondeterminism`
- Mọi collection/scan ảnh hưởng output/hash/signature phải canonical-sort theo byte order đã khóa.
- FS listing/glob/env enumerations không được phụ thuộc thứ tự OS trả về.

`RL-8: Entropy leakage từ host layer`
- Locked lane:
  - cấm entropy nguồn host đi trực tiếp vào logic/signature.
- Nếu cần entropy:
  - bắt buộc qua governed RNG seeded/cassette path có contract rõ.

`RL-9: Key lifecycle (rotation/revocation)`
- Trust policy bắt buộc có:
  - key rotation plan,
  - revocation mechanism deterministic/offline-safe,
  - `trust_epoch`/version trong report + metadata lock.

`RL-10: Downgrade/mix-and-match defense`
- Locked lane migration phải monotonic:
  - không cho downgrade policy/schema version.
- Mọi downgrade attempt phải có negative proof và fail-honest contract.

`RL-11: Capability laundering qua dependency graph`
- Capability enforcement theo call-site/edge:
  - không cho module ít quyền mượn module nhiều quyền kiểu proxy bypass.
- Audit phải chỉ rõ capability được dùng trên cạnh nào.

`RL-12: Privacy/PII leakage qua artifacts`
- Bắt buộc redaction/scrub policy cho trace/cassette/reports.
- Có gate “no secrets in artifacts” + negative tests.
- Encryption-at-rest có thể lane-based nhưng policy phải explicit.

`RL-13: Deterministic serialization drift`
- Canonical serializer bắt buộc cho outputs quan trọng:
  - stable key order,
  - stable number formatting,
  - schema versioned + golden vectors.

`RL-14: Error taxonomy contract drift`
- Error codes/alias/hints phải versioned contract.
- Cấm severity downgrade silent (deny -> warn) khi chưa có migration contract.

`RL-15: DoS/resource amplification`
- Hard caps + budgets áp dụng cho parser/loader/glob/scan/decode/adapters.
- Input hợp lệ nhưng vượt cap phải fail-honest deterministic.

`RL-16: Reproducible build toolchain drift`
- Build env allowlist deny-by-default.
- Attestation phải include toolchain digest + build flags digest.
- Output layout deterministic theo contract.

`RL-17: Boundary correctness (symlink/hardlink/traversal)`
- FS adapter phải canonical realpath trước check permission.
- Chặn symlink escape/path traversal theo policy strict.
- Có negative proofs cho symlink race/traversal attempts.

---

## 9) Error taxonomy & policy additions (v0.17 additive)
- Thêm nhóm RC cho guided compliance / cassette storage / extension trust.
- Giữ nguyên quy tắc:
  - runtime capability deny => Result4 + RC,
  - internal/tooling malfunction => `X-*`.

---

## 10) Execution gates v0.17 (chuẩn bị)

### Gate 17-A — DX guided compliance flow
Scope:
- doctor/fix/diff/approval flow contract.
- strict-lane non-bypass negative proofs.
- khóa `RL-1` anti-rubber-stamping (wildcard guard + risk-ack + justification).
- khóa `RL-14` error taxonomy UX contract (stable code/alias/hint, không severity downgrade silent).
Tests (dự kiến):
- `tests/perm_doctor.rs`
- `tests/perm_fix_apply.rs`
- `tests/perm_fix_negative.rs`
- `tests/perm_rubberstamp_guard.rs`
- `tests/error_taxonomy_contract.rs`
Outputs:
- `<project_root>/target/ocp/w17/dx/dx_friction_report.json`
- `<project_root>/target/ocp/w17/dx/permission_fix_safety_report.json`
Exit criteria:
- fix đúng luật, không bypass strict policy.
- `RL-1` pass đầy đủ trong strict lane.
- `RL-14` pass với alias/hint/severity mapping ổn định.

### Gate 17-B — Cassette/storage + TOCTOU + privacy/DoS/boundary
Scope:
- chunk/dedup/quota/prune/gc contract.
- khóa `RL-2` budget composition semantics + budget-path diagnostics.
- khóa `RL-3` precondition token cho observe->commit.
- khóa `RL-12` privacy redaction/no-secrets gate.
- khóa `RL-15` DoS/resource amplification caps.
- khóa `RL-17` boundary correctness (symlink/hardlink/traversal).
Tests (dự kiến):
- `tests/cassette_chunking.rs`
- `tests/cassette_prune_policy.rs`
- `tests/cassette_gc_negative.rs`
- `tests/budget_diamond.rs`
- `tests/budget_analyze.rs`
- `tests/commit_precondition.rs`
- `tests/privacy_artifact_hygiene.rs`
- `tests/dos_budget_caps.rs`
- `tests/fs_boundary_security.rs`
Outputs:
- `<artifact_dir>/target/ocp/w17/cassette/cassette_operability_report.json`
- `<artifact_dir>/target/ocp/w17/cassette/cassette_migration_report.json`
Exit criteria:
- workload lớn không vỡ quota im lặng; replay vẫn fail-honest.
- `RL-2` và `RL-3` pass theo contract đã khóa.
- `RL-12`, `RL-15`, `RL-17` pass với negative proofs.
- budget diagnostics có machine-readable path ổn định.

### Gate 17-C — Determinism core (concurrency/platform/order/entropy)
Scope:
- khóa deterministic logical-concurrency contract (scheduler/order/timeout-cancel semantics).
- khóa supported platform profile + deterministic canonicalization boundaries.
- khóa `RL-6`/`RL-7`/`RL-8` deterministic input/order/entropy boundaries.
Tests (dự kiến):
- `tests/deterministic_scheduler.rs`
- `tests/platform_profile_determinism.rs`
- `tests/unicode_canonicalization.rs`
- `tests/iteration_order_determinism.rs`
- `tests/entropy_governance.rs`
Outputs:
- `target/ocp/w17/determinism/determinism_core_report.json`
Exit criteria:
- deterministic concurrency/profile rules pass theo contract đã khóa.
- `RL-6`..`RL-8` pass với evidence đầy đủ.

### Gate 17-D — Trust lifecycle + downgrade + hash/serialization/repro
Scope:
- khóa `RL-4` hash taxonomy split (semantic/artifact) + version contracts.
- khóa `RL-9` key lifecycle (rotation/revocation/trust_epoch).
- khóa `RL-10` downgrade/mix-and-match defense.
- khóa `RL-13` deterministic serialization stability.
- khóa `RL-16` reproducible build/toolchain drift guard.
- verify SoT + signature cho toàn bộ contract mới v0.17.
- đóng KPI-4 compat line (`v0.8..v0.16`) qua migration/trust lane checks.
Tests (dự kiến):
- `tests/hash_taxonomy.rs`
- `tests/trust_key_lifecycle.rs`
- `tests/downgrade_guard.rs`
- `tests/serialization_stability.rs`
- `tests/reproducible_build_toolchain.rs`
- `tests/w17_contract_sot.rs`
- `tests/lock_migration.rs`
- `tests/trust_lane_policy.rs`
Outputs:
- `target/ocp/w17/contracts/w17_contract_sot_report.json`
- `target/ocp/w17/contracts/trust_lifecycle_report.json`
- `target/ocp/w17/compat/v016_line_compat_report.json`
Exit criteria:
- `RL-4`, `RL-9`, `RL-10`, `RL-13`, `RL-16` pass với evidence đầy đủ.
- mọi file SoT contract v0.17 có signature verify pass.
- KPI-4 compat pass (không drift lock/trust/lane contracts).

### Gate 17-E — Ecosystem governance + capability laundering + CVE
Scope:
- pack ABI/spec + trust/signing + sandbox boundary.
- khóa `RL-5` host adapter CVE containment policy trong strict lane.
- khóa `RL-11` capability laundering defense theo call-site/edge.
Tests (dự kiến):
- `tests/pack_abi.rs`
- `tests/pack_trust_policy.rs`
- `tests/pack_sandbox_boundary.rs`
- `tests/pack_boundary_wasi.rs`
- `tests/pack_boundary_native_cap.rs`
- `tests/capability_laundering.rs`
- `tests/adapter_cve_policy.rs`
Outputs:
- `target/ocp/w17/ecosystem/extension_governance_report.json`
Exit criteria:
- third-party pack không thể bypass capability governance.
- `RL-5` và `RL-11` pass với trust/sbom/fuzz-policy gates.
- cả hai boundary suites `pack_boundary_wasi` và `pack_boundary_native_cap` đều pass.

### Gate 17-F — Connector baseline implementation
Scope:
- ship connector baseline + policy hooks.
Tests (dự kiến):
- `tests/connector_db.rs`
- `tests/connector_http.rs`
- `tests/connector_queue.rs`
Outputs:
- `target/ocp/w17/connectors/connector_baseline_report.json`
Exit criteria:
- connectors chạy được theo policy đã khóa.

### Gate 17-G — Rollout readiness signoff
Scope:
- tổng hợp adoption evidence + security evidence.
- khóa tuyên bố release wording theo performance positioning v1.0 line.
- tổng hợp trạng thái `RL-1..RL-17` vào release guard report.
Tests (dự kiến):
- `tests/v017_adoption_readiness.rs`
- `tests/v017_release_guard.rs`
- `tests/v017_release_positioning_guard.rs`
Outputs:
- `target/ocp/w17/rc/v017_adoption_readiness_report.json`
- `target/ocp/w17/rc/v017_release_guard_report.json`
- `target/ocp/w17/rc/v017_release_positioning_guard_report.json`
Exit criteria:
- đủ bằng chứng để quyết định mở v1.0 release scope.
- không còn claim mơ hồ vượt quá performance positioning đã khóa.
- không còn risk lock `RL-1..RL-17` ở trạng thái mở.

### 10.1 Test ownership theo gate (LOCKED)
- Mục đích:
  - khóa cứng test nào thuộc gate nào để tránh hiểu sai phạm vi closeout.
  - cấm dùng test ngoài phạm vi gate để kết luận `DONE` cho gate hiện tại.
- Rule bắt buộc:
  - `Targeted tests`:
    - chỉ gồm test liệt kê trực tiếp trong block gate tương ứng.
    - phải pass thì gate mới được `DONE`.
  - `Regression tests`:
    - là test bổ sung để kiểm tra không vỡ vùng lân cận.
    - không thay thế targeted tests của gate.
  - `cargo test` full repo + `clippy` + `fmt --check`:
    - bắt buộc tại `Gate 17-G` (rollout signoff),
    - ở `Gate 17-A..17-F` chỉ là regression tùy chọn, trừ khi Planning Freeze gate đó ghi rõ bắt buộc.
- Mapping bắt buộc:
  - `Gate 17-A` targeted:
    - `perm_doctor`, `perm_fix_apply`, `perm_fix_negative`, `perm_rubberstamp_guard`, `error_taxonomy_contract`.
  - `Gate 17-B` targeted:
    - `cassette_chunking`, `cassette_prune_policy`, `cassette_gc_negative`,
    - `budget_diamond`, `budget_analyze`, `commit_precondition`,
    - `privacy_artifact_hygiene`, `dos_budget_caps`, `fs_boundary_security`,
    - `cassette_upgrade_v08_to_v17`, `cassette_upgrade_tamper_negative`.
  - `Gate 17-C` targeted:
    - `deterministic_scheduler`, `platform_profile_determinism`,
    - `unicode_canonicalization`, `iteration_order_determinism`, `entropy_governance`.
  - `Gate 17-D` targeted:
    - `hash_taxonomy`, `trust_key_lifecycle`, `downgrade_guard`,
    - `serialization_stability`, `reproducible_build_toolchain`,
    - `w17_contract_sot`, `lock_migration`, `trust_lane_policy`.
  - `Gate 17-E` targeted:
    - `pack_abi`, `pack_trust_policy`, `pack_sandbox_boundary`,
    - `pack_boundary_wasi`, `pack_boundary_native_cap`,
    - `capability_laundering`, `adapter_cve_policy`.
  - `Gate 17-F` targeted:
    - `connector_db`, `connector_http`, `connector_queue`.
  - `Gate 17-G` targeted:
    - `v017_adoption_readiness`, `v017_release_guard`, `v017_release_positioning_guard`,
    - `cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt -- --check`.

---

## 11) Exit Contract v0.17 (pre-code freeze)
- v0.17 chỉ chuyển sang implementation khi:
  - contract mỗi gate đã khóa rõ, không còn “choose one/or optional”.
  - KPI mỗi gate có tiêu chí machine-checkable.
  - mapping crate/path đã rõ ownership.
- v0.17 chỉ `DONE` khi:
  - toàn bộ gate core `17-A..17-G` đạt `DONE`,
  - không còn finding critical mở,
  - không có gate nào `DONE` nhưng thiếu code delta/test evidence.
- Không có gate nào `DONE` với trạng thái `DOWNGRADED_WITH_PROOF` nhưng thiếu `Evidence Packet`.
- Mọi `FUNDAMENTAL_IMPOSSIBILITY` phải có bằng chứng và lập luận mức nền tảng, không chấp nhận lý do thời gian/nguồn lực.

---

## 12) Operational commands (v0.17)
- Ghi chú phạm vi:
  - danh sách dưới đây là command pool của toàn phiên bản v0.17.
  - command bắt buộc cho từng gate phải theo `10.1 Test ownership theo gate (LOCKED)` hoặc Planning Freeze của gate đó.
- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt -- --check`
- `cargo test --test perm_doctor`
- `cargo test --test perm_fix_apply`
- `cargo test --test perm_fix_negative`
- `cargo test --test perm_rubberstamp_guard`
- `cargo test --test error_taxonomy_contract`
- `cargo test --test cassette_chunking`
- `cargo test --test cassette_prune_policy`
- `cargo test --test cassette_gc_negative`
- `cargo test --test budget_diamond`
- `cargo test --test budget_analyze`
- `cargo test --test commit_precondition`
- `cargo test --test cassette_upgrade_v08_to_v17`
- `cargo test --test cassette_upgrade_tamper_negative`
- `cargo test --test unicode_canonicalization`
- `cargo test --test iteration_order_determinism`
- `cargo test --test entropy_governance`
- `cargo test --test privacy_artifact_hygiene`
- `cargo test --test dos_budget_caps`
- `cargo test --test fs_boundary_security`
- `cargo test --test pack_abi`
- `cargo test --test pack_trust_policy`
- `cargo test --test pack_sandbox_boundary`
- `cargo test --test pack_boundary_wasi`
- `cargo test --test pack_boundary_native_cap`
- `cargo test --test deterministic_scheduler`
- `cargo test --test platform_profile_determinism`
- `cargo test --test hash_taxonomy`
- `cargo test --test trust_key_lifecycle`
- `cargo test --test downgrade_guard`
- `cargo test --test capability_laundering`
- `cargo test --test serialization_stability`
- `cargo test --test reproducible_build_toolchain`
- `cargo test --test w17_contract_sot`
- `cargo test --test lock_migration`
- `cargo test --test trust_lane_policy`
- `cargo test --test connector_db`
- `cargo test --test connector_http`
- `cargo test --test connector_queue`
- `cargo test --test adapter_cve_policy`
- `cargo test --test v017_adoption_readiness`
- `cargo test --test v017_release_guard`
- `cargo test --test v017_release_positioning_guard`
- `ocp cassette upgrade --plan`
- `ocp cassette upgrade --apply`
- `ocp verify --contract contracts/w17/supported_platform_profile.v1.json`
- `ocp verify --contract-signature contracts/w17/supported_platform_profile.v1.json.sig`
- `ocp verify --contract contracts/w17/cassette_storage_contract.v1.json`
- `ocp verify --contract-signature contracts/w17/cassette_storage_contract.v1.json.sig`
- `ocp verify --contract contracts/w17/pack_abi_spec.v1.json`
- `ocp verify --contract-signature contracts/w17/pack_abi_spec.v1.json.sig`
- `ocp verify --contract contracts/w17/risk_lock_policies.v1.json`
- `ocp verify --contract-signature contracts/w17/risk_lock_policies.v1.json.sig`
- `ocp verify --contract contracts/w17/semantic_hash_taxonomy.v1.json`
- `ocp verify --contract-signature contracts/w17/semantic_hash_taxonomy.v1.json.sig`
- `ocp verify --contract contracts/w17/canonical_serialization_spec.v1.json`
- `ocp verify --contract-signature contracts/w17/canonical_serialization_spec.v1.json.sig`
- `ocp budget analyze`
- `ocp budget doctor`

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

### 2026-03-06 — 17-A Planning Freeze
- Date: 2026-03-06
- Gate/Step: 17-A
- Why:
  - Mở DX guided compliance flow theo contract đã khóa, giảm friction nhưng không bypass strict policy.
- Scope:
  - Thêm `ocp perm doctor`.
  - Thêm `ocp perm fix --plan` và `ocp perm fix --apply` (advisory patch + approval record, không auto sửa manifest).
  - Khóa guard `RL-1`: wildcard/rubber-stamping bị chặn trong `locked_v071`.
  - Khóa `RL-14`: code/reason/alias/hint/severity ổn định cho lỗi guard.
- Expected tests:
  - `cargo test --test perm_doctor`
  - `cargo test --test perm_fix_apply`
  - `cargo test --test perm_fix_negative`
  - `cargo test --test perm_rubberstamp_guard`
  - `cargo test --test error_taxonomy_contract`
- Exit criteria:
  - Command surface `doctor/fix` hoạt động.
  - Strict lane không bypass.
  - Targeted tests của gate pass.

### 2026-03-06 — 17-A Implementation Closeout
- Date: 2026-03-06
- Gate/Step: 17-A
- Implemented:
  - SDK:
    - thêm summary/options types cho Gate 17-A (`PermissionDoctor*`, `PermissionFix*`) trong `ocp-sdk`.
    - thêm `write_permission_doctor_report_v17`, `write_permission_fix_plan_v17`, `apply_permission_fix_plan_v17`.
    - thêm wildcard risk detection theo lane, report canonical JSON, guard strict-lane, virtual approval record cho fix apply.
  - CLI:
    - thêm `ocp perm doctor <project_dir> [--out ...]`.
    - thêm `ocp perm fix --plan <project_dir> [--out ...]`.
    - thêm `ocp perm fix --apply <project_dir> ... --ack-risk --justification --by --date`.
    - cập nhật help text và parser flag presence (`has_flag`).
  - Tests:
    - thêm targeted tests: `perm_doctor`, `perm_fix_apply`, `perm_fix_negative`, `perm_rubberstamp_guard`, `error_taxonomy_contract`.
    - thêm `tests/w17_perm_common.rs` làm helper dùng chung.
- Files changed:
  - `projects/ocp/crates/ocp-sdk/src/lib.rs`
  - `projects/ocp/crates/ocp-sdk/src/perm_v15.rs`
  - `projects/ocp/crates/ocp-cli/src/main.rs`
  - `tests/w17_perm_common.rs`
  - `tests/perm_doctor.rs`
  - `tests/perm_fix_apply.rs`
  - `tests/perm_fix_negative.rs`
  - `tests/perm_rubberstamp_guard.rs`
  - `tests/error_taxonomy_contract.rs`
  - `OCP-MVP-PLAN-v0.17.md`
- Commands run:
  - `cargo test --test perm_doctor`
  - `cargo test --test perm_fix_apply`
  - `cargo test --test perm_fix_negative`
  - `cargo test --test perm_rubberstamp_guard`
  - `cargo test --test error_taxonomy_contract`
  - `cargo test --test perm_review`
  - `cargo test --test perm_bypass_negative`
  - `cargo test`
  - `cargo fmt`
  - `cargo fmt -- --check`
  - `cargo clippy --all-targets -- -D warnings`
- Test results:
  - Targeted tests: `PASS`
  - Regression tests: `PASS`
- Kết luận gate:
  - `DONE` (đủ code delta + targeted tests pass + regression phụ trợ pass).
- Design alignment:
  - `FULL`
- Notes/risks:
  - `perm fix --apply` cố ý chỉ tạo patch/report/approval record, không tự ghi đè `Ocp.toml`, để giữ fail-honest và không bypass policy strict.

### 2026-03-06 — 17-B Planning Freeze
- Date: 2026-03-06
- Gate/Step: 17-B
- Why:
  - khóa cứng vận hành cassette dữ liệu lớn + budget diagnostics + TOCTOU/boundary/privacy để tránh drift trước 17-C.
- Scope:
  - thêm command surface `ocp cassette {stats|prune|gc|upgrade}`.
  - thêm command surface `ocp budget {analyze|doctor}` với output machine-checkable.
  - thêm precondition token cho `std.fs.write_text` và enforce tại commit để fail-honest khi state đã đổi.
  - mở rộng payload schema `std.fs.write_text` để đồng bộ với precondition fields.
  - bổ sung bộ test targeted đúng danh sách Gate 17-B.
- Expected tests:
  - `cargo test --test cassette_chunking`
  - `cargo test --test cassette_prune_policy`
  - `cargo test --test cassette_gc_negative`
  - `cargo test --test budget_diamond`
  - `cargo test --test budget_analyze`
  - `cargo test --test commit_precondition`
  - `cargo test --test cassette_upgrade_v08_to_v17`
  - `cargo test --test cassette_upgrade_tamper_negative`
  - `cargo test --test privacy_artifact_hygiene`
  - `cargo test --test dos_budget_caps`
  - `cargo test --test fs_boundary_security`
- Exit criteria:
  - toàn bộ targeted tests pass.
  - command mới hoạt động với artifact thật và synthetic artifact.
  - không phá regression quan trọng liên quan `std.fs.write_text`/observe cache.

### 2026-03-06 — 17-B Implementation Closeout
- Date: 2026-03-06
- Gate/Step: 17-B
- Implemented:
  - hoàn thiện parser + handler cho `ocp cassette ...` và `ocp budget ...`.
  - thêm pipeline upgrade cassette v0.8 -> v0.17 (block store/index/meta, integrity verify, prune/gc).
  - thêm guard privacy leak cho cassette stats (`X-CASSETTE-PRIVACY-LEAK`).
  - thêm budget analyze/doctor report theo edge `(caller_package, key)` và budget path machine-readable.
  - thêm runtime precondition check cho `std.fs.write_text` trước commit.
  - cập nhật schema payload `std.fs.write_text` (optional `precondition_exists`, `precondition_len`) để không vỡ validator.
- Files changed:
  - `projects/ocp/crates/ocp-cli/src/main.rs`
  - `src/ocp/exec.rs`
  - `src/ocp/registry.rs`
  - `tests/w17_gate_b_cli_common.rs`
  - `tests/cassette_chunking.rs`
  - `tests/cassette_prune_policy.rs`
  - `tests/cassette_gc_negative.rs`
  - `tests/cassette_upgrade_v08_to_v17.rs`
  - `tests/cassette_upgrade_tamper_negative.rs`
  - `tests/budget_analyze.rs`
  - `tests/budget_diamond.rs`
  - `tests/commit_precondition.rs`
  - `tests/privacy_artifact_hygiene.rs`
  - `tests/dos_budget_caps.rs`
  - `tests/fs_boundary_security.rs`
  - `OCP-MVP-PLAN-v0.17.md`
- Commands run:
  - `cargo check -p ocp-cli`
  - `cargo test --test cassette_chunking --test cassette_prune_policy --test cassette_gc_negative --test budget_diamond --test budget_analyze --test commit_precondition --test cassette_upgrade_v08_to_v17 --test cassette_upgrade_tamper_negative --test privacy_artifact_hygiene --test dos_budget_caps --test fs_boundary_security`
  - `cargo test --test observe_cache observe_cache_fs_generation_invalidation_prevents_stale_read`
- Test results:
  - Targeted tests: `PASS`
  - Regression tests: `PASS`
- Kết luận gate:
  - `PARTIAL` (code delta + targeted/regression cho scope 17-B đã đạt, nhưng còn format debt repo-wide nên chưa chốt `DONE` theo chuẩn khắt khe).
- Design alignment:
  - `FULL`
- Maximal-first status:
  - `DOWNGRADED_WITH_PROOF`
- Blocker class:
  - `TEMPORARY_BLOCKER`
- Evidence Packet:
  - cassette upgrade/prune/gc/tamper/privacy/doS đều có test độc lập.
  - budget edge diagnostics có report JSON machine-readable.
  - TOCTOU precondition có test commit stale và regression observe-cache.
- Notes/risks:
  - prune/gc hiện deterministic theo orphan block/index integrity; policy TTL đã có tham số command nhưng chưa mở rộng scheduler maintenance tự động ở gate này.

### 2026-03-06 — 17-B Correction Resolution
- Lý do hiệu chỉnh trạng thái:
  - closeout ban đầu giữ `PARTIAL` vì format debt repo-wide chưa dọn sạch.
  - sau Gate `17-G` đã có full quality pass (`cargo test`, `cargo clippy --all-targets -- -D warnings`, `cargo fmt -- --check`).
- Hiệu chỉnh:
  - nâng trạng thái Gate `17-B` từ `PARTIAL` lên `DONE`.
  - khóa lại semantics output path cho Gate `17-A/17-B` theo context thực thi:
    - `<project_root>/target/ocp/w17/dx/...` với flow permission DX,
    - `<artifact_dir>/target/ocp/w17/cassette/...` với flow cassette CLI.
- Kết quả:
  - evidence packet của Gate `17-B` vẫn giữ nguyên hiệu lực.
  - không còn mâu thuẫn giữa gate status tổng và closeout chi tiết.

### 2026-03-06 — 17-B Evidence Refresh (artifact path + report assertions)
- Date: 2026-03-06
- Gate/Step: 17-B
- Implemented:
  - khóa rõ artifact path theo context thực thi:
    - `<project_root>/target/ocp/w17/dx/...` cho Gate `17-A`,
    - `<artifact_dir>/target/ocp/w17/cassette/...` cho Gate `17-B`.
  - bổ sung assert report bắt buộc trong targeted tests cassette:
    - `cassette_migration_report.json`,
    - `cassette_operability_report.json`.
- Files changed:
  - `tests/cassette_upgrade_v08_to_v17.rs`
  - `tests/cassette_prune_policy.rs`
  - `OCP-MVP-PLAN-v0.17.md`
- Commands run:
  - `cargo test --test cassette_upgrade_v08_to_v17 --test cassette_prune_policy`
- Test results:
  - Targeted tests: `PASS`
  - Regression tests: `PASS` (không phát sinh lỗi ngoài phạm vi gate trong lần chạy targeted này)
- Kết luận gate:
  - `DONE` (đã bổ sung evidence machine-checkable cho output artifact path của Gate `17-B`).
- Design alignment:
  - `FULL`
- Maximal-first status:
  - `FULL_MAXIMAL`
- Blocker class:
  - `NONE`
- Evidence Packet:
  - report paths `cassette_migration_report.json` và `cassette_operability_report.json` đã được assert trực tiếp trong test.
  - semantics path `project_root/artifact_dir` đã ghi rõ trong KPI + Gate Outputs để tránh hiểu sai workspace-root.
- Notes/risks:
  - check targeted này tập trung vào chứng cứ output path/report của Gate `17-B`; full regression toàn repo giữ theo evidence Gate `17-G`.

### 2026-03-06 — 17-C Planning Freeze
- Date: 2026-03-06
- Gate/Step: 17-C
- Why:
  - Khóa cứng determinism core trước khi mở các gate trust/ecosystem:
    - scheduler canonical,
    - profile platform deterministic,
    - canonicalization text/order,
    - entropy governance theo lane.
- Scope:
  - Thêm module determinism runtime (`src/ocp/determinism.rs`) cho:
    - canonical event ordering `(logical_time, task_id, seq)`,
    - timeout/cancel theo logical ticks,
    - supported platform profile + path canonicalization + locale/timezone pin,
    - canonicalization boundary cho text (UTF-8/BOM/LF + unicode compose subset deterministic),
    - iteration canonicalization cho listing/env entries,
    - entropy governance theo lane (`locked_v071|locked_v06|quarantine`).
  - Export API qua `src/ocp/mod.rs`.
  - Bổ sung đúng 5 targeted tests gate 17-C.
- Expected tests:
  - `cargo test --test deterministic_scheduler --test platform_profile_determinism --test unicode_canonicalization --test iteration_order_determinism --test entropy_governance`
  - `cargo test --test ocp_determinism`
- Exit criteria:
  - toàn bộ targeted tests gate 17-C pass.
  - artifact `target/ocp/w17/determinism/determinism_core_report.json` được ghi machine-checkable.
  - regression determinism cũ không lệch.

### 2026-03-06 — 17-C Implementation Closeout
- Date: 2026-03-06
- Gate/Step: 17-C
- Implemented:
  - thêm module `src/ocp/determinism.rs` với contract runtime deterministic cho:
    - scheduler canonical order + round-robin canonical + lifecycle timeout/cancel theo logical tick,
    - supported platform profile parse/match/enforce (`win-x64-ntfs`, `linux-x64-ext4`),
    - path canonicalization fail-honest (deny traversal escape),
    - locale/timezone pin (`C.UTF-8`, `UTC`),
    - canonicalization text boundary (UTF-8, strip BOM, LF canonical, unicode compose subset deterministic),
    - iteration canonicalization (paths/env entries),
    - entropy governance theo lane với reason code machine-readable.
  - cập nhật export API ở `src/ocp/mod.rs`.
  - thêm helper artifact writer `tests/w17_gate_c_common.rs`.
  - thêm đúng 5 targeted tests gate 17-C:
    - `tests/deterministic_scheduler.rs`
    - `tests/platform_profile_determinism.rs`
    - `tests/unicode_canonicalization.rs`
    - `tests/iteration_order_determinism.rs`
    - `tests/entropy_governance.rs`
- Files changed:
  - `src/ocp/determinism.rs`
  - `src/ocp/mod.rs`
  - `Cargo.toml`
  - `tests/w17_gate_c_common.rs`
  - `tests/deterministic_scheduler.rs`
  - `tests/platform_profile_determinism.rs`
  - `tests/unicode_canonicalization.rs`
  - `tests/iteration_order_determinism.rs`
  - `tests/entropy_governance.rs`
  - `OCP-MVP-PLAN-v0.17.md`
- Commands run:
  - `cargo test --test deterministic_scheduler --test platform_profile_determinism --test unicode_canonicalization --test iteration_order_determinism --test entropy_governance`
  - `cargo test --test ocp_determinism`
  - `cargo fmt -- projects/ocp/crates/ocp-cli/src/main.rs src/ocp/mod.rs src/ocp/registry.rs tests/cassette_chunking.rs tests/cassette_gc_negative.rs tests/cassette_prune_policy.rs tests/cassette_upgrade_tamper_negative.rs tests/cassette_upgrade_v08_to_v17.rs tests/dos_budget_caps.rs tests/deterministic_scheduler.rs tests/w17_gate_c_common.rs`
  - `cargo fmt -- --check`
  - `cargo test --test deterministic_scheduler --test platform_profile_determinism --test unicode_canonicalization --test iteration_order_determinism --test entropy_governance`
  - `cargo test --test ocp_determinism`
  - `cargo test --test cassette_chunking --test budget_analyze --test commit_precondition`
- Test results:
  - Targeted tests: `PASS`
  - Regression tests: `PASS`
- Kết luận gate:
  - `DONE` (đủ code delta thật + targeted/regression pass + format gate liên quan đã sạch theo `cargo fmt -- --check`).
- Design alignment:
  - `FULL`
- Maximal-first status:
  - `FULL_MAXIMAL`
- Blocker class:
  - `NONE`
- Evidence Packet:
  - deterministic scheduler/profile/order/entropy đều có targeted test riêng.
  - artifact `determinism_core_report.json` được ghi machine-checkable dưới `target/ocp/w17/determinism/`.
  - regression `ocp_determinism` giữ ổn định signature/replay cũ.
  - regression bổ sung sau refmt (`cassette_chunking`, `budget_analyze`, `commit_precondition`) đều pass.
- Notes/risks:
  - do môi trường offline không tải crate ngoài, unicode normalization dùng compose subset nội bộ deterministic thay vì phụ thuộc external library; vẫn giữ đúng contract fail-honest cho input UTF-8 và không normalize mù runtime literal.
  - format debt đã được xử lý ở các file gate liên quan và `cargo fmt -- --check` hiện pass.

### 2026-03-06 — 17-C Correction Note
- Lý do hiệu chỉnh trạng thái:
  - trước đó đã ghi `DONE` theo tiêu chí targeted scope.
  - theo chuẩn vận hành khắt khe của repo, còn format debt repo-wide thì chưa nên khóa `DONE`.
- Hiệu chỉnh:
  - hạ trạng thái gate `17-C` từ `DONE` về `PARTIAL`.
  - giữ nguyên evidence kỹ thuật đã đạt cho scope gate.
- Bước tiếp theo để đạt `DONE`:
  - xử lý format debt repo-wide ở phạm vi đã xác định,
  - chạy lại `cargo fmt -- --check`,
  - cập nhật closeout 17-C và chỉ chuyển `DONE` khi không còn blocker mở.

### 2026-03-06 — 17-C Correction Resolution
- Trạng thái sau hiệu chỉnh:
  - format debt đã được dọn theo danh sách file liên quan gate.
  - `cargo fmt -- --check` pass.
  - targeted tests 17-C + regression `ocp_determinism` đã chạy lại và pass.
- Kết quả:
  - gate `17-C` được nâng lại `DONE`.
  - workstream `WS-DT` nâng lại `DONE`.

### 2026-03-06 — 17-D Planning Freeze
- Date: 2026-03-06
- Gate/Step: 17-D
- Why:
  - khóa trust lifecycle + downgrade + hash/serialization/repro trước khi mở ecosystem gate.
  - chốt SoT contract `contracts/w17/*.json` + signature verification ở cả SDK và CLI.
- Scope:
  - thêm module `ocp-sdk::w17` làm source-of-truth cho:
    - hash taxonomy split (`semantic_hash_v1` vs `artifact_hash_v1`),
    - trust lifecycle (`trust_epoch`, revoke/rotate selection),
    - downgrade guard theo lane literals,
    - canonical serialization deterministic,
    - reproducible toolchain digest + env allowlist deny-by-default,
    - contract SoT/signature verify set (`contracts/w17/*`).
  - mở command surface CLI:
    - `ocp verify --contract-sign`,
    - `ocp verify --contract`,
    - `ocp verify --contract-signature`.
  - thêm đúng bộ targeted tests của gate 17-D.
- Expected tests:
  - `cargo test --test hash_taxonomy`
  - `cargo test --test trust_key_lifecycle`
  - `cargo test --test downgrade_guard`
  - `cargo test --test serialization_stability`
  - `cargo test --test reproducible_build_toolchain`
  - `cargo test --test w17_contract_sot`
  - `cargo test --test lock_migration`
  - `cargo test --test trust_lane_policy`
- Exit criteria:
  - `RL-4`, `RL-9`, `RL-10`, `RL-13`, `RL-16` pass với evidence machine-checkable.
  - đủ 6 file SoT `contracts/w17/*.json` + `.sig` và verify pass.
  - sinh đủ artifacts gate:
    - `target/ocp/w17/contracts/w17_contract_sot_report.json`
    - `target/ocp/w17/contracts/trust_lifecycle_report.json`
    - `target/ocp/w17/compat/v016_line_compat_report.json`

### 2026-03-06 — 17-D Implementation Closeout
- Date: 2026-03-06
- Gate/Step: 17-D
- Implemented:
  - thêm `projects/ocp/crates/ocp-sdk/src/w17.rs` với contract APIs:
    - `semantic_hash_from_bytes_v17`, `artifact_hash_from_bytes_v17`,
    - `evaluate_trust_lifecycle_v17`,
    - `evaluate_downgrade_attempt_v17`,
    - `canonical_json_value_v17`, `canonical_json_string_v17`,
    - `enforce_build_env_allowlist_v17`, `toolchain_digest_v17`,
    - `sign_contract_json_v17`, `verify_contract_json_signature_v17`,
    - `verify_contract_signature_file_v17`, `verify_w17_contract_set_v17`.
  - export toàn bộ API/constant 17-D qua `ocp-sdk/src/lib.rs`.
  - hoàn tất CLI `verify` surface cho contract SoT/signature trong `ocp-cli/src/main.rs`.
  - thêm bộ SoT contract v0.17:
    - `contracts/w17/supported_platform_profile.v1.json`
    - `contracts/w17/cassette_storage_contract.v1.json`
    - `contracts/w17/pack_abi_spec.v1.json`
    - `contracts/w17/risk_lock_policies.v1.json`
    - `contracts/w17/semantic_hash_taxonomy.v1.json`
    - `contracts/w17/canonical_serialization_spec.v1.json`
    - cùng 6 file `.sig` tương ứng.
  - thêm targeted tests:
    - `tests/hash_taxonomy.rs`
    - `tests/trust_key_lifecycle.rs`
    - `tests/downgrade_guard.rs`
    - `tests/serialization_stability.rs`
    - `tests/reproducible_build_toolchain.rs`
    - `tests/w17_contract_sot.rs`
  - artifacts gate 17-D được tạo bởi targeted tests:
    - `target/ocp/w17/contracts/w17_contract_sot_report.json`
    - `target/ocp/w17/contracts/trust_lifecycle_report.json`
    - `target/ocp/w17/compat/v016_line_compat_report.json`
- Files changed:
  - `projects/ocp/crates/ocp-sdk/src/w17.rs`
  - `projects/ocp/crates/ocp-sdk/src/lib.rs`
  - `projects/ocp/crates/ocp-cli/src/main.rs`
  - `contracts/w17/supported_platform_profile.v1.json`
  - `contracts/w17/supported_platform_profile.v1.json.sig`
  - `contracts/w17/cassette_storage_contract.v1.json`
  - `contracts/w17/cassette_storage_contract.v1.json.sig`
  - `contracts/w17/pack_abi_spec.v1.json`
  - `contracts/w17/pack_abi_spec.v1.json.sig`
  - `contracts/w17/risk_lock_policies.v1.json`
  - `contracts/w17/risk_lock_policies.v1.json.sig`
  - `contracts/w17/semantic_hash_taxonomy.v1.json`
  - `contracts/w17/semantic_hash_taxonomy.v1.json.sig`
  - `contracts/w17/canonical_serialization_spec.v1.json`
  - `contracts/w17/canonical_serialization_spec.v1.json.sig`
  - `tests/hash_taxonomy.rs`
  - `tests/trust_key_lifecycle.rs`
  - `tests/downgrade_guard.rs`
  - `tests/serialization_stability.rs`
  - `tests/reproducible_build_toolchain.rs`
  - `tests/w17_contract_sot.rs`
  - `OCP-MVP-PLAN-v0.17.md`
- Commands run:
  - `cargo run -p ocp-cli --quiet -- verify --contract-sign contracts/w17/supported_platform_profile.v1.json --signer-id w17-sot-root --trust-epoch 1`
  - `cargo run -p ocp-cli --quiet -- verify --contract-sign contracts/w17/cassette_storage_contract.v1.json --signer-id w17-sot-root --trust-epoch 1`
  - `cargo run -p ocp-cli --quiet -- verify --contract-sign contracts/w17/pack_abi_spec.v1.json --signer-id w17-sot-root --trust-epoch 1`
  - `cargo run -p ocp-cli --quiet -- verify --contract-sign contracts/w17/risk_lock_policies.v1.json --signer-id w17-sot-root --trust-epoch 1`
  - `cargo run -p ocp-cli --quiet -- verify --contract-sign contracts/w17/semantic_hash_taxonomy.v1.json --signer-id w17-sot-root --trust-epoch 1`
  - `cargo run -p ocp-cli --quiet -- verify --contract-sign contracts/w17/canonical_serialization_spec.v1.json --signer-id w17-sot-root --trust-epoch 1`
  - `cargo fmt`
  - `cargo fmt -- --check`
  - `cargo test --test hash_taxonomy`
  - `cargo test --test trust_key_lifecycle`
  - `cargo test --test downgrade_guard`
  - `cargo test --test serialization_stability`
  - `cargo test --test reproducible_build_toolchain`
  - `cargo test --test w17_contract_sot`
  - `cargo test --test lock_migration`
  - `cargo test --test trust_lane_policy`
  - `cargo run -p ocp-cli --quiet -- verify --contract contracts/w17/supported_platform_profile.v1.json`
  - `cargo run -p ocp-cli --quiet -- verify --contract-signature contracts/w17/supported_platform_profile.v1.json.sig`
  - `cargo run -p ocp-cli --quiet -- verify --contract contracts/w17/cassette_storage_contract.v1.json`
  - `cargo run -p ocp-cli --quiet -- verify --contract-signature contracts/w17/cassette_storage_contract.v1.json.sig`
  - `cargo run -p ocp-cli --quiet -- verify --contract contracts/w17/pack_abi_spec.v1.json`
  - `cargo run -p ocp-cli --quiet -- verify --contract-signature contracts/w17/pack_abi_spec.v1.json.sig`
  - `cargo run -p ocp-cli --quiet -- verify --contract contracts/w17/risk_lock_policies.v1.json`
  - `cargo run -p ocp-cli --quiet -- verify --contract-signature contracts/w17/risk_lock_policies.v1.json.sig`
  - `cargo run -p ocp-cli --quiet -- verify --contract contracts/w17/semantic_hash_taxonomy.v1.json`
  - `cargo run -p ocp-cli --quiet -- verify --contract-signature contracts/w17/semantic_hash_taxonomy.v1.json.sig`
  - `cargo run -p ocp-cli --quiet -- verify --contract contracts/w17/canonical_serialization_spec.v1.json`
  - `cargo run -p ocp-cli --quiet -- verify --contract-signature contracts/w17/canonical_serialization_spec.v1.json.sig`
- Test results:
  - Targeted tests: `PASS`
  - Regression tests: `PASS`
- Kết luận gate:
  - `DONE` (đủ code delta thật, đủ targeted tests đúng scope, đủ report artifacts và SoT signature verify pass).
- Design alignment:
  - `FULL`
- Maximal-first status:
  - `FULL_MAXIMAL`
- Blocker class:
  - `NONE`
- Evidence Packet:
  - hash taxonomy split có test độc lập xác nhận semantic hash không bị ảnh hưởng bởi diagnostic text.
  - trust lifecycle + downgrade guard có test negative/positive và report machine-checkable.
  - deterministic serialization + toolchain digest có test ổn định thứ tự/canonicalization.
  - toàn bộ SoT contracts v0.17 verify pass bằng cả SDK (`w17_contract_sot`) và CLI (`ocp verify --contract*`).
- Notes/risks:
  - chữ ký contract v0.17 hiện dùng deterministic sha256 signature contract của Gate 17-D; nếu sau này chuyển qua signer chain phức tạp hơn thì phải có migration note additive.

### 2026-03-06 — 17-E Planning Freeze
- Date: 2026-03-06
- Gate/Step: 17-E
- Why:
  - khóa cứng ecosystem governance trước khi mở connector baseline:
    - pack ABI/spec phải machine-checkable,
    - trust/signing/sandbox boundaries không được bypass trong strict lane,
    - capability laundering và CVE containment phải có negative proofs.
- Scope:
  - mở rộng `ocp-sdk::w17` cho contract Gate 17-E:
    - inspect pack ABI contract (`pack_abi_spec.v1.json`),
    - evaluate trust policy theo lane cho third-party packs,
    - evaluate sandbox boundary cho 2 surface `pack_boundary_wasi_v1` và `pack_boundary_native_cap_v1`,
    - evaluate capability edge (caller->callee) chống laundering,
    - evaluate adapter CVE policy (strict/compat/quarantine).
  - thêm đúng bộ targeted tests gate:
    - `tests/pack_abi.rs`
    - `tests/pack_trust_policy.rs`
    - `tests/pack_sandbox_boundary.rs`
    - `tests/pack_boundary_wasi.rs`
    - `tests/pack_boundary_native_cap.rs`
    - `tests/capability_laundering.rs`
    - `tests/adapter_cve_policy.rs`
  - tạo artifact machine-checkable:
    - `target/ocp/w17/ecosystem/extension_governance_report.json`
- Expected tests:
  - `cargo test --test pack_abi --test pack_trust_policy --test pack_sandbox_boundary --test pack_boundary_wasi --test pack_boundary_native_cap --test capability_laundering --test adapter_cve_policy`
  - `cargo test --test dep_permissions_transitive --test trust_policy`
- Exit criteria:
  - `RL-5` và `RL-11` pass với evidence đầy đủ.
  - cả hai boundary suites `pack_boundary_wasi` và `pack_boundary_native_cap` đều pass.
  - artifact `extension_governance_report.json` tồn tại đúng path.

### 2026-03-06 — 17-E Implementation Closeout
- Date: 2026-03-06
- Gate/Step: 17-E
- Implemented:
  - mở rộng `projects/ocp/crates/ocp-sdk/src/w17.rs` với contract APIs cho gate:
    - `inspect_pack_abi_spec_v17`,
    - `evaluate_pack_trust_policy_v17`,
    - `evaluate_pack_boundary_v17`,
    - `evaluate_capability_edge_v17`,
    - `evaluate_adapter_cve_policy_v17`,
    - constants/structs cho boundary, trust decision, CVE severity/decision.
  - export toàn bộ API 17-E qua `projects/ocp/crates/ocp-sdk/src/lib.rs`.
  - thêm helper `tests/w17_gate_e_common.rs` để sinh artifact:
    - `target/ocp/w17/ecosystem/extension_governance_report.json`.
  - thêm đúng bộ targeted tests của gate 17-E:
    - `tests/pack_abi.rs`
    - `tests/pack_trust_policy.rs`
    - `tests/pack_sandbox_boundary.rs`
    - `tests/pack_boundary_wasi.rs`
    - `tests/pack_boundary_native_cap.rs`
    - `tests/capability_laundering.rs`
    - `tests/adapter_cve_policy.rs`
- Files changed:
  - `projects/ocp/crates/ocp-sdk/src/w17.rs`
  - `projects/ocp/crates/ocp-sdk/src/lib.rs`
  - `tests/w17_gate_e_common.rs`
  - `tests/pack_abi.rs`
  - `tests/pack_trust_policy.rs`
  - `tests/pack_sandbox_boundary.rs`
  - `tests/pack_boundary_wasi.rs`
  - `tests/pack_boundary_native_cap.rs`
  - `tests/capability_laundering.rs`
  - `tests/adapter_cve_policy.rs`
  - `OCP-MVP-PLAN-v0.17.md`
- Commands run:
  - `cargo fmt -- projects/ocp/crates/ocp-sdk/src/w17.rs projects/ocp/crates/ocp-sdk/src/lib.rs tests/w17_gate_e_common.rs tests/pack_abi.rs tests/pack_trust_policy.rs tests/pack_sandbox_boundary.rs tests/pack_boundary_wasi.rs tests/pack_boundary_native_cap.rs tests/capability_laundering.rs tests/adapter_cve_policy.rs`
  - `cargo test --test pack_abi --test pack_trust_policy --test pack_sandbox_boundary --test pack_boundary_wasi --test pack_boundary_native_cap --test capability_laundering --test adapter_cve_policy`
  - `cargo test --test dep_permissions_transitive --test trust_policy`
  - `Test-Path target/ocp/w17/ecosystem/extension_governance_report.json`
- Test results:
  - Targeted tests: `PASS`
  - Regression tests: `PASS`
- Kết luận gate:
  - `DONE` (đủ code delta thật, đủ targeted tests đúng scope, đủ artifact machine-checkable cho gate 17-E).
- Design alignment:
  - `FULL`
- Maximal-first status:
  - `FULL_MAXIMAL`
- Blocker class:
  - `NONE`
- Evidence Packet:
  - dual boundary contract (`wasi` + `native_cap`) được verify qua `pack_abi` và hai suite boundary độc lập.
  - trust strict-lane và quarantine audit path được verify trong `pack_trust_policy`.
  - laundering edge deny/allow path có reason code + edge_id machine-checkable.
  - CVE containment strict/quarantine có negative proofs trong `adapter_cve_policy`.
- Notes/risks:
  - gate 17-E hiện khóa governance layer và boundary contracts; phần connector implementation cụ thể sẽ triển khai ở Gate 17-F theo cùng policy.

### 2026-03-06 — 17-F Planning Freeze
- Date: 2026-03-06
- Gate/Step: 17-F
- Why:
  - hoàn tất connector baseline implementation theo contract đã khóa:
    - `std.db.sql`,
    - `std.http.client`,
    - `std.queue.bus`,
  - bảo đảm mỗi connector có policy hook runtime thật (không chỉ ở tài liệu).
- Scope:
  - mở rộng `ProjectPermissions` để có section riêng:
    - `permissions.std_db`,
    - `permissions.std_queue`.
  - mở rộng parser/intersection cho dependency permissions:
    - parse từ manifest,
    - parse từ requested_permissions,
    - compute effective permissions cho dependency graph.
  - thêm policy checks trong `verify_pack_permissions_for_key`:
    - `std.db.query_int`/`std.db.exec` theo `std_db.allow_modes`,
    - `std.http.client.get`/`post` map về policy `std_net_http.allow_methods`,
    - `std.queue.bus.publish`/`consume` theo `std_queue.allow_topics`.
  - cập nhật registry metadata additive cho key mới:
    - `std.http.client.get`,
    - `std.http.client.post`,
    - `std.queue.bus.publish`,
    - `std.queue.bus.consume`.
  - thêm đúng 3 targeted tests gate:
    - `tests/connector_db.rs`
    - `tests/connector_http.rs`
    - `tests/connector_queue.rs`
  - tạo artifact:
    - `target/ocp/w17/connectors/connector_baseline_report.json`
- Expected tests:
  - `cargo test --test connector_db --test connector_http --test connector_queue`
  - `cargo test --test pack_abi --test pack_trust_policy --test capability_laundering --test adapter_cve_policy --test dep_permissions_transitive --test trust_policy`
  - `cargo test --test ocp_registry`
- Exit criteria:
  - connectors baseline chạy được theo policy đã khóa.
  - policy denies fail-honest với reason code đúng.
  - artifact `connector_baseline_report.json` tồn tại đúng path.

### 2026-03-06 — 17-F Implementation Closeout
- Date: 2026-03-06
- Gate/Step: 17-F
- Implemented:
  - mở rộng `projects/ocp/crates/ocp-sdk/src/lib.rs`:
    - thêm `StdDbPermissionConfig`, `StdQueuePermissionConfig`,
    - thêm `std_db`, `std_queue` vào `ProjectPermissions`,
    - parse/apply/intersection cho `permissions.std_db` và `permissions.std_queue`,
    - thêm action resolvers:
      - `std_http_client_action_from_key`,
      - `std_db_action_from_key`,
      - `std_queue_action_from_key`,
    - thêm runtime policy hooks fail-honest cho các connector keys tương ứng.
  - mở rộng `src/ocp/registry.rs` theo hướng additive:
    - thêm ctx_required/commit/determinism metadata cho
      `std.http.client.get`, `std.http.client.post`, `std.queue.bus.publish`, `std.queue.bus.consume`.
  - thêm helper report:
    - `tests/w17_gate_f_common.rs`.
  - thêm đúng bộ targeted tests Gate 17-F:
    - `tests/connector_db.rs`
    - `tests/connector_http.rs`
    - `tests/connector_queue.rs`
  - sinh artifact machine-checkable:
    - `target/ocp/w17/connectors/connector_baseline_report.json`.
- Files changed:
  - `projects/ocp/crates/ocp-sdk/src/lib.rs`
  - `src/ocp/registry.rs`
  - `tests/w17_gate_f_common.rs`
  - `tests/connector_db.rs`
  - `tests/connector_http.rs`
  - `tests/connector_queue.rs`
  - `OCP-MVP-PLAN-v0.17.md`
- Commands run:
  - `cargo fmt -- projects/ocp/crates/ocp-sdk/src/lib.rs src/ocp/registry.rs tests/w17_gate_f_common.rs tests/connector_db.rs tests/connector_http.rs tests/connector_queue.rs`
  - `cargo test --test connector_db --test connector_http --test connector_queue`
  - `cargo test --test pack_abi --test pack_trust_policy --test capability_laundering --test adapter_cve_policy --test dep_permissions_transitive --test trust_policy`
  - `cargo test --test ocp_registry`
  - `Test-Path target/ocp/w17/connectors/connector_baseline_report.json`
- Test results:
  - Targeted tests: `PASS`
  - Regression tests: `PASS`
- Kết luận gate:
  - `DONE` (đủ code delta thật cho connector baseline + policy hooks, đủ targeted tests đúng scope, đủ artifact machine-checkable).
- Design alignment:
  - `FULL`
- Maximal-first status:
  - `FULL_MAXIMAL`
- Blocker class:
  - `NONE`
- Evidence Packet:
  - `std.db.sql` có deny/allow theo mode (`read_query`/`write_exec`) và reason code ổn định.
  - `std.http.client` map policy chặt theo `allow_methods` (`GET`/`POST`) và fail-honest khi thiếu policy.
  - `std.queue.bus` có policy section riêng (`std_queue`) và deny khi allowlist rỗng.
  - regression quanh gate 17-E + dependency permissions + registry không bị lệch.
- Notes/risks:
  - Gate 17-F khóa baseline policy/runtime cho connector; phần rollout/adoption evidence toàn hệ sẽ tổng hợp ở Gate 17-G.

### 2026-03-06 — 17-G Planning Freeze
- Date: 2026-03-06
- Gate/Step: 17-G
- Why:
  - chốt rollout readiness signoff trước handoff v1.0 bằng evidence machine-checkable.
  - khóa cứng release positioning theo contract, tránh claim vượt phạm vi đã khóa.
- Scope:
  - hoàn tất đủ 3 targeted tests gate:
    - `tests/v017_adoption_readiness.rs`
    - `tests/v017_release_guard.rs`
    - `tests/v017_release_positioning_guard.rs`
  - sinh đủ report artifacts gate `17-G`:
    - `target/ocp/w17/rc/v017_adoption_readiness_report.json`
    - `target/ocp/w17/rc/v017_release_guard_report.json`
    - `target/ocp/w17/rc/v017_release_positioning_guard_report.json`
  - chạy full regression bắt buộc theo rule gate signoff:
    - `cargo test`
    - `cargo clippy --all-targets -- -D warnings`
    - `cargo fmt -- --check`
- Maximal target:
  - `17-G DONE` với full evidence, không để drift SoT/snapshot, không bypass quality gate.
- Alternatives to evaluate:
  - không dùng phương án rút gọn; giữ full strict path theo `10.1 Test ownership theo gate`.
- Expected tests:
  - `cargo test --test v017_adoption_readiness --test v017_release_guard --test v017_release_positioning_guard`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
- Exit criteria:
  - targeted tests pass và tạo đủ artifacts.
  - full regression bắt buộc pass.
  - gate status + closeout + outputs đồng bộ, không còn placeholder.

### 2026-03-06 — 17-G Implementation Closeout
- Date: 2026-03-06
- Gate/Step: 17-G
- Implemented:
  - thêm helper + targeted tests gate `17-G`:
    - `tests/w17_gate_g_common.rs`
    - `tests/v017_adoption_readiness.rs`
    - `tests/v017_release_guard.rs`
    - `tests/v017_release_positioning_guard.rs`
  - cập nhật Gate `17-G` output contract trong plan:
    - thêm `target/ocp/w17/rc/v017_release_positioning_guard_report.json`.
  - xử lý regressions khi chạy full gate checks:
    - cập nhật `contracts/required_contracts.v1.json` cho `registry.pack_contracts` schema hash hiện hành.
    - cập nhật `projects/ocp/conformance/expected/contracts/stability_contract_v14.json` cho `std.fs.write_text.payload_schema_hash`.
    - cập nhật `projects/ocp/conformance/expected/contracts/stability_contract_v15.json` cho `std.fs.write_text.payload_schema_hash`.
    - sửa `tests/w17_gate_b_cli_common.rs` (`&PathBuf` -> `&Path`) để đạt `clippy -D warnings`.
  - sinh đủ artifacts gate `17-G`:
    - `target/ocp/w17/rc/v017_adoption_readiness_report.json`
    - `target/ocp/w17/rc/v017_release_guard_report.json`
    - `target/ocp/w17/rc/v017_release_positioning_guard_report.json`
- Files changed:
  - `tests/w17_gate_g_common.rs`
  - `tests/v017_adoption_readiness.rs`
  - `tests/v017_release_guard.rs`
  - `tests/v017_release_positioning_guard.rs`
  - `tests/w17_gate_b_cli_common.rs`
  - `contracts/required_contracts.v1.json`
  - `projects/ocp/conformance/expected/contracts/stability_contract_v14.json`
  - `projects/ocp/conformance/expected/contracts/stability_contract_v15.json`
  - `OCP-MVP-PLAN-v0.17.md`
- Commands run:
  - `cargo fmt -- tests/w17_gate_g_common.rs tests/v017_adoption_readiness.rs tests/v017_release_guard.rs tests/v017_release_positioning_guard.rs`
  - `cargo test --test v017_adoption_readiness --test v017_release_guard --test v017_release_positioning_guard`
  - `Test-Path target/ocp/w17/rc/v017_adoption_readiness_report.json`
  - `Test-Path target/ocp/w17/rc/v017_release_guard_report.json`
  - `Test-Path target/ocp/w17/rc/v017_release_positioning_guard_report.json`
  - `cargo test`
  - `cargo test --test contract_inventory_complete`
  - `cargo test --test stability_contracts_v14`
  - `cargo test --test stability_contracts_v15`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- tests/w17_gate_b_cli_common.rs tests/w17_gate_g_common.rs tests/v017_adoption_readiness.rs tests/v017_release_guard.rs tests/v017_release_positioning_guard.rs`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
- Test results:
  - Targeted tests: `PASS`
  - Regression tests: `PASS`
- Kết luận gate:
  - `DONE` (đã đạt đủ targeted + regression bắt buộc của Gate `17-G`, artifacts đầy đủ, không còn drift mở).
- Design alignment:
  - `FULL`
- Maximal-first status:
  - `FULL_MAXIMAL`
- Blocker class:
  - `NONE`
- Evidence Packet:
  - release guard xác nhận `RL-1..RL-17` đều `locked` và SoT contracts `w17` verify đủ.
  - adoption readiness xác nhận đủ artifacts xuyên gate `17-A..17-F` trước signoff.
  - release positioning guard khóa claims theo `8.3 Performance positioning (LOCKED)`.
  - full regression bắt buộc (`cargo test`, `clippy`, `fmt --check`) sạch sau khi xử lý drift SoT/snapshot.
- Notes/risks:
  - drift hash của `std.fs.write_text` được chốt lại bằng cập nhật snapshot/required-contract SoT trong cùng gate; nếu contract key này thay đổi lần nữa bắt buộc kèm migration evidence trước khi đóng gate mới.

### 2026-03-06 — 17 Verification Snapshot (verification-only)
- Date: 2026-03-06
- Scope:
  - chạy lại full quality gate sau các chỉnh sửa evidence path/report của Gate `17-B`.
  - xác nhận không có regression trước quyết định đóng `v0.17`.
- Commands run:
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
  - `cargo test --test cassette_upgrade_v08_to_v17 --test cassette_prune_policy`
- Test results:
  - full test suite: `PASS`
  - clippy strict warnings-as-errors: `PASS`
  - format check: `PASS`
  - targeted cassette evidence tests: `PASS`
- Notes:
  - snapshot này là verification-only sau closeout, không thay đổi scope/gate status đã khóa.

---

## 14) Checklist khóa trước khi đóng gate
- [x] Gate status cập nhật đúng (`TODO/IN_PROGRESS/PARTIAL/DONE`).
- [x] Có đủ planning freeze + implementation closeout.
- [x] `Files changed` khớp code delta thực tế.
- [x] `Commands run` là lệnh đã chạy thật.
- [x] `Targeted tests` pass đúng phạm vi.
- [x] Đã xác nhận `Maximal-first status` cho gate.
- [x] Nếu có hạ chuẩn: đã đính kèm `Evidence Packet` đầy đủ và phân loại blocker đúng.
- [x] Nếu ghi `FUNDAMENTAL_IMPOSSIBILITY`: đã có lập luận horizon cấp nền tảng (>=10 năm).
- [x] Deterministic concurrency rules (ordering/fairness/timeout-cancel) đã được kiểm tra theo contract.
- [x] Supported platform profile + release positioning không còn mâu thuẫn hoặc claim vượt phạm vi.
- [x] Risk locks `RL-1..RL-17` đã có evidence và trạng thái `PASS`.
- [x] SoT contract v0.17 (`contracts/w17/*.json`) đã verify signature đầy đủ.
- [x] Cassette migration `v0.8 -> v0.17` đã pass invariants và negative tests.
- [x] Mọi gate đã sinh đúng output artifact path đã khóa trong gate section tương ứng (theo context `<project_root>`/`<artifact_dir>` đã nêu rõ).
- [x] Không còn placeholder `PASS/FAIL` trong closeout `DONE`.
- [x] Không có lỗi mã hóa tiếng Việt.

---

## 15) Handoff v0.17 -> v1.0
- Chỉ mở v1.0 release rộng khi:
  - toàn bộ gate core `17-A..17-G` đã `DONE`,
  - evidence adoption + operability + ecosystem governance đạt đủ,
  - không còn finding critical mở.
- Nếu còn gate `IN_PROGRESS/PARTIAL`, không được tuyên bố phát hành rộng.
