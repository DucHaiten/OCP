# OCL v0.15 — Supply-chain Hardening + LTS Rehearsal (Security-by-Design Distribution)

Ngày tạo: 2026-03-03  
Trạng thái: `DONE (15-A..15-F DONE)`  
Phạm vi: **OCL-only**.  
Tiền đề: v0.10 đã có packaging/lock/signing + permission manifests; v0.14 đã có conformance + stability sprint; v0.8 có quarantine+cassette; v0.11 có debugger/diff/minimizer.

Mục tiêu v0.15: đưa OCL tới mức **production readiness** ở góc độ supply-chain + governance:
- ký & xác thực chain-of-trust cho packages/engines/packs,
- policy review và attestation cho permissions,
- reproducible builds/attested artifacts,
- LTS rehearsal: thử đóng băng hợp đồng và vận hành upgrade trong thời gian dài mà không breaking.

---

## 0) Governance + Tracking v0.15

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
  - hardening supply-chain + attestation + LTS rehearsal, không mở semantics mới.
- Trạng thái tổng quan:
  - `DONE (15-A..15-F DONE)`.
- Gate đang làm/đã xong/chưa làm:
  - `15-A`, `15-B`, `15-C`, `15-D`, `15-E` đã xong; `15-F` đã xong (stretch).
- Bước kế tiếp ngay:
  - rà cuối v0.15 theo Exit Contract, sau đó chuẩn bị handoff `v1.0`.
- Lệnh kiểm chứng chuẩn:
  - xem `12) Operational commands (v0.15)`.
