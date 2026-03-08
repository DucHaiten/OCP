# MAINTAINER GUIDE OCP v1.0 (VI)

Tài liệu này dành cho vai trò maintainer/release, không phải luồng onboarding người dùng mới.

## Mục tiêu
- Chốt release theo quy trình có bằng chứng machine-checkable.
- Không để `PASS giả`: mỗi gate phải có code delta phù hợp + targeted tests.
- Giữ chain-of-trust xuyên suốt lock/perm/attest/replay/artifact manifest.

## 1) Luồng trust + attest + LTS
Luồng production tối thiểu:
1. `ocp lock sign <project_dir> --key <keyid> --lock deps.lock.v3`
2. `ocp lock verify <project_dir> --lock deps.lock.v3`
3. `ocp build <project_dir> --attest [--attest-key <keyid>]`
4. `ocp verify --attest <artifact_dir>`
5. `ocp verify --repro <artifact_dir>`
6. `ocp lts check <project_dir> --out <report_file> [--json]`
7. `ocp lts report <report_file> [--json]`

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
- `target/ocp/w16/meta/run_manifest.json`
- `target/ocp/w16/contracts/contract_freeze_report.json`
- `target/ocp/w16/conformance/unified_conformance_report.json`
- `target/ocp/w16/determinism/determinism_soak_report.json`
- `target/ocp/w16/security/security_chain_report.json`
- `target/ocp/w16/rc/release_artifact_manifest.json`

## 3) DX operations cho strict lane
Chuỗi doctor/fix/budget/cassette:
- `ocp perm doctor <project_dir> [--out <report.json>]`
- `ocp perm fix --plan <project_dir> [--out <permission_fix_plan.json>]`
- `ocp perm fix --apply <project_dir> [--plan-file <permission_fix_plan.json>] [--patch <permission_fix.patch.toml>] [--out <permission_fix_safety_report.json>] [--approval <permissions.approval.toml>] --ack-risk --justification <text> --by <id> --date <YYYY-MM-DD>`
- `ocp budget analyze <artifact_dir|audit.jsonl> [--json]`
- `ocp budget doctor <artifact_dir|audit.jsonl> [--json]`
- `ocp cassette stats <artifact_dir> [--json]`
- `ocp cassette prune <artifact_dir> --plan|--apply [--ttl-days <u32>] [--json]`
- `ocp cassette gc <artifact_dir> [--json]`
- `ocp cassette upgrade <artifact_dir> --plan|--apply [--max-entries <u32>] [--max-block-bytes <u32>] [--json]`

Quy tắc cứng:
- `perm fix --apply` thiếu `--ack-risk|--justification|--by|--date` phải fail.
- `locked_v071` không cho bypass approval.
- cassette thiếu block/chunk phải fail-honest.

## 4) SoT + migration + release packaging
Kiểm tra SoT:
- `ocp verify --contract <contract.json>`
- `ocp verify --contract-signature <contract.json.sig|contract.json>`

Rehearsal migration:
- `ocp cassette upgrade <artifact_dir> --plan --json`
- `ocp cassette upgrade <artifact_dir> --apply --json`
- `ocp lock verify <project_dir> --lock deps.lock.v3`
- `ocp deps verify <project_dir>`

## 5) Editor release checks (v0.19 line)
Checklist tối thiểu:
- VSIX cài được và nhận diện `.ocp`.
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
- `target/ocp/w20/meta/run_manifest.json`
- `target/ocp/w20/regression/cross_platform_signature_aggregate_report.json`
- `target/ocp/w20/hardcore/fuzz_report.json`
- `target/ocp/w20/hardcore/chaos_report.json`
- `target/ocp/w20/security/redteam_report.json`
- `target/ocp/w20/release/v1_rc_manifest.json`
- `target/ocp/w20/signoff/final_findings_report.json`
- `target/ocp/w20/signoff/v1_release_go_no_go.json`

GO/NO-GO:
- Chỉ `GO` khi không còn finding `CRITICAL/HIGH` mở.
- `final_findings_report.json` phải có đủ nhóm `CRITICAL/HIGH/MEDIUM/LOW`.
- Chaos gates không chạy với `--features w20_chaos` => chưa đạt.

## 7) Nơi xem lịch sử kỹ thuật
- `docs/plans/history/OCP-MVP-PLAN-v0.1.md` đến `v0.20.md`
- `docs/plans/history/README.md`
