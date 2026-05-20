[CmdletBinding()]
param(
    [string]$Serial,
    [string]$OutputRoot,
    [ValidateSet('current-state', 'empty-setup', 'profile-missing-token', 'profile-with-media', 'profile-active-remux')]
    [string]$FixtureState = 'current-state',
    [int]$FixtureServerPort = 3018,
    [string]$FixtureAccessToken = 'demo-fixture-token',
    [switch]$SkipBuild,
    [switch]$SkipAppBuild,
    [switch]$SkipFixtureServerBuild,
    [switch]$ResetAppData
)

$ErrorActionPreference = 'Stop'

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
        throw 'No connected Android devices were found. Start an emulator, confirm adb devices shows it in device state, then re-run the smoke script.'
    }

    throw "Multiple connected Android devices were found: $($connected -join ', '). Re-run with -Serial."
}

function Invoke-Adb {
    param(
        [string]$AdbPath,
        [string[]]$Arguments,
        [string]$FailureMessage
    )

    $output = & $AdbPath @Arguments 2>&1
    if ($LASTEXITCODE -ne 0) {
        if ($output) {
            throw "$FailureMessage`n$output"
        }

        throw $FailureMessage
    }
}

function Wait-ForHttpHealth {
    param(
        [string]$BaseUrl,
        [System.Diagnostics.Process]$Process,
        [int]$TimeoutSeconds = 20
    )

    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    $healthUrl = "$BaseUrl/health"
    $lastError = $null
    while ((Get-Date) -lt $deadline) {
        if ($Process -ne $null -and $Process.HasExited) {
            throw "Fixture server exited before health check passed. Exit code: $($Process.ExitCode)"
        }

        try {
            $response = Invoke-WebRequest -Uri $healthUrl -UseBasicParsing -TimeoutSec 2
            if ($response.StatusCode -ge 200 -and $response.StatusCode -lt 300) {
                return
            }
        } catch {
            $lastError = $_.Exception.Message
        }

        Start-Sleep -Milliseconds 500
    }

    throw "Fixture server health check did not pass at '$healthUrl'. Last error: $lastError"
}

function Resolve-SmokeMediaResumeFixture {
    param(
        [string]$BaseUrl,
        [string]$AccessToken
    )

    $headers = @{}
    if (-not [string]::IsNullOrWhiteSpace($AccessToken)) {
        $headers['Authorization'] = "Bearer $AccessToken"
    }

    $itemsUrl = "$BaseUrl/items?limit=1&offset=0"
    $items = Invoke-RestMethod -Uri $itemsUrl -Headers $headers -TimeoutSec 10
    $itemCandidates = @()
    if ($null -ne $items.items) {
        $itemCandidates = @($items.items)
    }
    if ($itemCandidates.Count -eq 0) {
        throw "Fixture server did not return any Media Items at '$itemsUrl'."
    }

    $item = $itemCandidates[0]
    $itemId = [string]$item.id
    if ([string]::IsNullOrWhiteSpace($itemId)) {
        throw 'Fixture server returned a Media Item without an id.'
    }

    $encodedItemId = [System.Uri]::EscapeDataString($itemId)
    $detailUrl = "$BaseUrl/items/$encodedItemId"
    $detail = Invoke-RestMethod -Uri $detailUrl -Headers $headers -TimeoutSec 10
    $sourceCandidates = @()
    if ($null -ne $detail.sources) {
        $sourceCandidates = @($detail.sources)
    }
    if ($sourceCandidates.Count -eq 0) {
        throw "Fixture server did not return any Media Sources at '$detailUrl'."
    }

    $source = $sourceCandidates[0]
    $sourceId = [string]$source.id
    if ([string]::IsNullOrWhiteSpace($sourceId)) {
        throw 'Fixture server returned a Media Source without an id.'
    }

    return [pscustomobject]@{
        MediaItemId = $itemId
        SourceId = $sourceId
        PositionMs = 1000
        DurationMs = 2000
        Title = [string]$item.metadata.title
        FileName = [string]$source.file_name
    }
}

function Write-SmokeMediaServerResumeFixture {
    param(
        [string]$OutputDir,
        [string]$BaseUrl,
        [string]$AccessToken,
        [string]$MediaItemId,
        [string]$SourceId,
        [long]$PositionMs,
        [long]$DurationMs
    )

    if ([string]::IsNullOrWhiteSpace($AccessToken)) {
        throw 'Fixture access token must be non-empty.'
    }
    if ([string]::IsNullOrWhiteSpace($MediaItemId)) {
        throw 'Fixture resume Media Item id is required.'
    }
    if ([string]::IsNullOrWhiteSpace($SourceId)) {
        throw 'Fixture resume Media Source id is required.'
    }
    if ($PositionMs -le 0) {
        throw 'Fixture resume position must be positive.'
    }

    $headers = @{
        Authorization = "Bearer $AccessToken"
        'Content-Type' = 'application/json'
    }
    $progressBody = @{
        source_id = $SourceId
        position_ms = $PositionMs
        duration_ms = if ($DurationMs -gt 0) { $DurationMs } else { $null }
    } | ConvertTo-Json -Depth 4 -Compress
    $unwatchBody = @{
        watched = $false
        source_id = $SourceId
        position_ms = $null
        duration_ms = if ($DurationMs -gt 0) { $DurationMs } else { $null }
    } | ConvertTo-Json -Depth 4 -Compress

    $encodedItemId = [System.Uri]::EscapeDataString($MediaItemId)
    $stateUrl = "$BaseUrl/users/me/playback-state/items/$encodedItemId"
    $watchedUrl = "$BaseUrl/users/me/playback-state/items/$encodedItemId/watched"
    $progressUrl = "$BaseUrl/users/me/playback-state/items/$encodedItemId/progress"
    $continueUrl = "$BaseUrl/users/me/playback-state/continue-watching?limit=12&offset=0"
    $initialResponse = Invoke-RestMethod -Uri $stateUrl -Headers $headers -TimeoutSec 10
    $unwatchResponse = Invoke-RestMethod -Uri $watchedUrl -Method Put -Headers $headers -Body $unwatchBody -TimeoutSec 10
    $progressResponse = Invoke-RestMethod -Uri $progressUrl -Method Put -Headers $headers -Body $progressBody -TimeoutSec 10
    $continueResponse = Invoke-RestMethod -Uri $continueUrl -Headers $headers -TimeoutSec 10
    $continueCount = @($continueResponse.items).Count
    if ($continueCount -lt 1) {
        throw "Server-backed Continue Watching did not return any rows after progress seed at '$continueUrl'."
    }

    $initialJson = $initialResponse | ConvertTo-Json -Depth 12
    $unwatchJson = $unwatchResponse | ConvertTo-Json -Depth 12
    $stateJson = $progressResponse | ConvertTo-Json -Depth 12
    $continueJson = $continueResponse | ConvertTo-Json -Depth 12
    $seedPath = Join-Path $OutputDir 'profile-with-media-server-resume.txt'
    Write-Utf8File -Path $seedPath -Content @"
Server User Playback State seed:
State URL: $stateUrl
Watched URL: $watchedUrl
Progress URL: $progressUrl
Continue Watching URL: $continueUrl
Access token: <redacted>
Resume Media Item id: $MediaItemId
Resume Media Source id: $SourceId
Resume position: $PositionMs
Resume duration: $DurationMs
Initial state response:
$initialJson
Unwatch response:
$unwatchJson
Progress response:
$stateJson
Continue Watching response:
$continueJson
"@
}

