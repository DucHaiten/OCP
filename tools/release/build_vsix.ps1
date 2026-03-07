param(
    [string]$BinarySource = "",
    [string]$LspSource = "",
    [string]$DapSource = "",
    [string]$ExtensionRoot = "editor/vscode/ocp-ocl",
    [string]$StageRoot = "target/ocl/w100/stage/vsix",
    [string]$ReleaseRoot = "target/ocl/w100/release",
    [string]$Version = "1.0.0"
)

$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..\..")).Path
$extensionRootPath = Join-Path $repoRoot $ExtensionRoot
$stageRootPath = Join-Path $repoRoot $StageRoot
$releaseRootPath = Join-Path $repoRoot $ReleaseRoot
$packageRoot = Join-Path $stageRootPath "package"
$extensionStageRoot = Join-Path $packageRoot "extension"
$vsixPath = Join-Path $releaseRootPath "ocp-ocl-vscode-v1.0.0.vsix"

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

    throw "Could not find built Windows binary for VSIX bundle: $FriendlyName"
}

if (-not (Test-Path $extensionRootPath)) {
    throw "Extension root not found: $extensionRootPath"
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

$required = @(
    "package.json",
    "language-configuration.json",
    "dist\dapRuntime.js",
    "dist\extension.js",
    "dist\commands.js",
    "dist\lspClient.js",
    "dist\runtimePaths.js",
    "syntaxes\ocp-ocl.tmLanguage.json",
    "icons\ocp-ocl.svg",
    "icons\ocp-ocl-icon-theme.json",
    "snippets\ocp-ocl.json"
)

foreach ($rel in $required) {
    $full = Join-Path $extensionRootPath $rel
    if (-not (Test-Path $full)) {
        throw "Required extension asset missing: $full"
    }
}

if (Test-Path $packageRoot) {
    Remove-Item -Recurse -Force $packageRoot
}
New-Item -ItemType Directory -Force -Path $extensionStageRoot | Out-Null
New-Item -ItemType Directory -Force -Path $releaseRootPath | Out-Null

$copyTargets = @(
    "dist",
    "icons",
    "snippets",
    "syntaxes",
    "language-configuration.json"
)

foreach ($rel in $copyTargets) {
    $src = Join-Path $extensionRootPath $rel
    $dst = Join-Path $extensionStageRoot $rel
    if (Test-Path $src -PathType Container) {
        Copy-Item $src $dst -Recurse -Force
    } else {
        Copy-Item $src $dst -Force
    }
}

$binStageRoot = Join-Path $extensionStageRoot "bin\win-x64"
New-Item -ItemType Directory -Force -Path $binStageRoot | Out-Null
Copy-Item $cliBinaryPath (Join-Path $binStageRoot "ocl.exe") -Force
Copy-Item $lspBinaryPath (Join-Path $binStageRoot "ocl-lsp.exe") -Force
Copy-Item $dapBinaryPath (Join-Path $binStageRoot "ocl-dap.exe") -Force

$packageJsonPath = Join-Path $extensionRootPath "package.json"
$packageJson = Get-Content -Path $packageJsonPath -Raw | ConvertFrom-Json
$packageJson.version = $Version
$packageJson.main = "./dist/extension.js"
$packageJsonRendered = $packageJson | ConvertTo-Json -Depth 100
[System.IO.File]::WriteAllText(
    (Join-Path $extensionStageRoot "package.json"),
    $packageJsonRendered,
    (New-Object System.Text.UTF8Encoding($false))
)

$readme = @(
    "# OCP OCL VSCode Extension",
    "",
    "Bundled with OCP-OCL v1.0.0 release installer.",
    "",
    "If auto-install did not run, install this file manually:",
    "- Extensions view -> ... -> Install from VSIX",
    '- or `code --install-extension ocp-ocl-vscode-v1.0.0.vsix --force`'
) -join [Environment]::NewLine
[System.IO.File]::WriteAllText((Join-Path $extensionStageRoot "README.md"), $readme, [System.Text.Encoding]::UTF8)

$contentTypes = @'
<?xml version="1.0" encoding="utf-8"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="json" ContentType="application/json" />
  <Default Extension="js" ContentType="application/javascript" />
  <Default Extension="md" ContentType="text/markdown" />
  <Default Extension="svg" ContentType="image/svg+xml" />
  <Default Extension="txt" ContentType="text/plain" />
  <Default Extension="xml" ContentType="text/xml" />
  <Override PartName="/extension.vsixmanifest" ContentType="text/xml" />
</Types>
'@
[System.IO.File]::WriteAllText((Join-Path $packageRoot "[Content_Types].xml"), $contentTypes, [System.Text.Encoding]::UTF8)

$manifest = @'
<?xml version="1.0" encoding="utf-8"?>
<PackageManifest Version="2.0.0" xmlns="http://schemas.microsoft.com/developer/vsx-schema/2011" xml:lang="en-US">
  <Metadata>
    <Identity Id="ocp-ocl" Version="1.0.0" Language="en-US" Publisher="ocp-ocl" />
    <DisplayName>OCP OCL</DisplayName>
    <Description xml:space="preserve">Observation Collapse Language support for VSCode.</Description>
    <Tags>ocl,ocp-ocl</Tags>
    <Categories>Programming Languages</Categories>
    <Properties>
      <Property Id="Microsoft.VisualStudio.Code.Engine" Value="^1.90.0" />
      <Property Id="Microsoft.VisualStudio.Code.ExtensionKind" Value="workspace" />
    </Properties>
  </Metadata>
  <Installation>
    <InstallationTarget Id="Microsoft.VisualStudio.Code" />
  </Installation>
  <Dependencies />
  <Assets>
    <Asset Type="Microsoft.VisualStudio.Code.Manifest" Path="extension/package.json" Addressable="true" />
    <Asset Type="Microsoft.VisualStudio.Services.Content.Details" Path="extension/README.md" Addressable="true" />
  </Assets>
</PackageManifest>
'@
[System.IO.File]::WriteAllText((Join-Path $packageRoot "extension.vsixmanifest"), $manifest, [System.Text.Encoding]::UTF8)

if (Test-Path $vsixPath) {
    Remove-Item -Force $vsixPath
}

Add-Type -AssemblyName System.IO.Compression.FileSystem
[System.IO.Compression.ZipFile]::CreateFromDirectory($packageRoot, $vsixPath)

Write-Host "VSIX built at $vsixPath"
