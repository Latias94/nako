#!/usr/bin/env pwsh
[CmdletBinding()]
param(
    [ValidateSet('docs', 'fast', 'db', 'api', 'postgres', 'workspace', 'all')]
    [string]$Mode = 'fast',

    [string]$PostgresUrl = $env:TARU_TEST_POSTGRES_URL,

    [switch]$SkipRedactionInventory
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot '..')
Set-Location $RepoRoot

$ReleaseGateOutput = Join-Path $RepoRoot 'target/release-gate'
$RedactionInventoryPath = Join-Path $ReleaseGateOutput 'redaction-inventory.txt'
$RedactionInventoryPattern = 'storage_uri|managed-artwork://|source_uri|cache_uri|content_hash|artifact_root|local_path|database_url|token|secret'

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

function Invoke-Inventory {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Name,

        [Parameter(Mandatory = $true)]
        [string[]]$Arguments,

        [Parameter(Mandatory = $true)]
        [string]$OutputPath
    )

    Write-Host ""
    Write-Host "==> $Name"
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $OutputPath) | Out-Null

    $output = & rg @Arguments 2>&1
    $exitCode = $LASTEXITCODE
    $output | Set-Content -Path $OutputPath -Encoding utf8

    if ($exitCode -eq 1) {
        Write-Host "No matches. Inventory written to $OutputPath."
        return
    }

    if ($exitCode -ne 0) {
        $output | Write-Host
        throw "$Name failed with exit code $exitCode."
    }

    $matchCount = @($output).Count
    Write-Host "$matchCount matches written to $OutputPath."
}

function Test-Mode {
    param(
        [Parameter(Mandatory = $true)]
        [string[]]$Names
    )

    return ($Names -contains $Mode) -or ($Mode -eq 'all')
}

function Invoke-PostgresContracts {
    param(
        [string]$DatabaseUrl
    )

    if ([string]::IsNullOrWhiteSpace($DatabaseUrl)) {
        Invoke-Step 'scripts/postgres-contract-harness.ps1 -Suite managed-artwork' {
            pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/postgres-contract-harness.ps1 -Suite managed-artwork
        }
    } else {
        Invoke-Step 'scripts/postgres-contract-harness.ps1 -Suite managed-artwork -DatabaseUrl <provided>' {
            pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/postgres-contract-harness.ps1 -Suite managed-artwork -DatabaseUrl $DatabaseUrl
        }
    }
}

function Invoke-ApiSdkGate {
    Invoke-Step 'cargo check -p taru-api --tests' {
        cargo check -p taru-api --tests
    }

    Invoke-Step 'cargo check -p taru-client --tests' {
        cargo check -p taru-client --tests
    }

    Invoke-Step 'cargo check -p taru-client-protocol --tests' {
        cargo check -p taru-client-protocol --tests
    }

    Invoke-Step 'cargo nextest run -p taru-api openapi --no-fail-fast' {
        cargo nextest run -p taru-api openapi --no-fail-fast
    }

    Invoke-Step 'cargo nextest run -p taru-api sdk --no-fail-fast' {
        cargo nextest run -p taru-api sdk --no-fail-fast
    }

    Invoke-Step 'cargo nextest run -p taru-api admin_contract --no-fail-fast' {
        cargo nextest run -p taru-api admin_contract --no-fail-fast
    }

    Invoke-Step 'cargo nextest run -p taru-client --no-fail-fast' {
        cargo nextest run -p taru-client --no-fail-fast
    }

    Invoke-Step 'cargo nextest run -p taru-client-protocol --no-fail-fast' {
        cargo nextest run -p taru-client-protocol --no-fail-fast
    }

    Invoke-Step 'cargo tree -p taru-client' {
        cargo tree -p taru-client
    }

    Invoke-Step 'cargo tree -p taru-client-protocol' {
        cargo tree -p taru-client-protocol
    }

    Invoke-Step 'npm run generate --prefix sdk/typescript' {
        npm run generate --prefix sdk/typescript
    }

    Invoke-Step 'npm run check --prefix sdk/typescript' {
        npm run check --prefix sdk/typescript
    }

    Invoke-Step 'npm run generate:admin-api --prefix apps/admin-web' {
        npm run generate:admin-api --prefix apps/admin-web
    }

    Invoke-Step 'npm run check --prefix apps/admin-web' {
        npm run check --prefix apps/admin-web
    }

    Invoke-Step 'git diff --check' {
        git diff --check
    }
}

Write-Host "Taru release gate"
Write-Host "Mode: $Mode"
Write-Host "Repository: $RepoRoot"

Invoke-Step 'cargo fmt --all -- --check' {
    cargo fmt --all -- --check
}

Invoke-Step 'git diff --check' {
    git diff --check
}

if (-not $SkipRedactionInventory) {
    Invoke-Inventory `
        'redaction inventory scan' `
        @('-n', $RedactionInventoryPattern, 'crates', 'docs') `
        $RedactionInventoryPath
}

if ($Mode -eq 'docs') {
    Write-Host ""
    Write-Host 'Docs-safe release gate completed.'
    exit 0
}

if (Test-Mode @('fast', 'db')) {
    Invoke-Step 'cargo check -p taru-db --tests' {
        cargo check -p taru-db --tests
    }

    Invoke-Step 'cargo nextest run -p taru-db sqlite_managed_artwork_contract --no-fail-fast' {
        cargo nextest run -p taru-db sqlite_managed_artwork_contract --no-fail-fast
    }
}

if (Test-Mode @('fast', 'api')) {
    Invoke-Step 'cargo check -p taru-server --tests' {
        cargo check -p taru-server --tests
    }

    Invoke-ApiSdkGate

    Invoke-Step 'cargo nextest run -p taru-api managed_artwork --no-fail-fast' {
        cargo nextest run -p taru-api managed_artwork --no-fail-fast
    }

    Invoke-Step 'cargo nextest run -p taru-server managed_artwork --no-fail-fast' {
        cargo nextest run -p taru-server managed_artwork --no-fail-fast
    }

    Invoke-Step 'cargo nextest run -p taru-server self_host_smoke --no-fail-fast' {
        cargo nextest run -p taru-server self_host_smoke --no-fail-fast
    }
}

if (Test-Mode @('workspace')) {
    Invoke-Step 'cargo check --workspace --tests' {
        cargo check --workspace --tests
    }

    Invoke-Step 'cargo nextest run --workspace --no-fail-fast' {
        cargo nextest run --workspace --no-fail-fast
    }
}

if (Test-Mode @('postgres')) {
    Invoke-PostgresContracts $PostgresUrl
}

Write-Host ""
Write-Host 'Taru release gate completed.'
