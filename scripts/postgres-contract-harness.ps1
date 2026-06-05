#!/usr/bin/env pwsh
[CmdletBinding()]
param(
    [ValidateSet('managed-artwork', 'storage-runtime', 'source-identity', 'all-contracts')]
    [string]$Suite = 'managed-artwork',

    [string]$DatabaseUrl = $env:NAKO_TEST_POSTGRES_URL,

    [int]$Port = 55432,

    [switch]$KeepData,

    [switch]$RequireTooling
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot '..')
Set-Location $RepoRoot

$HarnessRoot = Join-Path $RepoRoot 'target/postgres-contract'
$DataDir = Join-Path $HarnessRoot 'data'
$LogPath = Join-Path $HarnessRoot 'postgres.log'
$DatabaseName = 'nako_contract'
$UserName = 'nako'
$StartedLocalServer = $false

function Invoke-Native {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Name,

        [Parameter(Mandatory = $true)]
        [string[]]$Arguments
    )

    Write-Host ""
    Write-Host "==> $Name $($Arguments -join ' ')"
    & $Name @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "$Name failed with exit code $LASTEXITCODE."
    }
}

function Test-CommandAvailable {
    param([Parameter(Mandatory = $true)][string]$Name)
    return [bool](Get-Command $Name -ErrorAction SilentlyContinue)
}

function Stop-LocalPostgres {
    if ($script:StartedLocalServer) {
        try {
            Invoke-Native 'pg_ctl' @('stop', '-D', $DataDir, '-m', 'fast', '-w', '-t', '30')
        } catch {
            Write-Warning "Failed to stop local PostgreSQL cleanly: $_"
        }
    }
}

function Remove-HarnessData {
    if ($KeepData) {
        Write-Host ""
        Write-Host "Keeping PostgreSQL harness data at $HarnessRoot."
        return
    }

    $resolvedHarnessRoot = Resolve-Path $HarnessRoot -ErrorAction SilentlyContinue
    if ($null -eq $resolvedHarnessRoot) {
        return
    }

    $resolvedTarget = Resolve-Path (Join-Path $RepoRoot 'target')
    $harnessPath = [System.IO.Path]::GetFullPath($resolvedHarnessRoot.Path)
    $targetPath = [System.IO.Path]::GetFullPath($resolvedTarget.Path)
    if (-not $harnessPath.StartsWith($targetPath, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to remove PostgreSQL harness data outside target/: $harnessPath"
    }

    Remove-Item -LiteralPath $harnessPath -Recurse -Force
}

function Get-TestFilter {
    switch ($Suite) {
        'managed-artwork' { return 'postgres_managed_artwork_contract' }
        'storage-runtime' {
            return @(
                'postgres_storage_backend_health_contract',
                'postgres_vfs_staging_contract'
            )
        }
        'source-identity' {
            return @(
                'postgres_library_media_contract_preserves_library_scoped_source_identity',
                'postgres_scan_commit_contract_writes_full_source_unit_and_resolves_failure',
                'postgres_source_duplicate_contract',
                'postgres_vfs_staging_contract_round_trips_attribution_variants',
                'postgres_vfs_staging_contract_preserves_reservation_budget_and_leases'
            )
        }
        'all-contracts' { return 'postgres_' }
        default { throw "Unsupported suite: $Suite" }
    }
}

try {
    New-Item -ItemType Directory -Force -Path $HarnessRoot | Out-Null

    if ([string]::IsNullOrWhiteSpace($DatabaseUrl)) {
        $requiredTools = @('initdb', 'pg_ctl', 'createdb')
        $missingTools = @($requiredTools | Where-Object { -not (Test-CommandAvailable $_) })
        if ($missingTools.Count -gt 0) {
            $message = "Skipping PostgreSQL contract harness because NAKO_TEST_POSTGRES_URL was not provided and local PostgreSQL tooling is missing: $($missingTools -join ', ')."
            if ($RequireTooling) {
                throw $message
            }
            Write-Warning $message
            exit 0
        }

        Remove-HarnessData
        New-Item -ItemType Directory -Force -Path $HarnessRoot | Out-Null

        Invoke-Native 'initdb' @('-D', $DataDir, '-U', $UserName, '-A', 'trust', '-E', 'UTF8', '--no-locale')
        Invoke-Native 'pg_ctl' @(
            'start',
            '-D', $DataDir,
            '-l', $LogPath,
            '-w',
            '-t', '60',
            '-o', "-p $Port -h 127.0.0.1"
        )
        $StartedLocalServer = $true

        Invoke-Native 'createdb' @('-h', '127.0.0.1', '-p', "$Port", '-U', $UserName, $DatabaseName)
        $DatabaseUrl = "postgres://$UserName@127.0.0.1:$Port/$DatabaseName"
    } else {
        Write-Host "Using caller-provided PostgreSQL database URL."
    }

    $env:NAKO_TEST_POSTGRES_URL = $DatabaseUrl
    $testFilter = Get-TestFilter

    $nextestArgs = @(
        'nextest', 'run',
        '-p', 'nako-db'
    )
    $nextestArgs += $testFilter
    $nextestArgs += @(
        '--run-ignored', 'ignored-only',
        '--no-fail-fast'
    )
    Invoke-Native 'cargo' $nextestArgs
} finally {
    Stop-LocalPostgres
    Remove-HarnessData
}
