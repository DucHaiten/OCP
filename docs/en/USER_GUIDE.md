# OCP v1.0 User Guide (EN)

This is the main entry point for OCP v1.0 users.  
Goal: read once, then work from A to Z without hunting through many other files.

Scope:
- Install and verify release integrity.
- Learn enough OCP to write a basic program from scratch.
- Use the canonical CLI flow: `init -> lock -> perm -> build/verify attest -> run/replay -> debug`.
- Handle routine operations such as `doctor`, `fix`, and `budget analyze`.
- Diagnose the first real failures you will meet.

Notes:
- This file combines language learning and quick reference in one place.
- Advanced material is pushed below the onboarding sections.

Example labels used in this file:
- `Canonical pattern`: the pattern new users should learn and copy when writing new logic.
- `Template currently shipped`: mirrors the scaffold the CLI really ships today.
- `Smoke scaffold`: intentionally short sample for fast init/run/replay checks, not the cleanest long-term pattern.

---

## Table of Contents
- [Quick reference](#quick-reference)
- [One-file reading path (A->Z)](#learning-path)
- [Install](#install)
- [Quickstart](#quickstart)
- [Learn OCP](#learn-ocp)
- [5 code patterns you will copy often](#copy-patterns)
- [Template manifests](#template-manifests)
- [Capability starter pack](#capability-starter-pack)
- [Short language reference (runnable examples)](#language-reference-short)
- [Complete mini-game sample project](#mini-game-project)
- [Complete tool-http sample project](#tool-http-project)
- [Language Quickstart (write your first OCP code)](#language-quickstart)
- [Foundations you should know to use OCP](#concepts)
- [Permissions](#permissions)
- [Lock Trust Attest](#lock-trust-attest)
- [Run Replay](#run-replay)
- [Debug](#debug)
- [Full command catalog](#commands)
- [Important extended public surface](#public-surface)
- [Compose / Phenotype](#compose-phenotype)
- [Organ / Kit / Preset](#organ-kit-preset)
- [Editor workflow (VSCode/LSP/DAP)](#editor-workflow)
- [Reactor workflow (run/trace/profile)](#reactor-workflow)
- [Recommended workflow](#recommended-workflow)
- [Troubleshooting](#troubleshooting)
- [Advanced appendix](#advanced-appendix)
- [Reference](#reference)
- [Maintainer Guide](#maintainer-guide)

---

<a id="quick-reference"></a>
## Quick reference

If you need to start immediately, follow this order:
1. Install and verify the release (`Install`).
2. Write and run your first program (`Learn OCP` + `Language Quickstart`).
3. Run the canonical flow (`Quickstart` -> `Permissions` -> `Lock Trust Attest` -> `Run Replay` -> `Debug`).
4. If you get stuck, go to `Troubleshooting`.

Minimum command chain for a project:

```bash
ocp init <project_dir> --template tool-cli
ocp lock sync <project_dir>
ocp lock sign <project_dir> --key <keyid>
ocp lock verify <project_dir> --lock deps.lock.v3
ocp perm snapshot <project_dir>
ocp build <project_dir> --attest
ocp run <project_dir>
ocp replay <artifact_dir>
```

<a id="learning-path"></a>
## One-file reading path (A->Z)

Read in this order:
1. `Install`
2. `Quickstart`
3. `Learn OCP` + `Language Quickstart`
4. `Foundations you should know` (sections 1..6)
5. `Permissions` -> `Lock Trust Attest` -> `Run Replay` -> `Debug`
6. `Troubleshooting`
7. Only then move into the `Advanced appendix`

---

<a id="install"></a>
## [install] Install

### 1) Environment prerequisites
- Supported operating systems: Windows, Linux, macOS (per the v1.0 release profile).
- Write permission inside your project directory.
- VSCode only matters if you want the VSIX extension.

### 2) Install on Windows (installer)
1. Download `ocp-v1.0.0-setup-win-x64.exe` from the official GitHub Release.
2. Run the installer and choose:
   - the install directory on the drive you want;
   - whether to add `ocp` to `PATH`;
   - whether to install the VSCode extension immediately if VSCode is detected.
3. Open a new terminal and run:

```bash
ocp --version
```

If the command is not recognized:
- reopen the terminal;
- recheck `PATH`.

Installer scope:
- OCP icon
- install-directory picker
- explicit `PATH` confirmation
- `ocp.exe`, `ocp-lsp.exe`, `ocp-dap.exe`
- bundled `ocp-vscode-v1.0.0.vsix`

### 3) Portable install (Windows/Linux/macOS)
1. Download the correct portable package.
2. Extract it to a stable directory.
3. Add the directory containing `ocp` to `PATH`.
4. Verify:

```bash
ocp --version
```

### 4) Verify release integrity (required before production)
Minimum files:
- `release_artifact_manifest.json`
- `release_artifact_manifest.sig`
- `SHA256SUMS`
- `SHA256SUMS.sig`

Quick flow:
1. Download those four files together with the asset you want.
2. Compare the asset SHA256 against `SHA256SUMS`.
3. Verify the trust root first:

```bash
ocp verify --contract-signature contracts/security/v1.0/signing_trust_root.v1.json.sig
```

Expected output example:
- `verify contract signature ok (...)`

4. Verify the release signatures:
   - `schema = ocp.release.file.sig.v1`
   - `pubkey_id` and `trust_epoch` must belong to the trust root
   - `file_hash_sha256` must match the referenced file
5. Only install when hash and signature both verify.

Example hash commands on Windows:

```powershell
Get-FileHash release_artifact_manifest.json -Algorithm SHA256
Get-FileHash SHA256SUMS -Algorithm SHA256
```

If anything mismatches, stop immediately.

Deep-dive companion note:
`docs/en/security/verify-download.md`

### 5) Install the VSCode extension
If you use VSCode:
1. If the installer already installed the extension, reopen VSCode.
2. Otherwise use `ocp-vscode-v1.0.0.vsix`.
3. Install the VSIX in VSCode.
4. Open a `.ocp` file and check syntax highlighting.

---

<a id="quickstart"></a>
## [quickstart] Quickstart

### 1) Initialize a project

```bash
ocp init hello --template tool-cli
cd hello
```

Purpose:
- create a standard OCP project skeleton;
- make Quickstart stable by pinning `--template tool-cli`.

### 2) Sync the lock before running

```bash
ocp lock sync <project_dir>
ocp lock sign <project_dir> --key <keyid>
ocp lock verify <project_dir> --lock deps.lock.v3
```

Purpose:
- `lock sync`: update the lockfile;
- `lock sign`: produce the signature strict flow expects;
- `lock verify`: validate lock/trust before build and run.

### 3) Snapshot and review permissions

```bash
ocp perm snapshot <project_dir> --out-dir <perm_dir>
ocp perm diff <old_snapshot> <new_snapshot> --out <diff_report.json> --approval <permissions.approval.toml>
ocp perm approve <diff_report.json> --approval <permissions.approval.toml> --by <id> --date <YYYY-MM-DD>
```

Recommended baseline/current example:

```bash
ocp perm snapshot . --out-dir ./.ocp_perm/baseline
ocp perm snapshot . --out-dir ./.ocp_perm/current
ocp perm diff ./.ocp_perm/baseline/permissions.snapshot.json ./.ocp_perm/current/permissions.snapshot.json --out ./.ocp_perm/permission_diff_report.json --approval permissions.approval.toml
ocp perm approve ./.ocp_perm/permission_diff_report.json --approval permissions.approval.toml --by dev.local --date 2026-03-07
```

### 4) Build + verify attest

```bash
ocp build <project_dir> --attest
ocp verify --attest <artifact_dir>
```

### 5) Run and replay

```bash
ocp run <project_dir>
ocp replay ./.ocp_artifacts/<run_id>/
```

### 6) Debug a failure

```bash
ocp dbg <artifact_dir>
```

### 7) Fast path for file/state tools (v0.7.2)

```bash
ocp init my-tool --template tool-cli
cd my-tool
ocp test . --golden fixtures/expected --clean
ocp run .
ocp replay ./.ocp_artifacts/<run_id>/
```

### 8) Fast path for mini-game / shadow-preview (v0.7.3)

```bash
ocp init demo-game --template mini-game
ocp run demo-game
ocp replay ./.ocp_artifacts/<run_id>/
```

Or:

```bash
ocp init demo-shadow --template shadow-preview
ocp run demo-shadow --shadow branch-a --shadow-policy forbid_commit
ocp replay ./.ocp_artifacts/<run_id>/
```

---

<a id="learn-ocp"></a>
## Learn OCP

Goal: understand the language model well enough to write a basic OCP program instead of only editing a template by trial and error.

### OCP in 90 seconds
- `observe(...)`: get data or an effect handle from an authorized capability.
- `match` on `Result4`: decide what to do with the outcome.
- `commit(...)`: accept a side effect only when the contract and policy allow it.
- OCP does not assume IO always succeeds.
- `replay` is reproducibility evidence, not a bonus feature.

### 5 rules that let you write OCP
1. If you need external data, use `observe`.
2. After `observe`, always handle the `Result4` outcome.
3. Only `commit` on branches that the capability and lane allow.
4. Treat `INSUFFICIENT` and `DEFERRED` as fail-honest states.
5. After a run, verify with replay.

### Baseline v1.0 syntax (safe for onboarding)
This is the onboarding baseline, not the full language specification.

Guaranteed public workflow constructs:
- `module ...;`
- `let`
- record/list/string/number/bool literals
- `observe(...) -> result;`
- `match result { OK | DEGRADED | INSUFFICIENT | DEFERRED => ... }`
- `commit(result)`
- `condition(true|false)`

### Extended language surface (public v1.0, read after baseline)
Real v1.0 code may also use:
- `import` / `export`
- `fn` / `return`
- bounded loops such as `repeat`, `for ... cap N`, and compatibility wording around `for-range`
- list/map work
- `?`, `try/else`, `guard`, and `Result4` field access (`r.kind`, `r.reason_code`, `r.audit`)
- reactor tick integration

Rule:
1. when you open new syntax surface, run `ocp check <project_dir> --locked`;
2. rerun `ocp run` + `ocp replay` after refactoring.

### Minimum syntax you can use to start independently
- statements end with `;`
- blocks use `{ ... }`
- beginner-facing `match` should handle all 4 kinds
- `repeat N` and `for item in xs cap N` are bounded loops
- new code should prefer typed records over `ctx("...")`

### Standard `Result4` table

| Kind | Meaning | Beginner-safe next step |
| --- | --- | --- |
| `OK` | success under contract | continue, and `commit` if this is a side effect you want |
| `DEGRADED` | partial/limited success | treat carefully; for onboarding, default to non-commit |
| `INSUFFICIENT` | not enough permission, budget, or capability | stop and fix the cause |
| `DEFERRED` | intentionally postponed by runtime/policy | stop and handle later |

Stable fields:
- `r.kind`
- `r.reason_code`
- `r.audit`

### Common capability table for new users

| Capability | Use case | Commit? |
| --- | --- | --- |
| `std.fs.read_text` | read text/JSON input | no |
| `std.fs.mkdir` | create output directory | yes |
| `std.fs.write_text` | write text/JSON/report output | yes |
| `std.kv.put` | store small deterministic state | yes |
| `std.time.tick_info` | get deterministic logical time | no |

### Smallest possible OCP program

```ocp
module app.hello;
condition(true);
```

### Example 1: one observe + one match (no commit)

```ocp
module app.read_only;

let rd_req = { path: "./fixtures/in/sample.json" };
observe("std.fs.read_text", "tier2", rd_req, budget(5)) -> rd;

match rd {
  OK => { condition(true); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}
```

### Example 2: two chained steps (mkdir -> write_text)

```ocp
module app.pipeline;

let mk_req = { path: "./out", recursive: true };
observe("std.fs.mkdir", "tier2", mk_req, budget(5)) -> mk;
match mk {
  OK => { commit(mk); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}

let wr_req = { path: "./out/out.json", text: "{\"status\":\"OK\"}", overwrite: true };
observe("std.fs.write_text", "tier2", wr_req, budget(5)) -> wr;
match wr {
  OK => { commit(wr); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}

condition(true);
```

### Example 3: read input -> write report

```ocp
module app.report;

let rd_req = { path: "./fixtures/in/sample.txt" };
observe("std.fs.read_text", "tier2", rd_req, budget(5)) -> rd;

match rd {
  OK => {
    let wr_req = { path: "./out/report.txt", text: "report generated", overwrite: true };
    observe("std.fs.write_text", "tier2", wr_req, budget(5)) -> wr;

    match wr {
      OK => { commit(wr); condition(true); }
      DEGRADED => { condition(false); }
      INSUFFICIENT => { condition(false); }
      DEFERRED => { condition(false); }
    }
  }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}
```

### Minimum complete syntax (enough to start writing independently)
This is the "minimal but sufficient" part so you do not get stuck guessing syntax.
In this section:
- `module`, `let`, record/list literals, `observe`, `match`, `commit`, `condition` are the beginner baseline.
- `repeat` and `for ... cap N` are public extended syntax that becomes useful as soon as you start handling bounded loops or small batches.

Shortest syntax survey:

```ocp
// comment 1 dòng
module app.demo;

import toolkit.api.math;

fn dep_ready() {
  return true;
}

let ok = dep_ready();
let xs = [1, 2, 3];
let req = { path: "./out/out.json", text: "{\"status\":\"OK\"}", overwrite: true };

repeat 2 { let ping = true; }
for item in xs cap 2 { let seen = item; }

observe("std.fs.write_text", "tier2", req, budget(5)) -> wr;
match wr {
  OK => { commit(wr); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}
```

### From examples to your own code (checklist)
1. Choose the capability key.
2. Write the matching request record.
3. Set a reasonable budget.
4. `observe`.
5. `match` the 4 kinds.
6. Decide which branch may `commit`.
7. `ocp run <project_dir>`.
8. `ocp replay <artifact_dir>`.
9. If it fails, run `ocp dbg <artifact_dir>`.

### 3 common beginner errors
1. `RC-CTX-INVALID`
- Cause: payload shape does not match the capability schema.
- Fix: use a correctly shaped typed record.

2. `RC-*-PERMISSION-DENIED`
- Cause: missing permission in `Ocp.toml`.
- Fix: follow snapshot/diff/approve.

3. `INSUFFICIENT`
- Cause: not enough budget or permission.
- Fix: raise budget in a controlled way or reduce request scope.

### Beginner FAQ
#### Why not write directly? Why observe then commit?
Because OCP separates “seeing the effect” from “accepting the effect”, so policy and audit can control side effects explicitly.

#### Can `DEGRADED` be committed?
Possibly, but only when the capability contract and lane policy allow it. For onboarding, default to committing only on `OK`.

#### How is `condition(false)` different from a crash?
`condition(false)` is a contract-level fail-honest signal, not an uncontrolled crash.

#### When should I use `ctx("k=v;...")`?
Only for compatibility with older scripts or currently shipped templates.

### Result4 sugar and field access (from beginner to practical)

```ocp
observe("std.json.parse", "tier2", { text: "{\"ok\":true}" }, budget(5)) -> rs;
let parsed = try rs else { 0 };
let reason = try rs else { r.reason_code };

observe("std.fs.read_text", "tier2", { path: "./fixtures/in/sample.json" }, budget(5)) -> r;
guard r;
let k = r.kind;
let a = r.audit;
condition(true);
```

Rules:
- Sugar must preserve canonical `match` semantics.
- `std.json.parse` here is only a sugar example key; inspect `ocp doc packs --json` for exact schema.
- Inside `try r else { ... }`, the implicit `r` refers to the original `Result4`; it is valid syntax, not a typo.
- The `r` inside `else` is a local implicit binding, not the outer-scope `r`.
- After refactoring to sugar, rerun `ocp check --locked` and `ocp replay`.

### 15-minute practice path
1. `ocp init hello-ocp --template tool-cli`
2. Replace `src/main.ocp` with Example 2.
3. `ocp run .`
4. `ocp replay ./.ocp_artifacts/<run_id>/`
5. Lower one `budget(5)` to `budget(1)`, observe `INSUFFICIENT`, then fix it.

### 3 progressive exercises to gain confidence

#### Exercise 1: read one JSON file
Goal:
- write one `observe` + `match` flow correctly.

Requirements:
1. Create `fixtures/in/sample.json`.
2. Call `std.fs.read_text`.
3. On `OK`, only do `condition(true);`.

Done when:
- `ocp run .` passes.
- `ocp replay <artifact_dir>` passes.

#### Exercise 2: read input then write a report
Goal:
- chain two IO steps under control.

Requirements:
1. Read `./fixtures/in/sample.json`.
2. Create `./out`.
3. Write `./out/report.txt`.
4. Only `commit` on `OK`.

Done when:
- `./out/report.txt` is produced.
- Replay still passes.

#### Exercise 3: fail on purpose, then repair
Goal:
- learn to read the error instead of guessing.

Requirements:
1. Lower one `budget(5)` to `budget(1)`.
2. Run `ocp run .` and observe `INSUFFICIENT` or its reason code.
3. Raise the budget again or reduce request scope.
4. Rerun `ocp run .` and `ocp replay <artifact_dir>`.

Done when:
- you can identify the `reason_code`;
- you fix the real cause and replay passes again.

### Complete small sample project (copy and run)
If you do not want to start from a template and edit gradually, create this exact tree:

Example label:
- `Canonical pattern`: this sample is meant for new users to copy and extend in a v1.0-friendly style.

```text
hello-guide/
  Ocp.toml
  src/
    main.ocp
  fixtures/
    in/
      sample.json
    expected/
      out.json
```

Sample `Ocp.toml`:

```toml
[package]
name = "hello_guide"
version = "0.1.0"

[project]
lane = "locked_v071"
entry = "src/main.ocp"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[language]
guard_mode = "return"

[permissions.package]
allow = ["std.fs.*", "std.kv.*", "std.time.*"]
deny = []

[permissions.std_fs]
read = ["./fixtures/in/**", "./out/**"]
write = ["./out/**"]
remove = ["./out/**"]
rename = ["./out/**"]
list = ["./fixtures/in/**", "./out/**"]
max_read_bytes = 1048576
max_write_bytes = 1048576
max_list_entries = 500

[permissions.std_kv]
enabled = true
max_keys = 512
max_value_bytes = 65536
key_prefix = "tool."

[permissions.std_time]
enabled = true
tick_mode = "logical"
dt_ms = 16
```

Notes:
- This manifest is an onboarding sample so you can learn quickly.
- It intentionally grants enough permissions for the guide examples. It is not the tightest production configuration.
- For real projects, narrow `permissions.*` to the exact keys you use.

Sample `src/main.ocp`:

```ocp
module app.tool_cli;

let rd_req = { path: "./fixtures/in/sample.json" };
observe("std.fs.read_text", "tier2", rd_req, budget(5)) -> rd;
match rd {
  OK => { }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}

let mk_req = { path: "./out", recursive: true };
observe("std.fs.mkdir", "tier2", mk_req, budget(5)) -> mk;
match mk {
  OK => { commit(mk); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}

let wr_req = { path: "./out/out.json", text: "{\"status\":\"OK\"}", overwrite: true };
observe("std.fs.write_text", "tier2", wr_req, budget(5)) -> wr;
match wr {
  OK => { commit(wr); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}

let kv_req = { key: "tool.last_run", value: "ok", overwrite: true };
observe("std.kv.put", "tier2", kv_req, budget(5)) -> kvp;
match kvp {
  OK => { commit(kvp); condition(true); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}
```

Sample `fixtures/in/sample.json`:

```json
{
  "input": "sample"
}
```

Minimum `fixtures/expected/out.json`:

```json
{"status":"OK"}
```

Run it:
1. `ocp check . --locked`
2. `ocp test . --golden fixtures/expected --clean`
3. `ocp run .`
4. `ocp replay ./.ocp_artifacts/<run_id>/`

Important note:
- `ocp init --template tool-cli` currently generates `src/main.ocp` in compatibility style using `ctx("...")`.
- This guide immediately switches to typed records to teach the canonical v1.0-facing style.

<a id="copy-patterns"></a>
### 10) 5 code patterns you will copy often
These are the five most practical patterns to start from when you do not remember the whole language yet.

#### Pattern 1: read a file, no commit

```ocp
let rd_req = { path: "./fixtures/in/sample.json" };
observe("std.fs.read_text", "tier2", rd_req, budget(5)) -> rd;
match rd {
  OK => { condition(true); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}
```

#### Pattern 2: create a directory then write a file

```ocp
let mk_req = { path: "./out", recursive: true };
observe("std.fs.mkdir", "tier2", mk_req, budget(5)) -> mk;
match mk {
  OK => { commit(mk); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}

let wr_req = { path: "./out/out.json", text: "{\"status\":\"OK\"}", overwrite: true };
observe("std.fs.write_text", "tier2", wr_req, budget(5)) -> wr;
match wr {
  OK => { commit(wr); condition(true); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}
```

#### Pattern 3: read input then write a report

```ocp
let rd_req = { path: "./fixtures/in/sample.txt" };
observe("std.fs.read_text", "tier2", rd_req, budget(5)) -> rd;
match rd {
  OK => {
    let wr_req = { path: "./out/report.txt", text: "report generated", overwrite: true };
    observe("std.fs.write_text", "tier2", wr_req, budget(5)) -> wr;
    match wr {
      OK => { commit(wr); condition(true); }
      DEGRADED => { condition(false); }
      INSUFFICIENT => { condition(false); }
      DEFERRED => { condition(false); }
    }
  }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}
```

#### Pattern 4: write local state via KV

```ocp
let kv_req = { key: "tool.last_run", value: "ok", overwrite: true };
observe("std.kv.put", "tier2", kv_req, budget(5)) -> kvp;
match kvp {
  OK => { commit(kvp); condition(true); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}
```

#### Pattern 5: `guard` and `try ... else`

```ocp
observe("std.fs.read_text", "tier2", { path: "./fixtures/in/sample.json" }, budget(5)) -> r;
guard r;

observe("std.json.parse", "tier2", { text: "{\"ok\":true}" }, budget(5)) -> rs;
let parsed = try rs else { 0 };
let reason = try rs else { r.reason_code };
condition(true);
```

<a id="template-manifests"></a>
### 11) Template manifests
This section explains which template to start from and what runtime shape it assumes.

#### When should you use `tool-cli`?
- when you are building a file tool, local automation, or a small stateful CLI;
- this is the safest first template for new users.

Minimum flow:

```bash
ocp init hello-ocp --template tool-cli
cd hello-ocp
```

#### When should you use `tool-http`?
- when the project needs real HTTP calls and cassette-based replay;
- this means `quarantine`, not the locked deterministic lane.

Example `Ocp.toml`:

```toml
[package]
name = "tool_http"
version = "0.1.0"

[project]
lane = "quarantine"
entry = "src/main.ocp"

[quarantine]
mode = "record"
max_cassette_bytes = 10485760
max_entries = 2000

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[language]
guard_mode = "return"

[compat]
ctx_string = "deny"
ctx_extra_fields = "warn"

[permissions.package]
allow = ["std.net.http.*", "std.fs.*"]
deny = []

[permissions.std_net_http]
enabled = true
allow_hosts = ["mock.local"]
allow_methods = ["GET"]
timeout_ms = 3000
max_body_bytes = 64

[permissions.std_fs]
read = ["./out/**"]
write = ["./out/**"]
remove = ["./out/**"]
rename = ["./out/**"]
list = ["./out/**"]
max_read_bytes = 1048576
max_write_bytes = 1048576
max_list_entries = 500
```

Minimum flow:

```bash
ocp init my-http --template tool-http
cd my-http
export OCP_QUARANTINE=1
ocp run .
ocp replay ./.ocp_artifacts/<run_id>/
```

```powershell
$env:OCP_QUARANTINE="1"
ocp check .
ocp run .
```

```bash
export OCP_QUARANTINE=1
ocp check .
ocp run .
```

Important note:
- this sample mainly teaches `quarantine` + cassette record/replay;
- when you need to read HTTP response payloads such as `status` or `body`, inspect the schema through `ocp doc packs --json` and a replay artifact from a real run.

```powershell
ocp init my-http --template tool-http
cd my-http
$env:OCP_QUARANTINE="1"
ocp run .
ocp replay ./.ocp_artifacts/<run_id>/
```

#### When should you use `mini-game`?
- when you want to try OCP as an app/game runtime instead of a file-oriented tool.

Example `Ocp.toml`:

```toml
[package]
name = "mini_game"
version = "0.1.0"

[project]
lane = "locked_v071"
entry = "src/main.ocp"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[language]
guard_mode = "return"

[permissions.package]
allow = ["engine.game.*"]
deny = []

[permissions.std_game]
enabled = true
fixed_dt_ms = 16
rng_streams = ["main", "loot"]
rng_max_count = 32
state_delta_max_bytes = 65536
```

Minimum flow:

```bash
ocp init demo-game --template mini-game
cd demo-game
ocp run .
ocp replay ./.ocp_artifacts/<run_id>/
```

Keep in mind:
- the current `mini-game` template still ships compatibility-style source using `ctx("...")` for `engine.game.run`;
- that accurately reflects the shipped template, but you can keep the manifest and rewrite `src/main.ocp` in a more canonical style after init.

<a id="capability-starter-pack"></a>
### 12) Capability starter pack
This section is your starter map for the capabilities you are most likely to use first.

How to read it:
- this is a starter pack for writing code, not the full capability catalog;
- it only includes keys already backed by examples, templates, or tests in the repo;
- the most precise schema source is still:

```bash
ocp doc packs
ocp doc packs --json
```

#### File system group (`permissions.std_fs`)

`std.fs.read_text`
- use case: read text/JSON input files;
- minimum request:

```ocp
{ path: "./fixtures/in/sample.json" }
```

- common extended request:

```ocp
{ path: "./README.md", max_bytes: 128 }
```

- commit needed: no;
- beginner value: this is the best first key for onboarding;
- result/payload starter:

```text
OK/DEGRADED => { text, truncated, bytes }
```

- stable tested shape:
  - `text`
  - `truncated`
  - `bytes`
- if the file is too large for the limit, you often get `DEGRADED` with `truncated=true`.

`std.fs.mkdir`
- use case: create output or work directories;
- minimum request:

```ocp
{ path: "./out", recursive: true }
```

- commit needed: yes;
- beginner rule: commit only on `OK`.

`std.fs.write_text`
- use case: write text/JSON/report output;
- minimum request:

```ocp
{ path: "./out/out.json", text: "{\"status\":\"OK\"}", overwrite: true }
```

- commit needed: yes;
- if the file is named `.json`, write real JSON instead of loose text.

#### KV group (`permissions.std_kv`)

`std.kv.get`
- use case: read local deterministic state;
- minimum request:

```ocp
{ key: "app.flag" }
```

- commit needed: no.

`std.kv.put`
- use case: write checkpoints, markers, or small local state;
- minimum request:

```ocp
{ key: "tool.last_run", value: "ok", overwrite: true }
```

- tested variant:

```ocp
{ key: "app.answer", value_json: 42, overwrite: true }
```

- commit needed: yes;
- if the contract supports structured values, prefer `value_json`.

#### Time group (`permissions.std_time`)

`std.time.tick_info`
- use case: deterministic logical time in locked lanes;
- minimum request:

```ocp
{ scope: "tool" }
```

- tested variant:

```ocp
{ tick: 7, dt_ms: 20 }
```

- commit needed: no;
- result/payload starter:

```text
OK => { tick, dt_ms }
```

- current runtime/test shape often encodes these values as strings such as `"tick" = "7"` and `"dt_ms" = "20"`.

`std.time.wallclock.now`
- use case: real wallclock time in `quarantine`;
- minimum request:

```ocp
{ scope: "tool" }
```

- commit needed: no;
- this is nondeterministic, so it belongs in `quarantine`.

#### Network / process group (`quarantine`)

`std.net.http.request`
- use case: real HTTP calls with cassette record/replay;
- minimum request:

```ocp
{ method: "GET", url: "http://mock.local/demo", timeout_ms: 3000, max_body_bytes: 16 }
```

- commit needed: no;
- notes:
  - lane must be `quarantine`;
  - host/method/body limits must match permissions;
  - replay must never fall back to real network;
- result/cassette starter:
  - denied host -> `INSUFFICIENT`
  - denied method -> `INSUFFICIENT`
  - tested cassette evidence includes:

```text
cap = "std.net.http.request"
call_id
status
truncated
req_hash
```

- if you need the full payload map for actual business logic, inspect `ocp doc packs --json` and a replay artifact from a real run first.

`std.proc.exec`
- use case: run a real process in `quarantine`;
- minimum request from the shipped template:

```ocp
{ bin: "mock.proc", args: "--template", timeout_ms: 3000, max_stdout_bytes: 32 }
```

- commit needed: no.

#### App/game/shadow group

`engine.game.run`
- use case: deterministic app/game loop under the runtime pack;
- starter request currently used by shipped templates:

```ocp
ctx("entry_module=app.mini_game;phase=frame;tick=1;stream=main;count=4;input_cap=8;events=key:Space|text:start;draw_cap=8;draw_list=text:1,1,mini-game,12;state_json={\"score\":0}")
```

- commit needed: no in the starter template;
- the current template still uses compat `ctx("...")`.

`std.shadow.search`
- use case: bounded shadow branch search and compare;
- starter request currently present in fixtures/templates:

```ocp
ctx("policy=round_robin;variants_json=[{\"x\":1},{\"x\":2},{\"x\":3}];max_branches=3;per_branch_step_cap=80;per_branch_budget_cap=2000;global_step_cap=240;global_budget_cap=12000;top_k=3;rounds=2")
```

- commit needed: no in standard shadow search flow.

Checklist for a new capability:
1. Find the closest key in this starter pack.
2. Run `ocp doc packs` to inspect `permission_class`, `ctx_schema`, `payload_schema`, and `example`.
3. Write the request as a record or `ctx("...")` exactly for that key.
4. Run `ocp check . --locked` or the corresponding lane.
5. Run `ocp run .` and `ocp replay <artifact_dir>`.

#### How to read `result/payload` by capability
The most stable layer is always the `Result4` wrapper:
- `r.kind`
- `r.reason_code`
- `r.audit`

Some payload shapes already backed by tests:

`std.kv.get`
- successful payload commonly looks like:

```text
{ found: true|false, value: ... }
```

`std.kv.keys`
- when truncated, it may return `DEGRADED` with payload like:

```text
{ keys: [...], truncated: true }
```

`engine.game.run`
- successful payload may contain:

```text
{ entry_module, tick, stream, count, rng_values, ... }
```

- Meaning: this is not just “the game ran”; it also carries enough state/report data for replay and debug.

For keys such as `std.fs.read_text`, `std.time.tick_info`, `std.net.http.request`, and `std.shadow.search`:
- this guide pins the starter request shape;
- when you need real payload details for logic, inspect:
  1. `ocp doc packs`
  2. `ocp doc packs --json`
  3. the replay/audit artifact from that exact run

<a id="language-reference-short"></a>
### 13) Short language reference (runnable examples)
This does not replace a full language specification. It simply gathers the constructs you are most likely to encounter early.

Note:
- some imports or bindings below exist only to illustrate syntax; if the editor warns about `unused`, you may remove them without changing the teaching point.

#### `module`, `import`, `fn`, `return`

```ocp
module app.demo;
import toolkit.api.math;

fn dep_ready() {
  return true;
}

let ok = dep_ready();
condition(ok);
```

#### `repeat` and `for ... cap`

```ocp
module app.loops;

let xs = [1, 2, 3];
repeat 2 { let ping = true; }
for item in xs cap 2 { let seen = item; }
condition(true);
```

#### `guard` and `try ... else`

```ocp
module app.sugar;

observe("std.fs.read_text", "tier2", { path: "./fixtures/in/sample.json" }, budget(5)) -> r;
guard r;

observe("std.json.parse", "tier2", { text: "{\"ok\":true}" }, budget(5)) -> rs;
let parsed = try rs else { 0 };
let reason = try rs else { r.reason_code };
condition(true);
```

Rule:
- write the flow with `match` first;
- only then refactor to sugar;
- after each sugar refactor, run `ocp check . --locked` and `ocp replay <artifact_dir>`.

<a id="mini-game-project"></a>
### 14) Complete mini-game sample project
This is the minimal sample for users who want to try OCP as an app/game runtime instead of a file tool.

Example label:
- `Canonical pattern` for new users: the code below adds `match frame` so it stays consistent with the onboarding rules.
- `Template currently shipped`: the CLI still generates a shorter scaffold; see notes below.

Project tree:

```text
demo-game/
  Ocp.toml
  src/
    main.ocp
```

`Ocp.toml`:

```toml
[package]
name = "mini_game"
version = "0.1.0"

[project]
lane = "locked_v071"
entry = "src/main.ocp"

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[language]
guard_mode = "return"

[permissions.package]
allow = ["engine.game.*"]
deny = []

[permissions.std_game]
enabled = true
fixed_dt_ms = 16
rng_streams = ["main", "loot"]
rng_max_count = 32
state_delta_max_bytes = 65536
```

`src/main.ocp`:

```ocp
module app.mini_game;

observe("engine.game.run", "tier2", ctx("entry_module=app.mini_game;phase=frame;tick=1;stream=main;count=4;input_cap=8;events=key:Space|text:start;draw_cap=8;draw_list=text:1,1,mini-game,12;state_json={\"score\":0}"), budget(10)) -> frame;
match frame {
  OK => { condition(true); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}
```

Run:

```bash
ocp check . --locked
ocp run .
ocp replay ./.ocp_artifacts/<run_id>/
```

Expected outcome:
- `.ocp_artifacts/<run_id>/audit.jsonl`
- `.ocp_artifacts/<run_id>/signature.txt`
- `.ocp_artifacts/<run_id>/replay.toml`
- stable replay

Notes:
- This is the `canonical pattern` version for new users; it intentionally handles `frame` with `match`.
- The shipped `mini-game` template is shorter and still uses `ctx("...")` for `engine.game.run`.
- That shipped scaffold is fine for smoke-test init/run/replay, but it is not the cleanest pattern for extending new logic.

<a id="tool-http-project"></a>
### 15) Complete tool-http sample project
This is the minimal sample for users who want to try `quarantine` + cassette record/replay over HTTP.

Example label:
- `Canonical pattern` for new users: the code below handles `Result4` explicitly.
- `Template currently shipped`: the CLI generates a shorter scaffold; see notes below.

Project tree:

```text
my-http/
  Ocp.toml
  src/
    main.ocp
```

`Ocp.toml`:

```toml
[package]
name = "tool_http"
version = "0.1.0"

[project]
lane = "quarantine"
entry = "src/main.ocp"

[quarantine]
mode = "record"
max_cassette_bytes = 10485760
max_entries = 2000

[targets]
default = "main"

[dependencies]
std = "0.1.0"

[language]
guard_mode = "return"

[compat]
ctx_string = "deny"
ctx_extra_fields = "warn"

[permissions.package]
allow = ["std.net.http.*", "std.fs.*"]
deny = []

[permissions.std_net_http]
enabled = true
allow_hosts = ["mock.local"]
allow_methods = ["GET"]
timeout_ms = 3000
max_body_bytes = 64

[permissions.std_fs]
read = ["./out/**"]
write = ["./out/**"]
remove = ["./out/**"]
rename = ["./out/**"]
list = ["./out/**"]
max_read_bytes = 1048576
max_write_bytes = 1048576
max_list_entries = 500
```

`src/main.ocp`:

```ocp
module app.tool_http;

observe("std.net.http.request", "tier2", { method: "GET", url: "http://mock.local/template-http", timeout_ms: 3000, max_body_bytes: 32 }, budget(8)) -> net;
match net {
  OK => { }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}

observe("std.fs.mkdir", "tier2", { path: "./out", recursive: true }, budget(5)) -> mk;
match mk {
  OK => { commit(mk); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}

observe("std.fs.write_text", "tier2", { path: "./out/http.txt", text: "HTTP_TOOL_OK", overwrite: true }, budget(5)) -> wr;
match wr {
  OK => { commit(wr); condition(true); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}
```

Run in record mode:

```powershell
$env:OCP_QUARANTINE="1"
ocp check .
ocp run .
```

```bash
export OCP_QUARANTINE=1
ocp check .
ocp run .
```

Replay:

```powershell
$env:OCP_QUARANTINE="1"
ocp replay ./.ocp_artifacts/<run_id>/
```

```bash
export OCP_QUARANTINE=1
ocp replay ./.ocp_artifacts/<run_id>/
```

Expected outcome:
- `./out/http.txt`
- `.ocp_artifacts/<run_id>/cassette/cassette.jsonl`
- cassette entry with `cap = "std.net.http.request"`
- replay passes while the env gate remains enabled

Fail-honest rules:
- if `OCP_QUARANTINE="1"` is not enabled, both `run` and `replay` for the `quarantine` lane must fail;
- if host or method is outside the allowlist, the runtime must return `INSUFFICIENT`, not silently weaken policy.

Notes:
- unlike `tool-cli`, the shipped `tool-http` template already uses typed record requests for HTTP/fs;
- this sample is the `canonical pattern` version for new users; it handles `net`, `mk`, and `wr` with `match`;
- the shipped `tool-http` template is shorter and acceptable as a smoke scaffold, but it is not the cleanest pattern for new logic;
- this sample mainly teaches `quarantine` + cassette + controlled side effects; when you need real `status`/`body` response logic, inspect `ocp doc packs --json` and the replay artifact from a real run.

<a id="language-quickstart"></a>
### 16) Write your first OCP code (Language Quickstart)
Goal: edit `src/main.ocp` and make it run immediately.

Example label:
- `Canonical pattern`: this is the primary learning path for new users, using typed records and full `match`.

Guide convention for v1.0:
- new code: prefer typed record payloads;
- `ctx("k=v;...")`: keep it only for compatibility with older scripts or shipped templates.

Step 1 - create a project from a template with real IO:

```bash
ocp init hello-ocp --template tool-cli
cd hello-ocp
```

Step 2 - minimum project structure to know:
- `Ocp.toml`: lane, permissions, entrypoint;
- `src/main.ocp`: main code;
- `fixtures/`: sample test data.

Notes:
- the current `tool-cli` template still generates compatibility-style source with `ctx("k=v;...")`;
- in this guide you replace it immediately with typed records to learn the canonical v1.0-facing style.

Step 3 - replace `src/main.ocp` with this runnable sample:

```ocp
module app.tool_cli;

let mk_req = { path: "./out", recursive: true };
observe("std.fs.mkdir", "tier2", mk_req, budget(5)) -> mk;
match mk {
  OK => { commit(mk); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}

let wr_req = { path: "./out/out.json", text: "{\"status\":\"OK\"}", overwrite: true };
observe("std.fs.write_text", "tier2", wr_req, budget(5)) -> wr;
match wr {
  OK => { commit(wr); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}

let ti_req = { scope: "tool" };
observe("std.time.tick_info", "tier2", ti_req, budget(5)) -> ti;

let kv_req = { key: "tool.last_run", value: "ok", overwrite: true };
observe("std.kv.put", "tier2", kv_req, budget(5)) -> kvp;
match kvp {
  OK => { commit(kvp); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}

condition(true);
```

Step 4 - run:

```bash
ocp run .
ocp replay ./.ocp_artifacts/<run_id>/
```

Step 5 - read the result:
- output file: `./out/out.json`
- replay artifact: `.ocp_artifacts/<run_id>/`

Minimum syntax cheat sheet:
- variable declaration: `let x = 1;`
- typed record: `let req = { path: "./out/a.txt", overwrite: true };`
- list: `let items = ["a", "b"];`
- safe observe chain:
  1. `observe(...) -> r;`
  2. `match r { OK => commit(r); ... }`
- canonical `Result4` pattern:

```ocp
match r {
  OK => { /* nhánh thành công */ }
  DEGRADED => { /* nhánh giảm cấp */ }
  INSUFFICIENT => { /* thiếu ngân sách/quyền */ }
  DEFERRED => { /* cần hoãn */ }
}
```

Fail-honest file-read sample:

```ocp
let rd_req = { path: "./fixtures/in/sample.json" };
observe("std.fs.read_text", "tier2", rd_req, budget(5)) -> rd;
match rd {
  OK => { /* xử lý dữ liệu đọc được */ condition(true); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}
```

---

<a id="concepts"></a>
## [concepts] Foundations you should know to use OCP

This section covers practical operating knowledge.  
Detailed version-by-version history lives in `docs/plans/history/*`.

How to read this section:
- sections `1` to `6`: core material, read now;
- sections `7` to `15`: advanced material, read later.

### 1) Core execution model
1. All reads of external data go through `observe(...)`.
2. Outcomes are always represented as `OK/DEGRADED/INSUFFICIENT/DEFERRED`.
3. All side effects go through `commit(...)`, controlled by policy.
4. The runtime is bounded: budget and caps exist, and infinite execution is not allowed.

### 2) Locked vs unlocked (the most important distinction)
- `--locked`:
  - enforces pinned lockfiles/trust/signatures,
  - fails hard when lock/trust is missing or invalid.
- `--unlocked`:
  - useful for internal development, but not release-grade.

Important lanes:
- `locked_v071`
- `locked_v06`
- `quarantine`

### 2.1) Commonly confused terms
- `tier2`: default trust/runtime tier for observe keys in shipped templates.
- `budget(5)`: work budget for one observe call.
- `guard_mode = "return"`: return early on guard failure.
- `locked_v071` / `locked_v06` / `quarantine`: current standard / compatibility / nondeterministic lane.
- `std.ui.*`: governed draw-list capability, not a desktop widget toolkit.

### 3) Reproducibility and replay
- `run` executes real work.
- `replay` proves reproducibility from recorded evidence.
- `trace/profile/dbg` provide structured investigation evidence.

### 4) When to use cosmology / hive / shadow
- `--universe/--domain`: isolate environment, policy, or state region.
- `--shadow`: compare without committing to the truth world.
- hive/runtime advanced scheduling: use when the problem requires coordinated multi-task execution.

### 5) Supply-chain and trust
Before calling a build valid, at minimum do:
1. lock sync/verify
2. build + attest
3. verify attest/supply

### 6) Minimum manifest for the locked lane
A minimal `Ocp.toml` should include:

```toml
[project]
lane = "locked_v071"
entry = "src/main.ocp"

[language]
guard_mode = "return"

[permissions.package]
allow = ["std.log.info"]
deny = []
```

Meaning:
- `lane`: runtime/policy lane;
- `guard_mode`: early-exit behavior for guards;
- `[permissions.package]`: baseline permission grant in locked flow.

<a id="permissions"></a>
## [permissions] Permissions

### 1) Why permissions are mandatory
OCP does not allow side effects to float freely.  
All IO permissions must be explicit, reviewable, and traceable.

### 2) Meaning of each permission command
- `ocp perm snapshot <project_dir> [--out-dir <dir>]`
  - capture the permission baseline.
- `ocp perm diff <old> <new> [--out <report.json>] [--approval <permissions.approval.toml>]`
  - compare new permissions against baseline.
- `ocp perm approve <diff_report.json> [--approval <permissions.approval.toml>] --by <id> --date <YYYY-MM-DD> [--note <text>]`
  - approve a valid diff under the lane policy.

### 3) Safe approval rules
- always read the diff first;
- avoid wildcards in strict lanes;
- if you do not understand why permissions grew, stop and investigate first.

### 4) Standard workflow
1. change the code;
2. run `ocp perm snapshot <project_dir> --out-dir ./.ocp_perm`;
3. run `ocp perm diff <old_snapshot> <new_snapshot> --out permission_diff_report.json --approval permissions.approval.toml`;
4. if valid and justified: `ocp perm approve permission_diff_report.json --approval permissions.approval.toml --by <id> --date <YYYY-MM-DD>`;
5. keep the diff report and approval file as evidence.

---

<a id="lock-trust-attest"></a>
## [lock-trust-attest] Lock Trust Attest

### 1) `ocp lock sync <project_dir>`
- synchronize the lockfile with current dependency state;
- prepare stable inputs for verify/build.

### 2) `ocp lock sign <project_dir> --key <keyid> [--lock deps.lock.v3]`
- sign `deps.lock.v3` and produce `deps.lock.v3.sig`;
- needed before `lock verify` in strict signed flows.

### 3) `ocp lock verify <project_dir> [--lock deps.lock.v3]`
- check lock/trust policy;
- catch mismatches early.

### 4) `ocp build <project_dir> --attest`
- build with attestation evidence.

### 5) `ocp verify --attest <artifact_dir>`
- verify the attestation just produced.

### 6) When to run the full lock/trust/attest flow
- before merge
- before release
- before production deployment

---

<a id="run-replay"></a>
## [run-replay] Run Replay

### 1) `ocp run <project_dir>`
- run the program under the current policy/lane;
- generate trace/artifacts for audit and debug.

### 2) `ocp replay <artifact_dir>`
- replay behavior from recorded data;
- prove reproducibility.

Minimum artifact bundle:
- `.ocp_artifacts/<run_id>/audit.jsonl`
- `.ocp_artifacts/<run_id>/signature.txt`
- `.ocp_artifacts/<run_id>/replay.toml`

Example:

```bash
ocp replay ./.ocp_artifacts/000123/
```

### 3) Fail-honest principle
Replay must fail clearly when:
- required cassette/chunks are missing;
- replay data diverges from expectation;
- policy does not allow continuation.

### 4) Quick replay-failure checklist
1. verify lock/trust
2. verify cassette/chunk completeness
3. run `ocp dbg`

---

<a id="debug"></a>
## [debug] Debug

### 1) When to use `ocp dbg <artifact_dir>`
- run failed and the reason is unclear;
- replay mismatch;
- you need the exact state transition that caused the problem.

### 2) Goal of debug in OCP
- not just “see an error”;
- trace the real cause through structured evidence.

### 3) Short debug procedure
1. run `ocp dbg <artifact_dir>`;
2. identify the failing stage (observe/match/commit);
3. cross-check permission/lock/attest if relevant;
4. fix, then rerun the minimum flow.

---

<a id="commands"></a>
## Full command catalog

### A) Canonical command set (main v1.0 workflow)
- `ocp init <project_dir> [--template tool-cli|tool-http|tool-proc|tool-wallclock|mini-game|shadow-preview|dep-permission] [--preset workflow_basic|agent_swarm_basic]`
- `ocp lock sync <project_dir>`
- `ocp lock sign <project_dir> --key <keyid> [--lock deps.lock.v3]`
- `ocp lock verify <project_dir> [--lock deps.lock.v3]`
- `ocp perm snapshot <project_dir> [--out-dir <dir>]`
- `ocp perm diff <old> <new> [--out <report.json>] [--approval <permissions.approval.toml>]`
- `ocp perm approve <diff_report.json> [--approval <permissions.approval.toml>] --by <id> --date <YYYY-MM-DD> [--note <text>]`
- `ocp build <project_dir> --attest`
- `ocp verify --attest <artifact_dir>`
- `ocp run <project_dir>`
- `ocp replay <artifact_dir>`
- `ocp dbg <artifact_dir> [--script <file>]`
- `ocp doctor <project_dir> [--out <report.json>]`
- `ocp fix --plan <project_dir> [--out <permission_fix_plan.json>]`
- `ocp fix --apply <project_dir> [--plan-file <permission_fix_plan.json>] [--patch <permission_fix.patch.toml>] [--out <permission_fix_safety_report.json>] [--approval <permissions.approval.toml>] --ack-risk --justification <text> --by <id> --date <YYYY-MM-DD>`
- `ocp budget analyze <artifact_dir|audit.jsonl> [--json]`

### B) Advanced commands (available in the CLI, use as needed)

#### 1) Advanced execution, checking, and profiling
- `ocp check <project_dir> [--json] [--locked] [--universe <id>]`
- `ocp run <project_dir> [--engine interpreter|bytecode|dual] [--reactor --ticks N --runtime deterministic|throughput --socket-listen <addr> --runtime-report <file> --replay-audit <file>] [--shadow <id> --shadow-policy forbid_commit|shadow_commit_log] [--locked] [--universe <id>] [--domain <id>] [--view <id>]`
- `ocp fmt <project_dir> [--check]`
- `ocp test <project_dir> [--locked] [--universe <id>] [--domain <id>] [...]`
- `ocp cache stats <project_dir> [--json]`
- `ocp cache clean <project_dir> --yes [--json]`
- `ocp cache bench <project_dir> [--engine ...] [--locked] [--json]`
- `ocp trace run <project_dir> [--out <file>] [--engine interpreter|bytecode|dual] [--reactor --ticks N --runtime deterministic|throughput] [--shadow <id> --shadow-policy forbid_commit|shadow_commit_log] [--locked] [--universe <id>] [--domain <id>] [--view <id>]`
- `ocp trace view <artifact_dir|audit.jsonl|legacy.trace> [...]`
- `ocp trace diff <artifactA|auditA> <artifactB|auditB> [...]`
- `ocp minimize <artifact_dir> --goal <...> [...]`
- `ocp profile run <project_dir> [--out <file>] [--engine interpreter|bytecode|dual] [--reactor --ticks N --runtime deterministic|throughput] [--shadow <id> --shadow-policy forbid_commit|shadow_commit_log] [--locked] [--universe <id>] [--domain <id>] [--view <id>]`
- `ocp profile view <profile_file> [--top N] [--json]`
- `ocp budget doctor <artifact_dir|audit.jsonl> [--json]`
- `ocp doc packs [--json]`

#### 2) Supply-chain, package, and dependency
- `ocp publish <artifact.ocppkg> [--registry <dir>]`
- `ocp fetch <artifact|package> [--registry <dir>] [--out <dir>]`
- `ocp verify-supply <artifact.ocppkg>`
- `ocp deps resolve <project_dir> [--write-legacy-lock]`
- `ocp deps update <project_dir> [pkg] [--write-legacy-lock]`
- `ocp deps verify <project_dir> [--no-lane-policy]`
- `ocp pack build <project_dir> [--locked]`
- `ocp pack sign <artifact.ocppkg>`
- `ocp pack publish <artifact.ocppkg> [--registry <dir>]`
- `ocp pack verify <artifact.ocppkg>`

#### 3) Policy / cosmos / plugin / organ / kit
- `ocp init <project_dir> --preset workflow_basic|agent_swarm_basic`
- `ocp policy lock sync <project_dir>`
- `ocp cosmos init <project_dir> [--preset default|ci]`
- `ocp cosmos lock sync <project_dir> [--locked] [...]`
- `ocp plugin lock sync <project_dir>`
- `ocp plugin verify <project_dir>`
- `ocp organ lock sync <project_dir> [--registry <index.toml>] [--json]`
- `ocp organ verify <project_dir> [--locked|--unlocked] [--json]`
- `ocp organ install <name> <version> [--project <dir>] [...]`
- `ocp kit list [<project_dir>] [--json]`
- `ocp kit doctor <project_dir> [--locked|--json]`
- `ocp compose <project_dir> --phenotype <file> [...]`

Quick distinction:
- `ocp init --preset workflow_basic|agent_swarm_basic`: public project-level onboarding preset
- `ocp cosmos init --preset default|ci`: deeper cosmos configuration preset

#### 4) Advanced verify
- `ocp verify --attest <artifact_dir>`
- `ocp verify --repro <artifact_dir>`
- `ocp verify <project_dir> --phenotype <file> [...]`

### C) Useful aliases
- `ocp doctor ...` is an alias of `ocp perm doctor ...`
- `ocp fix --plan|--apply ...` is an alias of `ocp perm fix ...`
- `ocp perm review ...` is an alias of `ocp perm diff ...`

### D) Safe usage recommendations
- always run `ocp --help` or `ocp <group> --help` before advanced commands;
- for commands that change locks/permissions/artifacts, prefer a separate branch first.

Quick lookup:
- `docs/en/reference/cli.md`

<a id="public-surface"></a>
## Important extended public surface

Beyond the core flow, v1.0 also includes public branches that are easy to miss if you only glance at command names:
1. extended language surface (import/export/fn/return/bounded loops such as `repeat`, `for ... cap N`, compatibility wording around `for-range`, list-map, `?`, `try-else`, `guard`, reactor tick)
2. compose / phenotype
3. organ / kit / preset
4. editor workflow

<a id="compose-phenotype"></a>
## Compose / Phenotype

Use this when you want to generate modules/config from a phenotype contract instead of hand-writing everything.

Short workflow:
1. prepare the phenotype file;
2. run compose:

```bash
ocp compose <project_dir> --phenotype <phenotype_file>
```

3. verify phenotype/build consistency:

```bash
ocp verify <project_dir> --phenotype <phenotype_file>
```

Common artifacts after compose:
- `src/generated/*.ocp`
- `src/generated/mod.ocp`
- `assembly_proof.toml`

<a id="organ-kit-preset"></a>
## Organ / Kit / Preset

Use this when you want to bootstrap a workspace/team flow instead of assembling everything manually.

Short workflow:
1. initialize from a public workflow preset:

```bash
ocp init <project_dir> --preset workflow_basic
# hoặc
ocp init <project_dir> --preset agent_swarm_basic
```

2. the preset pulls the required lock chain, then sync the organ graph:

```bash
ocp organ lock sync <project_dir>
```

3. verify organ under the target lane:

```bash
ocp organ verify <project_dir> --locked
```

4. install more organs and check the kit:

```bash
ocp organ install <name> <version> --project <project_dir>
ocp kit doctor <project_dir> --locked
```

Preset guidance:
- `workflow_basic`: lighter standard application flow
- `agent_swarm_basic`: bounded swarm/hive scenarios with wiring already in place

<a id="editor-workflow"></a>
## Editor workflow (VSCode/LSP/DAP)

Recommended editor workflow:
1. install the VSIX and open a `.ocp` file;
2. by default you get TextMate highlighting + snippets; bridge commands use bundled CLI in trusted workspaces;
3. trust behavior:
   - untrusted workspace: static shell only, no runtime command bridge;
   - trusted workspace: runtime bridge commands such as `fmt/check/doctor/fix/debug trace` are enabled from the bundled CLI;
4. the editor bridge uses bundled CLI binaries from the extension package, not host `PATH`;
5. current editor runtime status in the repo:
   - bridge commands use bundled CLI for `fmt`, `check`, `doctor`, `fix --plan`, and opening trace artifacts;
   - editor runtimes are bundled as `ocp-lsp` / `ocp-dap` in the release pack;
6. before commit, run:

```bash
ocp fmt <project_dir> --check
ocp check <project_dir> --locked
ocp doctor <project_dir>
ocp fix --plan <project_dir>
```

7. if you already have a failing artifact or replay mismatch:

```bash
ocp dbg <artifact_dir>
```

<a id="reactor-workflow"></a>
## Reactor workflow (run/trace/profile)

Use reactor when you need bounded tick loops or separate runtime report/audit files.

Basic reactor run:

```bash
ocp run <project_dir> --reactor --ticks 50 --runtime deterministic --locked --universe dev --domain main --view default
```

Reactor trace:

```bash
ocp trace run <project_dir> --reactor --ticks 50 --runtime deterministic --locked --out target/ocp/trace/reactor.audit.jsonl
ocp trace view target/ocp/trace/reactor.audit.jsonl --tail 50
```

Reactor profile:

```bash
ocp profile run <project_dir> --reactor --ticks 50 --runtime deterministic --locked --out target/ocp/profile/reactor.profile.json
ocp profile view target/ocp/profile/reactor.profile.json --top 20
```

Notes:
- `--socket-listen`, `--runtime-report`, and `--replay-audit` are only valid with `--reactor`;
- if your team pins `--universe/--domain/--view`, keep them fixed to reduce drift across machines.

---

<a id="recommended-workflow"></a>
## Recommended workflow

Default team flow:
1. `ocp init <project_dir>` (or enter an existing project)
2. `ocp lock sync <project_dir>` + `ocp lock sign <project_dir> --key <keyid>` + `ocp lock verify <project_dir> --lock deps.lock.v3`
3. `ocp perm snapshot <project_dir>` + `ocp perm diff <old> <new>` + `ocp perm approve <diff_report.json> --by <id> --date <YYYY-MM-DD>`
4. `ocp build <project_dir> --attest` + `ocp verify --attest <artifact_dir>`
5. `ocp run <project_dir>` + `ocp replay <artifact_dir>`
6. If something breaks: `ocp doctor <project_dir>` -> `ocp fix --plan <project_dir>` -> `ocp fix --apply <project_dir> ...` -> `ocp dbg <artifact_dir>`
7. If you suspect budget issues: `ocp budget analyze <artifact_dir|audit.jsonl>`

---

<a id="troubleshooting"></a>
## [troubleshooting] Troubleshooting

### 1) Command not recognized after installation
Symptom:
- `ocp` is not recognized.

Fix:
1. open a new terminal
2. check `PATH`
3. run `ocp --version` again

### 2) `perm diff` shows suspicious permission growth
Symptom:
- many new permissions appear without a clear reason.

Fix:
1. do not approve immediately
2. review recent code/dependency changes
3. rerun `perm snapshot`

### 3) `lock verify` does not pass
Symptom:
- lock/trust mismatch.

Fix:
1. rerun `ocp lock sync`
2. if `deps.lock.v3.sig` is missing, run `ocp lock sign <project_dir> --key <keyid>`
3. verify again
4. if it still fails, inspect trust policy/metadata

### 4) `verify --attest` fails
Symptom:
- attestation mismatch.

Fix:
1. rebuild with `ocp build --attest`
2. verify immediately after build
3. inspect environment and artifact paths if it still diverges

### 5) `replay` fails because data is missing
Symptom:
- missing cassette/chunk data.

Fix:
1. record data again with the correct flow
2. do not use real IO fallback
3. use `ocp dbg`

### 6) Tool IO is blocked (`RC-*-PERMISSION-DENIED`)
Symptom:
- `std.fs.*`, `std.kv.*`, or `std.time.*` returns `INSUFFICIENT` due to policy.

Fix:
1. inspect:
   - `[permissions.std_fs]`
   - `[permissions.std_kv]`
   - `[permissions.std_time]`
2. compare the actual key with allow/deny rules
3. rerun `ocp check . --locked`
4. rerun `ocp test . --golden fixtures/expected --clean`

### 7) `quarantine` lane blocked during run/replay
Symptom:
- the CLI reports `quarantine` is denied or replay refuses to run.

Fix:
1. confirm which lane the project/artifact uses
2. if it is `quarantine`, enable the env gate:
   - PowerShell: `$env:OCP_QUARANTINE="1"`
   - bash/zsh: `export OCP_QUARANTINE=1`
3. do not replay `quarantine` artifacts like locked-lane artifacts
4. rerun `ocp run ...` then `ocp replay <artifact_dir>`

### 8) Need deeper help
- `docs/en/troubleshooting/common-errors.md`
- `docs/en/troubleshooting/editor.md`
- `docs/en/troubleshooting/replay.md`

---

<a id="advanced-appendix"></a>
## [advanced-appendix] Advanced appendix (read after onboarding)

Everything below is advanced material.  
If this is your first time learning OCP, reading through `Troubleshooting` is enough for daily work.

### 7) Tool-grade IO packs you should know about (advanced)
Locked-lane tool work commonly uses:
- `std.fs.*`
- `std.kv.*`
- `std.time.*`

When building an IO tool:
1. initialize from the template:
   - `ocp init <project_dir> --template tool-cli`
2. test with golden output:
   - `ocp test <project_dir> --golden fixtures/expected --clean`
3. replay from the artifact:
   - `ocp replay ./.ocp_artifacts/<run_id>/`

Important IO metadata artifacts:
- `io/fixtures_manifest.json`
- `state/kv_start.json`
- `replay.toml`

### 8) Consumer packs for app/game (advanced)
OCP also provides consumer-style capability families:
- `std.ui.*`
- `engine.game.*`
- `std.shadow.*`

Fast start:
1. `ocp init <project_dir> --template mini-game`
2. `ocp run <project_dir>`
3. `ocp replay ./.ocp_artifacts/<run_id>/`

Or:
1. `ocp init <project_dir> --template shadow-preview`
2. `ocp run <project_dir> --shadow <id> --shadow-policy forbid_commit`
3. `ocp replay ./.ocp_artifacts/<run_id>/`

Lane note:
- `quarantine` is separate for nondeterministic capabilities;
- when using that lane, enable `OCP_QUARANTINE=1`;
- `quarantine` artifacts/signatures must not be mixed with locked-lane artifacts.

### 9) Quarantine + cassette record/replay (advanced)
When your tool needs nondeterministic capabilities such as HTTP/proc/wallclock, use `quarantine`.

Correct operating checklist:
1. in `Ocp.toml`, set lane `quarantine` and declare the corresponding permissions
2. enable the env gate before running:

```powershell
$env:OCP_QUARANTINE="1"
```

```bash
export OCP_QUARANTINE=1
```

3. run the project with `ocp run <project_dir>`
4. replay from the run artifact:

```bash
ocp replay ./.ocp_artifacts/<run_id>/
```

5. for longer cassette operations, use:
   - `ocp cassette stats`
   - `ocp cassette prune`
   - `ocp cassette gc`
   - `ocp cassette upgrade`

Shipped example projects:
- `tool-http`
- `tool-proc`
- `tool-wallclock`

Important rules:
- missing required cassette/chunks => replay must fail honestly and must not call real IO
- `quarantine` artifact/signature sets must remain separate from locked-lane ones
- use `quarantine` only when you really need nondeterministic capabilities

### 10) Schema + typing for ctx/payload/effects (advanced)
OCP pushes contract checks earlier:
- `ctx` should be written as typed records where possible
- `match`/`try`/`guard` expect proper `Result4`
- `commit(...)` is only legal for observe keys whose contract allows commit

Recommended sample:

```ocp
let req = { path: "./out/result.json", overwrite: true };
observe("std.fs.write_text", "tier2", req, budget(5)) -> wr;
match wr {
  OK => { commit(wr); }
  DEGRADED => { commit(wr); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}
```

If you are migrating an older script that still uses `ctx("...")`, you may temporarily enable compatibility in `Ocp.toml`:

```toml
[compat]
ctx_string = "warn"
ctx_extra_fields = "warn"
```

Safe migration path:
1. set compat to `warn`
2. gradually rewrite scripts to typed records
3. switch back to `deny`

Useful schema-pack commands:
- `ocp doc packs`
- `ocp doc packs --json`

Common errors:
- `RC-CTX-INVALID`
- `T-TRY-NOT-RESULT4` / `T-GUARD-NOT-RESULT4`
- compile-time commit denial for observe-only keys

### 11) Packaging + lockfile + trust (advanced)
OCP provides a full dependency/package flow so modules can be reused across projects while keeping supply-chain control.

What to remember:
- the lockfile SoT is `deps.lock.v3`
- dependency permissions are computed with least privilege per package
- lane `locked_v071` requires non-builtin dependencies to be signed and trusted

Standard flow when adding/updating dependencies:
1. resolve the graph:

```bash
ocp deps resolve <project_dir>
```

2. verify lock + trust:

```bash
ocp deps verify <project_dir>
```

3. only then continue with build/run.

If you package a reusable artifact:

```bash
ocp pack build <project_dir>
ocp pack sign <artifact.ocppkg>
ocp pack verify <artifact.ocppkg>
```

Artifacts worth tracking:
- `deps.lock.v3`
- `permissions_effective.json`
- audit event `DepsResolved`

### 12) Replay-native debugging: trace, diff, dbg, minimize (advanced)
OCP gives you artifact-based debugging instead of relying on hand-written logging.

Recommended incident flow:
1. run and capture the artifact
2. inspect trace for the first failure point:

```bash
ocp trace view <artifact_dir|audit.jsonl> --tail 50
```

3. if you need a baseline comparison:

```bash
ocp trace diff <artifactA|auditA> <artifactB|auditB> --mode align --out <out_dir>
```

4. open the replay debugger:

```bash
ocp dbg <artifact_dir>
```

5. if the case is too large, minimize the repro:

```bash
ocp minimize <artifact_dir> --goal <error_code:X|divergence|kind:KIND> --out <out_dir>
```

### 13) Shadow scheduling + incremental reuse (advanced)
`shadow-preview` includes a bounded search engine:
- policies: `round_robin`, `beam`, `portfolio`
- deterministic scoring
- v2 report with best-branch ranking and prune reasons

Fast path:
1. initialize the template:

```bash
ocp init <project_dir> --template shadow-preview
```

2. run and produce an artifact:

```bash
ocp run <project_dir>
```

3. replay to confirm stability:

```bash
ocp replay ./.ocp_artifacts/<run_id>/
```

4. when comparing branches/builds:
   - `ocp trace view <artifact_dir>`
   - `ocp trace diff <artifactA> <artifactB> --mode align`

### 14) IR + reproducible cache (advanced)
OCP adds performance layers while keeping the principle “determinism first, speed second”:
- compile cache
- execution cache for pure branches
- observe cache under safe policy

Cache commands:

```bash
ocp cache stats <project_dir> [--json]
ocp cache bench <project_dir> [--engine interpreter|bytecode|dual] [--locked] [--json]
ocp cache clean <project_dir> --yes [--json]
```

Practical use:
1. `ocp cache stats` to inspect compile/exec/observe hit/miss
2. `ocp cache bench` to compare cold/warm/no-cache and verify `signature_equal`
3. `ocp cache clean --yes` only when you truly need a reset

### 15) Conformance suite + upgrade-check (advanced)
OCP provides system-level contract verification, not only isolated unit tests.

Main command set:

```bash
ocp test --conformance list --manifest <manifest_file>
ocp test --conformance run --manifest <manifest_file> --out <report_file> [--locked]
```

Equivalent aliases:

```bash
ocp conformance list --manifest <manifest_file>
ocp conformance run --manifest <manifest_file> --out <report_file> [--locked]
```

Practical example:

```bash
ocp test --conformance list --manifest projects/ocp/conformance/conformance.v5.toml
ocp test --conformance run --manifest projects/ocp/conformance/conformance.v5.toml --out target/ocp/w14/reports/conformance_report.json
```

Upgrade check before changing runtime:

```bash
ocp upgrade-check <project_dir> --manifest <manifest_file> --out <report_file> [--locked]
```

### 16) Advanced maintainer/release flow
To avoid mixing end-user and release-operator roles, these topics live in a separate document:
- trust + attestation + LTS production
- product-readiness audit/checklist
- migration/release packaging/signoff
- final audit gate flow (v0.20 line)

See:
- `docs/vi/MAINTAINER_GUIDE.md`
- `docs/plans/history/*`

---

<a id="reference"></a>
## Reference (supporting, not required to learn A->Z)

Prefer looking up directly inside this file:
- [Quick reference](#quick-reference)
- [Language Quickstart](#language-quickstart)
- [Foundations you should know to use OCP](#concepts)
- [Permissions](#permissions)
- [Full command catalog](#commands)
- [Troubleshooting](#troubleshooting)
- [Advanced appendix](#advanced-appendix)

External companion files:
- `docs/en/reference/cli.md`
- `docs/en/ops/doctor-and-fix.md`
- `docs/en/ops/budget-analyze.md`
- `docs/en/security/verify-download.md`

---

<a id="maintainer-guide"></a>
## Maintainer Guide

Release/gate material is separated so `USER_GUIDE` can stay focused on writing and running OCP:
- [docs/vi/MAINTAINER_GUIDE.md](/docs/vi/MAINTAINER_GUIDE.md)