function Write-SmokeMediaServerReadbackArtifact {
    param(
        [string]$OutputDir,
        [string]$BaseUrl,
        [string]$AccessToken,
        [string]$MediaItemId,
        [int]$TimeoutSeconds = 25
    )

    if ([string]::IsNullOrWhiteSpace($AccessToken)) {
        throw 'Fixture access token must be non-empty.'
    }
    if ([string]::IsNullOrWhiteSpace($MediaItemId)) {
        throw 'Fixture readback Media Item id is required.'
    }

    $headers = @{
        Authorization = "Bearer $AccessToken"
        'Content-Type' = 'application/json'
    }
    $encodedItemId = [System.Uri]::EscapeDataString($MediaItemId)
    $stateUrl = "$BaseUrl/users/me/playback-state/items/$encodedItemId"
    $continueUrl = "$BaseUrl/users/me/playback-state/continue-watching?limit=12&offset=0"
    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    $attempt = 0
    $stateResponse = $null
    $continueResponse = $null
    while ((Get-Date) -lt $deadline) {
        $attempt += 1
        $stateResponse = Invoke-RestMethod -Uri $stateUrl -Headers $headers -TimeoutSec 10
        $continueResponse = Invoke-RestMethod -Uri $continueUrl -Headers $headers -TimeoutSec 10
        $watched = [bool]$stateResponse.state.watched
        $continueCount = @($continueResponse.items).Count
        if ($watched -and $continueCount -eq 0) {
            $stateJson = $stateResponse | ConvertTo-Json -Depth 12
            $continueJson = $continueResponse | ConvertTo-Json -Depth 12
            $readbackPath = Join-Path $OutputDir 'profile-with-media-server-readback.txt'
            Write-Utf8File -Path $readbackPath -Content @"
Server User Playback State readback:
State URL: $stateUrl
Continue Watching URL: $continueUrl
Attempts: $attempt
Expected watched state: true
Expected continue-watching rows: 0
Observed watched state: $watched
Observed continue-watching rows: $continueCount
State response:
$stateJson
Continue Watching response:
$continueJson
"@
            return $readbackPath
        }

        Start-Sleep -Milliseconds 750
    }

    $finalWatched = if ($stateResponse) { [bool]$stateResponse.state.watched } else { $false }
    $finalContinueCount = if ($continueResponse) { @($continueResponse.items).Count } else { -1 }
    throw "Server User Playback State did not settle to watched=true with an empty Continue Watching list after player exit. Last observed watched=$finalWatched, continue_rows=$finalContinueCount."
}

function Get-HttpHeaderValue {
    param(
        [object]$Headers,
        [string]$Name
    )

    if ($Headers -eq $null -or [string]::IsNullOrWhiteSpace($Name)) {
        return $null
    }

    foreach ($key in $Headers.Keys) {
        if ([string]$key -ieq $Name) {
            $value = $Headers[$key]
            if ($value -is [array]) {
                return [string]($value | Select-Object -First 1)
            }

            return [string]$value
        }
    }

    return $null
}

function Start-SmokePlaybackSessionProbe {
    param(
        [string]$BaseUrl,
        [string]$AccessToken,
        [string]$SourceId,
        [string]$RemuxQuery = 'container=mkv&video_codec=h264&audio_codec=aac&output_container=mkv'
    )

    if ([string]::IsNullOrWhiteSpace($AccessToken)) {
        throw 'Fixture access token must be non-empty.'
    }
    if ([string]::IsNullOrWhiteSpace($SourceId)) {
        throw 'Fixture playback session Media Source id is required.'
    }

    $headers = @{
        Authorization = "Bearer $AccessToken"
    }
    $encodedSourceId = [System.Uri]::EscapeDataString($SourceId)
    $sessionHeaderName = 'x-taru-playback-session-id'
    $preflightUrl = "$BaseUrl/sources/$encodedSourceId/stream/remux?$RemuxQuery"
    $startedAt = Get-Date
    $preflightResponse = Invoke-WebRequest -Uri $preflightUrl -Method Head -Headers $headers -UseBasicParsing -TimeoutSec 60
    $sessionId = Get-HttpHeaderValue -Headers $preflightResponse.Headers -Name $sessionHeaderName
    if ([string]::IsNullOrWhiteSpace($sessionId)) {
        throw "Remux preflight did not expose '$sessionHeaderName' at '$preflightUrl'."
    }

    return [pscustomobject]@{
        SourceId = $SourceId
        PreflightUrl = $preflightUrl
        SessionHeaderName = $sessionHeaderName
        SessionId = $sessionId
        StatusCode = [int]$preflightResponse.StatusCode
        CreatedAt = $startedAt.ToString('o')
    }
}

function Write-SmokePlaybackSessionReadbackArtifact {
    param(
        [string]$OutputDir,
        [string]$BaseUrl,
        [string]$AccessToken,
        [object]$SessionProbe,
        [int]$TimeoutSeconds = 25
    )

    if ([string]::IsNullOrWhiteSpace($AccessToken)) {
        throw 'Fixture access token must be non-empty.'
    }
    if ($SessionProbe -eq $null -or [string]::IsNullOrWhiteSpace([string]$SessionProbe.SessionId)) {
        throw 'Fixture playback session probe is required before readback.'
    }

    $headers = @{
        Authorization = "Bearer $AccessToken"
    }
    $encodedSessionId = [System.Uri]::EscapeDataString([string]$SessionProbe.SessionId)
    $sessionUrl = "$BaseUrl/playback/sessions/$encodedSessionId"
    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    $attempt = 0
    $sessionResponse = $null
    while ((Get-Date) -lt $deadline) {
        $attempt += 1
        $sessionResponse = Invoke-RestMethod -Uri $sessionUrl -Headers $headers -TimeoutSec 10
        $observedId = [string]$sessionResponse.session.id
        $observedKind = [string]$sessionResponse.session.kind
        $observedState = [string]$sessionResponse.session.state
        if ($observedId -eq [string]$SessionProbe.SessionId -and
            $observedKind -eq 'remux' -and
            -not [string]::IsNullOrWhiteSpace($observedState)) {
            $sessionJson = $sessionResponse | ConvertTo-Json -Depth 12
            if ($sessionJson -match 'output_path|[A-Za-z]:\\\\|file://|local://') {
                throw 'Public playback session readback contained a forbidden local path or server-only field.'
            }

            $readbackPath = Join-Path $OutputDir 'profile-with-media-session-readback.txt'
            Write-Utf8File -Path $readbackPath -Content @"
Public playback session readback:
Preflight URL: $($SessionProbe.PreflightUrl)
Preflight method: HEAD
Preflight status: $($SessionProbe.StatusCode)
Session header: $($SessionProbe.SessionHeaderName)
Access token: <redacted>
Session URL: $sessionUrl
Created before Android player exit: true
Observed after Android player exit: true
Attempts: $attempt
Expected source id: $($SessionProbe.SourceId)
Expected session id: $($SessionProbe.SessionId)
Expected kind: remux
Observed session id: $observedId
Observed kind: $observedKind
Observed state: $observedState
Session response:
$sessionJson
"@
            return $readbackPath
        }

        Start-Sleep -Milliseconds 750
    }

    $finalState = if ($sessionResponse -and $sessionResponse.session) { [string]$sessionResponse.session.state } else { 'n/a' }
    throw "Public playback session readback did not return the expected remux session before timeout. Last observed state=$finalState."
}

function Write-SmokePlaybackSessionCancelledReadbackArtifact {
    param(
        [string]$OutputDir,
        [string]$BaseUrl,
        [string]$AccessToken,
        [string]$SessionId,
        [int]$TimeoutSeconds = 45
    )

    if ([string]::IsNullOrWhiteSpace($AccessToken)) {
        throw 'Fixture access token must be non-empty.'
    }
    if ([string]::IsNullOrWhiteSpace($SessionId)) {
        throw 'Fixture playback session id is required before cancellation readback.'
    }

    $headers = @{
        Authorization = "Bearer $AccessToken"
    }
    $encodedSessionId = [System.Uri]::EscapeDataString($SessionId)
    $sessionUrl = "$BaseUrl/playback/sessions/$encodedSessionId"
    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    $attempt = 0
    $sessionResponse = $null
    while ((Get-Date) -lt $deadline) {
        $attempt += 1
        $sessionResponse = Invoke-RestMethod -Uri $sessionUrl -Headers $headers -TimeoutSec 10
        $observedId = [string]$sessionResponse.session.id
        $observedKind = [string]$sessionResponse.session.kind
        $observedState = [string]$sessionResponse.session.state
        $observedFailure = [string]$sessionResponse.session.failure_category
        if ($observedId -eq $SessionId -and
            $observedKind -eq 'remux' -and
            $observedState -eq 'cancelled' -and
            $observedFailure -eq 'cancelled') {
            $sessionJson = $sessionResponse | ConvertTo-Json -Depth 12
            if ($sessionJson -match 'output_path|[A-Za-z]:\\\\|file://|local://') {
                throw 'Public playback session cancellation readback contained a forbidden local path or server-only field.'
            }

            $readbackPath = Join-Path $OutputDir 'profile-active-remux-session-cancelled.txt'
            Write-Utf8File -Path $readbackPath -Content @"
Public active playback session cancellation readback:
Session URL: $sessionUrl
Access token: <redacted>
Observed after Android player exit: true
Attempts: $attempt
Expected session id: $SessionId
Expected kind: remux
Expected state: cancelled
Expected failure category: cancelled
Observed session id: $observedId
Observed kind: $observedKind
Observed state: $observedState
Observed failure category: $observedFailure
Session response:
$sessionJson
"@
            return $readbackPath
        }

        Start-Sleep -Milliseconds 750
    }

    $finalState = if ($sessionResponse -and $sessionResponse.session) { [string]$sessionResponse.session.state } else { 'n/a' }
    $finalFailure = if ($sessionResponse -and $sessionResponse.session) { [string]$sessionResponse.session.failure_category } else { 'n/a' }
    throw "Public playback session cancellation readback did not observe cancelled remux session before timeout. Last observed state=$finalState, failure=$finalFailure."
}

