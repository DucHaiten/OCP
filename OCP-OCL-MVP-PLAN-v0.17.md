# OCL v0.17 — Adoption Hardening + Cassette Operability + Governed Ecosystem (Pre-v1.0 Design Freeze)

Ngày tạo: 2026-03-06  
Trạng thái: `IN_PROGRESS (17-A..17-B DONE, đang chuẩn bị 17-C)`  
Phạm vi: **OCL-only**.  
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
  - khóa backlog productization trước v1.0, chưa code vội.
- Trạng thái tổng quan:
  - `IN_PROGRESS (17-A..17-B DONE, đang chuẩn bị 17-C)`.
- Gate đang làm/đã xong/chưa làm:
  - `17-A..17-B DONE`, `17-C..17-G TODO`.
- Bước kế tiếp ngay:
  - mở Planning Freeze cho `17-C` (determinism core: concurrency/platform/order/entropy).
- Lệnh kiểm chứng chuẩn:
  - xem `12) Operational commands (v0.17)`.
- File code trọng yếu đã thay đổi ở 17-A..17-B:
  - `projects/ocp-ocl/crates/ocl-cli/src/main.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/src/perm_v15.rs`
  - `src/ocp_ocl/exec.rs`
  - `src/ocp_ocl/registry.rs`
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
  - `OCP-OCL-MVP-PLAN-v0.17.md`

### 0.3 Trạng thái Workstreams/Gates v0.17 (tracking)
#### Workstreams
- WS-DX (governed workflow UX): `DONE`
- WS-CS (cassette storage/retention operability): `DONE`
- WS-DT (determinism core: concurrency/platform/order/entropy): `TODO`
- WS-TR (trust lifecycle + hash/serialization/repro): `TODO`
- WS-EX (governed extension surface + capability governance): `TODO`
- WS-CN (connector baseline): `TODO`
- WS-RC (pre-v1.0 rollout readiness): `TODO`

#### Gate status (17-A .. 17-G)
- Gate 17-A (DX guided compliance flow): `DONE`
- Gate 17-B (Cassette/storage + TOCTOU + privacy/DoS/boundary): `DONE`
- Gate 17-C (Determinism core: concurrency/platform/order/entropy): `TODO`
- Gate 17-D (Trust lifecycle + downgrade + hash/serialization/repro): `TODO`
- Gate 17-E (Ecosystem governance + capability laundering + CVE): `TODO`
- Gate 17-F (Connector baseline implementation): `TODO`
- Gate 17-G (Rollout readiness + final signoff): `TODO`

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
- OCL sẵn sàng phát hành rộng với trải nghiệm tuân thủ mượt hơn.
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
  - `target/ocl/w17/dx/dx_friction_report.json`
  - `target/ocl/w17/dx/permission_fix_safety_report.json`
- Baseline ref:
  - `target/ocl/baseline/v016/dx_friction_report.json`
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
  - `target/ocl/w17/cassette/cassette_operability_report.json`
  - `target/ocl/w17/cassette/cassette_migration_report.json`
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
  - `target/ocl/w17/ecosystem/extension_governance_report.json`
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
  - `target/ocl/w17/compat/v016_line_compat_report.json`
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
  - crate: `ocl-cli`
  - path: `projects/ocp-ocl/crates/ocl-cli/src/main.rs`
