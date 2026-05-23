#!/usr/bin/env pwsh
[CmdletBinding()]
param(
    [string]$AddonRepo = '',
    [ValidateSet('cargo-install', 'workspace')]
    [string]$AddonBinarySource = 'cargo-install',
    [string]$AddonVersion = '0.1.0-alpha.1',
    [string]$NakoImage = 'ghcr.io/latias94/nako-server:0.1.0-alpha.1',
    [int]$NakoPort = 30130,
    [int]$SidecarPort = 19100,
    [switch]$SkipAddonBuild,
    [switch]$ForceAddonInstall,
    [switch]$NoCleanup
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
Set-Location $RepoRoot

if ([string]::IsNullOrWhiteSpace($AddonRepo)) {
    $AddonRepo = (Resolve-Path (Join-Path $RepoRoot '..\nako-official-addons')).Path
} else {
    $AddonRepo = (Resolve-Path $AddonRepo).Path
}

$SmokeScript = Join-Path $AddonRepo 'addons\metadata-scraper\smoke.local.ps1'
if (-not (Test-Path $SmokeScript)) {
    throw "Official metadata scraper smoke script not found: $SmokeScript"
}

function Invoke-NativeStep {
    param(
        [Parameter(Mandatory = $true)][string]$Name,
        [Parameter(Mandatory = $true)][scriptblock]$Action
    )

    Write-Host ''
    Write-Host "==> $Name"
    & $Action
    if ($LASTEXITCODE -ne 0) {
        throw "$Name failed with exit code $LASTEXITCODE."
    }
}

function Test-TcpPortOpen {
    param(
        [Parameter(Mandatory = $true)][string]$HostName,
        [Parameter(Mandatory = $true)][int]$Port
    )

    $client = [System.Net.Sockets.TcpClient]::new()
    try {
        $connect = $client.BeginConnect($HostName, $Port, $null, $null)
        if (-not $connect.AsyncWaitHandle.WaitOne(300)) {
            return $false
        }

        $client.EndConnect($connect)
        return $true
    } catch {
        return $false
    } finally {
        $client.Close()
    }
}

function Wait-HttpJson {
    param(
        [Parameter(Mandatory = $true)][string]$Url,
        [Parameter(Mandatory = $true)][string]$Name,
        [int]$TimeoutSeconds = 90
    )

    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    do {
        try {
            return Invoke-RestMethod -Uri $Url -Method GET -TimeoutSec 3
        } catch {
            if ((Get-Date) -ge $deadline) {
                throw "$Name did not become ready at $Url. Last error: $($_.Exception.Message)"
            }

            Start-Sleep -Seconds 2
        }
    } while ($true)
}

function Invoke-NakoAdminJson {
    param(
        [Parameter(Mandatory = $true)][string]$Method,
        [Parameter(Mandatory = $true)][string]$Path,
        [object]$Body = $null
    )

    $headers = @{
        Authorization = "Bearer $AdminToken"
    }
    $uri = "http://127.0.0.1:${NakoPort}$Path"
    if ($null -eq $Body) {
        return Invoke-RestMethod -Uri $uri -Method $Method -Headers $headers -TimeoutSec 10
    }

    $headers['Content-Type'] = 'application/json'
    $json = $Body | ConvertTo-Json -Depth 20
    return Invoke-RestMethod -Uri $uri -Method $Method -Headers $headers -Body $json -TimeoutSec 10
}

function Save-DockerLogs {
    param(
        [Parameter(Mandatory = $true)][string]$Container,
        [Parameter(Mandatory = $true)][string]$Path
    )

    try {
        docker logs $Container 2>&1 | Out-File -FilePath $Path -Encoding utf8
    } catch {
        "Unable to capture docker logs: $($_.Exception.Message)" |
            Out-File -FilePath $Path -Encoding utf8
    }
}

if (Test-TcpPortOpen -HostName '127.0.0.1' -Port $NakoPort) {
    throw "Nako host port is already in use: 127.0.0.1:$NakoPort"
}
if (Test-TcpPortOpen -HostName '127.0.0.1' -Port $SidecarPort) {
    throw "Metadata scraper sidecar port is already in use: 127.0.0.1:$SidecarPort"
}

$RunId = Get-Date -Format 'yyyyMMdd-HHmmss'
$RunRoot = Join-Path $RepoRoot "target\oae2e-alpha2-hosted\$RunId"
$DataDir = Join-Path $RunRoot 'data'
$CacheDir = Join-Path $RunRoot 'cache'
$MediaDir = Join-Path $RunRoot 'media'
$LogDir = Join-Path $RunRoot 'logs'
New-Item -ItemType Directory -Force -Path $DataDir, $CacheDir, $MediaDir, $LogDir | Out-Null

$ConfigPath = (Resolve-Path 'deploy\container\sqlite.nako.toml').Path
$AdminToken = "oae2e-alpha2-local-token-$([guid]::NewGuid())"
$Container = "nako-oae2e-alpha2-$PID"
$SidecarProcess = $null

$BinaryName = if ($IsWindows) { 'nako-metadata-scraper.exe' } else { 'nako-metadata-scraper' }
$AddonInstallRoot = Join-Path $RepoRoot "target\oae2e-alpha2-addon-install\$AddonVersion"
$SidecarBinary = if ($AddonBinarySource -eq 'cargo-install') {
    Join-Path $AddonInstallRoot "bin\$BinaryName"
} else {
    Join-Path $AddonRepo "target\debug\$BinaryName"
}

try {
    Write-Host "Run root: $RunRoot"
    Write-Host "Nako image: $NakoImage"
    Write-Host "Addon repo: $AddonRepo"
    Write-Host "Addon binary source: $AddonBinarySource"

    Invoke-NativeStep "docker run $NakoImage" {
        $dockerArgs = @(
            'run', '-d',
            '--name', $Container,
            '--add-host', 'host.docker.internal:host-gateway',
            '-p', "127.0.0.1:${NakoPort}:3000",
            '-e', "NAKO_ADMIN_TOKEN=$AdminToken",
            '--mount', "type=bind,source=$ConfigPath,target=/config/nako.toml,readonly",
            '--mount', "type=bind,source=$DataDir,target=/data",
            '--mount', "type=bind,source=$CacheDir,target=/cache",
            '--mount', "type=bind,source=$MediaDir,target=/media/movies,readonly",
            '--entrypoint', '/usr/bin/tini',
            $NakoImage,
            '--',
            'sh',
            '-c',
            'nako-server --config /config/nako.toml config-check --create-dirs && exec nako-server --config /config/nako.toml serve'
        )

        $containerId = docker @dockerArgs
        if ($LASTEXITCODE -eq 0 -and $containerId) {
            Write-Host "Started Nako container $($containerId.Substring(0, [Math]::Min(12, $containerId.Length)))."
        }
    }

    Wait-HttpJson -Url "http://127.0.0.1:${NakoPort}/health" -Name 'Nako health' -TimeoutSeconds 120 | Out-Null
    Write-Host "Nako health OK at http://127.0.0.1:${NakoPort}/health"

    if ($AddonBinarySource -eq 'cargo-install') {
        if ($ForceAddonInstall -or -not (Test-Path $SidecarBinary)) {
            Invoke-NativeStep "cargo install nako-metadata-scraper@$AddonVersion" {
                cargo install nako-metadata-scraper --version $AddonVersion --root $AddonInstallRoot --force --locked
            }
        } else {
            Write-Host "Using installed metadata scraper binary: $SidecarBinary"
        }
    } elseif (-not $SkipAddonBuild) {
        Invoke-NativeStep 'cargo build -p nako-metadata-scraper' {
            Push-Location $AddonRepo
            try {
                cargo build -p nako-metadata-scraper
            } finally {
                Pop-Location
            }
        }
    }

    if (-not (Test-Path $SidecarBinary)) {
        throw "Metadata scraper binary not found: $SidecarBinary"
    }

    $SidecarOut = Join-Path $LogDir 'sidecar.out.log'
    $SidecarErr = Join-Path $LogDir 'sidecar.err.log'
    $env:NAKO_METADATA_SCRAPER_LISTEN_ADDR = "0.0.0.0:${SidecarPort}"
    $env:NAKO_METADATA_SCRAPER_BASE_URL = "http://host.docker.internal:${SidecarPort}"
    $env:NAKO_METADATA_SCRAPER_PROVIDER_FIXTURE_ENABLED = 'true'
    $env:NAKO_METADATA_SCRAPER_PROVIDER_TMDB_ENABLED = 'false'
    $env:NAKO_METADATA_SCRAPER_PROVIDER_BANGUMI_ENABLED = 'false'
    $env:NAKO_METADATA_SCRAPER_LANGUAGE = 'en-US'

    Write-Host "Starting metadata scraper sidecar on 127.0.0.1:$SidecarPort for host callers."
    Write-Host "Sidecar manifest base_url will be http://host.docker.internal:$SidecarPort for the Nako container."
    $startArgs = @{
        FilePath = $SidecarBinary
        WorkingDirectory = $AddonRepo
        RedirectStandardOutput = $SidecarOut
        RedirectStandardError = $SidecarErr
        PassThru = $true
    }
    if ($IsWindows) {
        $startArgs['WindowStyle'] = 'Hidden'
    }
    $SidecarProcess = Start-Process @startArgs

    $manifest = $null
    $deadline = (Get-Date).AddSeconds(90)
    do {
        if ($SidecarProcess.HasExited) {
            $stderr = if (Test-Path $SidecarErr) { Get-Content $SidecarErr -Raw } else { '' }
            throw "Metadata scraper sidecar exited early with code $($SidecarProcess.ExitCode). stderr: $stderr"
        }

        try {
            $manifest = Invoke-RestMethod -Uri "http://127.0.0.1:${SidecarPort}/manifest.json" -Method GET -TimeoutSec 3
            break
        } catch {
            if ((Get-Date) -ge $deadline) {
                throw "Metadata scraper manifest did not become ready. Last error: $($_.Exception.Message)"
            }

            Start-Sleep -Seconds 2
        }
    } while ($true)

    Write-Host "Metadata scraper manifest OK: $($manifest.id)@$($manifest.version), protocol=$($manifest.protocol_version)"

    $env:NAKO_ADMIN_TOKEN = $AdminToken
    Invoke-NativeStep 'official metadata scraper Admin-mediated smoke' {
        pwsh -NoProfile -ExecutionPolicy Bypass -File $SmokeScript `
            -SidecarBaseUrl "http://127.0.0.1:${SidecarPort}" `
            -NakoBaseUrl "http://127.0.0.1:${NakoPort}" `
            -RegisterInNako `
            -Enable `
            -RunResourceCall `
            -RequireNako
    }

    Invoke-NativeStep 'Nako manager plan confirmation smoke' {
        $addons = Invoke-NakoAdminJson -Method GET -Path '/admin/v1/addons'
        $addon = $addons.addons |
            Where-Object {
                $_.manifest_id -eq $manifest.id -and
                $_.base_url -eq "http://host.docker.internal:${SidecarPort}"
            } |
            Select-Object -First 1
        if ($null -eq $addon) {
            throw "Registered addon not found for manifest $($manifest.id)."
        }

        $plan = Invoke-NakoAdminJson -Method POST -Path "/admin/v1/addons/$($addon.id)/manager-plan" -Body @{
            intent = 'update'
            operator_confirmed = $true
        }

        if ($plan.intent -ne 'update') {
            throw "Manager plan intent mismatch: expected update, got $($plan.intent)."
        }
        if (-not $plan.operator_confirmed) {
            throw "Manager plan should be operator-confirmed."
        }
        if ($plan.addon_id -ne $addon.id) {
            throw "Manager plan addon_id mismatch: expected $($addon.id), got $($plan.addon_id)."
        }
    }

    Save-DockerLogs -Container $Container -Path (Join-Path $LogDir 'nako-container.log')
    Write-Host ''
    Write-Host '[ok] Official Addon E2E smoke completed.'
    Write-Host "Logs: $LogDir"
} catch {
    Save-DockerLogs -Container $Container -Path (Join-Path $LogDir 'nako-container.log')
    Write-Warning $_.Exception.Message
    Write-Warning "Logs: $LogDir"
    throw
} finally {
    if ($null -ne $SidecarProcess -and -not $SidecarProcess.HasExited) {
        Stop-Process -Id $SidecarProcess.Id -Force -ErrorAction SilentlyContinue
    }

    if (-not $NoCleanup) {
        docker rm -f $Container *> $null
    }
}
