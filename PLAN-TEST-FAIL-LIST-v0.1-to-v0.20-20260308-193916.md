# Danh sách FAIL v0.1 -> v0.20

- Nguồn dữ liệu: `E:\OCP-OCL\target\ocp\plan_test_run_results_v0.1_to_v0.20_20260308_193916.json`
- Tổng lệnh: `586`
- PASS: `528`
- FAIL: `52`
- SKIP placeholder: `6`

## FAIL theo plan

| Plan | FAIL | SKIP |
|---|---:|---:|
| OCP-MVP-PLAN-v0.1.md | 0 | 0 |
| OCP-MVP-PLAN-v0.10.md | 1 | 0 |
| OCP-MVP-PLAN-v0.11.md | 3 | 0 |
| OCP-MVP-PLAN-v0.12.md | 0 | 0 |
| OCP-MVP-PLAN-v0.13.md | 0 | 0 |
| OCP-MVP-PLAN-v0.14.md | 5 | 0 |
| OCP-MVP-PLAN-v0.15.md | 0 | 0 |
| OCP-MVP-PLAN-v0.16.md | 0 | 0 |
| OCP-MVP-PLAN-v0.17.md | 0 | 0 |
| OCP-MVP-PLAN-v0.18.md | 0 | 0 |
| OCP-MVP-PLAN-v0.19.md | 0 | 0 |
| OCP-MVP-PLAN-v0.2.md | 0 | 0 |
| OCP-MVP-PLAN-v0.20.md | 0 | 0 |
| OCP-MVP-PLAN-v0.3.md | 6 | 1 |
| OCP-MVP-PLAN-v0.4.md | 14 | 2 |
| OCP-MVP-PLAN-v0.5.md | 13 | 1 |
| OCP-MVP-PLAN-v0.6.md | 5 | 0 |
| OCP-MVP-PLAN-v0.7.1.md | 4 | 0 |
| OCP-MVP-PLAN-v0.7.2.md | 1 | 0 |
| OCP-MVP-PLAN-v0.7.3.md | 0 | 1 |
| OCP-MVP-PLAN-v0.8.md | 0 | 1 |
| OCP-MVP-PLAN-v0.9.md | 0 | 0 |

## Top lệnh FAIL (theo tần suất)

| Count | Command |
|---:|---|
| 5 | `cargo test -p ocp-cli` |
| 4 | `cargo test -p ocp-runtime-core -p ocp-sdk -p ocp-cli` |
| 4 | `cargo clippy -p ocp-runtime-core -p ocp-sdk -p ocp-cli --all-targets -- -D warnings` |
| 3 | `cargo test -p ocp-sdk --test w9_conformance` |
| 2 | `cargo test -p ocp-sdk --test w4_supply` |
| 2 | `cargo test -p ocp-sdk` |
| 2 | `cargo test -p ocp-sdk --test m5_conformance` |
| 2 | `cargo clippy -p ocp-sdk -p ocp-cli --all-targets -- -D warnings` |
| 2 | `cargo test -p ocp-cli w9_cli_` |
| 2 | `cargo test -p ocp-sdk --test v5_w3_shadow` |
| 1 | `cargo test -p ocp-cli w5_cli_trace_and_profile_non_reactor_pass --offline` |
| 1 | `cargo test -p ocp-cli --offline` |
| 1 | `cargo test -p ocp-cli v7_g_cli_replay_signature_match_pass --offline` |
| 1 | `cargo test -p ocp-cli w14_cli_conformance_list_alias_json_pass -- --nocapture` |
| 1 | `cargo test -p ocp-cli w14_cli_test_conformance_list_mode_pass -- --nocapture` |
| 1 | `cargo test -p ocp-cli w14_cli_test_conformance_run_subcommand_pass -- --nocapture` |
| 1 | `cargo test -p ocp-cli w9_cli_conformance_single_scenario_pass -- --nocapture` |
| 1 | `$env:OCP_UPDATE_V14_CONTRACT_SNAPSHOT='1'; cargo test --test stability_contracts_v14` |
| 1 | `cargo test --workspace` |
| 1 | `cargo clippy --workspace --all-targets -- -D warnings` |
| 1 | `cargo test -p ocp-cli --test m5_conformance` |
| 1 | `cargo test -p ocp-sdk --test m3_packaging` |
| 1 | `cargo test -p ocp-cli w5_` |
| 1 | `cargo test -p ocp-cli w6_` |
| 1 | `cargo test -p ocp-cli w7_` |
| 1 | `cargo test -p ocp-cli v5_w3_cli` |
| 1 | `cargo test -p ocp-sdk -p ocp-cli` |
| 1 | `cargo test -p ocp-cli w6` |
| 1 | `cargo test -p ocp-cli w9` |
| 1 | `cargo test -p ocp-runtime-core -p ocp-runtime-rt -p ocp-sdk -p ocp-cli` |
| 1 | `cargo clippy -p ocp-runtime-core -p ocp-runtime-rt -p ocp-sdk -p ocp-cli --all-targets -- -D warnings` |
| 1 | `cargo test -p ocp-cli v7_a_cli_run_emits_artifacts_on_success` |
| 1 | `cargo test -p ocp-cli v7_a_cli_run_emits_artifacts_on_fail` |
| 1 | `cargo test -p ocp-cli v7_g_cli_replay_` |

