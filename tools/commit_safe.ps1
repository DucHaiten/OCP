param(
    [Parameter(Mandatory = $true, Position = 0)]
    [string]$Message,
    [switch]$StrictLargeDoc,
    [switch]$AllowGenerated
)

$ErrorActionPreference = "Stop"

Write-Host "[commit-safe] Unstage generated/build outputs..."
$generatedPaths = @(
    "target",
    "projects/ocp/crates/ocp-cli/target",
    "Rules/guard/__pycache__"
)

function Unstage-GeneratedPaths {
    param([string[]]$Paths)
    $stagedPaths = @(git diff --cached --name-only)
    foreach ($path in $Paths) {
        $hasStagedPath = $false
        foreach ($staged in $stagedPaths) {
            if (
                $staged -eq $path -or
                $staged.StartsWith("$path/") -or
                $staged.StartsWith("$path\")
            ) {
                $hasStagedPath = $true
                break
            }
        }

        if ($hasStagedPath) {
            git restore --staged -- $path *> $null
        }
    }
}

Unstage-GeneratedPaths -Paths $generatedPaths

Write-Host "[commit-safe] Auto-stage workspace changes..."
git add -A -- .
Unstage-GeneratedPaths -Paths $generatedPaths

$stagedForCommit = @(git diff --cached --name-only)
if ($stagedForCommit.Count -eq 0) {
    throw "[commit-safe] No staged changes to commit after auto-stage filter."
}

# Default an toàn cho workflow plan/docs:
# luôn cho phép large-doc rewrite có chủ đích, vẫn giữ tất cả guard khác.
if (-not $StrictLargeDoc) {
    $env:OCP_RULES_ALLOW_LARGE_DOC = "1"
}

if ($AllowGenerated) {
    $env:OCP_RULES_ALLOW_GENERATED = "1"
}

Write-Host "[commit-safe] Running staged guard..."
python Rules/guard/guard_repo.py --mode changed --staged
if ($LASTEXITCODE -ne 0) {
    throw "[commit-safe] Guard failed. Commit aborted."
}

Write-Host "[commit-safe] Committing..."
# Hook shell (sh/msys) có thể lỗi trên một số máy Windows.
# commit_safe đã chạy guard staged trước đó, nên commit với --no-verify để tránh fail giả do hook runtime.
git commit --no-verify -m $Message
if ($LASTEXITCODE -ne 0) {
    throw "[commit-safe] git commit failed."
}

Write-Host "[commit-safe] Done."
