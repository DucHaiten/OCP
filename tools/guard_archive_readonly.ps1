param(
    [switch]$AllowArchiveChanges
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Get-ChangedPaths {
    $paths = New-Object System.Collections.Generic.List[string]
    $statusLines = @(git status --porcelain --untracked-files=all 2>$null)
    foreach ($line in $statusLines) {
        if ([string]::IsNullOrWhiteSpace($line)) {
            continue
        }
        if ($line.Length -lt 4) {
            continue
        }
        $rawPath = $line.Substring(3).Trim()
        if ($rawPath -like "* -> *") {
            $segments = $rawPath -split " -> "
            if ($segments.Length -gt 0) {
                $rawPath = $segments[$segments.Length - 1]
            }
        }
        $rawPath = $rawPath.Trim('"')
        if (-not [string]::IsNullOrWhiteSpace($rawPath)) {
            $paths.Add($rawPath)
        }
    }
    return $paths | Sort-Object -Unique
}

function Is-ArchivePath([string]$path) {
    if ([string]::IsNullOrWhiteSpace($path)) {
        return $false
    }
    $normalized = $path.Replace("\", "/").TrimStart("./")
    return $normalized.StartsWith("_archive/", [System.StringComparison]::OrdinalIgnoreCase)
}

$changed = @(Get-ChangedPaths)
$archiveChanged = @()
foreach ($path in $changed) {
    if (Is-ArchivePath $path) {
        $archiveChanged += $path
    }
}

$envOverride = $env:ALLOW_ARCHIVE_CHANGES -eq "1"
$explicitOverride = $AllowArchiveChanges.IsPresent

if ($archiveChanged.Count -gt 0 -and -not ($envOverride -and $explicitOverride)) {
    Write-Error "Archive read-only guard blocked: phát hiện thay đổi dưới _archive/**. Dùng --AllowArchiveChanges và ALLOW_ARCHIVE_CHANGES=1 nếu cần override có chủ đích."
    $archiveChanged | ForEach-Object { Write-Host " - $_" }
    exit 2
}

Write-Host "Archive read-only guard passed."
exit 0
