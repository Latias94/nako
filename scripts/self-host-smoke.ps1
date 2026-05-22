#!/usr/bin/env pwsh
[CmdletBinding()]
param(
    [ValidateSet('sqlite', 'postgres')]
    [string]$Backend = 'sqlite',

    [switch]$PostgresContractsOnly
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot '..')
Set-Location $RepoRoot

function Invoke-Step {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Name,

        [Parameter(Mandatory = $true)]
        [scriptblock]$Action
    )

    Write-Host ""
    Write-Host "==> $Name"
    & $Action
    if ($LASTEXITCODE -ne 0) {
        throw "$Name failed with exit code $LASTEXITCODE."
    }
}

if ($Backend -eq 'postgres' -or $PostgresContractsOnly) {
    Invoke-Step 'scripts/postgres-contract-harness.ps1 -Suite managed-artwork' {
        pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/postgres-contract-harness.ps1 -Suite managed-artwork
    }
    exit 0
}

Invoke-Step 'cargo nextest run -p nako-server self_host_smoke --no-fail-fast' {
    cargo nextest run -p nako-server self_host_smoke --no-fail-fast
}
