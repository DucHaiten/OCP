$ErrorActionPreference = "Stop"

function Invoke-Step {
    param (
        [Parameter(Mandatory = $true)][string]$Command
    )
    Write-Host "[ocl-lane] $Command"
    Invoke-Expression $Command
    if ($LASTEXITCODE -ne 0) {
        throw "Command failed with exit code ${LASTEXITCODE}: $Command"
    }
}

function Quote-PS {
    param (
        [Parameter(Mandatory = $true)][string]$Value
    )
    return "'" + $Value.Replace("'", "''") + "'"
}

function New-IsolatedAppCopy {
    param (
        [Parameter(Mandatory = $true)][string]$SourcePath
    )
    $resolvedSource = (Resolve-Path -Path $SourcePath).Path
    $leaf = Split-Path -Path $resolvedSource -Leaf
    $stamp = Get-Date -Format "yyyyMMdd-HHmmssfff"
    $dest = Join-Path $env:TEMP ("ocl-lane-" + $leaf + "-" + $stamp)
    New-Item -ItemType Directory -Path $dest -Force | Out-Null
    Copy-Item -Path (Join-Path $resolvedSource "*") -Destination $dest -Recurse -Force
    return $dest
}

Invoke-Step "cargo test -p ocl-runtime-core"
Invoke-Step "cargo test -p ocl-sdk"
Invoke-Step "cargo test -p ocl-cli"

$canaryPath = New-IsolatedAppCopy "projects/ocp-ocl/app-ocl"
$canaryArg = Quote-PS $canaryPath
Invoke-Step "cargo run -p ocl-cli -- check $canaryArg --json"
Invoke-Step "cargo run -p ocl-cli -- run $canaryArg"
Invoke-Step "cargo run -p ocl-cli -- run $canaryArg --reactor --ticks 32"
Invoke-Step "cargo run -p ocl-cli -- fmt $canaryArg --check"
Invoke-Step "cargo run -p ocl-cli -- test $canaryArg"
Invoke-Step "cargo run -p ocl-cli -- build $canaryArg"

$demoApps = @(
    @{ Path = "projects/ocp-ocl/apps/hello-cli"; ReactorTicks = 0; Composer = $false },
    @{ Path = "projects/ocp-ocl/apps/web-fetch"; ReactorTicks = 0; Composer = $false },
    @{ Path = "projects/ocp-ocl/apps/mini-server"; ReactorTicks = 16; Composer = $false },
    @{ Path = "projects/ocp-ocl/apps/scheduler"; ReactorTicks = 16; Composer = $false },
    @{ Path = "projects/ocp-ocl/apps/composer-demo"; ReactorTicks = 0; Composer = $true }
)

foreach ($app in $demoApps) {
    $path = New-IsolatedAppCopy $app.Path
    $pathArg = Quote-PS $path
    Invoke-Step "cargo run -p ocl-cli -- lock sync $pathArg"

    if ($app.Composer) {
        $phenotypeArg = Quote-PS (Join-Path $path "phenotype.toml")
        $registryArg = Quote-PS (Join-Path $path "registry")
        Invoke-Step "cargo run -p ocl-cli -- compose $pathArg --phenotype $phenotypeArg --registry $registryArg --locked"
        Invoke-Step "cargo run -p ocl-cli -- verify $pathArg --phenotype $phenotypeArg --registry $registryArg --locked"
    }

    Invoke-Step "cargo run -p ocl-cli -- check $pathArg --json --locked"
    Invoke-Step "cargo run -p ocl-cli -- run $pathArg --locked"

    if ($app.ReactorTicks -gt 0) {
        Invoke-Step "cargo run -p ocl-cli -- run $pathArg --reactor --ticks $($app.ReactorTicks) --locked"
    }

    Invoke-Step "cargo run -p ocl-cli -- fmt $pathArg --check"
    Invoke-Step "cargo run -p ocl-cli -- test $pathArg --locked"
    Invoke-Step "cargo run -p ocl-cli -- build $pathArg --locked"
}
