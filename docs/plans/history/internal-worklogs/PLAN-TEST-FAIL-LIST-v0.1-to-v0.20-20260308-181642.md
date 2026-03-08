# PLAN TEST FAIL LIST v0.1 -> v0.20 (Đã cập nhật)

- File này được giữ lại tên cũ để bạn dễ theo dõi trong IDE.
- Trạng thái mới nhất đã chốt theo full matrix:
  - Tổng lệnh: `586`
  - PASS: `586`
  - FAIL: `0`
  - SKIP placeholder: `0`

## Nguồn kết quả mới nhất

- Results JSON:
  - `target/ocp/plan_test_run_results_v0.1_to_v0.20_20260308_205025.json`
- Summary JSON:
  - `target/ocp/plan_test_run_summary_v0.1_to_v0.20_20260308_205025.json`
- Fail-list đồng bộ:
  - `PLAN-TEST-FAIL-LIST-v0.1-to-v0.20-20260308-205025.md`

## Ghi chú

- Danh sách FAIL cũ (52 fail, 6 skip) đã được thay thế bởi kết quả chạy lại sau khi sửa lỗi.
- Nếu cần truy vết lịch sử, tham chiếu các snapshot fail-list theo mốc thời gian.

## Backlog Test Bổ Sung (không gồm đóng gói/phát hành v1.0)

### Quy ước chung để triển khai

- Mỗi test có mã cố định (`T-*`), không đổi tên khi đã merge.
- Mỗi test phải ghi rõ:
  - Input/fixture dùng gì.
  - Các bước chạy cụ thể.
  - Điều kiện `PASS` và điều kiện `FAIL`.
- Mỗi test tạo evidence tại `target/ocp/test_evidence/<test_id>/`.
- Các test ngẫu nhiên phải khóa seed để tái lập (`[11, 17, 23, 47, 89]` tối thiểu).
- Test nào bắt buộc chờ thời gian thực > 5 phút thì tạm loại khỏi lane mặc định.
- Ưu tiên mô phỏng/tăng tốc thời gian logic, không phụ thuộc thời gian thực.

### Danh sách test cần thêm

#### [x] T-ALG-001 - Differential Engine Equivalence (P0)
- Mục tiêu: cùng một chương trình phải cho cùng kết quả trên `interpreter`, `bytecode`, `dual`.
- File dự kiến: `tests/t_alg_001_differential_engine.rs`.
- Input:
  - Tối thiểu 30 chương trình OCP (bao gồm parser edge-case, loop, match, observe, commit).
  - Ưu tiên lấy từ `projects/ocp/apps/*` và fixture conformance hiện có.
- Steps:
  1. Parse + typecheck từng chương trình.
  2. Chạy 3 engine với cùng config.
  3. So sánh `exit_status`, `trace_signature`, `reason_code`, `steps`.
- PASS:
  - 30/30 case tương đương theo contract.
- FAIL:
  - Bất kỳ lệch nào giữa 3 engine mà không nằm trong allowlist contract.
- Lệnh chạy sau khi implement:
  - `cargo test --test t_alg_001_differential_engine -- --nocapture`

#### [x] T-ALG-002 - Metamorphic Invariance (P0)
- Mục tiêu: biến đổi không đổi nghĩa phải giữ nguyên outcome/signature.
- File dự kiến: `tests/t_alg_002_metamorphic_invariance.rs`.
- Input transform bắt buộc:
  - Đổi tên biến cục bộ.
  - Thêm/bớt whitespace.
  - Đổi thứ tự khai báo độc lập.
- Steps:
  1. Sinh bản gốc và bản biến đổi.
  2. Chạy cả hai bằng `dual`.
  3. So sánh `trace_signature`, `reason_code`, `steps`.
- PASS:
  - 100% case “no-op transform” giữ nguyên signature.
- FAIL:
  - Có case no-op làm thay đổi signature hoặc reason.
- Lệnh chạy:
  - `cargo test --test t_alg_002_metamorphic_invariance -- --nocapture`

#### [x] T-ALG-003 - Metamorphic Divergence (P0)
- Mục tiêu: biến đổi có đổi nghĩa phải tạo khác biệt đúng kỳ vọng.
- File dự kiến: `tests/t_alg_003_metamorphic_divergence.rs`.
- Input transform bắt buộc:
  - Đảo điều kiện logic.
  - Đổi hằng số điều khiển nhánh.
  - Đổi thứ tự commit có phụ thuộc.