## Chi tiết từng FAIL

### OCP-MVP-PLAN-v0.10.md (1 fail)

1. `cargo test -p ocp-sdk --test w4_supply`
   - exit: `1`
   - hint: `test w4_build_verify_publish_fetch_run_artifact_pass ... FAILED`

### OCP-MVP-PLAN-v0.11.md (3 fail)

1. `cargo test -p ocp-cli w5_cli_trace_and_profile_non_reactor_pass --offline`
   - exit: `1`
   - hint: `error: could not compile `ocp-cli` (bin "ocp" test) due to 11 previous errors`
2. `cargo test -p ocp-cli --offline`
   - exit: `1`
   - hint: `error: could not compile `ocp-cli` (bin "ocp" test) due to 11 previous errors`
3. `cargo test -p ocp-cli v7_g_cli_replay_signature_match_pass --offline`
   - exit: `1`
   - hint: `error: could not compile `ocp-cli` (bin "ocp" test) due to 11 previous errors`

### OCP-MVP-PLAN-v0.14.md (5 fail)

1. `cargo test -p ocp-cli w14_cli_conformance_list_alias_json_pass -- --nocapture`
   - exit: `1`
   - hint: `error: could not compile `ocp-cli` (bin "ocp" test) due to 11 previous errors`
2. `cargo test -p ocp-cli w14_cli_test_conformance_list_mode_pass -- --nocapture`
   - exit: `1`
   - hint: `error: could not compile `ocp-cli` (bin "ocp" test) due to 11 previous errors`
3. `cargo test -p ocp-cli w14_cli_test_conformance_run_subcommand_pass -- --nocapture`
   - exit: `1`
   - hint: `error: could not compile `ocp-cli` (bin "ocp" test) due to 11 previous errors`
4. `cargo test -p ocp-cli w9_cli_conformance_single_scenario_pass -- --nocapture`
   - exit: `1`
   - hint: `error: could not compile `ocp-cli` (bin "ocp" test) due to 11 previous errors`
5. `$env:OCP_UPDATE_V14_CONTRACT_SNAPSHOT='1'; cargo test --test stability_contracts_v14`
   - exit: `1`
   - hint: `test stability_contract_snapshot_v14_additive_only ... FAILED`

### OCP-MVP-PLAN-v0.3.md (6 fail)

1. `cargo test -p ocp-runtime-core -p ocp-sdk -p ocp-cli`
   - exit: `1`
   - hint: `error: could not compile `ocp-cli` (bin "ocp" test) due to 11 previous errors`
2. `cargo clippy -p ocp-runtime-core -p ocp-sdk -p ocp-cli --all-targets -- -D warnings`
   - exit: `1`
   - hint: `error: field assignment outside of initializer for an instance created with Default::default()`
3. `cargo test -p ocp-sdk`
   - exit: `1`
   - hint: `error: could not compile `ocp-sdk` (test "v5_w3_shadow") due to 2 previous errors`
4. `cargo test -p ocp-cli`
   - exit: `1`
   - hint: `error: could not compile `ocp-cli` (bin "ocp" test) due to 11 previous errors`
5. `cargo test --workspace`
   - exit: `1`
   - hint: `error: could not compile `ocp-cli` (bin "ocp" test) due to 11 previous errors`