- Policy schema, manifest/permission/approval structures:
  - crate: `ocl-sdk`
  - path: `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
- Cassette storage engine, chunk/dedup/quota/replay guards:
  - crate: `ocl-runtime-core`
  - path: `projects/ocp-ocl/crates/ocl-runtime-core/src/*`

### 3.4 Deterministic execution profile (Gate 17-C, LOCKED)
- `DEP-1: Single-thread deterministic core`:
  - mặc định v1.0 runtime không dựa shared-memory multithreading cho state OCL.
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
  - `ocl verify --contract contracts/w17/<name>.json`
  - `ocl verify --contract-signature contracts/w17/<name>.json.sig`
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
- `ocl perm doctor`
- `ocl perm diff <old> <new>`
- `ocl perm fix --plan`
- `ocl perm fix --apply` (chỉ tạo patch + approval record, không bypass policy)

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
- `ocl cassette stats`
- `ocl cassette prune --plan`
- `ocl cassette prune --apply`
- `ocl cassette gc`
- `ocl cassette upgrade --plan`
- `ocl cassette upgrade --apply`

### 5.5 Migration contract v0.8 -> v0.17 (LOCKED)
- Migration commands:
  - `ocl cassette upgrade --plan`
  - `ocl cassette upgrade --apply`
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
  - `ocl perm approve --all` chỉ hợp lệ khi có:
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
  - `ocl budget analyze`
  - `ocl budget doctor`

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
- `target/ocl/w17/dx/dx_friction_report.json`
- `target/ocl/w17/dx/permission_fix_safety_report.json`
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
- `target/ocl/w17/cassette/cassette_operability_report.json`
- `target/ocl/w17/cassette/cassette_migration_report.json`
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
- `target/ocl/w17/determinism/determinism_core_report.json`
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
- `target/ocl/w17/contracts/w17_contract_sot_report.json`
- `target/ocl/w17/contracts/trust_lifecycle_report.json`
- `target/ocl/w17/compat/v016_line_compat_report.json`
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
- `target/ocl/w17/ecosystem/extension_governance_report.json`
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
- `target/ocl/w17/connectors/connector_baseline_report.json`
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
- `target/ocl/w17/rc/v017_adoption_readiness_report.json`
- `target/ocl/w17/rc/v017_release_guard_report.json`
Exit criteria:
- đủ bằng chứng để quyết định mở v1.0 release scope.
- không còn claim mơ hồ vượt quá performance positioning đã khóa.
- không còn risk lock `RL-1..RL-17` ở trạng thái mở.

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
- `ocl cassette upgrade --plan`
- `ocl cassette upgrade --apply`
- `ocl verify --contract contracts/w17/supported_platform_profile.v1.json`
- `ocl verify --contract-signature contracts/w17/supported_platform_profile.v1.json.sig`
- `ocl verify --contract contracts/w17/cassette_storage_contract.v1.json`
- `ocl verify --contract-signature contracts/w17/cassette_storage_contract.v1.json.sig`
- `ocl verify --contract contracts/w17/pack_abi_spec.v1.json`
- `ocl verify --contract-signature contracts/w17/pack_abi_spec.v1.json.sig`
- `ocl verify --contract contracts/w17/risk_lock_policies.v1.json`
- `ocl verify --contract-signature contracts/w17/risk_lock_policies.v1.json.sig`
- `ocl verify --contract contracts/w17/semantic_hash_taxonomy.v1.json`
- `ocl verify --contract-signature contracts/w17/semantic_hash_taxonomy.v1.json.sig`
- `ocl verify --contract contracts/w17/canonical_serialization_spec.v1.json`
- `ocl verify --contract-signature contracts/w17/canonical_serialization_spec.v1.json.sig`
- `ocl budget analyze`
- `ocl budget doctor`

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
  - Thêm `ocl perm doctor`.
  - Thêm `ocl perm fix --plan` và `ocl perm fix --apply` (advisory patch + approval record, không auto sửa manifest).
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
    - thêm summary/options types cho Gate 17-A (`PermissionDoctor*`, `PermissionFix*`) trong `ocl-sdk`.
    - thêm `write_permission_doctor_report_v17`, `write_permission_fix_plan_v17`, `apply_permission_fix_plan_v17`.
    - thêm wildcard risk detection theo lane, report canonical JSON, guard strict-lane, virtual approval record cho fix apply.
  - CLI:
    - thêm `ocl perm doctor <project_dir> [--out ...]`.
    - thêm `ocl perm fix --plan <project_dir> [--out ...]`.
    - thêm `ocl perm fix --apply <project_dir> ... --ack-risk --justification --by --date`.
    - cập nhật help text và parser flag presence (`has_flag`).
  - Tests:
    - thêm targeted tests: `perm_doctor`, `perm_fix_apply`, `perm_fix_negative`, `perm_rubberstamp_guard`, `error_taxonomy_contract`.
    - thêm `tests/w17_perm_common.rs` làm helper dùng chung.
- Files changed:
  - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/src/perm_v15.rs`
  - `projects/ocp-ocl/crates/ocl-cli/src/main.rs`
  - `tests/w17_perm_common.rs`
  - `tests/perm_doctor.rs`
  - `tests/perm_fix_apply.rs`
  - `tests/perm_fix_negative.rs`
  - `tests/perm_rubberstamp_guard.rs`
  - `tests/error_taxonomy_contract.rs`
  - `OCP-OCL-MVP-PLAN-v0.17.md`
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
  - `perm fix --apply` cố ý chỉ tạo patch/report/approval record, không tự ghi đè `Ocl.toml`, để giữ fail-honest và không bypass policy strict.

### 2026-03-06 — 17-B Planning Freeze
- Date: 2026-03-06
- Gate/Step: 17-B
- Why:
  - khóa cứng vận hành cassette dữ liệu lớn + budget diagnostics + TOCTOU/boundary/privacy để tránh drift trước 17-C.
- Scope:
  - thêm command surface `ocl cassette {stats|prune|gc|upgrade}`.
  - thêm command surface `ocl budget {analyze|doctor}` với output machine-checkable.
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
  - hoàn thiện parser + handler cho `ocl cassette ...` và `ocl budget ...`.
  - thêm pipeline upgrade cassette v0.8 -> v0.17 (block store/index/meta, integrity verify, prune/gc).
  - thêm guard privacy leak cho cassette stats (`X-CASSETTE-PRIVACY-LEAK`).
  - thêm budget analyze/doctor report theo edge `(caller_package, key)` và budget path machine-readable.
  - thêm runtime precondition check cho `std.fs.write_text` trước commit.
  - cập nhật schema payload `std.fs.write_text` (optional `precondition_exists`, `precondition_len`) để không vỡ validator.
- Files changed:
  - `projects/ocp-ocl/crates/ocl-cli/src/main.rs`
  - `src/ocp_ocl/exec.rs`
  - `src/ocp_ocl/registry.rs`
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
  - `OCP-OCL-MVP-PLAN-v0.17.md`
- Commands run:
  - `cargo check -p ocl-cli`
  - `cargo test --test cassette_chunking --test cassette_prune_policy --test cassette_gc_negative --test budget_diamond --test budget_analyze --test commit_precondition --test cassette_upgrade_v08_to_v17 --test cassette_upgrade_tamper_negative --test privacy_artifact_hygiene --test dos_budget_caps --test fs_boundary_security`
  - `cargo test --test observe_cache observe_cache_fs_generation_invalidation_prevents_stale_read`
- Test results:
  - Targeted tests: `PASS`
  - Regression tests: `PASS`
- Kết luận gate:
  - `DONE` (đủ code delta thật + đủ targeted tests + regression phụ trợ pass).
- Design alignment:
  - `FULL`
- Maximal-first status:
  - `FULL_MAXIMAL`
- Blocker class:
  - `NONE`
- Evidence Packet:
  - cassette upgrade/prune/gc/tamper/privacy/doS đều có test độc lập.
  - budget edge diagnostics có report JSON machine-readable.
  - TOCTOU precondition có test commit stale và regression observe-cache.
- Notes/risks:
  - prune/gc hiện deterministic theo orphan block/index integrity; policy TTL đã có tham số command nhưng chưa mở rộng scheduler maintenance tự động ở gate này.

### YYYY-MM-DD — 17-C Planning Freeze
- Date:
- Gate/Step: 17-C
- Why:
- Scope:
- Expected tests:
- Exit criteria:

### YYYY-MM-DD — 17-C Implementation Closeout
- Date:
- Gate/Step: 17-C
- Implemented:
- Files changed:
- Commands run:
- Test results:
  - Targeted tests: `PASS`/`FAIL`
  - Regression tests: `PASS`/`FAIL`
- Kết luận gate:
- Design alignment:
- Notes/risks:

### YYYY-MM-DD — 17-D Planning Freeze
- Date:
- Gate/Step: 17-D
- Why:
- Scope:
- Expected tests:
- Exit criteria:

### YYYY-MM-DD — 17-D Implementation Closeout
- Date:
- Gate/Step: 17-D
- Implemented:
- Files changed:
- Commands run:
- Test results:
  - Targeted tests: `PASS`/`FAIL`
  - Regression tests: `PASS`/`FAIL`
- Kết luận gate:
- Design alignment:
- Notes/risks:

### YYYY-MM-DD — 17-E Planning Freeze
- Date:
- Gate/Step: 17-E
- Why:
- Scope:
- Expected tests:
- Exit criteria:

### YYYY-MM-DD — 17-E Implementation Closeout
- Date:
- Gate/Step: 17-E
- Implemented:
- Files changed:
- Commands run:
- Test results:
  - Targeted tests: `PASS`/`FAIL`
  - Regression tests: `PASS`/`FAIL`
- Kết luận gate:
- Design alignment:
- Notes/risks:

### YYYY-MM-DD — 17-F Planning Freeze
- Date:
- Gate/Step: 17-F
- Why:
- Scope:
- Expected tests:
- Exit criteria:

### YYYY-MM-DD — 17-F Implementation Closeout
- Date:
- Gate/Step: 17-F
- Implemented:
- Files changed:
- Commands run:
- Test results:
  - Targeted tests: `PASS`/`FAIL`
  - Regression tests: `PASS`/`FAIL`
- Kết luận gate:
- Design alignment:
- Notes/risks:

### YYYY-MM-DD — 17-G Planning Freeze
- Date:
- Gate/Step: 17-G
- Why:
- Scope:
- Expected tests:
- Exit criteria:

### YYYY-MM-DD — 17-G Implementation Closeout
- Date:
- Gate/Step: 17-G
- Implemented:
- Files changed:
- Commands run:
- Test results:
  - Targeted tests: `PASS`/`FAIL`
  - Regression tests: `PASS`/`FAIL`
- Kết luận gate:
- Design alignment:
- Notes/risks:

---

## 14) Checklist khóa trước khi đóng gate
- [ ] Gate status cập nhật đúng (`TODO/IN_PROGRESS/PARTIAL/DONE`).
- [ ] Có đủ planning freeze + implementation closeout.
- [ ] `Files changed` khớp code delta thực tế.
- [ ] `Commands run` là lệnh đã chạy thật.
- [ ] `Targeted tests` pass đúng phạm vi.
- [ ] Đã xác nhận `Maximal-first status` cho gate.
- [ ] Nếu có hạ chuẩn: đã đính kèm `Evidence Packet` đầy đủ và phân loại blocker đúng.
- [ ] Nếu ghi `FUNDAMENTAL_IMPOSSIBILITY`: đã có lập luận horizon cấp nền tảng (>=10 năm).
- [ ] Deterministic concurrency rules (ordering/fairness/timeout-cancel) đã được kiểm tra theo contract.
- [ ] Supported platform profile + release positioning không còn mâu thuẫn hoặc claim vượt phạm vi.
- [ ] Risk locks `RL-1..RL-17` đã có evidence và trạng thái `PASS`.
- [ ] SoT contract v0.17 (`contracts/w17/*.json`) đã verify signature đầy đủ.
- [ ] Cassette migration `v0.8 -> v0.17` đã pass invariants và negative tests.
- [ ] Mọi gate đã sinh đúng output artifact path đã khóa trong gate section tương ứng.
- [ ] Không còn placeholder `PASS/FAIL` trong closeout `DONE`.
- [ ] Không có lỗi mã hóa tiếng Việt.

---

## 15) Handoff v0.17 -> v1.0
- Chỉ mở v1.0 release rộng khi:
  - toàn bộ gate core `17-A..17-G` đã `DONE`,
  - evidence adoption + operability + ecosystem governance đạt đủ,
  - không còn finding critical mở.
- Nếu còn gate `IN_PROGRESS/PARTIAL`, không được tuyên bố phát hành rộng.
