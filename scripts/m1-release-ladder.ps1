#!/usr/bin/env pwsh
[CmdletBinding()]
param(
    [ValidateSet('docs', 'smoke', 'fast', 'release-fast', 'playback', 'container', 'postgres', 'workspace', 'all')]
    [string]$Mode = 'fast',

    [string]$PostgresUrl = $env:NAKO_TEST_POSTGRES_URL,

    [switch]$SkipRedactionInventory
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

function Write-LadderMap {
    $postgresState = if ([string]::IsNullOrWhiteSpace($PostgresUrl)) {
        'not provided; postgres gate will use its default harness behavior'
    } else {
        'provided via parameter or environment (redacted)'
    }

    Write-Host "Nako M1 release ladder"
    Write-Host "Mode: $Mode"
    Write-Host "Repository: $RepoRoot"
    Write-Host "PostgreSQL URL: $postgresState"
    Write-Host ""
    Write-Host "Mode map:"
    Write-Host "- docs: release docs hygiene, formatting, diff, and optional redaction inventory."
    Write-Host "- smoke: focused Product-Operator M1 journey smoke."
    Write-Host "- fast: docs plus Product-Operator M1 smoke; default local confidence path."
    Write-Host "- release-fast: existing technical release-gate fast mode."
    Write-Host "- playback/container/postgres/workspace: explicit expensive release dimensions."
    Write-Host "- all: docs, M1 smoke, release-fast, playback, container, postgres, and workspace."
}

function Invoke-ReleaseGate {
    param(
        [Parameter(Mandatory = $true)]
        [ValidateSet('docs', 'fast', 'playback', 'container', 'postgres', 'workspace')]
        [string]$GateMode,

        [switch]$SkipInventoryForThisGate
    )

    $commandArgs = @(
        '-NoProfile',
        '-ExecutionPolicy',
        'Bypass',
        '-File',
        'scripts/release-gate.ps1',
        '-Mode',
        $GateMode
    )

    if ($GateMode -eq 'postgres' -and -not [string]::IsNullOrWhiteSpace($PostgresUrl)) {
        $commandArgs += @('-PostgresUrl', $PostgresUrl)
    }

    if ($SkipRedactionInventory -or $SkipInventoryForThisGate) {
        $commandArgs += '-SkipRedactionInventory'
    }

    $displayName = if ($GateMode -eq 'postgres' -and -not [string]::IsNullOrWhiteSpace($PostgresUrl)) {
        'scripts/release-gate.ps1 -Mode postgres -PostgresUrl <provided>'
    } else {
        "scripts/release-gate.ps1 -Mode $GateMode"
    }

    if ($SkipRedactionInventory -or $SkipInventoryForThisGate) {
        $displayName = "$displayName -SkipRedactionInventory"
    }

    Invoke-Step $displayName {
        pwsh @commandArgs
    }
}

function Invoke-M1OperatorSmoke {
    param(
        [switch]$SkipDocsGate
    )

    $commandArgs = @(
        '-NoProfile',
        '-ExecutionPolicy',
        'Bypass',
        '-File',
        'scripts/m1-operator-journey-smoke.ps1',
        '-Mode',
        'fast'
    )

    $displayName = 'scripts/m1-operator-journey-smoke.ps1 -Mode fast'
    if ($SkipDocsGate) {
        $commandArgs += '-SkipDocsGate'
        $displayName = "$displayName -SkipDocsGate"
    }

    Invoke-Step $displayName {
        pwsh @commandArgs
    }
}

function Invoke-FastLadder {
    Invoke-ReleaseGate -GateMode 'docs'
    Invoke-M1OperatorSmoke -SkipDocsGate
}

function Invoke-AllLadder {
    Invoke-FastLadder
    Invoke-ReleaseGate -GateMode 'fast' -SkipInventoryForThisGate
    Invoke-ReleaseGate -GateMode 'playback' -SkipInventoryForThisGate
    Invoke-ReleaseGate -GateMode 'container' -SkipInventoryForThisGate
    Invoke-ReleaseGate -GateMode 'postgres' -SkipInventoryForThisGate
    Invoke-ReleaseGate -GateMode 'workspace' -SkipInventoryForThisGate
}

Write-LadderMap

switch ($Mode) {
    'docs' {
        Invoke-ReleaseGate -GateMode 'docs'
    }
    'smoke' {
        Invoke-M1OperatorSmoke
    }
    'fast' {
        Invoke-FastLadder
    }
    'release-fast' {
        Invoke-ReleaseGate -GateMode 'fast'
    }
    'playback' {
        Invoke-ReleaseGate -GateMode 'playback'
    }
    'container' {
        Invoke-ReleaseGate -GateMode 'container'
    }
    'postgres' {
        Invoke-ReleaseGate -GateMode 'postgres'
    }
    'workspace' {
        Invoke-ReleaseGate -GateMode 'workspace'
    }
    'all' {
        Invoke-AllLadder
    }
}

Write-Host ""
Write-Host 'Nako M1 release ladder completed.'