6. `cargo test -p ocp-sdk --test m5_conformance`
   - exit: `1`
   - hint: `thread 'm5_demo_apps_end_to_end_cli_and_fetch' (16504) panicked at projects\ocp\crates\ocp-sdk\tests\m5_conformance.rs:74:59:`

### OCP-MVP-PLAN-v0.4.md (14 fail)

1. `cargo clippy --workspace --all-targets -- -D warnings`
   - exit: `1`
   - hint: `error: field assignment outside of initializer for an instance created with Default::default()`
2. `cargo test -p ocp-cli --test m5_conformance`
   - exit: `1`
   - hint: `error: no test target named `m5_conformance` in `ocp-cli` package`
3. `cargo test -p ocp-sdk --test m5_conformance`
   - exit: `1`
   - hint: `thread 'm5_demo_apps_end_to_end_cli_and_fetch' (9504) panicked at projects\ocp\crates\ocp-sdk\tests\m5_conformance.rs:74:59:`
4. `cargo test -p ocp-cli`
   - exit: `1`
   - hint: `error: could not compile `ocp-cli` (bin "ocp" test) due to 11 previous errors`
5. `cargo clippy -p ocp-runtime-core -p ocp-sdk -p ocp-cli --all-targets -- -D warnings`
   - exit: `1`
   - hint: `error: field assignment outside of initializer for an instance created with Default::default()`
6. `cargo test -p ocp-runtime-core -p ocp-sdk -p ocp-cli`
   - exit: `1`
   - hint: `error: could not compile `ocp-cli` (bin "ocp" test) due to 11 previous errors`
7. `cargo test -p ocp-sdk --test m3_packaging`
   - exit: `1`
   - hint: `test m3_check_locked_passes_after_sync ... FAILED`
8. `cargo test -p ocp-sdk --test w4_supply`
   - exit: `1`
   - hint: `test w4_build_verify_publish_fetch_run_artifact_pass ... FAILED`
9. `cargo test -p ocp-cli w5_`
   - exit: `1`
   - hint: `error: could not compile `ocp-cli` (bin "ocp" test) due to 11 previous errors`
10. `cargo clippy -p ocp-sdk -p ocp-cli --all-targets -- -D warnings`
   - exit: `1`
   - hint: `error: field assignment outside of initializer for an instance created with Default::default()`
11. `cargo test -p ocp-cli w6_`
   - exit: `1`
   - hint: `error: could not compile `ocp-cli` (bin "ocp" test) due to 11 previous errors`
12. `cargo test -p ocp-cli w7_`
   - exit: `1`
   - hint: `error: could not compile `ocp-cli` (bin "ocp" test) due to 11 previous errors`
13. `cargo test -p ocp-sdk --test w9_conformance`
   - exit: `1`
   - hint: `thread 'w9_conformance_runner_locked_dual_pass' (16456) panicked at projects\ocp\crates\ocp-sdk\tests\w9_conformance.rs:62:5:`
14. `cargo test -p ocp-cli w9_cli_`
   - exit: `1`
   - hint: `error: could not compile `ocp-cli` (bin "ocp" test) due to 11 previous errors`

### OCP-MVP-PLAN-v0.5.md (13 fail)

1. `cargo test -p ocp-runtime-core -p ocp-sdk -p ocp-cli`
   - exit: `1`
   - hint: `error: could not compile `ocp-cli` (bin "ocp" test) due to 11 previous errors`
2. `cargo clippy -p ocp-runtime-core -p ocp-sdk -p ocp-cli --all-targets -- -D warnings`
   - exit: `1`
   - hint: `error: field assignment outside of initializer for an instance created with Default::default()`
3. `cargo test -p ocp-sdk --test v5_w3_shadow`
   - exit: `1`
   - hint: `error: could not compile `ocp-sdk` (test "v5_w3_shadow") due to 2 previous errors`
4. `cargo test -p ocp-cli`
   - exit: `1`
   - hint: `error: could not compile `ocp-cli` (bin "ocp" test) due to 11 previous errors`
5. `cargo test -p ocp-sdk --test w9_conformance`
   - exit: `1`
   - hint: `thread 'w9_conformance_runner_locked_dual_pass' (7916) panicked at projects\ocp\crates\ocp-sdk\tests\w9_conformance.rs:62:5:`
6. `cargo clippy -p ocp-sdk -p ocp-cli --all-targets -- -D warnings`
   - exit: `1`
   - hint: `error: field assignment outside of initializer for an instance created with Default::default()`
