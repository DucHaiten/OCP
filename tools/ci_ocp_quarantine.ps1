$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$outDir = "target/ocp/quarantine"
New-Item -ItemType Directory -Path $outDir -Force | Out-Null

$checks = @(
    @{
        name = "hello_cli_run_locked"
        action = {
            & cargo run -p ocp-cli -- run "projects/ocp/apps/hello-cli" --locked
        }
    },
    @{
        name = "mini_server_reactor_smoke"
        action = {
            & cargo run -p ocp-cli -- run "projects/ocp/apps/mini-server" --locked --reactor --ticks 8
        }
    },
    @{
        name = "composer_demo_verify_locked"
        action = {
            & cargo run -p ocp-cli -- test "projects/ocp/apps/composer-demo" --locked
        }
    },
    @{
        name = "w9_conformance_throughput_bytecode"
        action = {
            & cargo run -p ocp-cli -- test --conformance --locked --runtime throughput --engine bytecode --manifest "projects/ocp/conformance/conformance.v1.toml" --out "target/ocp/w9/reports/conformance_quarantine.json" --trust-store "projects/ocp/security/trust.store.toml" --signer-id "dev-root-1" --sign-key "projects/ocp/security/dev-root-1.signing.key.toml" --json
        }
    }
)

$results = @()

foreach ($check in $checks) {
    $start = Get-Date
    $status = "pass"
    $exitCode = 0
    $errorMessage = $null

    Write-Host "[quarantine] START $($check.name)"
    try {
        & $check.action
        if ($LASTEXITCODE -ne 0) {
            $status = "fail"
            $exitCode = $LASTEXITCODE
        }
    }
    catch {
        $status = "fail"
        $exitCode = if ($LASTEXITCODE -ne 0) { $LASTEXITCODE } else { 1 }
        $errorMessage = $_.Exception.Message
    }

    $durationMs = [int]((Get-Date) - $start).TotalMilliseconds
    $results += [pscustomobject]@{
        name = $check.name
        status = $status
        exit_code = $exitCode
        duration_ms = $durationMs
        error = $errorMessage
    }

    Write-Host "[quarantine] END   $($check.name) status=$status exit=$exitCode"
}

$failCount = @($results | Where-Object { $_.status -eq "fail" }).Count
$passCount = @($results | Where-Object { $_.status -eq "pass" }).Count

$report = [pscustomobject]@{
    generated_at = (Get-Date).ToString("o")
    suite = "ocp_quarantine_non_blocking"
    blocking = $false
    summary = [pscustomobject]@{
        total = $results.Count
        pass = $passCount
        fail = $failCount
    }
    results = $results
}

$reportPath = Join-Path $outDir "report.json"
$report | ConvertTo-Json -Depth 8 | Set-Content -Path $reportPath -Encoding UTF8

if ($failCount -gt 0) {
    Write-Warning "[quarantine] completed with failures (non-blocking). report=$reportPath"
}
else {
    Write-Host "[quarantine] PASS. report=$reportPath"
}

exit 0
