$ErrorActionPreference = "Stop"

function Invoke-Step {
    param (
        [Parameter(Mandatory = $true)][string]$Command
    )
    Write-Host "[ocp-lane] $Command"
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
    $dest = Join-Path $env:TEMP ("ocp-lane-" + $leaf + "-" + $stamp)
    New-Item -ItemType Directory -Path $dest -Force | Out-Null
    Copy-Item -Path (Join-Path $resolvedSource "*") -Destination $dest -Recurse -Force
    return $dest
}

Invoke-Step "cargo test -p ocp-runtime-core"
Invoke-Step "cargo test -p ocp-sdk"
Invoke-Step "cargo test -p ocp-cli"
Invoke-Step "cargo test -p ocp-runtime-core --test m1_language"
Invoke-Step "cargo test -p ocp-runtime-core --test w3_bytecode"
Invoke-Step "cargo test -p ocp-sdk --test w1_runtime"
Invoke-Step "cargo test -p ocp-sdk --test w2_permissions"
Invoke-Step "cargo test -p ocp-sdk --test w2_audit"
Invoke-Step "cargo test -p ocp-sdk --test w3_engine"
Invoke-Step "cargo test -p ocp-sdk --test w4_supply"
Invoke-Step "cargo test --test v100_ocp_extension_guard"

$canaryPath = New-IsolatedAppCopy "projects/ocp/app-ocp"
$canaryArg = Quote-PS $canaryPath
Invoke-Step "cargo run -p ocp-cli -- check $canaryArg --json"
Invoke-Step "cargo run -p ocp-cli -- run $canaryArg"
Invoke-Step "cargo run -p ocp-cli -- run $canaryArg --engine bytecode"
Invoke-Step "cargo run -p ocp-cli -- run $canaryArg --engine dual"
Invoke-Step "cargo run -p ocp-cli -- run $canaryArg --reactor --ticks 32"
Invoke-Step "cargo run -p ocp-cli -- fmt $canaryArg --check"
Invoke-Step "cargo run -p ocp-cli -- test $canaryArg"
Invoke-Step "cargo run -p ocp-cli -- build $canaryArg"

$demoApps = @(
    @{ Path = "projects/ocp/apps/hello-cli"; ReactorTicks = 0; Composer = $false },
    @{ Path = "projects/ocp/apps/web-fetch"; ReactorTicks = 0; Composer = $false },
    @{ Path = "projects/ocp/apps/mini-server"; ReactorTicks = 16; Composer = $false },
    @{ Path = "projects/ocp/apps/scheduler"; ReactorTicks = 16; Composer = $false },
    @{ Path = "projects/ocp/apps/tls-client"; ReactorTicks = 0; Composer = $false },
    @{ Path = "projects/ocp/apps/sqlite-app"; ReactorTicks = 0; Composer = $false },
    @{ Path = "projects/ocp/apps/composer-demo"; ReactorTicks = 0; Composer = $true }
)

foreach ($app in $demoApps) {
    $path = New-IsolatedAppCopy $app.Path
    $pathArg = Quote-PS $path
    Invoke-Step "cargo run -p ocp-cli -- lock sync $pathArg"

    if ($app.Composer) {
        $phenotypeArg = Quote-PS (Join-Path $path "phenotype.toml")
        $registryArg = Quote-PS (Join-Path $path "registry")
        Invoke-Step "cargo run -p ocp-cli -- compose $pathArg --phenotype $phenotypeArg --registry $registryArg --locked"
        Invoke-Step "cargo run -p ocp-cli -- verify $pathArg --phenotype $phenotypeArg --registry $registryArg --locked"
    }

    Invoke-Step "cargo run -p ocp-cli -- check $pathArg --json --locked"
    Invoke-Step "cargo run -p ocp-cli -- run $pathArg --locked"

    if ($app.ReactorTicks -gt 0) {
        Invoke-Step "cargo run -p ocp-cli -- run $pathArg --reactor --ticks $($app.ReactorTicks) --locked"
        if ($app.Path -eq "projects/ocp/apps/mini-server") {
            $reportDetArg = Quote-PS (Join-Path $path "target\\w1_runtime_det.json")
            $reportThrArg = Quote-PS (Join-Path $path "target\\w1_runtime_thr.json")
            $auditDetArg = Quote-PS (Join-Path $path "target\\w2_replay_det.audit.jsonl")
            $auditThrArg = Quote-PS (Join-Path $path "target\\w2_replay_thr.audit.jsonl")
            Invoke-Step "cargo run -p ocp-cli -- run $pathArg --reactor --ticks 16 --runtime deterministic --socket-listen 127.0.0.1:19091 --runtime-report $reportDetArg --replay-audit $auditDetArg --locked"
            Invoke-Step "cargo run -p ocp-cli -- run $pathArg --reactor --ticks 16 --runtime throughput --socket-listen 127.0.0.1:19092 --runtime-report $reportThrArg --replay-audit $auditThrArg --locked"
        }
    }

    Invoke-Step "cargo run -p ocp-cli -- fmt $pathArg --check"
    Invoke-Step "cargo run -p ocp-cli -- test $pathArg --locked"
    Invoke-Step "cargo run -p ocp-cli -- build $pathArg --locked"
}

Invoke-Step "cargo run -p ocp-cli -- test --conformance --locked --runtime deterministic --engine dual --manifest projects/ocp/conformance/conformance.v1.toml --out target/ocp/w9/reports/conformance_report.json --trust-store projects/ocp/security/trust.store.toml --signer-id dev-root-1 --sign-key projects/ocp/security/dev-root-1.signing.key.toml --json"