- File code trọng yếu đã thay đổi:
  - `projects/ocp-ocl/crates/ocl-sdk/src/attestation_v15.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/src/perm_v15.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
  - `projects/ocp-ocl/crates/ocl-cli/src/main.rs`
  - `tests/attestation.rs`
  - `tests/perm_review.rs`
  - `tests/lts_check.rs`
  - `tests/cassette_sign.rs`

### 0.3 Trạng thái Workstreams/Gates v0.15 (tracking)
#### Workstreams
- WS-TR (trust policy + signed lock): `DONE`
- WS-PR (permission review + approval gates): `DONE`
- WS-AT (attestation + repro): `DONE`
- WS-LTS (lts check/report + rehearsal): `DONE`
- WS-QH (cassette sign + secret hardening stretch): `DONE`

#### Gate status (15-A .. 15-F)
- Gate 15-A (Trust policy engine): `DONE`
- Gate 15-B (Signed lockfile): `DONE`
- Gate 15-C (Permission snapshot + diff + approval): `DONE`
- Gate 15-D (Build attestation): `DONE`
- Gate 15-E (LTS tooling): `DONE`
- Gate 15-F (Cassette signing + secret hardening stretch): `DONE`

---

## 1) Goals v0.15 (LOCKED)

### 1.1 North Star
- OCL có thể dùng trong môi trường “security-conscious”:
  - dependency provenance rõ ràng
  - không silent escalation
  - audit trails đầy đủ
- Dev/user có thể tin:
  - code chạy đúng quyền được cấp
  - upgrades có review và bị chặn nếu vi phạm policy

### 1.2 KPI bắt buộc (định lượng)
**KPI-1: Trust chain enforced**
- Mọi dependency ngoài `builtin` trong lane `locked_v071` phải:
  - có content_hash pinned trong lockfile
  - có signature hợp lệ
  - signer nằm trong trust policy
- Nếu thiếu bất kỳ điều kiện nào => build/run fail (không warn).

**KPI-2: Permission review workflow**
- Canonical workflow:
  - `ocl perm snapshot`
  - `ocl perm diff <old> <new>`
  - `ocl perm approve <diff_report>`
- `ocl perm review` (nếu giữ) chỉ là alias tổng hợp của `snapshot + diff`.
- Permission diff report bắt buộc cho:
  - project manifest thay đổi
  - dependency requested_permissions thay đổi (do update)
- Report phải machine-readable + human-readable.

**KPI-3: Attested artifacts**
- `ocl build --attest` tạo:
  - build manifest (deps + hashes + permissions effective + lock hash)
  - attestation signature (local key)
- `ocl verify --attest` xác thực artifact.

**KPI-4: LTS rehearsal**
- Thực hiện “LTS rehearsal window” (ví dụ 90 ngày mô phỏng):
  - chỉ chấp nhận changes additive
  - mọi release phải pass conformance + supply-chain checks
  - upgrade-check không báo breaking cho templates tiêu chuẩn

---

## 2) Axis Lock v0.15 (kế thừa, không đổi)
- Least privilege remains default.
- Deny remains final.
- Audit/replay remain mandatory.
- v0.15 tập trung “hardening”, không mở semantics mới lớn.
- Lane literals canonical:
  - `locked_v071`, `locked_v06`, `quarantine`
  - từ `locked` chỉ là alias mô tả trong văn bản, không phải literal manifest.

---

## 3) Scope v0.15

### 3.1 In-scope (ship)
A) **Trust policy system**
- `trust.toml` cấu hình trusted keys + registries + policies per lane.
- Mandatory verification pipeline for lane `locked_v071`.

B) **Signed lockfile / lock attestation**
- Lockfile có thể được ký bởi project owner:
  - chống tampering trong CI
- `ocl lock sign` / `ocl lock verify`
- SoT lockfile giữ nguyên từ v0.10:
  - `deps.lock.v3` là canonical source-of-truth.
  - chữ ký canonical: `deps.lock.v3.sig`.
  - `ocl.lock` chỉ là export/read-only view, không phải gate quyết định.

C) **Permission review & gating**
- Generate permission reports:
  - per package requested permissions
  - effective permissions after intersection
  - diffs on update
- Canonical commands:
  - `ocl perm snapshot`
  - `ocl perm diff <old> <new>`
  - `ocl perm approve <diff_report>`
- Alias compatibility:
  - `ocl perm review` (nếu tồn tại) phải forward sang flow canonical, không tạo semantics riêng.
- Policy gates:
  - “no new permissions without explicit approval”
  - “caps cannot increase without approval”
- Approval file bắt buộc trong lane `locked_v071`:
  - `permissions.approval.toml` checked in repo.

D) **Reproducible build attestation**
- Build artifact includes:
  - deps list + hashes
  - permission effective manifest
  - lock hash
  - runtime version
  - conformance suite hash used
- Attestation signed with local project key.

E) **Registry hardening**
- Mirror pinning
- allowlist registries
- transparency log deferred sau v0.15

F) **LTS rehearsal process + tooling**
- Define LTS policy profile:
  - additive-only schema changes
  - strict conformance gate
  - strict supply-chain gate
- `ocl lts check` verifies project compatibility.

G) **CLI compatibility contract (LOCKED)**
- Canonical commands của v0.15:
  - `ocl lock sign|verify`
  - `ocl perm snapshot|diff|approve`
  - `ocl build --attest`
  - `ocl verify --attest|--repro`
  - `ocl lts check|report`
- Alias compatibility:
  - `ocl perm review` là alias additive.
- Không được phá command/flag cũ đã tồn tại trước v0.15.

### 3.2 Out-of-scope (defer)
- Full transparency log infrastructure as a service.
- Remote attestation across multiple orgs.
- Formal verification.

### 3.3 Core vs stretch (LOCKED)
- Core v0.15 bắt buộc đóng:
  - 15-A, 15-B, 15-C, 15-D, 15-E.
- Stretch v0.15:
  - 15-F (cassette signing + secret hardening nâng cao).
- Quy tắc DONE phiên bản:
  - v0.15 có thể `DONE` khi 15-A..15-E hoàn tất theo contract.
  - 15-F nếu chưa làm phải giữ `TODO`/`PARTIAL`, không được gộp vào core completion.

### 3.4 Migration contract from v0.10-v0.14 (LOCKED)
- Giữ nguyên các trục đã khóa:
  - lockfile SoT = `deps.lock.v3` (v0.10),
  - lane literals = `locked_v071|locked_v06|quarantine` (v0.8+),
  - conformance/stability gate bắt buộc trước release (v0.14).
- v0.15 chỉ additive hardening, không đổi SoT lockfile và không thêm lane literal mới.

### 3.5 Module -> Crate -> Path mapping (LOCKED)
- Trust/lock/permission policy parser + conformance runner wiring:
  - crate: `ocl-sdk`
  - path: `projects/ocp-ocl/crates/ocl-sdk/src/*`
- Runtime enforcement marker/audit fields cho trust decisions:
  - crate: `ocl-runtime-core`
  - path: `projects/ocp-ocl/crates/ocl-runtime-core/src/*`
- CLI commands:
  - `ocl lock sign|verify`
  - `ocl perm snapshot|diff|approve|review(alias)`
  - `ocl build --attest`, `ocl verify --attest|--repro`, `ocl lts check|report`
  - crate: `ocl-cli`
  - path: `projects/ocp-ocl/crates/ocl-cli/src/main.rs`

---

## 4) Trust Policy (v0.15)

### 4.1 trust.toml schema
Example:

```toml
[policy]
mode = "strict"  # strict|warn
lane_locked_v071_requires_signed = true
lane_locked_v06_requires_signed = false
lane_quarantine_requires_signed = false

[registries]
allow = ["registry:main", "git:github.com/myorg/*"]

[[trusted_key]]
id = "publisher:engine-team"
alg = "ed25519"
public_key = "base64:..."
scope = ["engine-ui-widgets", "engine-game-loop"]

[[trusted_key]]
id = "publisher:std-team"
alg = "ed25519"
public_key = "base64:..."
scope = ["std-packs", "std-net", "std-proc"]
```

Rules:
- Lane literals hợp lệ:
  - `locked_v071`
  - `locked_v06`
  - `quarantine`
- Precedence theo lane:
  - `locked_v071`: luôn strict; nếu cấu hình `mode="warn"` thì coi là invalid config.
  - `locked_v06`: cho phép `warn` để compatibility.
  - `quarantine`: cho phép `warn`, nhưng bắt buộc emit audit marker.
- lane `locked_v071` must verify:
  - content_hash matches lock
  - signature valid
  - signer trusted for package scope
- deny unknown registries in locked by default.

### 4.2 Trust decisions in audit
- run artifacts include `trust_report.json`:
  - per package: verified yes/no, signer id, registry source

### 4.3 Registry allowlist + scope matching (LOCKED)
- Normalize source trước khi match:
  - lowercase scheme + host
  - strip trailing `/`
  - chuẩn hóa path separator thành `/`
- Registry pattern:
  - dùng glob deterministic một chiều (không regex tự do).
  - `*` match trong segment; không match qua dấu `/`.
- Package scope matching:
  - chỉ hỗ trợ exact package name hoặc prefix list đã khai báo.
  - không dùng regex runtime cho scope policy.

---

## 5) Signed Lockfile (v0.15)

### 5.1 lock signing
- `deps.lock.v3` plus `deps.lock.v3.sig` (SoT)
- sign input = `sha256-v1(canonical JSON view của deps.lock.v3 AST)`:
  - không ký raw text bytes của TOML lockfile.
- `ocl lock sign --key <keyid> --lock deps.lock.v3`
- `ocl lock verify --lock deps.lock.v3`
- `ocl.lock` nếu có chỉ là export/read-only view:
  - không dùng làm verify gate
  - có thể ký phụ trợ, nhưng không thay thế SoT signature.

### 5.2 CI gating
- In lane `locked_v071`, if `require_signed_lock=true`:
  - unsigned `deps.lock.v3` signature => fail
  - signature mismatch => fail
- In lane `locked_v06`:
  - cho phép warn mode để compat (nhưng bắt buộc audit marker).

---

## 6) Permission Review Workflow (v0.15)

### 6.1 Permission snapshot artifacts
Generate:
- `permissions_requested.json`
- `permissions_granted.json`
- `permissions_effective.json`
- Baseline SoT file (checked-in):
  - `permissions.snapshot.json`
  - được generate từ lock + manifest hiện tại theo canonicalization policy.

### 6.2 Diff report
`ocl perm diff <old> <new>` outputs:
- new capabilities requested by any package
- removed capabilities
- cap increases (e.g., max_draw_cmds)
- new paths added (fs globs)
- quarantine-only permissions changes
- machine-check field:
  - `permission_diff_hash = sha256-v1(canonical_bytes(permission_diff_report.json))`
- Pipeline old/new binding (LOCKED):
  - `old` = `permissions.snapshot.json` (SoT trong repo)
  - `new` = snapshot regenerate từ workspace hiện tại (temporary artifact).

### 6.3 Approval gating
Use `permissions.approval.toml` checked into repo:
- list approved permission deltas (hashes)
- approval entry schema:
  - `{diff_hash, approved_by, date, note}`
- CI checks:
  - if diff introduces new perms and `diff_hash` chưa được approve => fail
- `ocl perm approve` generates approval entry.
- Enforcement hook (LOCKED):
  - trong lane `locked_v071`, `ocl build`, `ocl run`, và `ocl lts check` phải:
    1. regenerate `new` permission snapshot,
    2. diff với `permissions.snapshot.json`,
    3. nếu `permission_diff_hash` không có trong `permissions.approval.toml` => fail-honest.

This turns permission review into an explicit workflow, not tribal knowledge.

---

## 7) Attested Builds (v0.15)

### 7.0 Hash & canonicalization policy (LOCKED)
- Security-relevant hash/proof đều dùng:
  - `sha256-v1`
- Canonical JSON bytes:
  - UTF-8
  - map key sort theo UTF-8 bytes
  - LF line endings
  - không phụ thuộc pretty-print.
- Canonical TOML bytes:
  - không hash raw TOML text
  - hash canonical JSON view của TOML AST với key order cố định.
- Applied to:
  - lock signature input
  - permission diff hash
  - build manifest hash/attestation hash
  - reproducibility comparison hashes

### 7.1 Build manifest
`build_manifest.json` includes:
- runtime version + commit
- lock hash + lock signature status
- deps list + hashes + signer ids
- effective permissions
- lane
- conformance suite version/hash
- cache mode (on/off)
- cassette hash (if quarantine replay)

### 7.2 Attestation signature
- `build_manifest.sig` signed by project key
- `ocl build --attest`
- `ocl verify --attest <artifact_dir>`

### 7.3 Reproducibility check
- `ocl verify --repro <artifact_dir>`:
  - rerun build in clean env and compare build_manifest hash.
  - allowed exclusions (locked list):
    - `created_at`
    - `machine_id`
    - `cwd`
  - mọi field ngoài danh sách exclusions phải byte-equal theo canonicalization policy.

---

## 8) LTS Rehearsal (v0.15)

### 8.1 LTS policy profile
- `lts_profile = "strict"`
Rules:
- no breaking changes to:
  - core syntax
  - manifest schema (additive only)
  - pack key contracts (additive only)
  - trace schema (additive only)
- require:
  - conformance suite pass
  - trust checks pass
  - permission diffs approved

### 8.2 LTS tooling
- `ocl lts check <project>`:
  - verifies `deps.lock.v3` signed
  - verifies deps signed/trusted
  - runs upgrade-check subset
  - runs conformance subset relevant to used packs
- `ocl lts report`:
  - outputs readiness report + outstanding risks

### 8.3 LTS rehearsal window
Define:
- N releases with zero breaking
- track metrics:
  - conformance failures
  - perf regressions
  - permission diff incidents

Outcome:
- if stable, you can *consider* an eventual “epoch freeze” (still 0.x if you want).

---

## 9) Quarantine + cassette hardening (v0.15)

### 9.1 Cassette signing (stretch v0.15)
- cassette bundle can be signed:
  - prevents tampering
- `cassette.sig` over `cassette_hash`
- enforce in replay if `require_signed_cassette=true`

### 9.2 Secret handling
- enforce redaction policies:
  - if headers/env not redacted but match sensitive patterns:
    - `locked_v071` => fail
    - `locked_v06|quarantine` => warn (emit audit marker)
- store redaction policy hash in cassette_meta

---

## 10) Error taxonomy v0.15 (additive)
- `X-TRUST-REGISTRY-DENIED`
- `X-TRUST-SIGNATURE-REQUIRED`
- `X-LOCK-SIGNATURE-MISSING`
- `X-LOCK-SIGNATURE-INVALID`
- `X-PERMISSION-REVIEW-REQUIRED`
- `X-PERMISSION-APPROVAL-MISSING`
- `X-ATTESTATION-INVALID`
- `X-LTS-CHECK-FAILED`

RC codes:
- `RC-TRUST-UNVERIFIED`
- `RC-PERMISSION-UNAPPROVED`

Policy for X vs RC:
- Build/resolve/lint stage failures dùng X-* (không phải runtime Result4).
- Trong lane `locked_v071`: trust fail phải dừng ở build/resolve stage với X-*, không đi vào runtime Result4.
- Runtime capability deny phải surface qua Result4 + RC-*:
  - thiếu trust/policy deny runtime => `INSUFFICIENT(RC-TRUST-UNVERIFIED)` hoặc RC policy cụ thể.
  - chặn do yêu cầu review/approval => `DEFERRED(RC-PERMISSION-UNAPPROVED)` khi còn thiếu phê duyệt.
- `RC-TRUST-UNVERIFIED` dùng cho `locked_v06|quarantine` compatibility paths hoặc tooling modes (`lint/report`), không thay thế hard gate của `locked_v071`.

---

## 11) Execution gates v0.15

### Gate 15-A — Trust policy engine
Scope:
- trust.toml parser
- verify deps signatures against trusted keys
- registry allowlist
Tests:
- `tests/trust_policy.rs`
Exit criteria:
- lane `locked_v071` rejects untrusted deps

### Gate 15-B — Signed lockfile
Scope:
- lock canonicalization + sign/verify
- CI gate flags
Tests:
- `tests/lock_sign.rs`
Exit criteria:
- tampered lockfile detected

### Gate 15-C — Permission snapshot + diff + approval
Scope:
- implement canonical commands:
  - `ocl perm snapshot`
  - `ocl perm diff <old> <new>`
  - `ocl perm approve <diff_report>`
- `ocl perm review` chỉ là alias compatibility
- approval file + hash verification trong CI
Tests:
- `tests/perm_review.rs`
Exit criteria:
- new permission requires approval to pass

### Gate 15-D — Build attestation
Scope:
- build_manifest generation
- attest sign/verify
Tests:
- `tests/attestation.rs`
Exit criteria:
- verify fails on tampered artifacts

### Gate 15-E — LTS tooling
Scope:
- `ocl lts check/report`
- integrate conformance + upgrade-check + trust gates
Tests:
- `tests/lts_check.rs`
Exit criteria:
- LTS report is deterministic and actionable

### Gate 15-F — Cassette signing + secret hardening (stretch)
Scope:
- sign/verify cassette bundles
- redaction enforcement
Tests:
- `tests/cassette_sign.rs`
Exit criteria:
- replay rejects tampered cassette (if required)
- gate này không chặn `DONE` của core v0.15 (15-A..15-E)

### Exit Contract v0.15 (machine-checkable)
1) No-new-semantics window
- Rule:
  - v0.15 không thêm semantics mới cho language/packs; chỉ bugfix + hardening + DX.
- PASS:
  - changelog/release note v0.15 không có mục `new language feature` hoặc `new pack semantics`.
  - snapshot contracts (pack keys + ctx/payload schema hashes + grammar surface) không có breaking diff ngoài allowlist bugfix.

2) Contract freeze
- Freeze list:
  - audit schema,
  - signature canonicalization,
  - artifact layout,
  - permissions manifest schema,
  - error taxonomy + alias shape,
  - lane rules.
- PASS:
  - có tài liệu `Contract Freeze v0.15`.
  - có test snapshot/parse schema cho từng freeze item.

3) Conformance green
- PASS:
  - `cargo test` pass.
  - conformance suite v0.14 pass (`ocl test --conformance run --suite all`).
  - platform target phải khóa rõ:
    - hoặc 1 OS cố định,
    - hoặc 2 OS nếu pipeline cross-OS đã sẵn sàng.

4) Determinism invariant holds
- PASS:
  - cùng seed/config => signature stable qua `N=20` runs.
  - path normalization stable across workspace roots khác nhau.
  - comparison dựa trên `signature.txt` byte-equal.

5) Compatibility rehearsal
- PASS:
  - lane compatibility cũ (`locked_v06`) vẫn chạy theo policy đã hứa.
  - mọi deprecation có alias + hint rõ ràng.
  - có targeted e2e tests cho compat + hint assertions.

6) Release rehearsal (dry-run)
- PASS:
  - chạy được pipeline dry-run:
    - version bump rehearsal,
    - tag rehearsal,
    - build artifacts,
    - verify hash/signature nội bộ,
    - publish staging (nếu có staging pipeline).
  - không publish production trong v0.15 rehearsal.
  - mục tiêu: chứng minh quy trình có thể lặp lại ổn định trước v1.0.

---

## 12) Operational commands (v0.15)
Core gates (required):
- `cargo test`
- `cargo test --test trust_policy`
- `cargo test --test lock_sign`
- `cargo test --test perm_review`
- `cargo test --test attestation`
- `cargo test --test lts_check`
- `cargo clippy --all-targets -- -D warnings`
- `cargo fmt -- --check`

Stretch gate (optional):
- `cargo test --test cassette_sign`

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

### 2026-03-05 — 15-A Planning Freeze
- Date: 2026-03-05
- Gate/Step: 15-A
- Why:
  - khóa trust policy engine cho lane `locked_v071` theo hợp đồng v0.15, giữ tương thích parser trust cũ.
- Scope:
  - mở rộng parser `trust.toml` hỗ trợ `[policy]`, `[registries]`, `[[trusted_key]]`;
  - enforce strict mode cho `locked_v071`, từ chối `mode="warn"` ở lane này;
  - enforce registry allowlist deterministic trong strict mode;
  - verify signer trusted theo scope package từ `[[trusted_key]]`;
  - thêm test targeted mới `tests/trust_policy.rs`.
- Expected tests:
  - `cargo test --test trust_policy`
  - `cargo test --test trust_lane_policy`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
- Exit criteria:
  - lane `locked_v071` từ chối deps untrusted đúng contract;
  - parser mới hoạt động và không phá trust format cũ;
  - targeted tests pass.

### 2026-03-05 — 15-A Implementation Closeout
- Date: 2026-03-05
- Gate/Step: 15-A
- Implemented:
  - mở rộng trust engine trong `ocl-sdk`:
    - parse `trust.toml` v0.15 (`[policy]`, `[registries]`, `[[trusted_key]]`) + giữ compat `trusted_signers.*`;
    - thêm lane policy precedence cho `locked_v071|locked_v06|quarantine`;
    - enforce registry allowlist deterministic + scope matching cho trusted key;
    - enforce hard gate cho `locked_v071` khi `mode="warn"` hoặc source không nằm trong allowlist strict.
  - thêm suite test mới `tests/trust_policy.rs` (3 test) cho Gate 15-A.
  - vá 1 lỗi clippy ngoài phạm vi gate nhưng chặn pipeline (`push_str("\n")` -> `push('\n')` trong `w9.rs`).
- Files changed:
  - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/src/w9.rs`
  - `tests/trust_policy.rs`
- Commands run:
  - `cargo test --test trust_policy`
  - `cargo test --test trust_lane_policy`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt`
  - `cargo fmt -- --check`
- Test results:
  - Targeted tests (must-pass for gate):
    - `PASS` (`trust_policy`: 3/3)
  - Regression tests (supporting only):
    - `PASS` (`trust_lane_policy`: 3/3)
    - `PASS` (`cargo test` toàn repo)
    - `PASS` (`cargo clippy --all-targets -- -D warnings`)
    - `PASS` (`cargo fmt -- --check`)
- Kết luận gate:
  - `DONE`
- Design alignment:
  - `FULL`
- Notes/risks:
  - chưa tạo `trust_report.json` artifact trong Gate 15-A; phần này được giữ cho gate tiếp theo theo scope v0.15.

### 2026-03-05 — 15-B Planning Freeze
- Date: 2026-03-05
- Gate/Step: 15-B
- Why:
  - khóa signed lockfile contract theo v0.15 với SoT `deps.lock.v3 + deps.lock.v3.sig` và CLI `ocl lock sign|verify`.
- Scope:
  - canonical hash input từ canonical JSON view của lock AST;
  - thêm SDK API sign/verify cho `deps.lock.v3.sig`;
  - enforce `require_signed_lock=true` trong lane `locked_v071`;
  - bổ sung CLI `ocl lock sign` và `ocl lock verify`;
  - thêm test targeted `tests/lock_sign.rs`.
- Expected tests:
  - `cargo test --test lock_sign`
  - `cargo test --test lock_resolve`
  - `cargo test --test trust_policy`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
- Exit criteria:
  - lock bị tamper phải bị detect;
  - signed lock verify pass khi dữ liệu đúng;
  - lane `locked_v071` fail-honest nếu bật `require_signed_lock=true` mà thiếu/chệch chữ ký.

### 2026-03-05 — 15-B Implementation Closeout
- Date: 2026-03-05
- Gate/Step: 15-B
- Implemented:
  - thêm signed-lock pipeline trong SDK:
    - canonical lock AST JSON hash (`sha256-v1`);
    - sign output `deps.lock.v3.sig`;
    - verify hash + signature + lock path consistency;
    - enforce `require_signed_lock=true` ở `locked_v071` trong đường `check/run` có lock.
  - mở rộng CLI:
    - `ocl lock sign <project_dir> --key <keyid> [--lock deps.lock.v3]`
    - `ocl lock verify <project_dir> [--lock deps.lock.v3]`
  - thêm test mới `tests/lock_sign.rs` cho Gate 15-B.
- Files changed:
  - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
  - `projects/ocp-ocl/crates/ocl-cli/src/main.rs`
  - `tests/lock_sign.rs`
- Commands run:
  - `cargo test --test lock_sign`
  - `cargo test --test lock_resolve`
  - `cargo test --test trust_policy`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt`
  - `cargo fmt -- --check`
- Test results:
  - Targeted tests (must-pass for gate):
    - `PASS` (`lock_sign`: 3/3)
  - Regression tests (supporting only):
    - `PASS` (`lock_resolve`: 2/2)
    - `PASS` (`trust_policy`: 3/3)
    - `PASS` (`cargo test` toàn repo)
    - `PASS` (`cargo clippy --all-targets -- -D warnings`)
    - `PASS` (`cargo fmt -- --check`)
- Kết luận gate:
  - `DONE`
- Design alignment:
  - `FULL`
- Notes/risks:
  - CLI `--lock` ở v0.15-B đang khóa vào `deps.lock.v3` để giữ SoT duy nhất; chưa mở multi-lock path ngoài scope gate.

### 2026-03-05 — 15-C Planning Freeze
- Date: 2026-03-05
- Gate/Step: 15-C
- Why:
  - Khóa workflow `perm snapshot/diff/approve` thành contract machine-checkable để lane `locked_v071` fail-honest khi xuất hiện quyền mới chưa được duyệt.
- Scope:
  - Thêm API SDK cho snapshot/diff/approve permissions.
  - Thêm CLI surface `ocl perm ...` (kèm alias `ocl perm review`).
  - Gắn enforcement approval vào path verify lock v3 cho lane `locked_v071`.
  - Bổ sung test targeted cho diff yêu cầu approval.
- Expected tests:
  - `cargo test --test perm_review`
  - `cargo test --test lock_sign`
  - `cargo test --test trust_policy`
- Exit criteria:
  - Có `permission_diff_hash` canonical (`sha256-v1`) và báo lỗi `X-PERMISSION-APPROVAL-REQUIRED` khi chưa có approval.
  - Có thể approve diff và chạy lại diff thành công với cùng hash.
  - Regression lane `locked_v071` không vỡ.

### 2026-03-05 — 15-C Implementation Closeout
- Date: 2026-03-05
- Gate/Step: 15-C
- Implemented:
  - Thêm module `perm_v15` trong SDK để tạo snapshot permissions, tạo diff report có hash canonical và approve diff vào `permissions.approval.toml`.
  - Thêm enforcement review policy vào verify lock v3 (`locked_v071`): có baseline snapshot nhưng diff quyền mới chưa được approve thì dừng fail-honest.
  - Mở rộng CLI:
    - `ocl perm snapshot`
    - `ocl perm diff`
    - `ocl perm approve`
    - `ocl perm review` (alias của diff flow)
  - Bổ sung test end-to-end xác nhận: chưa approve thì diff bị chặn, approve xong thì pass.
- Files changed:
  - `projects/ocp-ocl/crates/ocl-sdk/src/perm_v15.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/Cargo.toml`
  - `projects/ocp-ocl/crates/ocl-cli/src/main.rs`
  - `tests/perm_review.rs`
  - `Cargo.lock`
  - `OCP-OCL-MVP-PLAN-v0.15.md`
- Commands run:
  - `cargo test --test perm_review`
  - `cargo test --test lock_sign`
  - `cargo test --test trust_policy`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
- Test results:
  - Targeted tests (must-pass for gate):
    - `PASS` (`perm_review`: 1/1)
  - Regression tests (supporting only):
    - `PASS` (`lock_sign`: 3/3)
    - `PASS` (`trust_policy`: 3/3)
    - `PASS` (`cargo test` toàn repo)
    - `PASS` (`cargo clippy --all-targets -- -D warnings`)
    - `PASS` (`cargo fmt -- --check`)
- Kết luận gate:
  - `DONE`
- Design alignment:
  - `FULL`
- Notes/risks:
  - Enforcement `permissions.snapshot.json` chỉ kích hoạt khi baseline snapshot tồn tại; repository chưa có baseline sẽ không fail để tránh phá flow bootstrap, cần được chốt ở Gate 15-E (LTS check/report).

### 2026-03-05 — 15-D Planning Freeze
- Date: 2026-03-05
- Gate/Step: 15-D
- Why:
  - Khóa `attested build` thành workflow machine-checkable để phát hiện tamper và kiểm tra reproducibility bằng canonical hash.
- Scope:
  - Thêm SDK module cho:
    - `build_attestation_v15` (generate `build_manifest.json` + `build_manifest.sig`)
    - `verify_build_attestation_v15`
    - `verify_build_repro_v15` với allowlist exclusion khóa cứng (`created_at`, `machine_id`, `cwd`)
  - Mở rộng CLI:
    - `ocl build --attest [--attest-key <keyid>]`
    - `ocl verify --attest <artifact_dir>`
    - `ocl verify --repro <artifact_dir>`
  - Giữ nguyên path cũ `ocl verify <project_dir> --phenotype ...` để không phá backward compatibility.
  - Thêm test targeted `tests/attestation.rs`.
- Expected tests:
  - `cargo test --test attestation`
  - `cargo test --test lock_sign`
  - `cargo test --test trust_policy`
  - `cargo test --test perm_review`
- Exit criteria:
  - `verify --attest` fail-honest khi manifest bị tamper.
  - `verify --repro` pass khi không có drift ngoài allowlist exclusion.
  - Không regression trên trust/lock/permission gates đã đóng.

### 2026-03-05 — 15-D Implementation Closeout
- Date: 2026-03-05
- Gate/Step: 15-D
- Implemented:
  - Thêm module mới `attestation_v15` trong `ocl-sdk`:
    - canonical JSON + `sha256-v1` cho build manifest.
    - tạo `target/ocl/attestation/build_manifest.json`.
    - ký `build_manifest.sig` bằng deterministic local key context (v0.15 local project key flow).
    - verify chữ ký/hash manifest.
    - verify reproducibility bằng cách rebuild manifest và so hash sau khi loại trừ các trường được phép.
  - Nối SDK export trong `lib.rs` với summary types cho attestation/repro verification.
  - Nối CLI:
    - `build` hỗ trợ `--attest` (và `--attest-key` tùy chọn).
    - `verify` hỗ trợ `--attest` / `--repro` (đồng thời vẫn giữ `--phenotype` legacy path).
  - Bổ sung `tests/attestation.rs`:
    - case pass cho build+attest+verify+repro.
    - case fail khi tamper `build_manifest.json`.
- Files changed:
  - `projects/ocp-ocl/crates/ocl-sdk/src/attestation_v15.rs`
  - `projects/ocp-ocl/crates/ocl-sdk/src/lib.rs`
  - `projects/ocp-ocl/crates/ocl-cli/src/main.rs`
  - `tests/attestation.rs`
  - `OCP-OCL-MVP-PLAN-v0.15.md`
- Commands run:
  - `cargo test --test attestation`
  - `cargo test --test lock_sign`
  - `cargo test --test trust_policy`
  - `cargo test --test perm_review`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt`
  - `cargo fmt -- --check`
- Test results:
  - Targeted tests (must-pass for gate):
    - `PASS` (`attestation`: 2/2)
  - Regression tests (supporting only):
    - `PASS` (`lock_sign`: 3/3)
    - `PASS` (`trust_policy`: 3/3)
    - `PASS` (`perm_review`: 1/1)
    - `PASS` (`cargo test` toàn repo)
    - `PASS` (`cargo clippy --all-targets -- -D warnings`)
    - `PASS` (`cargo fmt -- --check`)
- Kết luận gate:
  - `DONE`
- Design alignment:
  - `FULL`
- Notes/risks:
  - `verify --repro` hiện bám vào `project_root` ghi trong `build_manifest.json`; nếu project đã bị di chuyển path sau khi attest thì check repro sẽ fail-honest do khác input build context.

### 2026-03-05 — 15-E Planning Freeze
- Date: 2026-03-05
- Gate/Step: 15-E
- Why:
  - khóa `ocl lts check/report` thành entrypoint machine-checkable cho Exit Contract v0.15, gom trust/lock/permission/conformance/upgrade-check vào một báo cáo deterministic.
- Scope:
  - mở rộng CLI `ocl lts check` và `ocl lts report`;
  - `lts check` phải:
    - verify signed lock v3;
    - enforce trust policy strict theo lane;
    - enforce permission review baseline (`permissions.snapshot.json` + `permissions.approval.toml`);
    - chạy subset conformance/upgrade-check theo contract v0.14/v0.15;
    - xuất report JSON deterministic + exit code machine-checkable.
  - thêm test targeted `tests/lts_check.rs`.
- Expected tests:
  - `cargo test --test lts_check`
  - `cargo test --test conformance_runner`
  - `cargo test --test upgrade_check`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
- Exit criteria:
  - `ocl lts check` pass khi toàn bộ gates nội bộ đạt;
  - fail-honest với mã lỗi đúng khi thiếu baseline permission approval;
  - report text/json nhất quán, actionable, deterministic.

### 2026-03-05 — 15-E Implementation Closeout
- Date: 2026-03-05
- Gate/Step: 15-E
- Implemented:
  - mở rộng CLI `ocl-cli`:
    - `ocl lts check <project_dir> [--manifest ... --target-runtime ... --out ... --json]`;
    - `ocl lts report <report_file> [--json]`.
  - triển khai pipeline `lts check`:
    - verify signed lock (`verify_deps_lock_v3_signature_v15`);
    - verify trust strict (`verify_deps_signing_and_trust_v10` theo lane policy);
    - enforce permission approval baseline trong `locked_v071` (`permissions.snapshot.json` + diff hash approval);
    - chạy conformance subset và upgrade-check subset;
    - tổng hợp report `ocl.lts_check.v1`, deterministic, có `ok/error_code/risks/gates`.
  - thêm targeted tests `tests/lts_check.rs`:
    - case pass đầy đủ gate;
    - case fail-honest khi thiếu `permissions.snapshot.json`.
- Files changed:
  - `projects/ocp-ocl/crates/ocl-cli/src/main.rs`
  - `tests/lts_check.rs`
  - `OCP-OCL-MVP-PLAN-v0.15.md`
- Commands run:
  - `cargo test --test lts_check`
  - `cargo test --test conformance_runner`
  - `cargo test --test upgrade_check`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
- Test results:
  - Targeted tests (must-pass for gate):
    - `PASS` (`lts_check`: 2/2)
    - `PASS` (`conformance_runner`: 3/3)
    - `PASS` (`upgrade_check`: 2/2)
  - Regression tests (supporting only):
    - `PASS` (`cargo test` toàn repo)
    - `PASS` (`cargo clippy --all-targets -- -D warnings`)
    - `PASS` (`cargo fmt -- --check`)
- Kết luận gate:
  - `DONE`
- Design alignment:
  - `FULL`
- Notes/risks:
  - `lts check` trong lane `locked_v071` yêu cầu baseline `permissions.snapshot.json`; thiếu baseline sẽ fail-honest với risk code permission-review để tránh bỏ qua approval gate.

### 2026-03-06 — 15-F Planning Freeze (stretch)
- Date: 2026-03-06
- Gate/Step: 15-F
- Why:
  - khóa hardening cho cassette quarantine: có thể yêu cầu cassette ký số và replay phải verify chữ ký khi policy bật, đồng thời bổ sung redaction-policy evidence trong cassette metadata.
- Scope:
  - thêm policy parse từ `[quarantine]`:
    - `require_signed_cassette` (bool),
    - `redact_headers`,
    - `redact_env_patterns`;
  - mở rộng cassette bundle writer:
    - ghi `redaction_policy_hash`,
    - ghi `redaction_warning_count`,
    - ghi `cassette.sig` khi `require_signed_cassette=true`;
  - mở rộng replay validator:
    - verify `cassette.sig` nếu signed-cassette required;
  - thêm targeted tests `tests/cassette_sign.rs`.
- Expected tests:
  - `cargo test --test cassette_sign`
  - `cargo test --test quarantine_gate`
  - `cargo test --test std_time_wallclock`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt -- --check`
- Exit criteria:
  - replay reject khi thiếu/tamper `cassette.sig` trong case `require_signed_cassette=true`;
  - cassette metadata có `redaction_policy_hash` và warning count deterministic;
  - không regression các flow quarantine hiện có.

### 2026-03-06 — 15-F Implementation Closeout (stretch)
- Date: 2026-03-06
- Gate/Step: 15-F
- Implemented:
  - mở rộng parsing policy `[quarantine]`:
    - `require_signed_cassette`,
    - `redact_headers`,
    - `redact_env_patterns`.
  - mở rộng cassette writer:
    - phát hiện header nhạy cảm chưa redact,
    - ghi `redaction_policy_hash`, `redaction_warning_count`, `require_signed_cassette` vào `cassette_meta.toml`,
    - ghi `redaction_warning.log` khi có cảnh báo,
    - tạo `cassette.sig` theo canonical `cassette_hash` khi bật signed-cassette.
  - mở rộng `replay.toml`:
    - persist `require_signed_cassette = true` khi policy bật.
  - mở rộng replay validator:
    - verify `cassette.sig` và reject khi thiếu/chỉnh sửa sai.
  - hardening vòng 2 cho chữ ký cassette:
    - ràng buộc chữ ký theo project-local secret (`.ocl_signing/cassette_sign.key.v15`) để chặn forge từ dữ liệu public,
    - replay verify bắt buộc đọc key local khi `require_signed_cassette=true`.
  - mở rộng redaction enforcement:
    - áp dụng `redact_env_patterns` cho stream `stdout/stderr` của `std.proc.exec` record,
    - loại bỏ `value_prefix` khỏi `redaction_warning.log` (không để lộ dữ liệu nhạy cảm).
  - thêm test integration `tests/cassette_sign.rs` (4 case).
- Files changed:
  - `projects/ocp-ocl/crates/ocl-cli/src/main.rs`
  - `tests/cassette_sign.rs`
  - `OCP-OCL-MVP-PLAN-v0.15.md`
- Commands run:
  - `cargo test --test cassette_sign`
  - `cargo test --test quarantine_gate`
  - `cargo test`
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo fmt`
  - `cargo fmt -- --check`
- Test results:
  - Targeted tests (must-pass for gate):
    - `PASS` (`cassette_sign`: 4/4)
  - Regression tests (supporting only):
    - `PASS` (`quarantine_gate`: 1/1)
    - `PASS` (`cargo test` toàn repo)
    - `PASS` (`cargo clippy --all-targets -- -D warnings`)
    - `PASS` (`cargo fmt -- --check`)
- Kết luận gate:
  - `DONE`
- Design alignment:
  - `FULL`
- Notes/risks:
  - Chữ ký cassette v0.15-F là deterministic project-local signing (gắn với key local của project) để phát hiện sai lệch/tamper trong flow CI nội bộ; chưa mở PKI external trust chain (ngoài scope stretch gate).
  - Redaction env patterns hiện enforce trên dữ liệu record stream của `std.proc.exec`; nếu cần coverage sâu hơn (full env map), sẽ mở ở scope kế tiếp bằng mở rộng record schema.

## 14) Checklist khóa trước khi đóng gate
- [x] Gate status đã cập nhật đúng (`TODO/IN_PROGRESS/PARTIAL/DONE`).
- [x] Có đủ cặp `Planning Freeze` + `Implementation Closeout` cho gate đang đóng.
- [x] `Files changed` khớp code delta thực tế của gate.
- [x] `Commands run` là lệnh đã chạy thật, không ghi lệnh dự kiến.
- [x] `Targeted tests (must-pass for gate)` đã pass cho đúng phạm vi thay đổi.
- [x] `Regression tests (supporting only)` đã ghi rõ phạm vi và kết quả.
- [x] Nếu chưa đạt 100%: giữ `IN_PROGRESS/PARTIAL`, không ghi `DONE`.
- [x] Không còn marker `FAIL`/placeholder `PASS/FAIL` trong closeout đã đánh dấu `DONE`.
- [x] Không có lỗi mã hóa tiếng Việt trong nội dung file theo hiển thị IDE.
- [x] Đã ghi `Design alignment: FULL` hoặc `Design alignment: PARTIAL` kèm lý do bất khả thi.

## 15) Handoff v0.15 -> v1.0 (pre-draft)
- Chỉ mở scope v1.0 sau khi các gate core `15-A..15-E` đã `DONE` và KPI core đạt đủ evidence.
- Gate stretch `15-F` không chặn core done, nhưng phải giữ trạng thái riêng (`TODO/IN_PROGRESS/DONE`) và không làm đổi kết luận core.
- Snapshot hợp đồng trước v1.0 phải được chốt và lưu:
  - lane policy (`locked_v071|locked_v06|quarantine`),
  - signature canonicalization,
  - trace/audit schema đang dùng,
  - lockfile SoT (`deps.lock.v3`) + trust/approval policy.
- Mọi thay đổi khác biệt với v0.15 phải đi qua migration contract rõ ràng, có compatibility path và test chứng minh.
- Release rehearsal v1.0 chỉ được mở khi:
  - conformance v0.14 pass,
  - determinism replay pass theo protocol đã khóa,
  - dry-run release pipeline pass và có artifact/hash evidence.

---

