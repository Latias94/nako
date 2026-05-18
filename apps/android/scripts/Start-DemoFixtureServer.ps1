[CmdletBinding()]
param(
    [int]$Port = 3018,
    [string]$FixtureRoot,
    [string]$ServerBinary,
    [string]$FfmpegPath,
    [string]$FfprobePath,
    [string]$Serial,
    [switch]$SkipBuild,
    [switch]$SkipSeed,
    [switch]$PrepareOnly,
    [switch]$AdbReverse
)

$ErrorActionPreference = 'Stop'

function Write-Utf8File {
    param(
        [string]$Path,
        [string]$Content
    )

    $utf8 = [System.Text.UTF8Encoding]::new($false)
    [System.IO.File]::WriteAllText($Path, $Content, $utf8)
}

function Resolve-CommandPath {
    param(
        [string]$Name,
        [string]$ProvidedPath
    )

    if (-not [string]::IsNullOrWhiteSpace($ProvidedPath)) {
        if (-not (Test-Path -LiteralPath $ProvidedPath)) {
            throw "$Name was not found at '$ProvidedPath'."
        }

        return (Resolve-Path -LiteralPath $ProvidedPath).Path
    }

    $command = Get-Command $Name -ErrorAction SilentlyContinue
    if ($command -and $command.Path) {
        return $command.Path
    }

    throw "$Name was not found. Add it to PATH or pass -$($Name.Substring(0, 1).ToUpper())$($Name.Substring(1))Path."
}

function Resolve-AdbPath {
    $command = Get-Command adb -ErrorAction SilentlyContinue
    if ($command -and $command.Path) {
        return $command.Path
    }

    foreach ($sdkRoot in @($env:ANDROID_SDK_ROOT, $env:ANDROID_HOME)) {
        if ([string]::IsNullOrWhiteSpace($sdkRoot)) {
            continue
        }

        $candidate = Join-Path $sdkRoot 'platform-tools\adb.exe'
        if (Test-Path -LiteralPath $candidate) {
            return (Resolve-Path -LiteralPath $candidate).Path
        }
    }

    throw 'adb was not found. Set ANDROID_SDK_ROOT or ANDROID_HOME, or add platform-tools to PATH.'
}

function Get-ConnectedDeviceSerial {
    param(
        [string]$AdbPath,
        [string]$RequestedSerial
    )

    & $AdbPath start-server | Out-Null
    if ($LASTEXITCODE -ne 0) {
        throw 'Failed to start adb server.'
    }

    $devices = & $AdbPath devices 2>$null
    if ($LASTEXITCODE -ne 0) {
        throw 'Failed to query connected Android devices.'
    }

    $connected = @()
    foreach ($line in $devices | Select-Object -Skip 1) {
        if ($line -match '^(?<serial>\S+)\s+device$') {
            $connected += $Matches.serial
        }
    }

    if (-not [string]::IsNullOrWhiteSpace($RequestedSerial)) {
        if ($connected -contains $RequestedSerial) {
            return $RequestedSerial
        }

        throw "Requested device '$RequestedSerial' is not connected. Connected devices: $($connected -join ', ')."
    }

    if ($connected.Count -eq 1) {
        return $connected[0]
    }

    if ($connected.Count -eq 0) {
        throw 'No connected Android devices were found. Start an emulator or omit -AdbReverse.'
    }

    throw "Multiple connected Android devices were found: $($connected -join ', '). Re-run with -Serial."
}

function Invoke-Native {
    param(
        [string]$Command,
        [string[]]$Arguments,
        [string]$FailureMessage
    )

    & $Command @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw $FailureMessage
    }
}

