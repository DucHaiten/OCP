param(
    [string]$BinarySource = "",
    [string]$LspSource = "",
    [string]$DapSource = "",
    [string]$VsixSource = "",
    [string]$StageRoot = "target/ocl/w100/stage/win-x64",
    [string]$ReleaseRoot = "target/ocl/w100/release",
    [string]$IsccPath = ""
)

$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$stageRootPath = Join-Path $repoRoot $StageRoot
$releaseRootPath = Join-Path $repoRoot $ReleaseRoot
$installerScript = Join-Path $repoRoot "installer\windows\ocl-win-x64.iss"
$installerIconPath = Join-Path $repoRoot "installer\windows\ocl-installer.ico"
$installerIconScript = Join-Path $repoRoot "tools\release\render_installer_icon.py"
$vsixBuildScript = Join-Path $repoRoot "tools\release\build_vsix.ps1"

function Resolve-BinarySource {
    param(
        [string]$InputPath,
        [string]$RepoRoot,
        [string]$FriendlyName,
        [string[]]$Fallbacks
    )

    $candidates = @()
    if ($InputPath -ne "") {
        if ([System.IO.Path]::IsPathRooted($InputPath)) {
            $candidates += $InputPath
        } else {
            $candidates += (Join-Path $RepoRoot $InputPath)
        }
    }

    foreach ($fallback in $Fallbacks) {
        $candidates += (Join-Path $RepoRoot $fallback)
    }

    foreach ($candidate in $candidates) {
        if ($candidate -and (Test-Path $candidate)) {
            return (Resolve-Path $candidate).Path
        }
    }

    throw "Could not find built Windows binary: $FriendlyName"
}

function Resolve-IsccPath {
    param([string]$InputPath)

    if ($InputPath -ne "") {
        if (-not (Test-Path $InputPath)) {
            throw "ISCC.exe not found at explicit path: $InputPath"
        }
        return (Resolve-Path $InputPath).Path
    }

    $candidates = @(
        "$env:ProgramFiles(x86)\Inno Setup 6\ISCC.exe",
        "$env:ProgramFiles\Inno Setup 6\ISCC.exe"
    )

    foreach ($candidate in $candidates) {
        if (Test-Path $candidate) {
            return (Resolve-Path $candidate).Path
        }
    }

    throw "ISCC.exe not found. Install Inno Setup 6 or pass -IsccPath explicitly."
}

function Resolve-VsixSource {
    param([string]$InputPath, [string]$RepoRoot, [string]$ReleaseRootPath, [string]$VsixBuildScriptPath)

    if ($InputPath -ne "") {
        $candidate = if ([System.IO.Path]::IsPathRooted($InputPath)) {
            $InputPath
        } else {
            Join-Path $RepoRoot $InputPath
        }
        if (-not (Test-Path $candidate)) {
            throw "Explicit VSIX source not found: $candidate"
        }
        return (Resolve-Path $candidate).Path
    }

    $null = & powershell -ExecutionPolicy Bypass -File $VsixBuildScriptPath -ReleaseRoot $ReleaseRoot
    if ($LASTEXITCODE -ne 0) {
        throw "VSIX build script failed with exit code $LASTEXITCODE"
    }

    $built = Join-Path $ReleaseRootPath "ocp-ocl-vscode-v1.0.0.vsix"
    if (-not (Test-Path $built)) {
        throw "VSIX not found after build: $built"
    }
    return (Resolve-Path $built).Path
}

function Ensure-InstallerIcon {
    param([string]$IconPath, [string]$ScriptPath)

    if (Test-Path $IconPath) {
        return
    }

    if (-not (Test-Path $ScriptPath)) {
        throw "Installer icon generator script not found: $ScriptPath"
    }

    & python $ScriptPath
    if ($LASTEXITCODE -ne 0) {
        throw "Installer icon generator failed with exit code $LASTEXITCODE"
    }

    if (-not (Test-Path $IconPath)) {
        throw "Installer icon not found after generation: $IconPath"
    }
}

