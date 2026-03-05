param(
    [switch]$IncludeSmokeRun,
    [switch]$IncludeCargoTests
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# This validation runner exists to catch integration regressions early after
# GUI/backend/model edits, before running long ABM sweeps.
$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")

function Invoke-Step {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][scriptblock]$Action
    )

    Write-Host "==> $Name" -ForegroundColor Cyan
    & $Action
    Write-Host "OK: $Name" -ForegroundColor Green
}

Push-Location $repoRoot
try {
    Invoke-Step -Name "Rust compile check" -Action { cargo check }
    Invoke-Step -Name "Node syntax check (GUI API server)" -Action { node --check gui/server/index.mjs }
    Invoke-Step -Name "GUI production build" -Action { npm.cmd --prefix gui run build }

    if ($IncludeCargoTests) {
        Invoke-Step -Name "Rust test suite" -Action { cargo test --quiet }
    }

    if ($IncludeSmokeRun) {
        Invoke-Step -Name "MVP smoke run (Phoenix config, 1 tick)" -Action {
            $tmp = Join-Path $repoRoot "outputs/_tmp_validation_smoke.toml"
            New-Item -ItemType Directory -Force -Path (Split-Path $tmp) | Out-Null
            $raw = Get-Content "configs/phoenix_basin.toml" -Raw
            $raw = $raw -replace '(?m)^ticks\s*=\s*\d+', 'ticks = 1'
            $raw = $raw -replace '(?m)^snapshot_every_ticks\s*=\s*\d+', 'snapshot_every_ticks = 1'
            $raw = $raw -replace '(?m)^live_update_every_ticks\s*=\s*\d+', 'live_update_every_ticks = 0'
            Set-Content -Path $tmp -Value $raw -Encoding UTF8
            cargo run --quiet -- $tmp
        }
    }

    Write-Host ""
    Write-Host "Validation complete." -ForegroundColor Green
}
finally {
    Pop-Location
}