- PASS:
  - Các case đổi nghĩa làm đổi `reason_code` hoặc `trace_signature` theo thiết kế.
- FAIL:
  - Case đổi nghĩa nhưng vẫn cho kết quả y hệt.
- Lệnh chạy:
  - `cargo test --test t_alg_003_metamorphic_divergence -- --nocapture`

#### [x] T-WF-001 - Workflow State Machine Idempotence (P0)
- Mục tiêu: chuỗi workflow chạy lặp không tạo drift ngầm.
- File dự kiến: `tests/t_wf_001_state_machine_idempotence.rs`.
- Workflow bắt buộc:
  - `init -> check -> run -> test -> build -> replay -> run`.
- Steps:
  1. Chạy chuỗi trên cùng project 3 vòng liên tiếp.
  2. So sánh hash các file state/lock/cache trọng yếu sau mỗi vòng.
- PASS:
  - Vòng 2 và vòng 3 không phát sinh diff ngoài timestamp cho phép.
- FAIL:
  - Có drift state không chủ đích giữa các vòng.
- Lệnh chạy:
  - `cargo test --test t_wf_001_state_machine_idempotence -- --nocapture`

#### [ ] T-WF-002 - Lock/Cache Invalidation Matrix (P0)
- Mục tiêu: invalidation đúng khi source/manifest/deps/policy thay đổi.
- File dự kiến: `tests/t_wf_002_lock_cache_invalidation.rs`.
- Matrix bắt buộc:
  - Đổi source.
  - Đổi dependency version.
  - Đổi policy permission.
  - Đổi manifest metadata không ảnh hưởng semantics.
- PASS:
  - Cache miss/hit đúng theo từng ô matrix đã định nghĩa.
- FAIL:
  - Dùng lại cache khi lẽ ra phải miss, hoặc ngược lại.
- Lệnh chạy:
  - `cargo test --test t_wf_002_lock_cache_invalidation -- --nocapture`

#### [ ] T-RT-001 - Reactor Determinism Repeatability (P0)
- Mục tiêu: reactor mode deterministic phải ổn định qua nhiều lần chạy.
- File dự kiến: `tests/t_rt_001_reactor_repeatability.rs`.
- Steps:
  1. Chạy cùng scenario 50 lần, cùng seed/config.
  2. Thu thập signature timeline mỗi lần.
- PASS:
  - 50/50 run có signature giống nhau.
- FAIL:
  - Có run khác signature/reason dù cùng input.
- Lệnh chạy:
  - `cargo test --test t_rt_001_reactor_repeatability -- --nocapture`

#### [ ] T-RT-002 - Reactor Backpressure/Timeout Safety (P1)
- Mục tiêu: tải cao vẫn fail-honest, không panic/treo.
- File dự kiến: `tests/t_rt_002_reactor_backpressure_timeout.rs`.
- Input:
  - Mailbox depth lớn, burst event cao, timeout thấp.
- PASS:
  - Không panic, lỗi đúng reason code, có audit marker đầy đủ.
- FAIL:
  - Panic, deadlock, hoặc reason code sai contract.
- Lệnh chạy:
  - `cargo test --test t_rt_002_reactor_backpressure_timeout -- --nocapture`

#### [ ] T-PERM-001 - Permission Lattice Precedence (P0)
- Mục tiêu: precedence `deny > allow`, transitive rule đúng cho graph phụ thuộc.
- File dự kiến: `tests/t_perm_001_lattice_precedence.rs`.
- Cases bắt buộc:
  - Parent allow nhưng child deny.
  - Dependency A có quyền, dependency B không được mượn quyền.
  - Module override đối nghịch package rule.
- PASS:
  - Tất cả case cho reason code đúng.
- FAIL:
  - Bypass quyền hoặc kết luận sai precedence.
- Lệnh chạy:
  - `cargo test --test t_perm_001_lattice_precedence -- --nocapture`

#### [ ] T-PERM-002 - Effective Permission Determinism (P1)
- Mục tiêu: report quyền hiệu lực phải ổn định theo cùng input.
- File dự kiến: `tests/t_perm_002_effective_permission_determinism.rs`.
- Steps:
  1. Sinh permissions report 10 lần cho cùng project.
  2. So hash report.
- PASS:
  - 10/10 hash giống nhau.
- FAIL:
  - Hash dao động không do input thay đổi.
