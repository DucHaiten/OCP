# Danh sách FAIL v0.1 -> v0.20

- Nguồn dữ liệu: `E:\OCP-OCL\target\ocp\plan_test_run_results_v0.1_to_v0.20_20260308_202048.json`
- Tổng lệnh: `586`
- PASS: `562`
- FAIL: `18`
- SKIP placeholder: `6`

## FAIL theo plan

| Plan | FAIL | SKIP |
|---|---:|---:|
| OCP-MVP-PLAN-v0.1.md | 0 | 0 |
| OCP-MVP-PLAN-v0.10.md | 1 | 0 |
| OCP-MVP-PLAN-v0.11.md | 0 | 0 |
| OCP-MVP-PLAN-v0.12.md | 0 | 0 |
| OCP-MVP-PLAN-v0.13.md | 0 | 0 |
| OCP-MVP-PLAN-v0.14.md | 1 | 0 |
| OCP-MVP-PLAN-v0.15.md | 0 | 0 |
| OCP-MVP-PLAN-v0.16.md | 0 | 0 |
| OCP-MVP-PLAN-v0.17.md | 0 | 0 |
| OCP-MVP-PLAN-v0.18.md | 0 | 0 |
| OCP-MVP-PLAN-v0.19.md | 0 | 0 |
| OCP-MVP-PLAN-v0.2.md | 0 | 0 |
| OCP-MVP-PLAN-v0.20.md | 0 | 0 |
| OCP-MVP-PLAN-v0.3.md | 3 | 1 |
| OCP-MVP-PLAN-v0.4.md | 5 | 2 |
| OCP-MVP-PLAN-v0.5.md | 6 | 1 |
| OCP-MVP-PLAN-v0.6.md | 2 | 0 |
| OCP-MVP-PLAN-v0.7.1.md | 0 | 0 |
| OCP-MVP-PLAN-v0.7.2.md | 0 | 0 |
| OCP-MVP-PLAN-v0.7.3.md | 0 | 1 |
| OCP-MVP-PLAN-v0.8.md | 0 | 1 |
| OCP-MVP-PLAN-v0.9.md | 0 | 0 |

## Top lệnh FAIL (theo tần suất)

| Count | Command |
|---:|---|
| 4 | `cargo test -p ocp-runtime-core -p ocp-sdk -p ocp-cli` |
| 3 | `cargo test -p ocp-sdk --test w9_conformance` |
| 2 | `cargo test -p ocp-sdk --test w4_supply` |
| 2 | `cargo test -p ocp-sdk` |
| 1 | `$env:OCP_UPDATE_V14_CONTRACT_SNAPSHOT='1'; cargo test --test stability_contracts_v14` |
| 1 | `cargo test --workspace` |
| 1 | `cargo clippy --workspace --all-targets -- -D warnings` |
| 1 | `cargo test -p ocp-cli --test m5_conformance` |
| 1 | `cargo test -p ocp-sdk -p ocp-cli` |
| 1 | `cargo test -p ocp-runtime-core -p ocp-runtime-rt -p ocp-sdk -p ocp-cli` |
| 1 | `cargo clippy -p ocp-runtime-core -p ocp-runtime-rt -p ocp-sdk -p ocp-cli --all-targets -- -D warnings` |

## Chi tiết từng FAIL

### OCP-MVP-PLAN-v0.10.md (1 fail)

1. `cargo test -p ocp-sdk --test w4_supply`
   - exit: `1`
   - hint: `test w4_build_verify_publish_fetch_run_artifact_pass ... FAILED`

### OCP-MVP-PLAN-v0.14.md (1 fail)

1. `$env:OCP_UPDATE_V14_CONTRACT_SNAPSHOT='1'; cargo test --test stability_contracts_v14`
   - exit: `1`
   - hint: `test stability_contract_snapshot_v14_additive_only ... FAILED`

### OCP-MVP-PLAN-v0.3.md (3 fail)

1. `cargo test -p ocp-runtime-core -p ocp-sdk -p ocp-cli`
   - exit: `1`
   - hint: `thread 'w4_locked_fails_when_dep_signature_missing' (13012) panicked at projects\ocp\crates\ocp-sdk\tests\w4_supply.rs:97:5:`
2. `cargo test -p ocp-sdk`
   - exit: `1`
   - hint: `thread 'w4_build_verify_publish_fetch_run_artifact_pass' (29552) panicked at projects\ocp\crates\ocp-sdk\tests\w4_supply.rs:51:53:`
