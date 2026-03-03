param(
    [ValidateSet("Before", "After")]
    [string]$Phase = "Before",
    [string]$ReportRoot = "target/ocl/v5/w0/reports"
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

function Ensure-Directory {
    param([Parameter(Mandatory = $true)][string]$Path)
    if (-not (Test-Path -LiteralPath $Path)) {
        New-Item -ItemType Directory -Path $Path -Force | Out-Null
    }
}

function Get-HashHex {
    param([Parameter(Mandatory = $true)][string]$InputText)
    $sha = [System.Security.Cryptography.SHA256]::Create()
    try {
        $bytes = [System.Text.Encoding]::UTF8.GetBytes($InputText)
        $hashBytes = $sha.ComputeHash($bytes)
        return ([System.BitConverter]::ToString($hashBytes)).Replace("-", "").ToLowerInvariant()
    }
    finally {
        $sha.Dispose()
    }
}

function Get-WorkspaceMembers {
    $raw = (& cargo metadata --no-deps --format-version 1) -join "`n"
    if ($LASTEXITCODE -ne 0) {
        throw "cargo metadata failed while collecting topology snapshot."
    }
    $obj = $raw | ConvertFrom-Json
    return @($obj.packages | ForEach-Object { [string]$_.name } | Sort-Object -Unique)
}

function Get-ConformanceScenarioOrder {
    $manifest = "projects/ocp-ocl/conformance/conformance.v1.toml"
    if (-not (Test-Path -LiteralPath $manifest)) {
        return @()
    }
    $names = New-Object System.Collections.Generic.List[string]
    foreach ($line in Get-Content -LiteralPath $manifest) {
        if ($line -match '^\s*name\s*=\s*"([^"]+)"\s*$') {
            $names.Add($Matches[1])
        }
    }
    return @($names)
}

function Get-PluginPlatformPins {
    $path = "projects/ocp-ocl/apps/plugin-demo/plugins.lock.v1"
    if (-not (Test-Path -LiteralPath $path)) {
        return @()
    }
    $pins = New-Object System.Collections.Generic.List[string]
    foreach ($line in Get-Content -LiteralPath $path) {
        if ($line.StartsWith("plugin=")) {
            $parts = $line.Split("`t")
            if ($parts.Length -ge 4) {
                $pins.Add($parts[0] + "|" + $parts[2])
            }
            else {
                $pins.Add($line)
            }
        }
    }
    return @($pins)
}

function Get-LanePins {
    $paths = @(
        "tools/ci_ocl_lane.ps1",
        "tools/ci_ocl_quarantine.ps1"
    )
    $pins = New-Object System.Collections.Generic.List[string]
    foreach ($path in $paths) {
        if (-not (Test-Path -LiteralPath $path)) {
            continue
        }
        foreach ($line in Get-Content -LiteralPath $path) {
            if ($line -match "test --conformance" -or $line -match "--engine bytecode" -or $line -match "plugin verify") {
                $pins.Add($line.Trim())
            }
        }
    }
    return @($pins)
}

function New-TopologySnapshot {
    $members = Get-WorkspaceMembers
    $scenarios = Get-ConformanceScenarioOrder
    $pluginPins = Get-PluginPlatformPins
    $lanePins = Get-LanePins

    $membersHash = Get-HashHex ([string]::Join("`n", $members))
    $scenariosHash = Get-HashHex ([string]::Join("`n", $scenarios))
    $pluginHash = Get-HashHex ([string]::Join("`n", $pluginPins))
    $laneHash = Get-HashHex ([string]::Join("`n", $lanePins))
    $overallHash = Get-HashHex ([string]::Join("`n", @($membersHash, $scenariosHash, $pluginHash, $laneHash)))

    return [ordered]@{
        generated_at = [DateTime]::UtcNow.ToString("o")
        workspace_members = $members
        conformance_scenario_order = $scenarios
        plugin_platform_pins = $pluginPins
        lane_pins = $lanePins
        hashes = [ordered]@{
            workspace_members = $membersHash
            conformance_scenario_order = $scenariosHash
            plugin_platform_pins = $pluginHash
            lane_pins = $laneHash
            overall = $overallHash
        }
    }
}

Ensure-Directory -Path $ReportRoot

$beforePath = Join-Path $ReportRoot "topology_freeze_before.json"
$afterPath = Join-Path $ReportRoot "topology_freeze_after.json"
$snapshotPath = Join-Path $ReportRoot "topology_freeze_hash_snapshot.json"
$statePath = Join-Path $ReportRoot "topology_freeze_state.json"

if ($Phase -eq "Before") {
    $before = New-TopologySnapshot
    $before | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $beforePath -Encoding UTF8
    $before | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $statePath -Encoding UTF8
    Write-Host "[v5-topology-freeze] PASS before snapshot recorded: $beforePath"
    exit 0
}

if (-not (Test-Path -LiteralPath $statePath)) {
    throw "Missing topology freeze baseline state. Run with -Phase Before first."
}

$beforeState = Get-Content -LiteralPath $statePath -Raw | ConvertFrom-Json
$after = New-TopologySnapshot
$after | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $afterPath -Encoding UTF8

$drift = New-Object System.Collections.Generic.List[string]
$keys = @("workspace_members", "conformance_scenario_order", "plugin_platform_pins", "lane_pins", "overall")
foreach ($key in $keys) {
    $beforeHash = [string]$beforeState.hashes.$key
    $afterHash = [string]$after.hashes.$key
    if ($beforeHash -ne $afterHash) {
        $drift.Add($key)
    }
}

$status = if ($drift.Count -eq 0) { "pass" } else { "fail" }
$summary = [ordered]@{
    status = $status
    drift = @($drift)
    before_snapshot = $beforePath
    after_snapshot = $afterPath
    before_hash = [string]$beforeState.hashes.overall
    after_hash = [string]$after.hashes.overall
}
$summary | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath $snapshotPath -Encoding UTF8

if ($drift.Count -gt 0) {
    throw "Topology freeze drift detected: $([string]::Join(', ', $drift))"
}

Write-Host "[v5-topology-freeze] PASS after snapshot recorded: $snapshotPath"
exit 0
