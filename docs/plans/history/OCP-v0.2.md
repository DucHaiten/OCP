# OCP v0.2 Core Freeze Notes

Date: 2026-03-03

## Canonical Contracts
- Diagnostics taxonomy:
  - Parser: `P-*`
  - Typechecker: `T-*`
  - Executor: `X-*`
  - Runtime/Bridge integration: `R-*`
  - Root reason: `RC-*`
- Legacy `E-*` is alias migration only.

## Condition Canonical Rule
- Internal evaluator may produce `Pass/Fail/Deferred/Insufficient`.
- Public executor surface:
  - `Fail -> X-COND-FALSE`
  - `Deferred -> X-COND-DEFERRED`
  - `Insufficient -> X-COND-INSUFFICIENT` with `root_reason`.

## Conformance Profiles
- `P0`: core shipped in v0.2.
- `P1`: extensions deferred to v0.2.1.

## Typed Context Contract (core semantic fields)
- `seed:u64`
- `tick:u64`
- `observer_id:u64`
- `policy_id:u32`

