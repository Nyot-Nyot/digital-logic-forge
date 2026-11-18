<#
.SYNOPSIS
    Automated test runner for Digital Logic Forge: runs Rust unit tests and Godot headless tests.
#>

$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

$GodotBin = "D:\Downloads\Godot_v4.7.2-stable_win64.exe\Godot_v4.7.2-stable_win64_console.exe"
if (-not (Test-Path $GodotBin)) {
    Write-Error "Godot binary not found at $GodotBin"
    exit 1
}

Write-Host "======================================================" -ForegroundColor Cyan
Write-Host "1. Running Rust Workspace Unit Tests..." -ForegroundColor Cyan
Write-Host "======================================================" -ForegroundColor Cyan
& cargo test --workspace

if ($LASTEXITCODE -ne 0) {
    Write-Error "Cargo test failed with exit code $LASTEXITCODE"
    exit $LASTEXITCODE
}

Write-Host "`n======================================================" -ForegroundColor Cyan
Write-Host "2. Building GDExtension Debug Library..." -ForegroundColor Cyan
Write-Host "======================================================" -ForegroundColor Cyan
& powershell -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot "build_extension.ps1")

if ($LASTEXITCODE -ne 0) {
    Write-Error "GDExtension build failed with exit code $LASTEXITCODE"
    exit $LASTEXITCODE
}

Write-Host "`n======================================================" -ForegroundColor Cyan
Write-Host "3. Running Godot Headless Smoke & Worker Test..." -ForegroundColor Cyan
Write-Host "======================================================" -ForegroundColor Cyan
& $GodotBin --headless --path (Join-Path $Root "app\godot") -s res://tests/smoke_test.gd

if ($LASTEXITCODE -ne 0) {
    Write-Error "Godot smoke test failed with exit code $LASTEXITCODE"
    exit $LASTEXITCODE
}

Write-Host "`n======================================================" -ForegroundColor Cyan
Write-Host "4. Running Godot Headless Canvas Prototype Test..." -ForegroundColor Cyan
Write-Host "======================================================" -ForegroundColor Cyan
& $GodotBin --headless --path (Join-Path $Root "app\godot") -s res://tests/canvas_test.gd

if ($LASTEXITCODE -ne 0) {
    Write-Error "Godot canvas test failed with exit code $LASTEXITCODE"
    exit $LASTEXITCODE
}

Write-Host "`n======================================================" -ForegroundColor Green
Write-Host "ALL DLF AUTOMATED TESTS (RUST + GODOT) PASSED! [OK]" -ForegroundColor Green
Write-Host "======================================================" -ForegroundColor Green
