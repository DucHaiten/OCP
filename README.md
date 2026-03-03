# OCP-OCL

**Bản tiếng Việt:**

OCP-OCL (Observation-Collapse Programming / Observation Collapse Language) là DSL + runtime cho mô hình observe → 4-kind → commit, deterministic và commit-gated I/O.

OCP-OCL được thiết kế quanh nguyên lý “quan sát rồi mới sụp trạng thái”: mọi world-data bắt buộc đi qua observe, mọi kết quả chuẩn hoá theo 4-kind (OK/DEGRADED/INSUFFICIENT/DEFERRED), và chỉ side-effect hợp lệ mới được đi qua commit. Mục tiêu là giữ tính xác định, bounded execution (budgets/caps), và kiểm soát I/O theo quyền hạn.

Từ v0.15, OCP-OCL mở rộng thành nền tảng capability/permission-driven và replay-first: packages kèm permission manifests, lockfile + signing + attestation, deterministic-by-recording (cassette) cho I/O không xác định, shadow/search bounded (song song các “nhánh thực tại” trong ngân sách đo được), cùng toolchain trace/debug/diff/minimizer, conformance suite và LTS rehearsal.

Nền tảng ý tưởng tổng hợp từ TSS, ELW, RGOK, SHSM, hướng Biologic, vũ trụ học (song song/chồng chập/chân trời) và Hive.

#

**English version:**

OCP-OCL (Observation-Collapse Programming / Observation Collapse Language) is a DSL + runtime built around observe → 4-kind → commit, with deterministic execution and commit-gated I/O.

OCP-OCL follows the principle “observe before collapse”: all world-data must go through observe, all outcomes are normalized into a 4-kind contract (OK/DEGRADED/INSUFFICIENT/DEFERRED), and only valid side-effects may pass through commit. The goal is deterministic behavior, bounded execution (budgets/caps), and permissioned control over I/O.

As of v0.15, OCP-OCL evolves into a capability/permission-driven and replay-first platform: packages ship with permission manifests, lockfile + signing + attestation, deterministic-by-recording cassettes for non-deterministic I/O, bounded shadow/search (parallel “realities” under measurable budgets), plus trace/debug/diff/minimizer tooling, a global conformance suite, and an LTS rehearsal track.

The underlying ideas synthesize concepts from TSS, ELW, RGOK, SHSM, a biologic direction, cosmology motifs (parallel/superposed/horizon), and Hive.