function Start-SmokeFixtureServer {
    param(
        [string]$ServerBinary,
        [string]$ConfigPath,
        [string]$OutputDir
    )

    $stdoutPath = Join-Path $OutputDir 'fixture-server.stdout.log'
    $stderrPath = Join-Path $OutputDir 'fixture-server.stderr.log'
    return Start-Process `
        -FilePath $ServerBinary `
        -ArgumentList @('--config', $ConfigPath, 'serve') `
        -PassThru `
        -WindowStyle Hidden `
        -RedirectStandardOutput $stdoutPath `
        -RedirectStandardError $stderrPath
}

function Stop-SmokeFixtureServer {
    param(
        [System.Diagnostics.Process]$Process
    )

    if ($Process -eq $null -or $Process.HasExited) {
        return
    }

    Stop-Process -Id $Process.Id -Force -ErrorAction SilentlyContinue
    $Process.WaitForExit(5000) | Out-Null
}

function Start-SmokeMediaFixtureProvider {
    param(
        [string]$ScriptDir,
        [string]$AndroidRoot,
        [string]$OutputDir,
        [int]$Port,
        [bool]$SkipServerBuild,
        [string]$FixtureRoot,
        [string]$VideoContainer = 'mp4',
        [bool]$SlowRemux = $false
    )

    $providerScript = Join-Path $ScriptDir 'Start-DemoFixtureServer.ps1'
    if (-not (Test-Path -LiteralPath $providerScript)) {
        throw "Fixture provider script was not found at '$providerScript'."
    }

    $providerArgs = @{
        PrepareOnly = $true
        Port = $Port
    }
    if ($SkipServerBuild) {
        $providerArgs.SkipBuild = $true
    }
    if (-not [string]::IsNullOrWhiteSpace($FixtureRoot)) {
        $providerArgs.FixtureRoot = $FixtureRoot
    }
    if (-not [string]::IsNullOrWhiteSpace($VideoContainer)) {
        $providerArgs.VideoContainer = $VideoContainer
    }
    if ($SlowRemux) {
        $providerArgs.SlowRemux = $true
    }

    & $providerScript @providerArgs
    if ($LASTEXITCODE -ne 0) {
        throw 'Demo fixture provider preparation failed.'
    }

    $summaryPath = if ([string]::IsNullOrWhiteSpace($FixtureRoot)) {
        Join-Path $AndroidRoot 'build\demo-fixtures\server-backed\summary.json'
    } else {
        Join-Path $FixtureRoot 'summary.json'
    }
    if (-not (Test-Path -LiteralPath $summaryPath)) {
        throw "Fixture provider summary was not found at '$summaryPath'."
    }

    $summary = Get-Content -LiteralPath $summaryPath -Raw | ConvertFrom-Json
    $process = Start-SmokeFixtureServer `
        -ServerBinary $summary.server_binary `
        -ConfigPath $summary.config `
        -OutputDir $OutputDir

    Wait-ForHttpHealth -BaseUrl $summary.base_url -Process $process

    return [pscustomobject]@{
        Summary = $summary
        Process = $process
    }
}

function Install-SmokeMediaProfileFixture {
    param(
        [string]$AdbPath,
        [string]$DeviceSerial,
        [string]$OutputDir,
        [string]$BaseUrl,
        [string]$AccessToken,
        [bool]$ForceRemux = $false
    )

    if ([string]::IsNullOrWhiteSpace($AccessToken)) {
        throw 'Fixture access token must be non-empty.'
    }

    $providerUri = 'content://dev.taru.android.smoke.fixture'
    $startedAt = [DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
    $seedOutput = $null
    $lastSeedError = $null
    for ($attempt = 1; $attempt -le 3; $attempt += 1) {
        $seedArguments = @(
            '-s', $DeviceSerial,
            'shell', 'content', 'call',
            '--uri', $providerUri,
            '--method', 'seed',
            '--arg', $BaseUrl,
            '--extra', "access_token:s:$AccessToken",
            '--extra', "checked_at_millis:l:$startedAt"
        )
        if ($ForceRemux) {
            $seedArguments += @('--extra', 'force_remux:b:true')
        }

        $output = & $AdbPath @seedArguments 2>&1
        $seedOutput = ($output | Out-String).TrimEnd()
        if ($LASTEXITCODE -eq 0 -and $seedOutput -match 'status=ok') {
            break
        }

        $lastSeedError = $seedOutput
        Start-Sleep -Milliseconds 750
    }

    if ([string]::IsNullOrWhiteSpace($seedOutput) -or $seedOutput -notmatch 'status=ok') {
        throw "Debug smoke fixture seed failed.`n$lastSeedError"
    }

    $safeOutput = (($output | Out-String) -replace [regex]::Escape($AccessToken), '<redacted>').TrimEnd()
    $safeSeedOutput = ($seedOutput -replace [regex]::Escape($AccessToken), '<redacted>').TrimEnd()
    $seedPath = Join-Path $OutputDir 'profile-with-media-seed.txt'
    Write-Utf8File -Path $seedPath -Content @"
Seed provider: $providerUri
Base URL: $BaseUrl
Display name: Smoke Server
Access token: <redacted>
Resume source: server User Playback State
Seed status:
$safeSeedOutput
ADB output:
$safeOutput
"@
}

function Wait-ForBootComplete {
    param(
        [string]$AdbPath,
        [string]$DeviceSerial
    )

    $deadline = (Get-Date).AddMinutes(2)
    while ((Get-Date) -lt $deadline) {
        $bootCompleted = (& $AdbPath -s $DeviceSerial shell getprop sys.boot_completed 2>$null).Trim()
        if ($LASTEXITCODE -eq 0 -and $bootCompleted -eq '1') {
            return
        }

        Start-Sleep -Seconds 2
    }

    throw "Device '$DeviceSerial' did not report boot completion within two minutes."
}

function Write-Utf8File {
    param(
        [string]$Path,
        [string]$Content
    )

    $utf8 = [System.Text.UTF8Encoding]::new($false)
    [System.IO.File]::WriteAllText($Path, $Content, $utf8)
}

