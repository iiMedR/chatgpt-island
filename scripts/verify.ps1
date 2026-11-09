[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$projectRoot = Split-Path -Parent $PSScriptRoot
function Invoke-Check {
    param([string]$Step, [string]$Executable, [string[]]$Arguments)
    Write-Host "`n== $Step =="
    & $Executable @Arguments
    if ($LASTEXITCODE -ne 0) { throw "$Step failed (exit $LASTEXITCODE)." }
}
Push-Location $projectRoot
try {
    Invoke-Check 'TypeScript check' 'npm.cmd' @('run', 'typecheck')
    Invoke-Check 'JavaScript tests' 'npm.cmd' @('test')
    Invoke-Check 'Frontend build' 'npm.cmd' @('run', 'build')
    Push-Location 'src-tauri'
    try {
        Invoke-Check 'Rust check' 'cargo' @('check', '--locked')
        Invoke-Check 'Rust tests' 'cargo' @('test', '--locked')
        Invoke-Check 'Rust formatting' 'cargo' @('fmt', '--check')
        Invoke-Check 'Rust Clippy' 'cargo' @('clippy', '--locked', '--', '-D', 'warnings')
    } finally { Pop-Location }
    Write-Host "`nAll native verification checks passed. GUI feasibility still requires manual testing."
} finally { Pop-Location }
