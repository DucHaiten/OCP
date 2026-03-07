# MAINTAINER GUIDE OCL v1.0 (VI)

Tài liệu này dành cho vai trò maintainer/release, không phải luồng onboarding người dùng mới.

## Mục tiêu
- Chốt release theo quy trình có bằng chứng machine-checkable.
- Không để `PASS giả`: mỗi gate phải có code delta phù hợp + targeted tests.
- Giữ chain-of-trust xuyên suốt lock/perm/attest/replay/artifact manifest.

## 1) Luồng trust + attest + LTS
Luồng production tối thiểu:
1. `ocl lock sign <project_dir> --key <keyid> --lock deps.lock.v3`
2. `ocl lock verify <project_dir> --lock deps.lock.v3`
3. `ocl build <project_dir> --attest [--attest-key <keyid>]`
4. `ocl verify --attest <artifact_dir>`
5. `ocl verify --repro <artifact_dir>`
6. `ocl lts check <project_dir> --out <report_file> [--json]`
7. `ocl lts report <report_file> [--json]`

## 2) Product-readiness checklist trước release
Checklist vận hành:
1. Contract freeze + historical evidence.
2. Unified conformance matrix.
3. Determinism/replay invariants.
4. Compatibility + migration rehearsal.
5. Perf/capacity verification.
6. Security + supply-chain verification.
7. RC dry-run + release artifact manifest.

Artifacts cốt lõi cần có:
- `target/ocl/w16/meta/run_manifest.json`
- `target/ocl/w16/contracts/contract_freeze_report.json`
- `target/ocl/w16/conformance/unified_conformance_report.json`
- `target/ocl/w16/determinism/determinism_soak_report.json`
- `target/ocl/w16/security/security_chain_report.json`
- `target/ocl/w16/rc/release_artifact_manifest.json`

## 3) DX operations cho strict lane
Chuỗi doctor/fix/budget/cassette:
- `ocl perm doctor <project_dir> [--out <report.json>]`
- `ocl perm fix --plan <project_dir> [--out <permission_fix_plan.json>]`
- `ocl perm fix --apply <project_dir> [--plan-file <permission_fix_plan.json>] [--patch <permission_fix.patch.toml>] [--out <permission_fix_safety_report.json>] [--approval <permissions.approval.toml>] --ack-risk --justification <text> --by <id> --date <YYYY-MM-DD>`
- `ocl budget analyze <artifact_dir|audit.jsonl> [--json]`
- `ocl budget doctor <artifact_dir|audit.jsonl> [--json]`
- `ocl cassette stats <artifact_dir> [--json]`
- `ocl cassette prune <artifact_dir> --plan|--apply [--ttl-days <u32>] [--json]`
- `ocl cassette gc <artifact_dir> [--json]`
- `ocl cassette upgrade <artifact_dir> --plan|--apply [--max-entries <u32>] [--max-block-bytes <u32>] [--json]`

Quy tắc cứng:
- `perm fix --apply` thiếu `--ack-risk|--justification|--by|--date` phải fail.
- `locked_v071` không cho bypass approval.
- cassette thiếu block/chunk phải fail-honest.

## 4) SoT + migration + release packaging
Kiểm tra SoT:
- `ocl verify --contract <contract.json>`
- `ocl verify --contract-signature <contract.json.sig|contract.json>`

Rehearsal migration:
- `ocl cassette upgrade <artifact_dir> --plan --json`
- `ocl cassette upgrade <artifact_dir> --apply --json`
- `ocl lock verify <project_dir> --lock deps.lock.v3`
- `ocl deps verify <project_dir>`

## 5) Editor release checks (v0.19 line)
Checklist tối thiểu:
- VSIX cài được và nhận diện `.ocl`.
- LSP features tối thiểu chạy được.
- Workspace trust policy đúng contract.
- Packaging manifest + signature verify pass.

## 6) Final exhaustive verification (v0.20 line)
Flow release-maintainer:
1. Contracts/regression gates.
2. Hardcore bug-hunt (fuzz/property/mutation).
3. Chaos/fault gates với `--features w20_chaos`.
4. Security + packaging + final signoff.

Artifacts chốt:
- `target/ocl/w20/meta/run_manifest.json`
- `target/ocl/w20/regression/cross_platform_signature_aggregate_report.json`
- `target/ocl/w20/hardcore/fuzz_report.json`
- `target/ocl/w20/hardcore/chaos_report.json`
- `target/ocl/w20/security/redteam_report.json`
- `target/ocl/w20/release/v1_rc_manifest.json`
- `target/ocl/w20/signoff/final_findings_report.json`
- `target/ocl/w20/signoff/v1_release_go_no_go.json`

GO/NO-GO:
- Chỉ `GO` khi không còn finding `CRITICAL/HIGH` mở.
- `final_findings_report.json` phải có đủ nhóm `CRITICAL/HIGH/MEDIUM/LOW`.
- Chaos gates không chạy với `--features w20_chaos` => chưa đạt.

## 7) Nơi xem lịch sử kỹ thuật
- `docs/plans/history/OCP-OCL-MVP-PLAN-v0.1.md` đến `v0.20.md`
- `docs/plans/history/README.md`
