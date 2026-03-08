param(
    [string]$StageRoot = "target/ocp/w100/stage/win-x64",
    [string]$ReleaseRoot = "target/ocp/w100/release",
    [string]$PortableStageRoot = "target/ocp/w100/stage/portable",
    [string]$Version = "1.0.0",
    [string]$LinuxBinaryRoot = "target/x86_64-unknown-linux-musl/release",
    [string]$LinuxArchiveSource = "",
    [string]$MacosArchiveSource = ""
)

$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$stageRootPath = Join-Path $repoRoot $StageRoot
$releaseRootPath = Join-Path $repoRoot $ReleaseRoot
$portableStageRootPath = Join-Path $repoRoot $PortableStageRoot
$winStagePath = Join-Path $portableStageRootPath "win-x64"
$linuxStagePath = Join-Path $portableStageRootPath "linux-x64"
$winZipPath = Join-Path $releaseRootPath ("ocp-v{0}-win-x64.zip" -f $Version)
$linuxTarPath = Join-Path $releaseRootPath ("ocp-v{0}-linux-x64.tar.gz" -f $Version)

function Require-File {
    param([string]$PathValue)

    if (-not (Test-Path $PathValue)) {
        throw "Required file missing: $PathValue"
    }
}

function Copy-OptionalArchive {
    param(
        [string]$InputPath,
        [string]$OutputPath
    )

    if ($InputPath -eq "") {
        return
    }

    $resolved = if ([System.IO.Path]::IsPathRooted($InputPath)) {
        $InputPath
    } else {
        Join-Path $repoRoot $InputPath
    }
    Require-File -PathValue $resolved
    Copy-Item $resolved $OutputPath -Force
}

function Build-LinuxArchiveFromBinaryRoot {
    param(
        [string]$BinaryRootInput,
        [string]$LinuxStagePathValue,
        [string]$LinuxTarPathValue
    )

    $binaryRoot = if ([System.IO.Path]::IsPathRooted($BinaryRootInput)) {
        $BinaryRootInput
    } else {
        Join-Path $repoRoot $BinaryRootInput
    }

    if (-not (Test-Path $binaryRoot)) {
        return $false
    }

    $cli = Join-Path $binaryRoot "ocp-cli"
    $lsp = Join-Path $binaryRoot "ocp-lsp"
    $dap = Join-Path $binaryRoot "ocp-dap"
    if (-not (Test-Path $cli) -or -not (Test-Path $lsp) -or -not (Test-Path $dap)) {
        return $false
    }

    if (Test-Path $LinuxStagePathValue) {
        Remove-Item -Recurse -Force $LinuxStagePathValue
    }
    New-Item -ItemType Directory -Force -Path $LinuxStagePathValue | Out-Null

    Copy-Item $cli (Join-Path $LinuxStagePathValue "ocp") -Force
    Copy-Item $lsp (Join-Path $LinuxStagePathValue "ocp-lsp") -Force
    Copy-Item $dap (Join-Path $LinuxStagePathValue "ocp-dap") -Force

    if (Test-Path $LinuxTarPathValue) {
        Remove-Item -Force $LinuxTarPathValue
    }

    tar -a -c -f $LinuxTarPathValue -C $LinuxStagePathValue .
    if ($LASTEXITCODE -ne 0) {
        throw "Failed to build Linux tar.gz archive"
    }
    return $true
}

Require-File -PathValue (Join-Path $stageRootPath "ocp.exe")
Require-File -PathValue (Join-Path $stageRootPath "ocp-lsp.exe")
Require-File -PathValue (Join-Path $stageRootPath "ocp-dap.exe")
Require-File -PathValue (Join-Path $stageRootPath "INSTALL-NEXT-STEPS.txt")

if (Test-Path $winStagePath) {
    Remove-Item -Recurse -Force $winStagePath
}
New-Item -ItemType Directory -Force -Path $winStagePath | Out-Null
New-Item -ItemType Directory -Force -Path $releaseRootPath | Out-Null

Copy-Item (Join-Path $stageRootPath "ocp.exe") (Join-Path $winStagePath "ocp.exe") -Force
Copy-Item (Join-Path $stageRootPath "ocp-lsp.exe") (Join-Path $winStagePath "ocp-lsp.exe") -Force
Copy-Item (Join-Path $stageRootPath "ocp-dap.exe") (Join-Path $winStagePath "ocp-dap.exe") -Force
Copy-Item (Join-Path $stageRootPath "INSTALL-NEXT-STEPS.txt") (Join-Path $winStagePath "INSTALL-NEXT-STEPS.txt") -Force

if (Test-Path $winZipPath) {
    Remove-Item -Force $winZipPath
}

Compress-Archive -Path (Join-Path $winStagePath "*") -DestinationPath $winZipPath

$linuxOut = Join-Path $releaseRootPath ("ocp-v{0}-linux-x64.tar.gz" -f $Version)
$macosOut = Join-Path $releaseRootPath ("ocp-v{0}-macos-arm64.tar.gz" -f $Version)
$linuxBuilt = Build-LinuxArchiveFromBinaryRoot -BinaryRootInput $LinuxBinaryRoot -LinuxStagePathValue $linuxStagePath -LinuxTarPathValue $linuxTarPath
if (-not $linuxBuilt) {
    Copy-OptionalArchive -InputPath $LinuxArchiveSource -OutputPath $linuxOut
}
Copy-OptionalArchive -InputPath $MacosArchiveSource -OutputPath $macosOut

Write-Host "Portable Windows zip built at $winZipPath"