- Lệnh chạy:
  - `cargo test --test t_perm_002_effective_permission_determinism -- --nocapture`

#### [ ] T-ERR-001 - Error Taxonomy Lock (P0)
- Mục tiêu: lỗi chuẩn hóa giữ ổn định code/message class.
- File dự kiến: `tests/t_err_001_taxonomy_lock.rs`.
- Input:
  - Bộ case âm: parse/type/runtime/permission/conformance.
- PASS:
  - Mỗi case map đúng `reason_code` đã khóa.
- FAIL:
  - Lệch namespace hoặc code không còn ổn định.
- Lệnh chạy:
  - `cargo test --test t_err_001_taxonomy_lock -- --nocapture`

#### [x] T-FUZZ-001 - Parser Fuzz Non-Panic (P1)
- Mục tiêu: parser không được panic với input ngẫu nhiên.
- File dự kiến: `tests/t_fuzz_001_parser_nonpanic.rs`.
- Input:
  - 2,000 mẫu token/chuỗi sinh ngẫu nhiên có seed cố định.
- PASS:
  - 0 panic; lỗi trả về phải thuộc taxonomy hợp lệ.
- FAIL:
  - Xuất hiện panic hoặc lỗi ngoài taxonomy.
- Lệnh chạy:
  - `cargo test --test t_fuzz_001_parser_nonpanic -- --nocapture`

#### [ ] T-FUZZ-002 - Mini Program Generator End-to-End (P1)
- Mục tiêu: mini program đa dạng chạy qua full flow không crash.
- File dự kiến: `tests/t_fuzz_002_miniprogram_e2e.rs`.
- Input:
  - 500 chương trình nhỏ hợp lệ sinh tự động (seed cố định).
- Steps:
  1. Parse.
  2. Typecheck.
  3. Run (dual).
- PASS:
  - Không panic; kết quả lỗi nếu có phải fail-honest.
- FAIL:
  - Panic, deadlock hoặc kết quả không tái lập.
- Lệnh chạy:
  - `cargo test --test t_fuzz_002_miniprogram_e2e -- --nocapture`

#### [ ] T-PERF-001 - Perf Regression Budget Guard (P1)
- Mục tiêu: chặn regression hiệu năng lõi sau mỗi thay đổi lớn.
- File dự kiến: `tests/t_perf_001_budget_guard.rs`.
- Chỉ số bắt buộc:
  - Runtime steps.
  - Thời gian chạy.
  - Cache hit ratio.
- PASS:
  - Không chỉ số nào vượt ngưỡng cho phép (mặc định 15% so baseline đã khóa).
- FAIL:
  - Vượt ngưỡng mà không có cờ cập nhật baseline hợp lệ.
- Lệnh chạy:
  - `cargo test --test t_perf_001_budget_guard -- --nocapture`

## Trạng thái triển khai backlog

- [x] Đã triển khai và chạy PASS: `T-ALG-001`, `T-ALG-002`, `T-ALG-003`, `T-WF-001`, `T-FUZZ-001`.
- [x] Đã chạy đúng phạm vi chỉ gồm test mới, không re-run bộ cũ.
- [ ] Các test còn lại giữ trạng thái TODO, triển khai theo ưu tiên P0 trước.
- Commands run (đã chạy thật):
  - `cargo test --test t_alg_001_differential_engine`
  - `cargo test --test t_alg_002_metamorphic_invariance`
  - `cargo test --test t_alg_003_metamorphic_divergence`
  - `cargo test --test t_wf_001_state_machine_idempotence`
  - `cargo test --test t_fuzz_001_parser_nonpanic`
- `cargo test --test t_alg_001_differential_engine --test t_alg_002_metamorphic_invariance --test t_alg_003_metamorphic_divergence --test t_wf_001_state_machine_idempotence --test t_fuzz_001_parser_nonpanic`

## Practical command-flow coverage (real OCP usage)

- [x] `T-CLI-REAL-001`: `tests/cli_real_shadow_hive_ops_e2e.rs::cli_real_shadow_command_flow_passes`
  - Scope: `init/check/run --shadow/replay/trace run/profile run`
  - Evidence: verifies generated artifacts, trace/profile outputs, and shadow report message.
