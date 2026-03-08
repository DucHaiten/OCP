param(
    [switch]$FullClean,
    [string]$SignerId = "dev-root-1",
    [string]$SignKey = "projects/ocp/security/dev-root-1.signing.key.toml",
    [string]$TrustStore = "projects/ocp/security/trust.store.toml"
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$ReportRoot = "target/ocp/v5/w0/reports"
$ManifestPath = "projects/ocp/conformance/conformance.v1.toml"
$DetRun1Report = Join-Path $ReportRoot "conformance.det.run1.json"
$DetRun2Report = Join-Path $ReportRoot "conformance.det.run2.json"
$FailManifest = Join-Path $ReportRoot "conformance.fail.intentional.toml"
$FailReport = Join-Path $ReportRoot "conformance.fail.intentional.json"
$VerifyPassReport = Join-Path $ReportRoot "verify_supply.pass.json"
$VerifyFailReport = Join-Path $ReportRoot "verify_supply.fail.tampered.json"
$PluginVerifyPassReport = Join-Path $ReportRoot "plugin.verify_supply.pass.json"
$StatusReport = Join-Path $ReportRoot "w0_status.json"

function Ensure-Directory {
    param([Parameter(Mandatory = $true)][string]$Path)
    if (-not (Test-Path -LiteralPath $Path)) {
        New-Item -ItemType Directory -Path $Path -Force | Out-Null
    }
}

function Assert-FileExists {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [Parameter(Mandatory = $true)][string]$Label
    )
    if (-not (Test-Path -LiteralPath $Path)) {
        throw "[w0-entry] missing ${Label}: $Path"
    }
}

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

function Invoke-Native {
    param(
        [Parameter(Mandatory = $true)][string]$Program,
        [Parameter(Mandatory = $true)][string[]]$Args
    )
    & $Program @Args
    if ($LASTEXITCODE -ne 0) {
        throw "[w0-entry] command failed: $Program $($Args -join ' ') (exit=$LASTEXITCODE)"
    }
}

function Set-DeterministicPins {
    $env:TZ = "UTC"
    $env:LANG = "C"
    $env:LC_ALL = "C"
    [System.Threading.Thread]::CurrentThread.CurrentCulture = [System.Globalization.CultureInfo]::InvariantCulture
    [System.Threading.Thread]::CurrentThread.CurrentUICulture = [System.Globalization.CultureInfo]::InvariantCulture
}

function Read-RequiredDigest {
    param([Parameter(Mandatory = $true)][string]$Path)
    $raw = Get-Content -LiteralPath $Path -Raw
    $obj = $raw | ConvertFrom-Json
    if (-not $obj.required_digest) {
        throw "[w0-entry] missing required_digest in report: $Path"
    }
    return [string]$obj.required_digest
}

function Run-Conformance {
    param(
        [Parameter(Mandatory = $true)][string]$Manifest,
        [Parameter(Mandatory = $true)][string]$OutPath
    )

    Invoke-Native -Program "cargo" -Args @(
        "run", "-p", "ocp-cli", "--",
        "test", "--conformance",
        "--locked",
        "--runtime", "deterministic",
        "--engine", "dual",
        "--manifest", $Manifest,
        "--out", $OutPath,
        "--trust-store", $TrustStore,
        "--signer-id", $SignerId,
        "--sign-key", $SignKey,
        "--json"
    )
}

Ensure-Directory -Path $ReportRoot

if ($FullClean) {
    Write-Host "[w0-entry] FullClean requested. Existing reports will be overwritten in-place."
}

Invoke-Step "deterministic_pins" {
    Set-DeterministicPins
}

Invoke-Step "guard_project_boundaries" {
    & "$PSScriptRoot/guard_project_boundaries.ps1"
}

Invoke-Step "guard_archive_readonly" {
    & "$PSScriptRoot/guard_archive_readonly.ps1"
}

Invoke-Step "signed_preflight" {
    Assert-FileExists -Path $SignKey -Label "sign key"
    Assert-FileExists -Path $TrustStore -Label "trust store"
    if ([string]::IsNullOrWhiteSpace($SignerId)) {
        throw "[w0-entry] missing signer id"
    }
}

Invoke-Step "topology_freeze_before" {
    & "$PSScriptRoot/guard_v5_topology_freeze.ps1" -Phase Before -ReportRoot $ReportRoot
}

Invoke-Step "cargo_check_workspace" {
    Invoke-Native -Program "cargo" -Args @("check", "--workspace")
}

Invoke-Step "cargo_fmt_check" {
    Invoke-Native -Program "cargo" -Args @("fmt", "--", "--check")
}

Invoke-Step "cargo_clippy_workspace" {
    Invoke-Native -Program "cargo" -Args @("clippy", "--workspace", "--all-targets", "--", "-D", "warnings")
}

Invoke-Step "ci_ocp_lane" {
    & "$PSScriptRoot/ci_ocp_lane.ps1"
}

Invoke-Step "deterministic_conformance_run1" {
    Run-Conformance -Manifest $ManifestPath -OutPath $DetRun1Report
    Assert-FileExists -Path $DetRun1Report -Label "deterministic report run1"
}

