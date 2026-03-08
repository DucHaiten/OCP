param(
    [string]$InstallerPath = "target/ocp/w100/release/ocp-v1.0.0-setup-win-x64.exe",
    [string]$InstallerScriptPath = "installer/windows/ocp-win-x64.iss"
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$resolvedInstallerPath = if ([System.IO.Path]::IsPathRooted($InstallerPath)) {
    $InstallerPath
} else {
    Join-Path $repoRoot $InstallerPath
}
 $resolvedInstallerScriptPath = if ([System.IO.Path]::IsPathRooted($InstallerScriptPath)) {
    $InstallerScriptPath
} else {
    Join-Path $repoRoot $InstallerScriptPath
}

if (-not (Test-Path -LiteralPath $resolvedInstallerPath)) {
    throw "[installer-branding] installer not found: $resolvedInstallerPath"
}

[byte[]]$bytes = [System.IO.File]::ReadAllBytes($resolvedInstallerPath)
$ascii = [System.Text.Encoding]::ASCII.GetString($bytes)
$utf16 = [System.Text.Encoding]::Unicode.GetString($bytes)

$legacyToken = "OCP-OCL"
if ($ascii.Contains($legacyToken) -or $utf16.Contains($legacyToken)) {
    throw "[installer-branding] legacy token detected in installer binary: $legacyToken"
}

if (-not (Test-Path -LiteralPath $resolvedInstallerScriptPath)) {
    throw "[installer-branding] installer script not found: $resolvedInstallerScriptPath"
}
$iss = Get-Content -LiteralPath $resolvedInstallerScriptPath -Raw
if (-not $iss.Contains('#define MyAppName "OCP"')) {
    throw "[installer-branding] installer script must define MyAppName as OCP"
}
if (-not $iss.Contains('DefaultDirName={autopf}\OCP')) {
    throw "[installer-branding] installer script must keep default install dir as {autopf}\\OCP"
}

Write-Host "[installer-branding] PASS $resolvedInstallerPath"
exit 0
