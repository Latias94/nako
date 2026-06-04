#!/usr/bin/env pwsh
[CmdletBinding()]
param(
    [ValidateSet('docs', 'fast', 'db', 'api', 'playback', 'postgres', 'container', 'workspace', 'all')]
    [string]$Mode = 'fast',

    [string]$PostgresUrl = $env:NAKO_TEST_POSTGRES_URL,

    [switch]$SkipRedactionInventory
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot '..')
Set-Location $RepoRoot

$ReleaseGateOutput = Join-Path $RepoRoot 'target/release-gate'
$RedactionInventoryPath = Join-Path $ReleaseGateOutput 'redaction-inventory.txt'
$PlaybackHardwareReportPath = Join-Path $ReleaseGateOutput 'playback-hardware-report.json'
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

function Invoke-ContainerGate {
    $composeEnv = @{
        NAKO_ADMIN_TOKEN = 'release-gate-admin-token'
        NAKO_POSTGRES_PASSWORD = 'release-gate-postgres-password'
        NAKO_MEDIA_ROOT = (Join-Path $RepoRoot 'target/release-gate/media/movies')
    }

    New-Item -ItemType Directory -Force -Path $composeEnv.NAKO_MEDIA_ROOT | Out-Null

    Invoke-Step 'cargo nextest run -p nako-server config --no-fail-fast' {
        cargo nextest run -p nako-server config --no-fail-fast
    }

    Invoke-Step 'docker compose config for Nako SQLite stack' {
        & {
            $env:NAKO_ADMIN_TOKEN = $composeEnv.NAKO_ADMIN_TOKEN
            $env:NAKO_MEDIA_ROOT = $composeEnv.NAKO_MEDIA_ROOT
            docker compose -f deploy/compose/nako-sqlite.yml config | Out-Null
        }
    }

    Invoke-Step 'docker compose config for Nako PostgreSQL stack' {
        & {
            $env:NAKO_ADMIN_TOKEN = $composeEnv.NAKO_ADMIN_TOKEN
            $env:NAKO_POSTGRES_PASSWORD = $composeEnv.NAKO_POSTGRES_PASSWORD
            $env:NAKO_MEDIA_ROOT = $composeEnv.NAKO_MEDIA_ROOT
            docker compose -f deploy/compose/nako-postgres.yml config | Out-Null
        }
    }
}

function Invoke-ApiSdkGate {
    Invoke-Step 'cargo check -p nako-api --tests' {
        cargo check -p nako-api --tests
    }

    Invoke-Step 'cargo check -p nako-client --tests' {
        cargo check -p nako-client --tests
    }

    Invoke-Step 'cargo check -p nako-client-protocol --tests' {
        cargo check -p nako-client-protocol --tests
    }

    Invoke-Step 'cargo nextest run -p nako-api openapi --no-fail-fast' {
        cargo nextest run -p nako-api openapi --no-fail-fast
    }

    Invoke-Step 'cargo nextest run -p nako-api sdk --no-fail-fast' {
        cargo nextest run -p nako-api sdk --no-fail-fast
    }

    Invoke-Step 'cargo nextest run -p nako-api admin_contract --no-fail-fast' {
        cargo nextest run -p nako-api admin_contract --no-fail-fast
    }

    Invoke-Step 'cargo nextest run -p nako-client --no-fail-fast' {
        cargo nextest run -p nako-client --no-fail-fast
    }

    Invoke-Step 'cargo nextest run -p nako-client-protocol --no-fail-fast' {
        cargo nextest run -p nako-client-protocol --no-fail-fast
    }

    Invoke-Step 'cargo tree -p nako-client' {
        cargo tree -p nako-client
    }

    Invoke-Step 'cargo tree -p nako-client-protocol' {
        cargo tree -p nako-client-protocol
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

function Invoke-PlaybackGate {
    Invoke-Step 'ffmpeg -version' {
        ffmpeg -version
    }

    Invoke-Step 'ffprobe -version' {
        ffprobe -version
    }

    Invoke-Step 'cargo check -p nako-transcode -p nako-server --tests' {
        cargo check -p nako-transcode -p nako-server --tests
    }

    Invoke-Step 'cargo nextest run -p nako-transcode hardware --no-fail-fast' {
        cargo nextest run -p nako-transcode hardware --no-fail-fast
    }

    Invoke-Step 'cargo run -p nako-transcode --example hardware-report -- --ffmpeg ffmpeg --output target/release-gate/playback-hardware-report.json' {
        cargo run -p nako-transcode --example hardware-report -- --ffmpeg ffmpeg --output $PlaybackHardwareReportPath
    }

    Invoke-Step 'cargo nextest run -p nako-transcode hls --no-fail-fast' {
        cargo nextest run -p nako-transcode hls --no-fail-fast
    }

    Invoke-Step 'cargo nextest run -p nako-server self_host_smoke --no-fail-fast' {
        cargo nextest run -p nako-server self_host_smoke --no-fail-fast
    }
}

Write-Host "Nako release gate"
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
    Invoke-Step 'cargo check -p nako-db --tests' {
        cargo check -p nako-db --tests
    }

    Invoke-Step 'cargo nextest run -p nako-db sqlite_managed_artwork_contract --no-fail-fast' {
        cargo nextest run -p nako-db sqlite_managed_artwork_contract --no-fail-fast
    }
}

if (Test-Mode @('fast', 'api')) {
    Invoke-Step 'cargo check -p nako-server --tests' {
        cargo check -p nako-server --tests
    }

    Invoke-ApiSdkGate

    Invoke-Step 'cargo nextest run -p nako-api managed_artwork --no-fail-fast' {
        cargo nextest run -p nako-api managed_artwork --no-fail-fast
    }

    Invoke-Step 'cargo nextest run -p nako-server managed_artwork --no-fail-fast' {
        cargo nextest run -p nako-server managed_artwork --no-fail-fast
    }

    Invoke-Step 'cargo nextest run -p nako-server self_host_smoke --no-fail-fast' {
        cargo nextest run -p nako-server self_host_smoke --no-fail-fast
    }
}

if (Test-Mode @('playback')) {
    Invoke-PlaybackGate
}

if (Test-Mode @('container')) {
    Invoke-ContainerGate
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
Write-Host 'Nako release gate completed.'
