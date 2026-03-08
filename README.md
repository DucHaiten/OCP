# OCP

**English:**

OCP (Observation-Collapse Programming) is a governed general-purpose language/runtime, built around observe -> 4-kind -> commit to achieve determinism, bounded execution, and capability/permission-controlled side effects.

OCP is not a “narrow domain-only DSL”: it can power tools/apps/games/agent runtimes via packs/engines, packaging + lockfiles, trace/debug/diff tooling, conformance suites, and an LTS rehearsal track. Yet it still feels DSL-strict because the core semantics are deliberately constrained: all reads must go through observe, all outcomes are normalized into 4 kinds (OK/DEGRADED/INSUFFICIENT/DEFERRED), and all I/O/side effects are commit-gated under explicit policy.

OCP follows the principle “observe before collapse”: all world-data must go through observe, all outcomes are normalized into a 4-kind contract (OK/DEGRADED/INSUFFICIENT/DEFERRED), and only valid side-effects may pass through commit. The goal is deterministic behavior, bounded execution (budgets/caps), and permissioned control over I/O.

OCP evolves into a capability/permission-driven and replay-first platform: packages ship with permission manifests, lockfile + signing + attestation, deterministic-by-recording cassettes for non-deterministic I/O, bounded shadow/search (parallel “realities” under measurable budgets), plus trace/debug/diff/minimizer tooling, a global conformance suite, and an LTS rehearsal track.

The underlying ideas synthesize concepts from TSS (TIERED STATE SYSTEM), ELW (ENTITY-LESS WORLD), RGOK (REALITY-GATED OBSERVATION KERNEL), SHSM (Synchronous Hyperdimensional State Matrix), a biologic direction, cosmology motifs (parallel/superposed/horizon), and Hive.

#

**Tiếng Việt:**

OCP (Observation-Collapse Programming) là một ngôn ngữ + runtime đa dụng có governance rõ ràng, được thiết kế quanh mô hình observe -> 4-kind -> commit để đạt determinism, bounded execution, và kiểm soát side-effect theo capability/permission.

OCP không phải “DSL thuần theo domain hẹp” nó có thể xây tool/app/game/agent runtime nhờ hệ packs/engines, packaging/lockfile, trace/debug/diff, conformance và LTS. Tuy nhiên nó vẫn mang “dáng dấp khắt khe của DSL” vì semantics lõi bị ràng buộc nghiêm ngặt: mọi đọc world-data phải qua observe, mọi kết quả đi qua 4-kind (OK/DEGRADED/INSUFFICIENT/DEFERRED), và mọi I/O/side-effect chỉ được phép xảy ra qua commit theo policy.

OCP được thiết kế quanh nguyên lý “quan sát rồi mới sụp trạng thái”: mọi world-data bắt buộc đi qua observe, mọi kết quả chuẩn hoá theo 4-kind (OK/DEGRADED/INSUFFICIENT/DEFERRED), và chỉ side-effect hợp lệ mới được đi qua commit. Mục tiêu là giữ tính xác định, bounded execution (budgets/caps), và kiểm soát I/O theo quyền hạn.

OCP mở rộng thành nền tảng capability/permission-driven và replay-first: packages kèm permission manifests, lockfile + signing + attestation, deterministic-by-recording (cassette) cho I/O không xác định, shadow/search bounded (song song các “nhánh thực tại” trong ngân sách đo được), cùng toolchain trace/debug/diff/minimizer, conformance suite và LTS rehearsal.

Nền tảng ý tưởng tổng hợp từ TSS (TIERED STATE SYSTEM), ELW (ENTITY-LESS WORLD), RGOK (REALITY-GATED OBSERVATION KERNEL), SHSM (Synchronous Hyperdimensional State Matrix), hướng Biologic, vũ trụ học (song song/chồng chập/chân trời) và Hive.

**Replayable programs with real I/O** - deterministic runs, bounded execution, and policy-gated side effects.

<p align="center">
  <a href="docs/en/USER_GUIDE.md">Docs (EN)</a> •
  <a href="docs/vi/USER_GUIDE.md">Tài liệu (VI)</a> •
  <a href="docs/en/security/verify-download.md">Verify Download</a> •
  <a href="GOVERNANCE.md">Governance</a> •
  <a href="LICENSE">AGPL-3.0-only</a>
</p>

## What is OCP
OCP is a general-purpose language/runtime for teams that need predictable behavior under real-world I/O, not only pure compute.