3. `cargo test --workspace`
   - exit: `1`
   - hint: `error: test failed, to rerun pass `-p ocp-sdk --test w4_supply``

### OCP-MVP-PLAN-v0.4.md (5 fail)

1. `cargo clippy --workspace --all-targets -- -D warnings`
   - exit: `1`
   - hint: `error: unnecessary closure used to substitute value for `Option::None``
2. `cargo test -p ocp-cli --test m5_conformance`
   - exit: `1`
   - hint: `error: no test target named `m5_conformance` in `ocp-cli` package`
3. `cargo test -p ocp-runtime-core -p ocp-sdk -p ocp-cli`
   - exit: `1`
   - hint: `thread 'w4_locked_fails_when_dep_signature_missing' (30216) panicked at projects\ocp\crates\ocp-sdk\tests\w4_supply.rs:97:5:`
4. `cargo test -p ocp-sdk --test w4_supply`
   - exit: `1`
   - hint: `test w4_build_verify_publish_fetch_run_artifact_pass ... FAILED`
5. `cargo test -p ocp-sdk --test w9_conformance`
   - exit: `1`
   - hint: `thread 'w9_conformance_runner_locked_dual_pass' (27572) panicked at projects\ocp\crates\ocp-sdk\tests\w9_conformance.rs:62:5:`

### OCP-MVP-PLAN-v0.5.md (6 fail)

1. `cargo test -p ocp-runtime-core -p ocp-sdk -p ocp-cli`
   - exit: `1`
   - hint: `thread 'w4_locked_fails_when_dep_signature_missing' (25964) panicked at projects\ocp\crates\ocp-sdk\tests\w4_supply.rs:97:5:`
2. `cargo test -p ocp-sdk --test w9_conformance`
   - exit: `1`
   - hint: `thread 'w9_conformance_runner_locked_dual_pass' (5388) panicked at projects\ocp\crates\ocp-sdk\tests\w9_conformance.rs:62:5:`
3. `cargo test -p ocp-sdk`
   - exit: `1`
   - hint: `thread 'w4_build_verify_publish_fetch_run_artifact_pass' (4476) panicked at projects\ocp\crates\ocp-sdk\tests\w4_supply.rs:51:53:`
4. `cargo test -p ocp-sdk -p ocp-cli`
   - exit: `1`
   - hint: `thread 'w4_build_verify_publish_fetch_run_artifact_pass' (7104) panicked at projects\ocp\crates\ocp-sdk\tests\w4_supply.rs:51:53:`
5. `cargo test -p ocp-runtime-core -p ocp-runtime-rt -p ocp-sdk -p ocp-cli`
   - exit: `1`
   - hint: `error: package ID specification `ocp-runtime-rt` did not match any packages`
6. `cargo clippy -p ocp-runtime-core -p ocp-runtime-rt -p ocp-sdk -p ocp-cli --all-targets -- -D warnings`
   - exit: `1`
   - hint: `error: package ID specification `ocp-runtime-rt` did not match any packages`

### OCP-MVP-PLAN-v0.6.md (2 fail)

1. `cargo test -p ocp-runtime-core -p ocp-sdk -p ocp-cli`
   - exit: `1`
   - hint: `thread 'w4_locked_fails_when_dep_signature_missing' (26852) panicked at projects\ocp\crates\ocp-sdk\tests\w4_supply.rs:97:5:`
2. `cargo test -p ocp-sdk --test w9_conformance`
   - exit: `1`
   - hint: `thread 'w9_conformance_runner_locked_dual_pass' (30268) panicked at projects\ocp\crates\ocp-sdk\tests\w9_conformance.rs:62:5:`

## Placeholder bị SKIP

1. `OCP-MVP-PLAN-v0.3.md` - `cargo clippy ... -D warnings`
2. `OCP-MVP-PLAN-v0.4.md` - `cargo clippy ...`
3. `OCP-MVP-PLAN-v0.4.md` - `cargo clippy ... -D warnings`
4. `OCP-MVP-PLAN-v0.5.md` - `cargo test/clippy`
5. `OCP-MVP-PLAN-v0.7.3.md` - `cargo test --test <targeted_suite>`
6. `OCP-MVP-PLAN-v0.8.md` - `cargo test --test <targeted_suite>`