function Convert-ToJsonPath {
    param(
        [string]$Path
    )

    if ([string]::IsNullOrWhiteSpace($Path)) {
        return $null
    }

    return $Path.Replace('\', '/')
}

function Resolve-FixtureState {
    param(
        [string]$RequestedFixtureState,
        [bool]$RequestedResetAppData
    )

    if ($RequestedResetAppData) {
        if ($RequestedFixtureState -eq 'current-state') {
            return 'empty-setup'
        }

        if ($RequestedFixtureState -notin @('empty-setup', 'profile-with-media', 'profile-active-remux')) {
            throw '-ResetAppData can only be combined with the default fixture state, -FixtureState empty-setup, -FixtureState profile-with-media, or -FixtureState profile-active-remux.'
        }
    }

    return $RequestedFixtureState
}

function Wake-Device {
    param(
        [string]$AdbPath,
        [string]$DeviceSerial
    )

    Invoke-Adb -AdbPath $AdbPath -Arguments @('-s', $DeviceSerial, 'shell', 'input', 'keyevent', 'KEYCODE_WAKEUP') -FailureMessage 'adb wake failed.'
    Invoke-Adb -AdbPath $AdbPath -Arguments @('-s', $DeviceSerial, 'shell', 'wm', 'dismiss-keyguard') -FailureMessage 'adb dismiss keyguard failed.'
}

function Wait-ForFocusedAppWindow {
    param(
        [string]$AdbPath,
        [string]$DeviceSerial,
        [string]$PackageName = 'dev.taru.android',
        [int]$TimeoutSeconds = 10
    )

    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    while ((Get-Date) -lt $deadline) {
        $windowDump = & $AdbPath -s $DeviceSerial shell dumpsys window 2>$null
        if ($LASTEXITCODE -eq 0) {
            $text = $windowDump | Out-String
            if ($text -match "mCurrentFocus=Window\{[^}]*$([regex]::Escape($PackageName))/" -or
                $text -match "mFocusedWindow=Window\{[^}]*$([regex]::Escape($PackageName))/") {
                return $true
            }
        }

        Start-Sleep -Milliseconds 500
    }

    return $false
}

function Recover-AppFocus {
    param(
        [string]$AdbPath,
        [string]$DeviceSerial
    )

    Wake-Device -AdbPath $AdbPath -DeviceSerial $DeviceSerial
    & $AdbPath -s $DeviceSerial shell am start -n 'dev.taru.android/.MainActivity' -a android.intent.action.MAIN -c android.intent.category.LAUNCHER *> $null
    Start-Sleep -Seconds 2
}

function Install-SmokeProfileFixture {
    param(
        [string]$AdbPath,
        [string]$DeviceSerial,
        [string]$OutputDir
    )

    $snapshotJson = '{"profiles":[{"id":"server-1","displayName":"Smoke Server","baseUrl":"https://smoke.invalid","tokenReference":"server-token:server-1","lastObservedApiVersion":"v1","lastSuccessfulConnectionAtMillis":null,"lastPublicError":null}],"activeProfileId":"server-1"}'
    $escapedSnapshot = [System.Security.SecurityElement]::Escape($snapshotJson)
    $xml = @"
<?xml version='1.0' encoding='utf-8' standalone='yes' ?>
<map>
    <string name="snapshot">$escapedSnapshot</string>
</map>
"@

    $localSeedPath = Join-Path ([System.IO.Path]::GetTempPath()) "taru-smoke-profile-$([System.Guid]::NewGuid()).xml"
    $remoteSeedPath = '/data/local/tmp/taru-smoke-profile.xml'
    try {
        Write-Utf8File -Path $localSeedPath -Content $xml

        Invoke-Adb -AdbPath $AdbPath -Arguments @('-s', $DeviceSerial, 'push', $localSeedPath, $remoteSeedPath) -FailureMessage 'adb push profile fixture failed.'
        Invoke-Adb -AdbPath $AdbPath -Arguments @('-s', $DeviceSerial, 'shell', 'chmod', '644', $remoteSeedPath) -FailureMessage 'adb chmod profile fixture failed.'
        Invoke-Adb -AdbPath $AdbPath -Arguments @('-s', $DeviceSerial, 'shell', 'run-as', 'dev.taru.android', 'mkdir', '-p', 'shared_prefs') -FailureMessage 'adb create shared_prefs failed.'
        Invoke-Adb -AdbPath $AdbPath -Arguments @('-s', $DeviceSerial, 'shell', 'run-as', 'dev.taru.android', 'cp', $remoteSeedPath, 'shared_prefs/taru_server_profiles.xml') -FailureMessage 'adb seed server profile failed.'
        Invoke-Adb -AdbPath $AdbPath -Arguments @('-s', $DeviceSerial, 'shell', 'run-as', 'dev.taru.android', 'chmod', '660', 'shared_prefs/taru_server_profiles.xml') -FailureMessage 'adb chmod server profile preferences failed.'
        Invoke-Adb -AdbPath $AdbPath -Arguments @('-s', $DeviceSerial, 'shell', 'rm', $remoteSeedPath) -FailureMessage 'adb cleanup profile fixture failed.'
    } finally {
        & $AdbPath -s $DeviceSerial shell rm $remoteSeedPath *> $null
        Remove-Item -LiteralPath $localSeedPath -Force -ErrorAction SilentlyContinue
    }
}

function Get-UiDump {
    param(
        [string]$AdbPath,
        [string]$DeviceSerial,
        [string]$OutputDir,
        [string]$Name
    )

    $safeName = $Name -replace '[^A-Za-z0-9_.-]', '-'
    $remoteDump = "/data/local/tmp/taru-android-smoke-$safeName.xml"
    $localDump = Join-Path $OutputDir "$safeName.uiautomator.xml"
    $lastError = $null

    for ($attempt = 1; $attempt -le 8; $attempt += 1) {
        try {
            if (-not (Wait-ForFocusedAppWindow -AdbPath $AdbPath -DeviceSerial $DeviceSerial -TimeoutSeconds 8)) {
                Recover-AppFocus -AdbPath $AdbPath -DeviceSerial $DeviceSerial
            }

            & $AdbPath -s $DeviceSerial shell rm $remoteDump *> $null
            & $AdbPath -s $DeviceSerial shell pkill -f uiautomator *> $null
            Start-Sleep -Milliseconds 500
            $dumpOutput = & $AdbPath -s $DeviceSerial shell uiautomator dump $remoteDump 2>&1
            if ($LASTEXITCODE -ne 0) {
                throw "adb UI dump failed for '$Name'.`n$($dumpOutput | Out-String)"
            }

            $remoteReady = $false
            for ($fileAttempt = 1; $fileAttempt -le 10; $fileAttempt += 1) {
                & $AdbPath -s $DeviceSerial shell ls $remoteDump *> $null
                if ($LASTEXITCODE -eq 0) {
                    $remoteReady = $true
                    break
                }

                Start-Sleep -Milliseconds 250
            }

            if (-not $remoteReady) {
                throw "UI dump '$Name' did not create remote hierarchy file '$remoteDump'. Dump output: $($dumpOutput | Out-String)"
            }

            Invoke-Adb -AdbPath $AdbPath -Arguments @('-s', $DeviceSerial, 'pull', $remoteDump, $localDump) -FailureMessage "adb pull UI dump failed for '$Name'."
            Invoke-Adb -AdbPath $AdbPath -Arguments @('-s', $DeviceSerial, 'shell', 'rm', $remoteDump) -FailureMessage "adb cleanup UI dump failed for '$Name'."
            return $localDump
        } catch {
            $lastError = $_.Exception.Message
            & $AdbPath -s $DeviceSerial shell pkill -f uiautomator *> $null
            Recover-AppFocus -AdbPath $AdbPath -DeviceSerial $DeviceSerial
            Start-Sleep -Milliseconds 1500
        }
    }

    throw "Could not capture UI hierarchy for '$Name'. Last error: $lastError"
}

function Get-UiTextValues {
    param(
        [xml]$Hierarchy
    )

    $values = New-Object System.Collections.Generic.List[string]
    foreach ($node in @($Hierarchy.SelectNodes('//node'))) {
        foreach ($attributeName in @('text', 'content-desc')) {
            $attribute = $node.Attributes[$attributeName]
            if ($attribute -ne $null -and -not [string]::IsNullOrWhiteSpace($attribute.Value)) {
                $values.Add($attribute.Value)
            }
        }
    }
    return $values.ToArray()
}

function Get-SmokePlaybackSessionIdFromUi {
    param(
        [string]$AdbPath,
        [string]$DeviceSerial,
        [string]$OutputDir,
        [string]$Name
    )

    $dumpPath = Get-UiDump -AdbPath $AdbPath -DeviceSerial $DeviceSerial -OutputDir $OutputDir -Name $Name
    [xml]$hierarchy = Get-Content -LiteralPath $dumpPath -Raw
    $values = Get-UiTextValues -Hierarchy $hierarchy
    $sessionId = $values |
        ForEach-Object {
            if ($_ -match '(?<session_id>[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})') {
                $Matches.session_id
            }
        } |
        Select-Object -First 1
    if ([string]::IsNullOrWhiteSpace($sessionId)) {
        throw "Could not find a playback session id in UI hierarchy '$Name'."
    }

    return [string]$sessionId
}

function Find-UiNode {
    param(
        [xml]$Hierarchy,
        [string]$Text
    )

    foreach ($node in @($Hierarchy.SelectNodes('//node'))) {
        foreach ($attributeName in @('text', 'content-desc')) {
            $attribute = $node.Attributes[$attributeName]
            if ($attribute -ne $null -and $attribute.Value -eq $Text) {
                return $node
            }
        }
    }

    return $null
}

function Dismiss-SystemAnrDialog {
    param(
        [string]$AdbPath,
        [string]$DeviceSerial,
        [xml]$Hierarchy
    )

    $values = Get-UiTextValues -Hierarchy $Hierarchy
    if ($values -notcontains "Process system isn't responding") {
        return $false
    }

    $waitNode = Find-UiNode -Hierarchy $Hierarchy -Text 'Wait'
    if ($waitNode -eq $null) {
        return $false
    }

    $bounds = $waitNode.Attributes['bounds'].Value
    $center = Get-BoundsCenter -Bounds $bounds
    Invoke-Adb -AdbPath $AdbPath -Arguments @('-s', $DeviceSerial, 'shell', 'input', 'tap', $center.X, $center.Y) -FailureMessage 'adb tap failed for system ANR wait button.'
    Start-Sleep -Seconds 2
    return $true
}

function Get-BoundsCenter {
    param(
        [string]$Bounds
    )

    if ($Bounds -notmatch '^\[(?<x1>\d+),(?<y1>\d+)\]\[(?<x2>\d+),(?<y2>\d+)\]$') {
        throw "Cannot parse UI bounds '$Bounds'."
    }

    $x = [Math]::Floor(([int]$Matches['x1'] + [int]$Matches['x2']) / 2)
    $y = [Math]::Floor(([int]$Matches['y1'] + [int]$Matches['y2']) / 2)
    return [pscustomobject]@{
        X = $x
        Y = $y
    }
}

function Wait-ForUiText {
    param(
        [string]$AdbPath,
        [string]$DeviceSerial,
        [string]$OutputDir,
        [string]$Text,
        [int]$TimeoutSeconds = 25
    )

    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    $lastError = $null
    while ((Get-Date) -lt $deadline) {
        try {
            $dumpPath = Get-UiDump -AdbPath $AdbPath -DeviceSerial $DeviceSerial -OutputDir $OutputDir -Name 'wait'
            [xml]$hierarchy = Get-Content -LiteralPath $dumpPath -Raw
            $values = Get-UiTextValues -Hierarchy $hierarchy
            if ($values -contains $Text) {
                return
            }

            if (Dismiss-SystemAnrDialog -AdbPath $AdbPath -DeviceSerial $DeviceSerial -Hierarchy $hierarchy) {
                Recover-AppFocus -AdbPath $AdbPath -DeviceSerial $DeviceSerial
            }
        } catch {
            $lastError = $_.Exception.Message
        }

        Start-Sleep -Milliseconds 750
    }

    if ([string]::IsNullOrWhiteSpace($lastError)) {
        throw "Timed out waiting for UI text '$Text'."
    }

    throw "Timed out waiting for UI text '$Text'. Last UI dump error: $lastError"
}

function Wait-ForAnyUiText {
    param(
        [string]$AdbPath,
        [string]$DeviceSerial,
        [string]$OutputDir,
        [string[]]$Text,
        [int]$TimeoutSeconds = 25
    )

    if ($Text.Count -eq 0) {
        throw 'At least one UI text value is required.'
    }

    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    $lastError = $null
    while ((Get-Date) -lt $deadline) {
        try {
            $dumpPath = Get-UiDump -AdbPath $AdbPath -DeviceSerial $DeviceSerial -OutputDir $OutputDir -Name 'wait-any'
            [xml]$hierarchy = Get-Content -LiteralPath $dumpPath -Raw
            $values = Get-UiTextValues -Hierarchy $hierarchy
            foreach ($candidate in $Text) {
                if ($values -contains $candidate) {
                    return $candidate
                }
            }

            if (Dismiss-SystemAnrDialog -AdbPath $AdbPath -DeviceSerial $DeviceSerial -Hierarchy $hierarchy) {
                Recover-AppFocus -AdbPath $AdbPath -DeviceSerial $DeviceSerial
            }
        } catch {
            $lastError = $_.Exception.Message
        }

        Start-Sleep -Milliseconds 750
    }

    $expected = $Text -join ', '
    if ([string]::IsNullOrWhiteSpace($lastError)) {
        throw "Timed out waiting for any UI text: $expected."
    }

    throw "Timed out waiting for any UI text: $expected. Last UI dump error: $lastError"
}

function Test-UiText {
    param(
        [string]$AdbPath,
        [string]$DeviceSerial,
        [string]$OutputDir,
        [string]$Text,
        [string]$DumpName
    )

    $dumpPath = Get-UiDump -AdbPath $AdbPath -DeviceSerial $DeviceSerial -OutputDir $OutputDir -Name $DumpName
    [xml]$hierarchy = Get-Content -LiteralPath $dumpPath -Raw
    $values = Get-UiTextValues -Hierarchy $hierarchy
    return $values -contains $Text
}

function Swipe-Up {
    param(
        [string]$AdbPath,
        [string]$DeviceSerial
    )

    Invoke-Adb -AdbPath $AdbPath -Arguments @('-s', $DeviceSerial, 'shell', 'input', 'swipe', '540', '1500', '540', '520', '450') -FailureMessage 'adb swipe up failed.'
}

function Swipe-UntilUiText {
    param(
        [string]$AdbPath,
        [string]$DeviceSerial,
        [string]$OutputDir,
        [string]$Text,
        [int]$MaxSwipes = 4
    )

    for ($attempt = 0; $attempt -le $MaxSwipes; $attempt += 1) {
        if (Test-UiText -AdbPath $AdbPath -DeviceSerial $DeviceSerial -OutputDir $OutputDir -Text $Text -DumpName "swipe-$Text-$attempt") {
            return
        }

        if ($attempt -lt $MaxSwipes) {
            Swipe-Up -AdbPath $AdbPath -DeviceSerial $DeviceSerial
            Start-Sleep -Milliseconds 750
        }
    }

    throw "Could not find UI text '$Text' after $MaxSwipes swipe(s)."
}

function Tap-UiText {
    param(
        [string]$AdbPath,
        [string]$DeviceSerial,
        [string]$OutputDir,
        [string]$Text
    )

    $dumpPath = Get-UiDump -AdbPath $AdbPath -DeviceSerial $DeviceSerial -OutputDir $OutputDir -Name "tap-$Text"
    [xml]$hierarchy = Get-Content -LiteralPath $dumpPath -Raw
    $node = Find-UiNode -Hierarchy $hierarchy -Text $Text
    if ($node -eq $null) {
        throw "Cannot tap UI text '$Text' because it was not found."
    }

    $bounds = $node.Attributes['bounds'].Value
    $center = Get-BoundsCenter -Bounds $bounds
    Invoke-Adb -AdbPath $AdbPath -Arguments @('-s', $DeviceSerial, 'shell', 'input', 'tap', $center.X, $center.Y) -FailureMessage "adb tap failed for '$Text'."
}

function Open-SmokeMediaDetail {
    param(
        [string]$AdbPath,
        [string]$DeviceSerial,
        [string]$OutputDir
    )

    Wait-ForUiText -AdbPath $AdbPath -DeviceSerial $DeviceSerial -OutputDir $OutputDir -Text 'Night Harbor' -TimeoutSeconds 35
    if (-not (Test-UiText -AdbPath $AdbPath -DeviceSerial $DeviceSerial -OutputDir $OutputDir -Text 'Open detail' -DumpName 'open-detail-check')) {
        Invoke-Adb -AdbPath $AdbPath -Arguments @('-s', $DeviceSerial, 'shell', 'input', 'keyevent', 'KEYCODE_BACK') -FailureMessage 'adb back failed while returning to home.'
        Wait-ForUiText -AdbPath $AdbPath -DeviceSerial $DeviceSerial -OutputDir $OutputDir -Text 'Open detail' -TimeoutSeconds 25
    }

    Tap-UiText -AdbPath $AdbPath -DeviceSerial $DeviceSerial -OutputDir $OutputDir -Text 'Open detail'
    Wait-ForUiText -AdbPath $AdbPath -DeviceSerial $DeviceSerial -OutputDir $OutputDir -Text 'Check source' -TimeoutSeconds 25
}

function Return-ToSmokeMediaDetail {
    param(
        [string]$AdbPath,
        [string]$DeviceSerial,
        [string]$OutputDir
    )

    Tap-UiText -AdbPath $AdbPath -DeviceSerial $DeviceSerial -OutputDir $OutputDir -Text 'Back'
    Wait-ForUiText -AdbPath $AdbPath -DeviceSerial $DeviceSerial -OutputDir $OutputDir -Text 'Night Harbor' -TimeoutSeconds 25
    Wait-ForUiText -AdbPath $AdbPath -DeviceSerial $DeviceSerial -OutputDir $OutputDir -Text 'Check source' -TimeoutSeconds 25
}

function Assert-SmokeFacetRoute {
    param(
        [string]$AdbPath,
        [string]$DeviceSerial,
        [string]$OutputDir,
        [string]$TapText,
        [string]$FacetLabel,
        [string]$FamilyLabel,
        [string]$Name,
        [string[]]$AdditionalRequiredText = @()
    )

    Swipe-UntilUiText -AdbPath $AdbPath -DeviceSerial $DeviceSerial -OutputDir $OutputDir -Text $TapText -MaxSwipes 6
    Tap-UiText -AdbPath $AdbPath -DeviceSerial $DeviceSerial -OutputDir $OutputDir -Text $TapText
    Wait-ForUiText -AdbPath $AdbPath -DeviceSerial $DeviceSerial -OutputDir $OutputDir -Text 'Related Media Items' -TimeoutSeconds 25
    $requiredText = @(
        $FacetLabel,
        $FamilyLabel,
        'API backed',
        '1 results',
        'Related Media Items',
        'Night Harbor'
    ) + $AdditionalRequiredText
    return Capture-SmokeSurface -AdbPath $AdbPath -DeviceSerial $DeviceSerial -OutputDir $OutputDir -Name $Name -RequiredText $requiredText
}

function Capture-SmokeSurface {
    param(
        [string]$AdbPath,
        [string]$DeviceSerial,
        [string]$OutputDir,
        [string]$Name,
        [string[]]$RequiredText = @(),
        [string[]]$ForbiddenText = @()
    )

    $remoteShot = "/sdcard/taru-android-smoke-$Name.png"
    $localShot = Join-Path $OutputDir "$Name.png"
    Invoke-Adb -AdbPath $AdbPath -Arguments @('-s', $DeviceSerial, 'shell', 'screencap', '-p', $remoteShot) -FailureMessage "adb screencap failed for '$Name'."
    Invoke-Adb -AdbPath $AdbPath -Arguments @('-s', $DeviceSerial, 'pull', $remoteShot, $localShot) -FailureMessage "adb pull screenshot failed for '$Name'."
    Invoke-Adb -AdbPath $AdbPath -Arguments @('-s', $DeviceSerial, 'shell', 'rm', $remoteShot) -FailureMessage "adb cleanup of remote screenshot failed for '$Name'."

    $dumpPath = Get-UiDump -AdbPath $AdbPath -DeviceSerial $DeviceSerial -OutputDir $OutputDir -Name $Name
    [xml]$hierarchy = Get-Content -LiteralPath $dumpPath -Raw
    $values = Get-UiTextValues -Hierarchy $hierarchy
    $criteriaPath = Join-Path $OutputDir "$Name.criteria.txt"
    $criteriaLines = New-Object System.Collections.Generic.List[string]
    $criteriaLines.Add("Surface: $Name")
    $criteriaLines.Add("Screenshot: $(Split-Path -Leaf $localShot)")
    $criteriaLines.Add("UI hierarchy: $(Split-Path -Leaf $dumpPath)")

    $missing = @()
    $unexpected = @()
    if ($RequiredText.Count -eq 0) {
        $criteriaLines.Add('Required text/content descriptions: none')
    } else {
        $criteriaLines.Add('Required text/content descriptions:')
        foreach ($text in $RequiredText) {
            $present = $values -contains $text
            $criteriaLines.Add("- $text : $(if ($present) { 'PASS' } else { 'MISSING' })")
            if (-not $present) {
                $missing += $text
            }
        }
    }

    if ($ForbiddenText.Count -eq 0) {
        $criteriaLines.Add('Forbidden text/content description fragments: none')
    } else {
        $criteriaLines.Add('Forbidden text/content description fragments:')
        foreach ($text in $ForbiddenText) {
            $present = $false
            foreach ($value in $values) {
                if ($value.IndexOf($text, [System.StringComparison]::OrdinalIgnoreCase) -ge 0) {
                    $present = $true
                    break
                }
            }
            $criteriaLines.Add("- $text : $(if ($present) { 'UNEXPECTED' } else { 'PASS' })")
            if ($present) {
                $unexpected += $text
            }
        }
    }

    $criteriaLines.Add("Result: $(if ($missing.Count -eq 0 -and $unexpected.Count -eq 0) { 'PASS' } else { 'FAIL' })")
    $criteriaLines | Out-File -LiteralPath $criteriaPath -Encoding utf8

    if ($missing.Count -gt 0) {
        throw "Surface '$Name' missing expected UI text/content descriptions: $($missing -join ', ')."
    }
    if ($unexpected.Count -gt 0) {
        throw "Surface '$Name' contained forbidden UI text/content description fragments: $($unexpected -join ', ')."
    }

    return [pscustomobject]@{
        Name = $Name
        Screenshot = Split-Path -Leaf $localShot
        Hierarchy = Split-Path -Leaf $dumpPath
        Criteria = Split-Path -Leaf $criteriaPath
    }
}

$scriptDir = $PSScriptRoot
$androidRoot = (Resolve-Path -LiteralPath (Join-Path $scriptDir '..')).Path
$repoRoot = Split-Path -Parent (Split-Path -Parent $androidRoot)
$gradlew = Join-Path $androidRoot 'gradlew.bat'
$adb = Resolve-AdbPath
$deviceSerial = Get-ConnectedDeviceSerial -AdbPath $adb -RequestedSerial $Serial
$stateMode = Resolve-FixtureState -RequestedFixtureState $FixtureState -RequestedResetAppData ([bool]$ResetAppData)
$clearsAppData = $stateMode -in @('empty-setup', 'profile-missing-token', 'profile-with-media', 'profile-active-remux')
$skipAndroidBuild = [bool]($SkipBuild -or $SkipAppBuild)
$skipServerBuild = [bool]($SkipBuild -or $SkipFixtureServerBuild)

if ([string]::IsNullOrWhiteSpace($OutputRoot)) {
    $OutputRoot = Join-Path $androidRoot 'build\smoke'
}

$timestamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$outputDir = Join-Path $OutputRoot "$timestamp-$stateMode-$deviceSerial"
New-Item -ItemType Directory -Force -Path $outputDir | Out-Null
$fixtureServerProcess = $null
$fixtureReversePort = $null
$fixtureBaseUrl = $null
$playbackSessionProbe = $null
$playbackSessionReadbackPath = $null
$playbackSessionCancellationReadbackPath = $null

try {

if (-not $skipAndroidBuild) {
    Push-Location $androidRoot
    try {
        & $gradlew :app:assembleDebug --no-daemon
        if ($LASTEXITCODE -ne 0) {
            throw 'Gradle assembleDebug failed.'
        }
    } finally {
        Pop-Location
    }
}

$apkPath = Join-Path $androidRoot 'app\build\outputs\apk\debug\app-debug.apk'
if (-not (Test-Path -LiteralPath $apkPath)) {
    throw "APK not found at $apkPath"
}

Invoke-Adb -AdbPath $adb -Arguments @('-s', $deviceSerial, 'wait-for-device') -FailureMessage 'adb wait-for-device failed.'
Wait-ForBootComplete -AdbPath $adb -DeviceSerial $deviceSerial
Invoke-Adb -AdbPath $adb -Arguments @('-s', $deviceSerial, 'install', '-r', '-d', $apkPath) -FailureMessage 'adb install failed.'
if ($clearsAppData) {
    Invoke-Adb -AdbPath $adb -Arguments @('-s', $deviceSerial, 'shell', 'pm', 'clear', 'dev.taru.android') -FailureMessage 'adb app data reset failed.'
}
if ($stateMode -in @('profile-with-media', 'profile-active-remux')) {
    $isActiveRemuxSmoke = $stateMode -eq 'profile-active-remux'
    $fixtureRoot = if ($isActiveRemuxSmoke) {
        Join-Path $outputDir 'demo-fixture'
    } else {
        Join-Path $androidRoot 'build\demo-fixtures\server-backed'
    }
    $fixtureProvider = Start-SmokeMediaFixtureProvider `
        -ScriptDir $scriptDir `
        -AndroidRoot $androidRoot `
        -OutputDir $outputDir `
        -Port $FixtureServerPort `
        -SkipServerBuild $skipServerBuild `
        -FixtureRoot $fixtureRoot `
        -VideoContainer $(if ($isActiveRemuxSmoke) { 'mkv' } else { 'mp4' }) `
        -SlowRemux $isActiveRemuxSmoke
    $fixtureServerProcess = $fixtureProvider.Process
    $fixtureBaseUrl = $fixtureProvider.Summary.base_url
    $resumeFixture = Resolve-SmokeMediaResumeFixture `
        -BaseUrl $fixtureBaseUrl `
        -AccessToken $FixtureAccessToken
    Write-SmokeMediaServerResumeFixture `
        -OutputDir $outputDir `
        -BaseUrl $fixtureBaseUrl `
        -AccessToken $FixtureAccessToken `
        -MediaItemId $resumeFixture.MediaItemId `
        -SourceId $resumeFixture.SourceId `
        -PositionMs $resumeFixture.PositionMs `
        -DurationMs $resumeFixture.DurationMs
    Invoke-Adb -AdbPath $adb -Arguments @('-s', $deviceSerial, 'reverse', "tcp:$FixtureServerPort", "tcp:$FixtureServerPort") -FailureMessage 'adb reverse failed for profile-with-media.'
    $fixtureReversePort = $FixtureServerPort
    Install-SmokeMediaProfileFixture `
        -AdbPath $adb `
        -DeviceSerial $deviceSerial `
        -OutputDir $outputDir `
        -BaseUrl $fixtureBaseUrl `
        -AccessToken $FixtureAccessToken `
        -ForceRemux $isActiveRemuxSmoke
    Invoke-Adb -AdbPath $adb -Arguments @('-s', $deviceSerial, 'shell', 'am', 'force-stop', 'dev.taru.android') -FailureMessage 'adb force-stop after profile-with-media seed failed.'
}
if ($stateMode -eq 'profile-missing-token') {
    Install-SmokeProfileFixture -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir
}
if ($stateMode -notin @('profile-with-media', 'profile-active-remux')) {
    Invoke-Adb -AdbPath $adb -Arguments @('-s', $deviceSerial, 'shell', 'am', 'force-stop', 'dev.taru.android') -FailureMessage 'adb force-stop failed.'
}
Wake-Device -AdbPath $adb -DeviceSerial $deviceSerial

$launchPath = Join-Path $outputDir 'launch.txt'
$launchOutput = & $adb -s $deviceSerial shell am start -n 'dev.taru.android/.MainActivity' -a android.intent.action.MAIN -c android.intent.category.LAUNCHER 2>&1
$launchText = ($launchOutput | Out-String).TrimEnd()
Write-Utf8File -Path $launchPath -Content $launchText
if ($LASTEXITCODE -ne 0) {
    throw 'adb am start failed.'
}

Start-Sleep -Seconds 6

$surfaceEvidence = @()
if ($stateMode -eq 'empty-setup') {
    Wait-ForUiText -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Text 'Connect to a server'
    $surfaceEvidence += Capture-SmokeSurface -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Name 'setup' -RequiredText @(
        'Taru',
        'Connect to a server',
        'Display name',
        'Server URL',
        'Access Token',
        'Server profiles',
        'No saved server profiles.'
    )
} elseif ($stateMode -eq 'profile-missing-token') {
    Wait-ForUiText -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Text 'Authentication required'
    $surfaceEvidence += Capture-SmokeSurface -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Name 'home' -RequiredText @(
        'Smoke Server',
        'Your Taru library',
        'Authentication required',
        'Re-authenticate this server before browsing.',
        'Settings'
    )

    Tap-UiText -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Text 'Settings'
    Wait-ForUiText -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Text 'Client identity, playback defaults, and safe diagnostics.'
    $surfaceEvidence += Capture-SmokeSurface -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Name 'settings' -RequiredText @(
        'Settings',
        'Client identity, playback defaults, and safe diagnostics.',
        'Smoke Server',
        'Server profile',
        'Account Access',
        'Playback'
    )

    Tap-UiText -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Text 'Profile'
    Wait-ForUiText -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Text 'Server Profile'
    $surfaceEvidence += Capture-SmokeSurface -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Name 'server-profile' -RequiredText @(
        'Server Profile',
        'Access, connection, and profile switching.',
        'Smoke Server',
        'Server Access Token',
        'Token reference is stored locally; token value is never shown.'
    )

    Tap-UiText -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Text 'Back'
    Wait-ForUiText -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Text 'Client identity, playback defaults, and safe diagnostics.'
    $surfaceEvidence += Capture-SmokeSurface -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Name 'settings-after-profile-back' -RequiredText @(
        'Settings',
        'Client identity, playback defaults, and safe diagnostics.',
        'Account Access',
        'Playback',
        'Server profile'
    )
} elseif ($stateMode -in @('profile-with-media', 'profile-active-remux')) {
    Wait-ForUiText -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Text 'Night Harbor' -TimeoutSeconds 35
    $surfaceEvidence += Capture-SmokeSurface -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Name 'home' -RequiredText @(
        'Smoke Server',
        'Night Harbor',
        'Continue Watching',
        'Media Libraries',
        '1 visible',
        'Open detail'
    )

    Open-SmokeMediaDetail -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir
    $surfaceEvidence += Capture-SmokeSurface -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Name 'detail' -RequiredText @(
        'Night Harbor',
        'Resume',
        'Check source',
        'Needs check'
    )

    Swipe-UntilUiText -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Text 'Mystery' -MaxSwipes 6
    $surfaceEvidence += Capture-SmokeSurface -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Name 'detail-metadata' -RequiredText @(
        'Metadata',
        'Mystery',
        'Lighthouse',
        '2026',
        'unknown'
    )
    $surfaceEvidence += Assert-SmokeFacetRoute -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -TapText 'Mystery' -FacetLabel 'Mystery' -FamilyLabel 'Genre' -Name 'facet-genre'
    Return-ToSmokeMediaDetail -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir
    $surfaceEvidence += Capture-SmokeSurface -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Name 'detail-after-facet-back' -RequiredText @(
        'Night Harbor',
        'Resume',
        'Check source',
        'Needs check'
    )

    Swipe-UntilUiText -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Text 'Lighthouse' -MaxSwipes 6
    $surfaceEvidence += Assert-SmokeFacetRoute -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -TapText 'Lighthouse' -FacetLabel 'Lighthouse' -FamilyLabel 'Tag' -Name 'facet-tag'
    Return-ToSmokeMediaDetail -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir

    Swipe-UntilUiText -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Text 'Cast & Crew' -MaxSwipes 7
    $surfaceEvidence += Capture-SmokeSurface -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Name 'detail-cast-crew' -RequiredText @(
        'Cast & Crew',
        'Actor / as Keeper',
        'Open related Media Items from this person.'
    )
    $surfaceEvidence += Assert-SmokeFacetRoute -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -TapText 'Actor / as Keeper' -FacetLabel 'Actor / as Keeper' -FamilyLabel 'Person' -Name 'facet-person' -AdditionalRequiredText @(
        'Mira Vale'
    )
    Return-ToSmokeMediaDetail -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir

    Swipe-UntilUiText -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Text 'Resume from server state'
    $surfaceEvidence += Capture-SmokeSurface -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Name 'source-picker-server-resume' -RequiredText @(
        'Source / Version',
        $(if ($stateMode -eq 'profile-active-remux') { 'Night Harbor.mkv' } else { 'Night Harbor.mp4' }),
        'Resume from server state',
        'Taru will use authoritative User Playback State after checking the selected source.',
        'Resume'
    ) -ForbiddenText @(
        'Resume on this device',
        'Local resume'
    )
    Tap-UiText -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Text 'Resume'
    Wait-ForUiText -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Text $(if ($stateMode -eq 'profile-active-remux') { 'Remux' } else { 'Direct' }) -TimeoutSeconds 25
    Swipe-UntilUiText -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Text 'Source / Version'
    Swipe-UntilUiText -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Text 'Start resume'
    $surfaceEvidence += Capture-SmokeSurface -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Name 'source-picker' -RequiredText @(
        'Source / Version',
        $(if ($stateMode -eq 'profile-active-remux') { 'Night Harbor.mkv' } else { 'Night Harbor.mp4' }),
        $(if ($stateMode -eq 'profile-active-remux') { 'Remux route prepared' } else { 'Direct route prepared' }),
        'Start resume'
    ) -ForbiddenText @(
        'Resume on this device',
        'Local resume'
    )

    Tap-UiText -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Text 'Start resume'
    $activeRemuxSessionId = $null
    if ($stateMode -eq 'profile-with-media') {
        Wait-ForUiText -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Text 'Tracks and subtitles use Media3 controls in this version.' -TimeoutSeconds 25
        $playbackSessionProbe = Start-SmokePlaybackSessionProbe `
            -BaseUrl $fixtureBaseUrl `
            -AccessToken $FixtureAccessToken `
            -SourceId $resumeFixture.SourceId
        Wait-ForUiText -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Text 'Ended' -TimeoutSeconds 25
    } else {
        $activeRemuxSessionProbe = Start-SmokePlaybackSessionProbe `
            -BaseUrl $fixtureBaseUrl `
            -AccessToken $FixtureAccessToken `
            -SourceId $resumeFixture.SourceId `
            -RemuxQuery 'direct_play=true&container=mp4&video_codec=h264&audio_codec=aac&output_container=mp4'
        $activeRemuxSessionId = [string]$activeRemuxSessionProbe.SessionId
        Wait-ForUiText -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Text 'Remux' -TimeoutSeconds 25
        Start-Sleep -Seconds 2
    }
    if ($stateMode -eq 'profile-with-media') {
        $playerRequiredText = @(
            'Night Harbor',
            'Direct',
            'Server resume 0:01',
            '00:02'
        )
        $playerRequiredText += 'Ended'
        $playerRequiredText += 'Tracks and subtitles use Media3 controls in this version.'
    } else {
        $playerRequiredText = @(
            'Night Harbor',
            'Remux',
            'Back'
        )
    }
    $surfaceEvidence += Capture-SmokeSurface -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Name 'player' -RequiredText $playerRequiredText -ForbiddenText @(
        'Local resume'
    )

    Tap-UiText -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Text 'Back'
    Wait-ForUiText -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Text 'Check source' -TimeoutSeconds 25
    if ($stateMode -eq 'profile-with-media') {
        $playbackSessionReadbackPath = Write-SmokePlaybackSessionReadbackArtifact `
            -OutputDir $outputDir `
            -BaseUrl $fixtureBaseUrl `
            -AccessToken $FixtureAccessToken `
            -SessionProbe $playbackSessionProbe
        $serverReadbackPath = Write-SmokeMediaServerReadbackArtifact -OutputDir $outputDir -BaseUrl $fixtureBaseUrl -AccessToken $FixtureAccessToken -MediaItemId $resumeFixture.MediaItemId
    } else {
        $playbackSessionCancellationReadbackPath = Write-SmokePlaybackSessionCancelledReadbackArtifact `
            -OutputDir $outputDir `
            -BaseUrl $fixtureBaseUrl `
            -AccessToken $FixtureAccessToken `
            -SessionId $activeRemuxSessionId
    }
    $surfaceEvidence += Capture-SmokeSurface -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Name 'detail-after-player-back' -RequiredText @(
        'Night Harbor',
        'Resume',
        'Check source',
        'Needs check'
    ) -ForbiddenText @(
        'Resume on this device',
        'Local resume'
    )
} else {
    $surfaceEvidence += Capture-SmokeSurface -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Name 'launch'
}

$surfaceReport = if ($surfaceEvidence.Count -eq 0) {
    '- none'
} else {
    ($surfaceEvidence | ForEach-Object {
        "  - $($_.Name): screenshot=$($_.Screenshot), hierarchy=$($_.Hierarchy), criteria=$($_.Criteria)"
    }) -join [Environment]::NewLine
}
$serverReadbackReport = if ([string]::IsNullOrWhiteSpace($serverReadbackPath)) {
    'n/a'
} else {
    Split-Path -Leaf $serverReadbackPath
}
$playbackSessionReadbackReport = if ([string]::IsNullOrWhiteSpace($playbackSessionReadbackPath)) {
    'n/a'
} else {
    Split-Path -Leaf $playbackSessionReadbackPath
}
$playbackSessionCancellationReadbackReport = if ([string]::IsNullOrWhiteSpace($playbackSessionCancellationReadbackPath)) {
    'n/a'
} else {
    Split-Path -Leaf $playbackSessionCancellationReadbackPath
}

$reportPath = Join-Path $outputDir 'report.md'
$jsonPath = Join-Path $outputDir 'report.json'
$surfaceReportItems = @(
    foreach ($surface in $surfaceEvidence) {
        [pscustomobject]@{
            name = $surface.Name
            screenshot = $surface.Screenshot
            hierarchy = $surface.Hierarchy
            criteria = $surface.Criteria
        }
    }
)
$jsonReport = [ordered]@{
    schema_version = 1
    kind = 'taru_android_smoke_state'
    timestamp = (Get-Date).ToString('o')
    result = 'PASS'
    fixture_state = $FixtureState
    state_mode = $stateMode
    device_serial = $deviceSerial
    apk = Convert-ToJsonPath -Path $apkPath
    build_step = if ($skipAndroidBuild) { 'skipped' } else { 'assembleDebug' }
    reset_app_data = [bool]$clearsAppData
    fixture_server = [ordered]@{
        base_url = if ($fixtureBaseUrl) { $fixtureBaseUrl } else { $null }
        reverse_port = if ($fixtureReversePort) { $fixtureReversePort } else { $null }
    }
    launch = [ordered]@{
        activity = 'dev.taru.android/.MainActivity'
        output = Convert-ToJsonPath -Path $launchPath
    }
    reports = [ordered]@{
        markdown = Convert-ToJsonPath -Path $reportPath
        json = Convert-ToJsonPath -Path $jsonPath
        evidence_directory = Convert-ToJsonPath -Path $outputDir
    }
    surfaces = $surfaceReportItems
    readbacks = [ordered]@{
        server_playback = Convert-ToJsonPath -Path $serverReadbackPath
        public_playback_session = Convert-ToJsonPath -Path $playbackSessionReadbackPath
        public_playback_session_cancellation = Convert-ToJsonPath -Path $playbackSessionCancellationReadbackPath
    }
    repo_root = Convert-ToJsonPath -Path $repoRoot
}
$report = @"
# Taru Android Smoke Evidence

- Timestamp: $(Get-Date -Format o)
- Device: $deviceSerial
- APK: $apkPath
- Build step: $(if ($skipAndroidBuild) { 'skipped' } else { 'assembleDebug' })
- State mode: $stateMode
- Reset app data: $clearsAppData
- Fixture server base URL: $(if ($fixtureBaseUrl) { $fixtureBaseUrl } else { 'n/a' })
- Fixture reverse port: $(if ($fixtureReversePort) { "tcp:$fixtureReversePort" } else { 'n/a' })
- Launch activity: dev.taru.android/.MainActivity
- Launch output: launch.txt
- Surface evidence:
$surfaceReport
- Server playback readback: $serverReadbackReport
- Public playback session readback: $playbackSessionReadbackReport
- Public playback session cancellation readback: $playbackSessionCancellationReadbackReport
- Repo root: $repoRoot
"@
Write-Utf8File -Path $reportPath -Content $report
Write-Utf8File -Path $jsonPath -Content ($jsonReport | ConvertTo-Json -Depth 12)

Write-Host "Smoke complete."
Write-Host "Evidence directory: $outputDir"
Write-Host "Report: $reportPath"
Write-Host "Structured report: $jsonPath"
Write-Host "Launch output: $launchPath"
Write-Host "Surface evidence:"
$surfaceEvidence | ForEach-Object {
    Write-Host "- $($_.Name): $($_.Screenshot)"
}
} finally {
    if ($fixtureReversePort -ne $null) {
        & $adb -s $deviceSerial reverse --remove "tcp:$fixtureReversePort" *> $null
    }
    Stop-SmokeFixtureServer -Process $fixtureServerProcess
}