Execution contract:
- `observe`: read world state through declared capabilities.
- `4-kind`: normalize outcomes to `OK | DEGRADED | INSUFFICIENT | DEFERRED`.
- `commit`: execute side effects only when permission/policy allows.

Why it is different in production:
- Replay is a first-class artifact (`run -> artifact -> replay`).
- Permission boundaries are explicit (`fs/net/proc/time`), not ambient.
- Bounded execution is built in (budgets/caps, bounded shadow/search).
- Ops/debug tooling is native (`trace view/diff`, `dbg`, `minimize`, `budget analyze`).

Use OCP when:
- You need reproducible automation, agents, or tool pipelines.
- You need auditable side effects and strict permission control.
- You need incident-debug loops based on replay evidence.

Do not use OCP when:
- You only need throwaway scripts with unconstrained I/O and zero governance.

## Install
Official install and verification:
- EN install: [docs/en/USER_GUIDE.md#install](docs/en/USER_GUIDE.md#install)
- VI install: [docs/vi/USER_GUIDE.md#install](docs/vi/USER_GUIDE.md#install)
- EN verify download: [docs/en/security/verify-download.md](docs/en/security/verify-download.md)
- VI verify download: [docs/vi/security/verify-download.md](docs/vi/security/verify-download.md)

Windows setup package:
- [Download `ocp-v1.0.0-setup-win-x64.exe`](https://github.com/DucHaiten/OCP/blob/main/installer/releases/ocp-v1.0.0-setup-win-x64.exe?raw=1)
- [All releases](https://github.com/DucHaiten/OCP/releases)

Install from source (git):
```bash
git clone https://github.com/DucHaiten/OCP.git
cd OCP
cargo run --bin ocp-cli -- --version
```

## Quickstart
60-second demo (real I/O, replayable):

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
  OK => { commit(wr); condition(true); }
  DEGRADED => { condition(false); }
  INSUFFICIENT => { condition(false); }
  DEFERRED => { condition(false); }
}
```

Run flow:
```bash
ocp init hello --template tool-cli
cd hello
ocp lock sync .
ocp run .
ocp replay ./.ocp_artifacts/<run_id>/
```

Expected result:
- `./out/out.json` is created.
- Run artifacts are created under `.ocp_artifacts/<run_id>/`.
- Replay reproduces the run behavior deterministically.
- Full production flow (`lock sign/verify`, `perm`, `attest`) is in USER_GUIDE.

Full guides:
- EN: [docs/en/USER_GUIDE.md#quickstart](docs/en/USER_GUIDE.md#quickstart)
- VI: [docs/vi/USER_GUIDE.md#quickstart](docs/vi/USER_GUIDE.md#quickstart)

## Editor
VSCode/editor workflow:
- EN: [docs/en/USER_GUIDE.md#editor-workflow](docs/en/USER_GUIDE.md#editor-workflow)
- VI: [docs/vi/USER_GUIDE.md#editor-workflow](docs/vi/USER_GUIDE.md#editor-workflow)

Practical notes from v1.0 docs:
- `.ocp` association, snippets, and diagnostics are supported.
- In trusted workspaces, bridge commands can run bundled CLI workflows (`fmt`, `check`, `doctor`, `fix`, trace/debug actions).

## Security
Security and supply-chain references:
- EN verify: [docs/en/security/verify-download.md](docs/en/security/verify-download.md)
- VI verify: [docs/vi/security/verify-download.md](docs/vi/security/verify-download.md)
- Governance: [GOVERNANCE.md](GOVERNANCE.md)
- Project boundary: [docs/PROJECT-BOUNDARY.md](docs/PROJECT-BOUNDARY.md)

Key operational controls in v1.0:
- Lock chain: `lock sync -> lock sign -> lock verify`
- Build provenance: `build --attest` + `verify --attest`
- Permission review flow: `perm snapshot -> perm diff -> perm approve`

## License
OSS licensing is `AGPL-3.0-only`.
- License file: [LICENSE](LICENSE)
- EN legal details: [docs/en/legal/licensing.md](docs/en/legal/licensing.md)
- VI legal details: [docs/vi/legal/licensing.md](docs/vi/legal/licensing.md)

## Commercial Use
Commercial terms and scope:
- EN: [docs/en/legal/commercial.md](docs/en/legal/commercial.md)
- VI: [docs/vi/legal/commercial.md](docs/vi/legal/commercial.md)

## Contributing
Contribution workflow and standards:
- [CONTRIBUTING.md](CONTRIBUTING.md)