- [x] `T-CLI-REAL-002`: `tests/cli_real_shadow_hive_ops_e2e.rs::cli_real_hive_reactor_command_flow_and_locked_guard`
  - Scope: `init --preset agent_swarm_basic`, reactor run with universe/domain, runtime report + replay audit.
  - Guard: tamper `cosmos.lock.v1` hive row and verify locked run fails with `V-HIVE-LOCK-MISMATCH`.
- [x] `T-CLI-REAL-003`: `tests/cli_real_shadow_hive_ops_e2e.rs::cli_real_shadow_digest_stable_across_repeated_runs`
  - Scope: repeat `run --shadow` 5 times on same project.
  - Gate: `shadow_digest` must stay identical across all rounds.
- [x] `T-CLI-REAL-004`: `tests/cli_real_shadow_hive_ops_e2e.rs::cli_real_hive_dispatch_digest_stable_across_repeated_runs`
  - Scope: repeat reactor run 5 times with same universe/domain.
  - Gate: `dispatch_digest256` in runtime report must stay identical across all rounds.

- Supporting real-flow suites already present and re-run:
  - `tests/cli_shadow_preview_v2_e2e.rs`
  - `tests/v1_user_journey_smoke.rs`
  - `projects/ocp/crates/ocp-sdk/tests/v5_w4_hive.rs`

- Commands run for this practical block:
  - `cargo test --test cli_real_shadow_hive_ops_e2e`
  - `cargo test --test cli_real_shadow_hive_ops_e2e` (re-run x3 for flake check, all pass)
  - `cargo test --test cli_real_shadow_hive_ops_e2e --test cli_shadow_preview_v2_e2e --test v1_user_journey_smoke`
  - `cargo test --test cli_shadow_preview_v2_e2e`
  - `cargo test --test v1_user_journey_smoke`
  - `cargo test -p ocp-sdk --test v5_w4_hive`

## CI gate (pre-merge enforcement)

- [x] Added script gate: `tools/ci_ocp_real_ops_gate.ps1`
- [x] Hooked into lane script: `tools/ci_ocp_lane.ps1`
- [x] Added GitHub workflow gate: `.github/workflows/ocp-real-command-flow-gate.yml`
- Commands run:
  - `powershell -ExecutionPolicy Bypass -File tools/ci_ocp_real_ops_gate.ps1`

## Branch protection status

- Attempted to set required status check via GitHub API:
  - `powershell -ExecutionPolicy Bypass -File tools/set_branch_protection_required_check.ps1 -Owner DucHaiten -Repo OCP-OCL -Branch main -RequiredCheckContext 'real command-flow gate'`
  - Result: `HTTP 403` (`Upgrade to GitHub Pro or make this repository public to enable this feature`).
- Local hard enforcement enabled:
  - Added installer: `tools/install_local_real_ops_gate_hook.ps1`
  - Installed hook: `.git/hooks/pre-push`
  - Effect: local `git push` is blocked when `tools/ci_ocp_real_ops_gate.ps1` fails.

## Recheck v1.0 (v100_*) - 2026-03-08

- Full matrix re-run: `65/65 PASS`, `0 FAIL`.
- Result artifacts:
  - `target/ocp/v100_recheck_results_20260308_231437.json`
  - `target/ocp/v100_recheck_summary_20260308_231437.json`
- Fixes applied before final re-run:
  - Synced historical evidence hashes + signature:
    - `contracts/history/evidence_index.v1.json`
    - `contracts/history/evidence_index.v1.sig`
  - Removed legacy marker literal from branch-protection helper defaults:
    - `tools/set_branch_protection_required_check.ps1`

## Installer branding reality check (Windows setup UI)

- Source scripts/contracts already use `OCP`:
  - `installer/windows/ocp-win-x64.iss` (`AppName`, `DefaultDirName`)
  - `contracts/release/v1.0/installer_contract_win.v1.json`
- Added binary branding guard:
  - `tools/release/check_win_installer_branding.ps1`
  - `tools/release/build_win_installer.ps1` now runs branding check after ISCC build.
- Current local installer artifact is stale legacy build:
  - `target/ocp/w100/release/ocp-v1.0.0-setup-win-x64.exe` still contains `OCP-OCL` marker (UTF-16).
  - Verification command:
    - `powershell -ExecutionPolicy Bypass -File tools/release/check_win_installer_branding.ps1 -InstallerPath target/ocp/w100/release/ocp-v1.0.0-setup-win-x64.exe`
  - Result: FAIL (`legacy token detected in installer binary: OCP-OCL`).
