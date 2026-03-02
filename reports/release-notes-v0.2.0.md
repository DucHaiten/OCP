# Release Notes v0.2.0

Date: 2026-03-03

## Highlights
- Canonical diagnostics taxonomy freeze: `P-* / T-* / X-* / R-* / RC-*`.
- Added root reason propagation in diagnostics (`root_reason=RC-*`).
- Runtime strict context validation with canonical `R-CTX-*` surface.
- Bounded condition evaluator with explicit outcomes:
  - `X-COND-FALSE`
  - `X-COND-DEFERRED`
  - `X-COND-INSUFFICIENT`
- Bounded entangle runtime semantics:
  - session-local constraints
  - edge/degree/propagation caps
  - fail-honest canonical errors

## Compatibility
- Legacy `E-*` kept as migration alias path.
- Canonical output surface is `P/T/X/R`.

## Deferred to v0.2.1
- Typed context AST/parser/checker.
- `horizon=...` grammar and full runtime mapping.

