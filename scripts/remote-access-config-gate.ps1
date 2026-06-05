#!/usr/bin/env pwsh
[CmdletBinding()]
param(
    [string]$OutputDir
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot '..')
Set-Location $RepoRoot

if ([string]::IsNullOrWhiteSpace($OutputDir)) {
    $OutputDir = Join-Path $RepoRoot 'target/release-gate/remote-access'
}

$Fixtures = @(
    @{
        Name = 'reverse-proxy'
        Config = 'deploy/remote-access/reverse-proxy.nako.toml'
        ExpectedChecks = @('network.access', 'network.proxy', 'network.origins', 'network.tunnel_providers')
        SensitiveValues = @(
            '127.0.0.1',
            'nako-reverse.redaction.invalid',
            'player-reverse.redaction.invalid',
            'webdav-reverse.redaction.invalid',
            'reverse-url-token',
            '10.66.10.5',
            'remote-access-fixture-admin-token',
            'x-forwarded-host',
            'x-forwarded-proto'
        )
    },
    @{
        Name = 'tunnel-provider'
        Config = 'deploy/remote-access/tunnel-provider.nako.toml'
        ExpectedChecks = @('network.access', 'network.proxy', 'network.origins', 'network.tunnel_providers')
        SensitiveValues = @(
            '127.0.0.1',
            'nako-tunnel.redaction.invalid',
            'player-tunnel.redaction.invalid',
            'cloudflare-tunnel.redaction.invalid',
            'webdav-tunnel.redaction.invalid',
            'tunnel-url-token',
            'tunnel-library-token',
            'remote-access-fixture-admin-token',
            'remote-access-fixture-tunnel-token',
            'x-forwarded-host',
            'x-forwarded-proto'
        )
    }
)

function Set-GateEnv {
    return @{
        AdminToken = [Environment]::GetEnvironmentVariable('NAKO_ADMIN_TOKEN', 'Process')
        TunnelToken = [Environment]::GetEnvironmentVariable('NAKO_REMOTE_ACCESS_GATE_TUNNEL_TOKEN', 'Process')
    }
}

function Restore-GateEnv {
    param(
        [Parameter(Mandatory = $true)]
        [hashtable]$Snapshot
    )

    if ($null -eq $Snapshot.AdminToken) {
        Remove-Item Env:NAKO_ADMIN_TOKEN -ErrorAction SilentlyContinue
    } else {
        $env:NAKO_ADMIN_TOKEN = $Snapshot.AdminToken
    }

    if ($null -eq $Snapshot.TunnelToken) {
        Remove-Item Env:NAKO_REMOTE_ACCESS_GATE_TUNNEL_TOKEN -ErrorAction SilentlyContinue
    } else {
        $env:NAKO_REMOTE_ACCESS_GATE_TUNNEL_TOKEN = $Snapshot.TunnelToken
    }
}

function Invoke-RemoteAccessFixture {
    param(
        [Parameter(Mandatory = $true)]
        [hashtable]$Fixture
    )

    $configPath = Join-Path $RepoRoot $Fixture.Config
    $outputPath = Join-Path $OutputDir "$($Fixture.Name)-config-check.json"

    Write-Host ""
    Write-Host "==> remote access config-check: $($Fixture.Name)"
    New-Item -ItemType Directory -Force -Path (Split-Path -Parent $outputPath) | Out-Null

    $snapshot = Set-GateEnv
    try {
        $env:NAKO_ADMIN_TOKEN = 'remote-access-fixture-admin-token'
        $env:NAKO_REMOTE_ACCESS_GATE_TUNNEL_TOKEN = 'remote-access-fixture-tunnel-token'

        $output = & cargo run -q -p nako-server -- --config $configPath config-check --json --create-dirs
        if ($LASTEXITCODE -ne 0) {
            throw "config-check failed for $($Fixture.Name) with exit code $LASTEXITCODE"
        }
    } finally {
        Restore-GateEnv -Snapshot $snapshot
    }

    $json = $output -join [Environment]::NewLine
    $json | Set-Content -Path $outputPath -Encoding utf8
    $report = $json | ConvertFrom-Json

    if ($report.status -ne 'pass') {
        throw "$($Fixture.Name) expected pass status, got $($report.status)"
    }

    $checksById = @{}
    foreach ($check in $report.checks) {
        $checksById[$check.id] = $check.status
    }

    foreach ($checkId in $Fixture.ExpectedChecks) {
        if (-not $checksById.ContainsKey($checkId)) {
            throw "$($Fixture.Name) missing expected check $checkId"
        }
        if ($checksById[$checkId] -ne 'pass') {
            throw "$($Fixture.Name) expected $checkId to pass, got $($checksById[$checkId])"
        }
    }

    foreach ($value in $Fixture.SensitiveValues) {
        if ($json.IndexOf($value, [StringComparison]::OrdinalIgnoreCase) -ge 0) {
            throw "$($Fixture.Name) leaked sensitive fixture value: $value"
        }
    }

    Write-Host "Report: $outputPath"
}

Write-Host "Nako remote access config gate"
Write-Host "Repository: $RepoRoot"
Write-Host "Output: $OutputDir"

foreach ($fixture in $Fixtures) {
    Invoke-RemoteAccessFixture -Fixture $fixture
}

Write-Host ""
Write-Host "Remote access config gate completed."
