$ErrorActionPreference = "Stop"

$workspace = "projects/ocp-ocl/app-ocl"
if (-not (Test-Path $workspace)) {
    Write-Error "Workspace not found: $workspace"
    exit 1
}

$forbidden = @(Get-ChildItem -Path $workspace -Recurse -File | Where-Object {
    $_.Extension -in @(".rs", ".cpp", ".cc", ".cxx", ".c")
})

if (@($forbidden).Count -gt 0) {
    Write-Error "Boundary violation: host code detected in OCL app workspace."
    $forbidden | ForEach-Object { Write-Host $_.FullName }
    exit 1
}

Write-Host "Boundary guard passed."
