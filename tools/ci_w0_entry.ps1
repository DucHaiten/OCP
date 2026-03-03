$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

function Invoke-Step {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][scriptblock]$Action
    )

    Write-Host "[w0-entry] START $Name"
    & $Action
    $exitCode = 0
    if (Get-Variable -Name LASTEXITCODE -Scope Global -ErrorAction SilentlyContinue) {
        $exitCode = [int]$LASTEXITCODE
    }
    if ($exitCode -ne 0) {
        throw "[w0-entry] FAIL $Name (exit=$exitCode)"
    }
    Write-Host "[w0-entry] PASS  $Name"
}

Invoke-Step "guard_project_boundaries" {
    & "$PSScriptRoot/guard_project_boundaries.ps1"
}

Invoke-Step "guard_archive_readonly" {
    & "$PSScriptRoot/guard_archive_readonly.ps1"
}

Invoke-Step "cargo_check_workspace" {
    & cargo check --workspace
}

Invoke-Step "cargo_fmt_check" {
    & cargo fmt -- --check
}

Invoke-Step "cargo_clippy_workspace" {
    & cargo clippy --workspace --all-targets -- -D warnings
}

Invoke-Step "ci_ocl_lane" {
    & "$PSScriptRoot/ci_ocl_lane.ps1"
}

Invoke-Step "m5_conformance" {
    & cargo test -p ocl-sdk --test m5_conformance
}

Invoke-Step "cargo_metadata_no_deps" {
    & cargo metadata --no-deps
}

Write-Host "[w0-entry] DONE"
exit 0
