<#
.SYNOPSIS
    Builds the dlf-godot GDExtension library and copies it to the Godot project bin directory.
.PARAMETER Release
    If set, builds with --release optimizations.
#>
param(
    [switch]$Release
)

$ErrorActionPreference = "Stop"

$Root = Split-Path -Parent $PSScriptRoot
Set-Location $Root

$Profile = if ($Release) { "release" } else { "debug" }
$CargoArgs = @("build", "-p", "dlf-godot")
if ($Release) {
    $CargoArgs += "--release"
}

Write-Host "Building dlf-godot [$Profile]..." -ForegroundColor Cyan
& cargo @CargoArgs

if ($LASTEXITCODE -ne 0) {
    Write-Error "Cargo build failed with exit code $LASTEXITCODE"
    exit $LASTEXITCODE
}

$BinDir = Join-Path $Root "app\godot\bin"
if (-not (Test-Path $BinDir)) {
    New-Item -ItemType Directory -Path $BinDir -Force | Out-Null
}

$DllSource = Join-Path $Root "target\$Profile\dlf_godot.dll"
$DllDest = Join-Path $BinDir "dlf_godot.dll"

if (Test-Path $DllSource) {
    Copy-Item -Path $DllSource -Destination $DllDest -Force
    Write-Host "Copied $DllSource -> $DllDest" -ForegroundColor Green
} else {
    Write-Error "Compiled DLL not found at: $DllSource"
    exit 1
}

$PdbSource = Join-Path $Root "target\$Profile\dlf_godot.pdb"
if (Test-Path $PdbSource) {
    $PdbDest = Join-Path $BinDir "dlf_godot.pdb"
    Copy-Item -Path $PdbSource -Destination $PdbDest -Force
}

Write-Host "GDExtension build completed successfully." -ForegroundColor Green
