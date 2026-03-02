# OCL Project Boundary (M0-A)

## Source Of Truth
- OCL v0.3 track lives under `projects/ocp-ocl/`.
- Legacy engine code for v0.1/v0.2 remains in `src/ocp_ocl/` and is consumed as runtime core backend during M0-A.

## OCL-only Workspace Rule
- Application workspace must contain only:
  - `.ocl` sources
  - `Ocl.toml`
  - `tests/` with `.ocl` files
  - generated bundle output (`.oclbundle/`)
- No host-language app code (`.rs`, `.cpp`) is required to run OCL apps.

## Toolchain Boundary
- `ocl-runtime-core`: wraps parse/typecheck/exec primitives.
- `ocl-sdk`: project lifecycle APIs (`init/check/run/fmt/test/build`).
- `ocl-cli`: user-facing command entry.
