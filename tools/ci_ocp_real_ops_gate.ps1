$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

function Invoke-Step {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][scriptblock]$Action
    )
    Write-Host "[real-ops-gate] START $Name"
    & $Action
    if ($LASTEXITCODE -ne 0) {
        throw "[real-ops-gate] FAIL $Name (exit=$LASTEXITCODE)"
    }
    Write-Host "[real-ops-gate] PASS  $Name"
}

Invoke-Step "cli_real_shadow_hive_ops_e2e" {
    & cargo test --test cli_real_shadow_hive_ops_e2e
}

Invoke-Step "cli_shadow_preview_v2_e2e + v1_user_journey_smoke" {
    & cargo test --test cli_shadow_preview_v2_e2e --test v1_user_journey_smoke
}

Invoke-Step "ocp_sdk_v5_w4_hive" {
    & cargo test -p ocp-sdk --test v5_w4_hive
}

Invoke-Step "flake_probe_round_1" {
    & cargo test --test cli_real_shadow_hive_ops_e2e
}

Invoke-Step "flake_probe_round_2" {
    & cargo test --test cli_real_shadow_hive_ops_e2e
}

Write-Host "[real-ops-gate] DONE"
exit 0
