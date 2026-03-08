param(
    [string]$Owner = "",
    [string]$Repo = "",
    [string]$Branch = "main",
    [string]$RequiredCheckContext = "real command-flow gate",
    [switch]$DryRun
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

if ([string]::IsNullOrWhiteSpace($Owner) -or [string]::IsNullOrWhiteSpace($Repo)) {
    $repoView = & gh repo view --json nameWithOwner -q .nameWithOwner 2>$null
    if ($LASTEXITCODE -eq 0 -and -not [string]::IsNullOrWhiteSpace($repoView)) {
        $parts = $repoView.Trim().Split('/')
        if ($parts.Length -eq 2) {
            if ([string]::IsNullOrWhiteSpace($Owner)) { $Owner = $parts[0] }
            if ([string]::IsNullOrWhiteSpace($Repo)) { $Repo = $parts[1] }
        }
    }
}

if ([string]::IsNullOrWhiteSpace($Owner) -or [string]::IsNullOrWhiteSpace($Repo)) {
    throw "[branch-protection] missing owner/repo (pass -Owner/-Repo or set gh repo context)"
}

function Invoke-GhApiRaw {
    param(
        [Parameter(Mandatory = $true)][string]$Path,
        [string]$Method = "GET",
        [string]$InputFile = ""
    )

    $args = @("api", "--method", $Method, $Path)
    if (-not [string]::IsNullOrWhiteSpace($InputFile)) {
        $args += @("--input", $InputFile)
    }

    $output = & gh @args 2>&1
    $code = [int]$LASTEXITCODE
    return [pscustomobject]@{
        ExitCode = $code
        Output = [string]$output
    }
}

function Write-JsonToTempFile {
    param([Parameter(Mandatory = $true)]$Value)
    $path = Join-Path $env:TEMP ("ocp_branch_protection_" + [Guid]::NewGuid().ToString("N") + ".json")
    $json = $Value | ConvertTo-Json -Depth 32
    [System.IO.File]::WriteAllText($path, $json)
    return $path
}

$protectionPath = "repos/$Owner/$Repo/branches/$Branch/protection"
$requiredChecksPath = "$protectionPath/required_status_checks"

Write-Host "[branch-protection] target: $Owner/$Repo branch=$Branch context=$RequiredCheckContext"

$currentProtection = Invoke-GhApiRaw -Path $protectionPath -Method "GET"
$branchProtectionExists = $false
$contexts = @()

if ($currentProtection.ExitCode -eq 0) {
    $branchProtectionExists = $true
    $parsed = $currentProtection.Output | ConvertFrom-Json
    if ($null -ne $parsed.required_status_checks -and $null -ne $parsed.required_status_checks.contexts) {
        $contexts = @($parsed.required_status_checks.contexts)
    }
} elseif ($currentProtection.Output -match "HTTP 404") {
    $branchProtectionExists = $false
    $contexts = @()
} else {
    throw "[branch-protection] cannot read branch protection: $($currentProtection.Output)"
}

if ($contexts -notcontains $RequiredCheckContext) {
    $contexts += $RequiredCheckContext
}

if ($DryRun) {
    Write-Host "[branch-protection] dry-run mode"
    Write-Host "[branch-protection] existing_protection=$branchProtectionExists"
    Write-Host "[branch-protection] required_contexts=$($contexts -join ', ')"
    exit 0
}

if ($branchProtectionExists) {
    $payload = @{
        strict = $true
        contexts = $contexts
    }
    $payloadPath = Write-JsonToTempFile -Value $payload
    try {
        $setChecks = Invoke-GhApiRaw -Path $requiredChecksPath -Method "PATCH" -InputFile $payloadPath
        if ($setChecks.ExitCode -ne 0) {
            throw "[branch-protection] failed to patch required checks: $($setChecks.Output)"
        }
    }
    finally {
        Remove-Item -LiteralPath $payloadPath -Force -ErrorAction SilentlyContinue
    }
} else {
    $payload = @{
        required_status_checks = @{
            strict = $true
            contexts = $contexts
        }
        enforce_admins = $false
        required_pull_request_reviews = $null
        restrictions = $null
    }
    $payloadPath = Write-JsonToTempFile -Value $payload
    try {
        $setProtection = Invoke-GhApiRaw -Path $protectionPath -Method "PUT" -InputFile $payloadPath
        if ($setProtection.ExitCode -ne 0) {
            throw "[branch-protection] failed to create branch protection: $($setProtection.Output)"
        }
    }
    finally {
        Remove-Item -LiteralPath $payloadPath -Force -ErrorAction SilentlyContinue
    }
}

Write-Host "[branch-protection] done"
exit 0