7. `cargo test -p ocp-cli v5_w3_cli`
   - exit: `1`
   - hint: `error: could not compile `ocp-cli` (bin "ocp" test) due to 11 previous errors`
8. `cargo test -p ocp-sdk`
   - exit: `1`
   - hint: `error: could not compile `ocp-sdk` (test "v5_w3_shadow") due to 2 previous errors`
9. `cargo test -p ocp-sdk -p ocp-cli`
   - exit: `1`
   - hint: `error: could not compile `ocp-cli` (bin "ocp" test) due to 11 previous errors`
10. `cargo test -p ocp-cli w6`
   - exit: `1`
   - hint: `error: could not compile `ocp-cli` (bin "ocp" test) due to 11 previous errors`
11. `cargo test -p ocp-cli w9`
   - exit: `1`
   - hint: `error: could not compile `ocp-cli` (bin "ocp" test) due to 11 previous errors`
12. `cargo test -p ocp-runtime-core -p ocp-runtime-rt -p ocp-sdk -p ocp-cli`
   - exit: `1`
   - hint: `error: package ID specification `ocp-runtime-rt` did not match any packages`
13. `cargo clippy -p ocp-runtime-core -p ocp-runtime-rt -p ocp-sdk -p ocp-cli --all-targets -- -D warnings`
   - exit: `1`
   - hint: `error: package ID specification `ocp-runtime-rt` did not match any packages`

### OCP-MVP-PLAN-v0.6.md (5 fail)

1. `cargo test -p ocp-runtime-core -p ocp-sdk -p ocp-cli`
   - exit: `1`
   - hint: `error: could not compile `ocp-cli` (bin "ocp" test) due to 11 previous errors`
2. `cargo clippy -p ocp-runtime-core -p ocp-sdk -p ocp-cli --all-targets -- -D warnings`
   - exit: `1`
   - hint: `error: field assignment outside of initializer for an instance created with Default::default()`
3. `cargo test -p ocp-sdk --test w9_conformance`
   - exit: `1`
   - hint: `thread 'w9_conformance_runner_locked_dual_pass' (12956) panicked at projects\ocp\crates\ocp-sdk\tests\w9_conformance.rs:62:5:`
4. `cargo test -p ocp-cli w9_cli_`
   - exit: `1`
   - hint: `error: could not compile `ocp-cli` (bin "ocp" test) due to 11 previous errors`
5. `cargo test -p ocp-sdk --test v5_w3_shadow`
   - exit: `1`
   - hint: `error: could not compile `ocp-sdk` (test "v5_w3_shadow") due to 2 previous errors`

### OCP-MVP-PLAN-v0.7.1.md (4 fail)

1. `cargo test -p ocp-cli v7_a_cli_run_emits_artifacts_on_success`
   - exit: `1`
   - hint: `error: could not compile `ocp-cli` (bin "ocp" test) due to 11 previous errors`
2. `cargo test -p ocp-cli v7_a_cli_run_emits_artifacts_on_fail`
   - exit: `1`
   - hint: `error: could not compile `ocp-cli` (bin "ocp" test) due to 11 previous errors`
3. `cargo test -p ocp-cli v7_g_cli_replay_`
   - exit: `1`
   - hint: `error: could not compile `ocp-cli` (bin "ocp" test) due to 11 previous errors`
4. `cargo test -p ocp-cli`
   - exit: `1`
   - hint: `error: could not compile `ocp-cli` (bin "ocp" test) due to 11 previous errors`

### OCP-MVP-PLAN-v0.7.2.md (1 fail)

1. `cargo test -p ocp-cli`
   - exit: `1`
   - hint: `error: could not compile `ocp-cli` (bin "ocp" test) due to 11 previous errors`

## Placeholder bị SKIP

1. `OCP-MVP-PLAN-v0.3.md` - `cargo clippy ... -D warnings`
2. `OCP-MVP-PLAN-v0.4.md` - `cargo clippy ...`
3. `OCP-MVP-PLAN-v0.4.md` - `cargo clippy ... -D warnings`
4. `OCP-MVP-PLAN-v0.5.md` - `cargo test/clippy`
5. `OCP-MVP-PLAN-v0.7.3.md` - `cargo test --test <targeted_suite>`
6. `OCP-MVP-PLAN-v0.8.md` - `cargo test --test <targeted_suite>`