$cliBinaryPath = Resolve-BinarySource -InputPath $BinarySource -RepoRoot $repoRoot -FriendlyName "ocl.exe" -Fallbacks @(
    "target\release\ocl-cli.exe",
    "projects\ocp-ocl\crates\ocl-cli\target\release\ocl-cli.exe"
)
$lspBinaryPath = Resolve-BinarySource -InputPath $LspSource -RepoRoot $repoRoot -FriendlyName "ocl-lsp.exe" -Fallbacks @(
    "target\release\ocl-lsp.exe",
    "projects\ocp-ocl\crates\ocl-lsp\target\release\ocl-lsp.exe"
)
$dapBinaryPath = Resolve-BinarySource -InputPath $DapSource -RepoRoot $repoRoot -FriendlyName "ocl-dap.exe" -Fallbacks @(
    "target\release\ocl-dap.exe",
    "projects\ocp-ocl\crates\ocl-dap\target\release\ocl-dap.exe"
)
$iscc = Resolve-IsccPath -InputPath $IsccPath
$vsixPath = Resolve-VsixSource -InputPath $VsixSource -RepoRoot $repoRoot -ReleaseRootPath $releaseRootPath -VsixBuildScriptPath $vsixBuildScript
Ensure-InstallerIcon -IconPath $installerIconPath -ScriptPath $installerIconScript

New-Item -ItemType Directory -Force -Path $stageRootPath | Out-Null
New-Item -ItemType Directory -Force -Path $releaseRootPath | Out-Null

Copy-Item $cliBinaryPath (Join-Path $stageRootPath "ocl.exe") -Force
Copy-Item $lspBinaryPath (Join-Path $stageRootPath "ocl-lsp.exe") -Force
Copy-Item $dapBinaryPath (Join-Path $stageRootPath "ocl-dap.exe") -Force
Copy-Item $vsixPath (Join-Path $stageRootPath "ocp-ocl-vscode-v1.0.0.vsix") -Force

$nextSteps = @(
    "OCP-OCL v1.0 - Windows install next steps",
    "",
    "Fast start:",
    "- If VSCode extension was auto-installed, you can open VSCode and start coding in .ocl files immediately.",
    "- If the extension was not auto-installed, install the bundled VSIX and then start coding.",
    "",
    "Recommended quick checks:",
    "1. Open a new terminal.",
    "2. Run: ocl --version",
    "3. Run: ocl init hello --template tool-cli",
    "",
    "Required before production use:",
    "- Verify release integrity:",
    "  docs\vi\security\verify-download.md",
    "",
    "Installer behavior:",
    "- choose the install directory you want during setup",
    "- opt in to Add OCP-OCL CLI to PATH if you want terminal access everywhere",
    "",
    "VSCode extension:",
    "- installer will auto-install ocp-ocl-vscode-v1.0.0.vsix if VSCode is detected",
    '- supported detection paths: Program Files, LocalAppData, or any absolute code.cmd path returned by `where code.cmd`',
    "- install log after setup: <install-dir>\\vscode-extension-install.log",
    "- manual fallback: code --install-extension ocp-ocl-vscode-v1.0.0.vsix --force"
) -join [Environment]::NewLine

[System.IO.File]::WriteAllText(
    (Join-Path $stageRootPath "INSTALL-NEXT-STEPS.txt"),
    $nextSteps,
    [System.Text.Encoding]::UTF8
)

& $iscc "/DStageRoot=$stageRootPath" "/DReleaseRoot=$releaseRootPath" $installerScript
if ($LASTEXITCODE -ne 0) {
    throw "ISCC.exe failed with exit code $LASTEXITCODE"
}

Write-Host "Installer built at $releaseRootPath\ocl-v1.0.0-setup-win-x64.exe"
