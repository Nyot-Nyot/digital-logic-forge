<#
.SYNOPSIS
    Runs rustfmt and clippy across the entire workspace with zero tolerance for warnings.
#>

$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

Write-Host "Running cargo fmt check..." -ForegroundColor Cyan
& cargo fmt --check

if ($LASTEXITCODE -ne 0) {
    Write-Error "cargo fmt failed! Run 'cargo fmt' to fix."
    exit $LASTEXITCODE
}

Write-Host "Running cargo clippy (--all-targets -D warnings)..." -ForegroundColor Cyan
& cargo clippy --workspace --all-targets -- -D warnings

if ($LASTEXITCODE -ne 0) {
    Write-Error "cargo clippy found warnings/errors."
    exit $LASTEXITCODE
}

Write-Host "All formatting and lint checks passed cleanly! [OK]" -ForegroundColor Green
