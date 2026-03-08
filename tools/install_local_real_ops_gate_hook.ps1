param(
    [switch]$Force
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$hookDir = Join-Path ".git" "hooks"
$hookPath = Join-Path $hookDir "pre-push"

if (-not (Test-Path -LiteralPath $hookDir)) {
    throw "[install-hook] missing .git/hooks (run inside repository root)"
}

if ((Test-Path -LiteralPath $hookPath) -and -not $Force) {
    throw "[install-hook] pre-push hook already exists. Re-run with -Force to overwrite."
}

$script = @'
#!/usr/bin/env bash
set -euo pipefail

echo "[pre-push] running OCP real command-flow gate..."
powershell -ExecutionPolicy Bypass -File tools/ci_ocp_real_ops_gate.ps1
'@

[System.IO.File]::WriteAllText($hookPath, $script)

Write-Host "[install-hook] installed pre-push hook at $hookPath"
Write-Host "[install-hook] pushes will be blocked when real gate fails"