Invoke-Step "deterministic_conformance_run2" {
    Run-Conformance -Manifest $ManifestPath -OutPath $DetRun2Report
    Assert-FileExists -Path $DetRun2Report -Label "deterministic report run2"
}

$digestRun1 = ""
$digestRun2 = ""
Invoke-Step "deterministic_digest_compare" {
    $digestRun1 = Read-RequiredDigest -Path $DetRun1Report
    $digestRun2 = Read-RequiredDigest -Path $DetRun2Report
    if ($digestRun1 -ne $digestRun2) {
        throw "[w0-entry] deterministic digest mismatch: run1=$digestRun1 run2=$digestRun2"
    }
}

Invoke-Step "conformance_fail_intentional_report_written" {
    @(
        "version = 1",
        "",
        "[[scenario]]",
        "name = `"intentional-fail`"",
        "path = `"projects/ocp/apps/__intentional_missing__`"",
        "reactor_ticks = 0",
        "composer = false"
    ) | Set-Content -LiteralPath $FailManifest -Encoding ASCII

    & cargo run -p ocp-cli -- test --conformance --locked --runtime deterministic --engine dual --manifest $FailManifest --out $FailReport --trust-store $TrustStore --signer-id $SignerId --sign-key $SignKey --json
    $failExit = [int]$LASTEXITCODE
    if ($failExit -eq 0) {
        throw "[w0-entry] intentional fail conformance unexpectedly passed."
    }
    Assert-FileExists -Path $FailReport -Label "intentional fail conformance report"
    $global:LASTEXITCODE = 0
}

Invoke-Step "verify_supply_positive_and_negative" {
    $project = "projects/ocp/apps/hello-cli"
    $artifact = Join-Path $project ".ocppkg/hello_cli-0.1.0.ocppkg"
    $tampered = Join-Path $project ".ocppkg/hello_cli-0.1.0.tampered.ocppkg"

    Invoke-Native -Program "cargo" -Args @("run", "-p", "ocp-cli", "--", "build", $project, "--locked")
    Assert-FileExists -Path $artifact -Label "hello-cli artifact"

    Invoke-Native -Program "cargo" -Args @("run", "-p", "ocp-cli", "--", "verify-supply", $artifact)
    [ordered]@{
        valid = $true
        artifact = $artifact
    } | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $VerifyPassReport -Encoding UTF8

    Copy-Item -LiteralPath $artifact -Destination $tampered -Force
    Add-Content -LiteralPath $tampered -Value "`ntampered=1"

    & cargo run -p ocp-cli -- verify-supply $tampered
    $tamperedExit = [int]$LASTEXITCODE
    if ($tamperedExit -eq 0) {
        throw "[w0-entry] tampered artifact verify-supply unexpectedly passed."
    }
    [ordered]@{
        valid = $false
        artifact = $tampered
        expected_fail = $true
    } | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $VerifyFailReport -Encoding UTF8
    $global:LASTEXITCODE = 0
}

Invoke-Step "plugin_sub_gate" {
    $pluginProject = "projects/ocp/apps/plugin-demo"
    $pluginArtifact = Join-Path $pluginProject ".ocppkg/plugin_demo-0.1.0.ocppkg"

    Invoke-Native -Program "cargo" -Args @("run", "-p", "ocp-cli", "--", "plugin", "lock", "sync", $pluginProject)
    Invoke-Native -Program "cargo" -Args @("run", "-p", "ocp-cli", "--", "plugin", "verify", $pluginProject)
    Invoke-Native -Program "cargo" -Args @("run", "-p", "ocp-cli", "--", "build", $pluginProject, "--locked")
    Assert-FileExists -Path $pluginArtifact -Label "plugin-demo artifact"
    Invoke-Native -Program "cargo" -Args @("run", "-p", "ocp-cli", "--", "verify-supply", $pluginArtifact)
    Invoke-Native -Program "cargo" -Args @("run", "-p", "ocp-cli", "--", "run", $pluginProject, "--locked", "--engine", "dual")

    [ordered]@{
        valid = $true
        artifact = $pluginArtifact
        plugin_project = $pluginProject
    } | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath $PluginVerifyPassReport -Encoding UTF8
}

Invoke-Step "topology_freeze_after" {
    & "$PSScriptRoot/guard_v5_topology_freeze.ps1" -Phase After -ReportRoot $ReportRoot
}

Invoke-Step "write_w0_status_report" {
    $status = [ordered]@{
        status = "pass"
        signer_id = $SignerId
        trust_store = $TrustStore
        sign_key = $SignKey
        deterministic_digest_run1 = $digestRun1
        deterministic_digest_run2 = $digestRun2
        reports = @(
            $DetRun1Report,
            $DetRun2Report,
            $FailReport,
            $VerifyPassReport,
            $VerifyFailReport,
            $PluginVerifyPassReport,
            (Join-Path $ReportRoot "topology_freeze_before.json"),
            (Join-Path $ReportRoot "topology_freeze_hash_snapshot.json")
        )
    }
    $status | ConvertTo-Json -Depth 6 | Set-Content -LiteralPath $StatusReport -Encoding UTF8
}

Write-Host "[w0-entry] DONE"
exit 0
