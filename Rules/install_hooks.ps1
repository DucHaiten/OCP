param()

$ErrorActionPreference = "Stop"
$repoRoot = Split-Path -Parent $PSScriptRoot
$hooksPath = Join-Path $repoRoot "Rules/hooks"

Write-Host "[rules] set core.hooksPath -> $hooksPath"
git config core.hooksPath "$hooksPath"
Write-Host "[rules] done"
