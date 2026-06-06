#!/usr/bin/env pwsh
[CmdletBinding()]
param(
    [ValidateSet('docs', 'server', 'admin-web', 'fast')]
    [string]$Mode = 'fast',

    [switch]$SkipDocsGate
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

function Write-JourneyMap {
    Write-Host "Nako M1 operator journey smoke"
    Write-Host "Mode: $Mode"
    Write-Host "Repository: $RepoRoot"
    Write-Host ""
    Write-Host "Coverage map:"
    Write-Host "- Library config and scan/index: Admin Web App.test route coverage plus server self_host_smoke."
    Write-Host "- Browse and playback readiness: Media Web mediaSurface.test plus server self_host_smoke."
    Write-Host "- Diagnostics, repair, and redaction: Admin Web App.test plus docs-safe release gate."
    Write-Host "- Historical Web MVP evidence remains in docs/workstreams/web-mvp-live-smoke/EVIDENCE_AND_GATES.md."
}

function Invoke-DocsGate {
    if ($SkipDocsGate) {
        Write-Host ""
        Write-Host "==> docs-safe release gate"
        Write-Host "Skipped by -SkipDocsGate."
        return
    }

    Invoke-Step 'scripts/release-gate.ps1 -Mode docs -SkipRedactionInventory' {
        pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/release-gate.ps1 -Mode docs -SkipRedactionInventory
    }
}

function Invoke-ServerSmoke {
    Invoke-Step 'scripts/self-host-smoke.ps1' {
        pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/self-host-smoke.ps1
    }
}

function Invoke-AdminWebSmoke {
    Invoke-Step 'Admin Web M1 route/media smoke tests' {
        npm run test --prefix apps/admin-web -- App.test.tsx src/surfaces/media/mediaSurface.test.tsx
    }
}

Write-JourneyMap

if ($Mode -eq 'docs') {
    Invoke-DocsGate
    exit 0
}

Invoke-DocsGate

if ($Mode -eq 'server' -or $Mode -eq 'fast') {
    Invoke-ServerSmoke
}

if ($Mode -eq 'admin-web' -or $Mode -eq 'fast') {
    Invoke-AdminWebSmoke
}

Write-Host ""
Write-Host 'M1 operator journey smoke completed.'
