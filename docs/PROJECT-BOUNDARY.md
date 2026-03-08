# OCP Project Boundary (M0-A)

## Source Of Truth
- OCP v0.3 track lives under `projects/ocp/`.
- Legacy engine code for v0.1/v0.2 remains in `src/ocp/` and is consumed as runtime core backend during M0-A.

## OCP-only Workspace Rule
- Application workspace must contain only:
  - `.ocp` sources
  - `Ocp.toml`
  - `tests/` with `.ocp` files
  - generated bundle output (`.ocpbundle/`)
- No host-language app code (`.rs`, `.cpp`) is required to run OCP apps.

## Toolchain Boundary
- `ocp-runtime-core`: wraps parse/typecheck/exec primitives.
- `ocp-sdk`: project lifecycle APIs (`init/check/run/fmt/test/build`).
- `ocp-cli`: user-facing command entry.
