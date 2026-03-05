param(
    [switch]$InstallDeps
)

Set-StrictMode -Version Latest
$ErrorActionPreference = "Stop"

# This launcher keeps GUI + API startup consistent so ABM runs are reproducible.
$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$guiDir = Join-Path $repoRoot "gui"
$logsDir = Join-Path $repoRoot "outputs/dev_logs"

if (-not (Test-Path $guiDir)) {
    throw "Expected GUI directory at: $guiDir"
}

New-Item -ItemType Directory -Force -Path $logsDir | Out-Null

if ($InstallDeps -or -not (Test-Path (Join-Path $guiDir "node_modules"))) {
    Write-Host "Installing GUI dependencies..." -ForegroundColor Cyan
    & npm install --prefix $guiDir
    if ($LASTEXITCODE -ne 0) {
        throw "npm install failed."
    }
}

function Start-BackgroundProcess {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][string]$Command,
        [Parameter(Mandatory = $true)][string]$WorkingDirectory
    )

    $timestamp = Get-Date -Format "yyyyMMdd-HHmmss"
    $stdoutLog = Join-Path $logsDir "$Name-$timestamp.out.log"
    $stderrLog = Join-Path $logsDir "$Name-$timestamp.err.log"

    $proc = Start-Process `
        -FilePath "cmd.exe" `
        -ArgumentList "/c $Command" `
        -WorkingDirectory $WorkingDirectory `
        -RedirectStandardOutput $stdoutLog `
        -RedirectStandardError $stderrLog `
        -PassThru

    return [pscustomobject]@{
        Name = $Name
        Pid = $proc.Id
        StdoutLog = $stdoutLog
        StderrLog = $stderrLog
    }
}

Write-Host "Starting API on http://localhost:8787 ..." -ForegroundColor Cyan
$api = Start-BackgroundProcess -Name "api" -Command "npm run api" -WorkingDirectory $guiDir

Start-Sleep -Seconds 2

Write-Host "Starting GUI on http://localhost:5173 ..." -ForegroundColor Cyan
$gui = Start-BackgroundProcess -Name "gui" -Command "npm run dev" -WorkingDirectory $guiDir

$manifest = [pscustomobject]@{
    started_utc = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ssZ")
    api_pid = $api.Pid
    gui_pid = $gui.Pid
    api_stdout_log = $api.StdoutLog
    api_stderr_log = $api.StderrLog
    gui_stdout_log = $gui.StdoutLog
    gui_stderr_log = $gui.StderrLog
}

$manifestPath = Join-Path $logsDir "latest-dev-processes.json"
$manifest | ConvertTo-Json | Set-Content -Path $manifestPath -Encoding UTF8

Write-Host ""
Write-Host "Started background processes:" -ForegroundColor Green
Write-Host ("  API PID: {0}" -f $api.Pid)
Write-Host ("  GUI PID: {0}" -f $gui.Pid)
Write-Host ("  Manifest: {0}" -f $manifestPath)
Write-Host ("  API logs: {0}" -f $api.StdoutLog)
Write-Host ("  GUI logs: {0}" -f $gui.StdoutLog)
Write-Host ""
Write-Host "To stop them later:" -ForegroundColor Yellow
Write-Host ("  Stop-Process -Id {0},{1}" -f $api.Pid, $gui.Pid)
