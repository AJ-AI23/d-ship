# Deploy a MultiverseX contract locally to devnet.
# Prerequisites: Build first (.\scripts\build-contract.ps1), mxpy, wallet-owner.pem in project root.
# Usage: .\scripts\deploy-contract.ps1 [contract_path] [network]
# Example: .\scripts\deploy-contract.ps1 contracts/serial devnet

param(
    [Parameter(Position = 0)]
    [string]$ContractPath = "contracts/serial",
    [Parameter(Position = 1)]
    [ValidateSet("devnet", "testnet", "mainnet")]
    [string]$Network = "devnet"
)

$ErrorActionPreference = "Stop"
$projectRoot = Split-Path -Parent $PSScriptRoot
Set-Location $projectRoot

$pemPath = Join-Path $projectRoot "wallet-owner.pem"
if (-not (Test-Path $pemPath)) {
    Write-Error "Wallet not found: $pemPath. Add wallet-owner.pem to project root for local testing."
}

$wasmPath = Join-Path $projectRoot (Join-Path $ContractPath "output")
$wasmFile = Get-ChildItem -Path $wasmPath -Filter "*.wasm" -ErrorAction SilentlyContinue | Select-Object -First 1
if (-not $wasmFile) {
    Write-Error "No .wasm found in $wasmPath. Run .\scripts\build-contract.ps1 $ContractPath first."
}

$networkConfig = switch ($Network) {
    "mainnet" { @("https://gateway.multiversx.com", "1") }
    "testnet" { @("https://testnet-gateway.multiversx.com", "T") }
    default   { @("https://devnet-gateway.multiversx.com", "D") }
}
$proxy = $networkConfig[0]
$chain = $networkConfig[1]

Write-Host "Deploying $ContractPath to $Network..." -ForegroundColor Cyan
Write-Host "  WASM: $($wasmFile.FullName)" -ForegroundColor Gray
Write-Host "  PEM:  $pemPath" -ForegroundColor Gray
Write-Host "  Proxy: $proxy" -ForegroundColor Gray

# Ensure mxpy is available (requires Python/pip in PATH for auto-install)
if (-not (Get-Command mxpy -ErrorAction SilentlyContinue)) {
    Write-Host "mxpy not found. Attempting install..." -ForegroundColor Yellow
    $installed = $false
    foreach ($pipCmd in @("pip", "pip3")) {
        if (Get-Command $pipCmd -ErrorAction SilentlyContinue) {
            & $pipCmd install multiversx-sdk-cli --quiet 2>$null
            if (Get-Command mxpy -ErrorAction SilentlyContinue) { $installed = $true; break }
        }
    }
    if (-not $installed -and (Get-Command py -ErrorAction SilentlyContinue)) {
        py -m pip install multiversx-sdk-cli --quiet 2>$null
        if (Get-Command mxpy -ErrorAction SilentlyContinue) { $installed = $true }
    }
    if (-not (Get-Command mxpy -ErrorAction SilentlyContinue)) {
        Write-Error "mxpy required. Install Python (python.org), add to PATH, then: pip install multiversx-sdk-cli"
        exit 1
    }
}

$outFile = Join-Path $env:TEMP "deploy-output-$([guid]::NewGuid().ToString('N').Substring(0,8)).json"
try {
    mxpy contract deploy `
        --bytecode="$($wasmFile.FullName)" `
        --pem="$pemPath" `
        --proxy="$proxy" `
        --chain="$chain" `
        --arguments "str:{}" `
        --gas-limit=100000000 `
        --send `
        --outfile="$outFile"

    if (Test-Path $outFile) {
        $result = Get-Content $outFile -Raw | ConvertFrom-Json
        Write-Host "`nDeployed successfully!" -ForegroundColor Green
        Write-Host "  Contract: $($result.contractAddress)" -ForegroundColor Green
        Write-Host "  Tx hash:  $($result.emittedTransactionHash)" -ForegroundColor Green
    }
} finally {
    if (Test-Path $outFile) { Remove-Item $outFile -Force }
}
