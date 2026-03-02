# Build a MultiverseX contract locally.
# Usage: .\scripts\build-contract.ps1 [contract_path]
# Example: .\scripts\build-contract.ps1 contracts/serial

param(
    [Parameter(Position = 0)]
    [string]$ContractPath = "contracts/serial"
)

$ErrorActionPreference = "Stop"
$projectRoot = Split-Path -Parent $PSScriptRoot
if (-not $projectRoot) { $projectRoot = (Get-Location).Path }
Set-Location $projectRoot

Write-Host "Building $ContractPath..." -ForegroundColor Cyan
$metaPath = Join-Path $projectRoot (Join-Path $ContractPath "meta")
if (-not (Test-Path $metaPath)) {
    Write-Error "Meta folder not found: $metaPath"
}
Push-Location $metaPath
try {
    cargo run build
} finally {
    Pop-Location
}

if ($LASTEXITCODE -eq 0) {
    $wasmDir = Join-Path $projectRoot (Join-Path $ContractPath "output")
    Write-Host "`nWASM output: $wasmDir" -ForegroundColor Green
    if (Test-Path $wasmDir) {
        Get-ChildItem $wasmDir -Filter "*.wasm" | ForEach-Object { Write-Host "  - $($_.Name)" }
    }
}
