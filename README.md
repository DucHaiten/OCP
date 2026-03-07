# OCP-OCL

**English:**

OCP-OCL (Observation-Collapse Programming / Observation Collapse Language) is a governed general-purpose language/runtime, built around observe → 4-kind → commit to achieve determinism, bounded execution, and capability/permission-controlled side effects.

OCP-OCL is not a “narrow domain-only DSL”: it can power tools/apps/games/agent runtimes via packs/engines, packaging + lockfiles, trace/debug/diff tooling, conformance suites, and an LTS rehearsal track. Yet it still feels DSL-strict because the core semantics are deliberately constrained: all reads must go through observe, all outcomes are normalized into 4 kinds (OK/DEGRADED/INSUFFICIENT/DEFERRED), and all I/O/side effects are commit-gated under explicit policy.

OCP-OCL follows the principle “observe before collapse”: all world-data must go through observe, all outcomes are normalized into a 4-kind contract (OK/DEGRADED/INSUFFICIENT/DEFERRED), and only valid side-effects may pass through commit. The goal is deterministic behavior, bounded execution (budgets/caps), and permissioned control over I/O.

OCP-OCL evolves into a capability/permission-driven and replay-first platform: packages ship with permission manifests, lockfile + signing + attestation, deterministic-by-recording cassettes for non-deterministic I/O, bounded shadow/search (parallel “realities” under measurable budgets), plus trace/debug/diff/minimizer tooling, a global conformance suite, and an LTS rehearsal track.

The underlying ideas synthesize concepts from TSS (TIERED STATE SYSTEM), ELW (ENTITY-LESS WORLD), RGOK (REALITY-GATED OBSERVATION KERNEL), SHSM (Synchronous Hyperdimensional State Matrix), a biologic direction, cosmology motifs (parallel/superposed/horizon), and Hive.

#

**Tiếng Việt:**

OCP-OCL (Observation-Collapse Programming / Observation Collapse Language) là một ngôn ngữ + runtime đa dụng có governance rõ ràng, được thiết kế quanh mô hình observe → 4-kind → commit để đạt determinism, bounded execution, và kiểm soát side-effect theo capability/permission.

OCP-OCL không phải “DSL thuần theo domain hẹp” nó có thể xây tool/app/game/agent runtime nhờ hệ packs/engines, packaging/lockfile, trace/debug/diff, conformance và LTS. Tuy nhiên nó vẫn mang “dáng dấp khắt khe của DSL” vì semantics lõi bị ràng buộc nghiêm ngặt: mọi đọc world-data phải qua observe, mọi kết quả đi qua 4-kind (OK/DEGRADED/INSUFFICIENT/DEFERRED), và mọi I/O/side-effect chỉ được phép xảy ra qua commit theo policy.

OCP-OCL được thiết kế quanh nguyên lý “quan sát rồi mới sụp trạng thái”: mọi world-data bắt buộc đi qua observe, mọi kết quả chuẩn hoá theo 4-kind (OK/DEGRADED/INSUFFICIENT/DEFERRED), và chỉ side-effect hợp lệ mới được đi qua commit. Mục tiêu là giữ tính xác định, bounded execution (budgets/caps), và kiểm soát I/O theo quyền hạn.

OCP-OCL mở rộng thành nền tảng capability/permission-driven và replay-first: packages kèm permission manifests, lockfile + signing + attestation, deterministic-by-recording (cassette) cho I/O không xác định, shadow/search bounded (song song các “nhánh thực tại” trong ngân sách đo được), cùng toolchain trace/debug/diff/minimizer, conformance suite và LTS rehearsal.

Nền tảng ý tưởng tổng hợp từ TSS (TIERED STATE SYSTEM), ELW (ENTITY-LESS WORLD), RGOK (REALITY-GATED OBSERVATION KERNEL), SHSM (Synchronous Hyperdimensional State Matrix), hướng Biologic, vũ trụ học (song song/chồng chập/chân trời) và Hive.

## What is OCL
- EN: OCL is a governed general-purpose language/runtime with deterministic `observe -> 4-kind -> commit` execution.
- VI: OCL là ngôn ngữ/runtime đa dụng có kiểm soát, khóa theo mô hình `observe -> 4-kind -> commit`.

## Install
- EN: See `docs/en/USER_GUIDE.md#install`.
- VI: Xem `docs/vi/USER_GUIDE.md#install`.

## Quickstart
- EN: Start at `docs/en/USER_GUIDE.md#quickstart`.
- VI: Bắt đầu tại `docs/vi/USER_GUIDE.md#quickstart`.

## Editor
- EN: VSCode flow is documented in `docs/en/USER_GUIDE.md#editor-workflow`.
- VI: Quy trình VSCode nằm ở `docs/vi/USER_GUIDE.md#editor-workflow`.

## Security
- EN: Download verification guide: `docs/en/security/verify-download.md`.
- VI: Hướng dẫn verify tải về: `docs/vi/security/verify-download.md`.

## License
- EN: The OSS side is `AGPL-3.0-only`. See `LICENSE` and `docs/en/legal/licensing.md`.
- VI: Nhánh OSS là `AGPL-3.0-only`. Xem `LICENSE` và `docs/vi/legal/licensing.md`.

## Commercial Use
- EN: If you need terms beyond the OSS side, see `docs/en/legal/commercial.md`.
- VI: Nếu bạn cần điều khoản ngoài nhánh OSS, xem `docs/vi/legal/commercial.md`.

## Contributing
- EN: See `CONTRIBUTING.md`.
- VI: Xem `CONTRIBUTING.md`.