function Convert-ToPortablePath {
    param([string]$Path)

    return (Resolve-Path -LiteralPath $Path).Path.Replace('\', '/')
}

function Get-TaruServerBinary {
    param(
        [string]$RepoRoot,
        [string]$ProvidedBinary,
        [bool]$SkipBuild
    )

    if (-not [string]::IsNullOrWhiteSpace($ProvidedBinary)) {
        if (-not (Test-Path -LiteralPath $ProvidedBinary)) {
            throw "taru-server binary was not found at '$ProvidedBinary'."
        }

        return (Resolve-Path -LiteralPath $ProvidedBinary).Path
    }

    if (-not $SkipBuild) {
        Push-Location $RepoRoot
        try {
            Invoke-Native -Command 'cargo' -Arguments @('build', '-p', 'taru-server') -FailureMessage 'cargo build -p taru-server failed.'
        } finally {
            Pop-Location
        }
    }

    $candidate = Join-Path $RepoRoot 'target\debug\taru-server.exe'
    if (Test-Path -LiteralPath $candidate) {
        return (Resolve-Path -LiteralPath $candidate).Path
    }

    $unixCandidate = Join-Path $RepoRoot 'target\debug\taru-server'
    if (Test-Path -LiteralPath $unixCandidate) {
        return (Resolve-Path -LiteralPath $unixCandidate).Path
    }

    throw 'taru-server binary was not found. Run cargo build -p taru-server or pass -ServerBinary.'
}

function New-DemoVideo {
    param(
        [string]$Ffmpeg,
        [string]$OutputPath
    )

    if (Test-Path -LiteralPath $OutputPath) {
        return
    }

    $arguments = @(
        '-y',
        '-hide_banner',
        '-loglevel', 'error',
        '-f', 'lavfi',
        '-i', 'testsrc=size=640x360:rate=24',
        '-f', 'lavfi',
        '-i', 'sine=frequency=440:sample_rate=48000',
        '-t', '2',
        '-pix_fmt', 'yuv420p',
        '-c:v', 'libx264',
        '-preset', 'veryfast',
        '-c:a', 'aac',
        '-movflags', '+faststart',
        $OutputPath
    )

    & $Ffmpeg @arguments
    if ($LASTEXITCODE -eq 0) {
        return
    }

    $fallbackArguments = @(
        '-y',
        '-hide_banner',
        '-loglevel', 'error',
        '-f', 'lavfi',
        '-i', 'testsrc=size=640x360:rate=24',
        '-f', 'lavfi',
        '-i', 'sine=frequency=440:sample_rate=48000',
        '-t', '2',
        '-pix_fmt', 'yuv420p',
        '-c:v', 'mpeg4',
        '-c:a', 'aac',
        '-movflags', '+faststart',
        $OutputPath
    )

    Invoke-Native -Command $Ffmpeg -Arguments $fallbackArguments -FailureMessage 'ffmpeg could not create the demo MP4 fixture.'
}

function Write-DemoNfo {
    param([string]$Path)

    $xml = @'
<movie>
  <title>Night Harbor</title>
  <sorttitle>Night Harbor</sorttitle>
  <plot>A lighthouse keeper finds a signal buried inside a quiet coastal broadcast.</plot>
  <releasedate>2026-05-18</releasedate>
  <runtime>2</runtime>
  <tagline>Every signal has a shore.</tagline>
  <genre>Mystery</genre>
  <tag>Lighthouse</tag>
  <actor>
    <name>Mira Vale</name>
    <role>Keeper</role>
    <order>0</order>
  </actor>
</movie>
'@
    Write-Utf8File -Path $Path -Content $xml
}

function Write-DemoConfig {
    param(
        [string]$Path,
        [string]$DatabasePath,
        [string]$MediaRoot,
        [string]$CacheRoot,
        [string]$Ffmpeg,
        [string]$Ffprobe,
        [int]$ListenPort
    )

    $databaseUrl = "sqlite://$(Convert-ToPortablePath -Path $DatabasePath)"
    $mediaRootPath = Convert-ToPortablePath -Path $MediaRoot
    $cacheRootPath = Convert-ToPortablePath -Path $CacheRoot
    $ffmpegConfigPath = Convert-ToPortablePath -Path $Ffmpeg
    $ffprobeConfigPath = Convert-ToPortablePath -Path $Ffprobe

    $toml = @"
listen_addr = "127.0.0.1:$ListenPort"
database_url = "$databaseUrl"
ffprobe_path = "$ffprobeConfigPath"
ffmpeg_path = "$ffmpegConfigPath"
scan_concurrency = 1
probe_concurrency = 1
metadata_concurrency = 1
remux_concurrency = 1
webhook_concurrency = 1
remux_timeout_ms = 60000
remux_staging_root = "$cacheRootPath/remux"

[auth]
enabled = false

[staging]
cleanup_on_startup = true

[playback]
remote_stream_concurrency = 1
remote_stage_concurrency = 1

[[libraries]]
id = "018f0000-0000-7000-8000-000000000301"
name = "Movies"
root = "$mediaRootPath"
preset = "movies"
"@

    Write-Utf8File -Path $Path -Content $toml
}

$scriptDir = $PSScriptRoot
$androidRoot = (Resolve-Path -LiteralPath (Join-Path $scriptDir '..')).Path
$repoRoot = Split-Path -Parent (Split-Path -Parent $androidRoot)

if ([string]::IsNullOrWhiteSpace($FixtureRoot)) {
    $FixtureRoot = Join-Path $androidRoot 'build\demo-fixtures\server-backed'
}

$fixtureRootPath = $FixtureRoot
New-Item -ItemType Directory -Force -Path $fixtureRootPath | Out-Null
$fixtureRootPath = (Resolve-Path -LiteralPath $fixtureRootPath).Path

$mediaRoot = Join-Path $fixtureRootPath 'media'
$cacheRoot = Join-Path $fixtureRootPath 'cache'
$databaseRoot = Join-Path $fixtureRootPath 'db'
New-Item -ItemType Directory -Force -Path $mediaRoot, $cacheRoot, $databaseRoot | Out-Null

$ffmpeg = Resolve-CommandPath -Name 'ffmpeg' -ProvidedPath $FfmpegPath
$ffprobe = Resolve-CommandPath -Name 'ffprobe' -ProvidedPath $FfprobePath
$server = Get-TaruServerBinary -RepoRoot $repoRoot -ProvidedBinary $ServerBinary -SkipBuild ([bool]$SkipBuild)

$videoPath = Join-Path $mediaRoot 'Night Harbor.mp4'
$nfoPath = Join-Path $mediaRoot 'Night Harbor.nfo'
$databasePath = Join-Path $databaseRoot 'taru-demo.db'
$configPath = Join-Path $fixtureRootPath 'taru.toml'
$summaryPath = Join-Path $fixtureRootPath 'summary.json'

New-DemoVideo -Ffmpeg $ffmpeg -OutputPath $videoPath
Write-DemoNfo -Path $nfoPath
if (-not (Test-Path -LiteralPath $databasePath)) {
    New-Item -ItemType File -Path $databasePath | Out-Null
}
Write-DemoConfig -Path $configPath -DatabasePath $databasePath -MediaRoot $mediaRoot -CacheRoot $cacheRoot -Ffmpeg $ffmpeg -Ffprobe $ffprobe -ListenPort $Port

if (-not $SkipSeed) {
    Invoke-Native -Command $server -Arguments @('--config', $configPath, 'scan') -FailureMessage 'taru-server scan failed for the demo fixture.'
    Invoke-Native -Command $server -Arguments @('--config', $configPath, 'import-nfo') -FailureMessage 'taru-server import-nfo failed for the demo fixture.'
}

if ($AdbReverse) {
    $adb = Resolve-AdbPath
    $deviceSerial = Get-ConnectedDeviceSerial -AdbPath $adb -RequestedSerial $Serial
    Invoke-Native -Command $adb -Arguments @('-s', $deviceSerial, 'reverse', "tcp:$Port", "tcp:$Port") -FailureMessage 'adb reverse failed for the demo fixture server.'
}

$summary = [ordered]@{
    fixture = 'profile-with-media'
    base_url = "http://127.0.0.1:$Port"
    server_binary = $server
    config = $configPath
    media_root = $mediaRoot
    video = $videoPath
    nfo = $nfoPath
    database = $databasePath
    auth = 'disabled'
    android_smoke_token = 'demo-fixture-token'
    adb_reverse = [bool]$AdbReverse
}

Write-Utf8File -Path $summaryPath -Content ($summary | ConvertTo-Json -Depth 4)

Write-Host "Demo fixture prepared."
Write-Host "Base URL: http://127.0.0.1:$Port"
Write-Host "Config: $configPath"
Write-Host "Summary: $summaryPath"
Write-Host "Android smoke token placeholder: demo-fixture-token"
if ($AdbReverse) {
    Write-Host "ADB reverse: tcp:$Port -> tcp:$Port"
}

if ($PrepareOnly) {
    return
}

Write-Host "Starting taru-server. Stop this process with Ctrl+C when finished."
& $server --config $configPath serve
if ($LASTEXITCODE -ne 0) {
    throw 'taru-server serve failed.'
}
