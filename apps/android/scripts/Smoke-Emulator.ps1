[CmdletBinding()]
param(
    [string]$Serial,
    [string]$OutputRoot,
    [ValidateSet('current-state', 'empty-setup', 'profile-missing-token', 'profile-with-media')]
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
        [bool]$SkipServerBuild
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

    & $providerScript @providerArgs
    if ($LASTEXITCODE -ne 0) {
        throw 'Demo fixture provider preparation failed.'
    }

    $summaryPath = Join-Path $AndroidRoot 'build\demo-fixtures\server-backed\summary.json'
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
        [string]$ResumeMediaItemId,
        [string]$ResumeSourceId,
        [long]$ResumePositionMs = 0,
        [long]$ResumeDurationMs = 0
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
        if (-not [string]::IsNullOrWhiteSpace($ResumeMediaItemId) -and
            -not [string]::IsNullOrWhiteSpace($ResumeSourceId) -and
            $ResumePositionMs -gt 0) {
            $seedArguments += @(
                '--extra', "resume_media_item_id:s:$ResumeMediaItemId",
                '--extra', "resume_source_id:s:$ResumeSourceId",
                '--extra', "resume_position_ms:l:$ResumePositionMs"
            )
            if ($ResumeDurationMs -gt 0) {
                $seedArguments += @('--extra', "resume_duration_ms:l:$ResumeDurationMs")
            }
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
Resume Media Item id: $ResumeMediaItemId
Resume Media Source id: $ResumeSourceId
Resume position: $ResumePositionMs
Resume duration: $ResumeDurationMs
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

function Resolve-FixtureState {
    param(
        [string]$RequestedFixtureState,
        [bool]$RequestedResetAppData
    )

    if ($RequestedResetAppData) {
        if ($RequestedFixtureState -eq 'current-state') {
            return 'empty-setup'
        }

        if ($RequestedFixtureState -notin @('empty-setup', 'profile-with-media')) {
            throw '-ResetAppData can only be combined with the default fixture state, -FixtureState empty-setup, or -FixtureState profile-with-media.'
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
$clearsAppData = $stateMode -in @('empty-setup', 'profile-missing-token', 'profile-with-media')
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
if ($stateMode -eq 'profile-with-media') {
    $fixtureProvider = Start-SmokeMediaFixtureProvider `
        -ScriptDir $scriptDir `
        -AndroidRoot $androidRoot `
        -OutputDir $outputDir `
        -Port $FixtureServerPort `
        -SkipServerBuild $skipServerBuild
    $fixtureServerProcess = $fixtureProvider.Process
    $fixtureBaseUrl = $fixtureProvider.Summary.base_url
    $resumeFixture = Resolve-SmokeMediaResumeFixture `
        -BaseUrl $fixtureBaseUrl `
        -AccessToken $FixtureAccessToken
    Invoke-Adb -AdbPath $adb -Arguments @('-s', $deviceSerial, 'reverse', "tcp:$FixtureServerPort", "tcp:$FixtureServerPort") -FailureMessage 'adb reverse failed for profile-with-media.'
    $fixtureReversePort = $FixtureServerPort
    Install-SmokeMediaProfileFixture `
        -AdbPath $adb `
        -DeviceSerial $deviceSerial `
        -OutputDir $outputDir `
        -BaseUrl $fixtureBaseUrl `
        -AccessToken $FixtureAccessToken `
        -ResumeMediaItemId $resumeFixture.MediaItemId `
        -ResumeSourceId $resumeFixture.SourceId `
        -ResumePositionMs $resumeFixture.PositionMs `
        -ResumeDurationMs $resumeFixture.DurationMs
    Invoke-Adb -AdbPath $adb -Arguments @('-s', $deviceSerial, 'shell', 'am', 'force-stop', 'dev.taru.android') -FailureMessage 'adb force-stop after profile-with-media seed failed.'
}
if ($stateMode -eq 'profile-missing-token') {
    Install-SmokeProfileFixture -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir
}
if ($stateMode -ne 'profile-with-media') {
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
} elseif ($stateMode -eq 'profile-with-media') {
    Wait-ForUiText -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Text 'Night Harbor' -TimeoutSeconds 35
    $surfaceEvidence += Capture-SmokeSurface -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Name 'home' -RequiredText @(
        'Smoke Server',
        'Night Harbor',
        'Media Libraries',
        '1 visible',
        'Open detail'
    )

    Tap-UiText -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Text 'Open detail'
    Wait-ForUiText -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Text 'Check source' -TimeoutSeconds 25
    $surfaceEvidence += Capture-SmokeSurface -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Name 'detail' -RequiredText @(
        'Night Harbor',
        'Resume',
        'Check source',
        'Needs check'
    )

    Swipe-UntilUiText -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Text 'Resume on this device'
    $surfaceEvidence += Capture-SmokeSurface -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Name 'source-picker-local-resume' -RequiredText @(
        'Source / Version',
        'Night Harbor.mp4',
        'Resume on this device',
        'A device-local position exists for the selected source. Taru still checks the source before playback.',
        'Resume'
    ) -ForbiddenText @(
        'Continue Watching',
        'User Playback State'
    )
    Tap-UiText -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Text 'Resume'
    Wait-ForUiText -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Text 'Direct' -TimeoutSeconds 25
    Swipe-UntilUiText -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Text 'Source / Version'
    Swipe-UntilUiText -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Text 'Start resume'
    $surfaceEvidence += Capture-SmokeSurface -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Name 'source-picker' -RequiredText @(
        'Source / Version',
        'Night Harbor.mp4',
        'Direct route prepared',
        'Start resume'
    ) -ForbiddenText @(
        'Continue Watching',
        'User Playback State'
    )

    Tap-UiText -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Text 'Start resume'
    Wait-ForUiText -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Text 'Night Harbor' -TimeoutSeconds 25
    $surfaceEvidence += Capture-SmokeSurface -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir -Name 'player' -RequiredText @(
        'Night Harbor',
        'Direct',
        'Local resume 0:01',
        'Tracks and subtitles use Media3 controls in this version.'
    ) -ForbiddenText @(
        'Continue Watching',
        'User Playback State'
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

$reportPath = Join-Path $outputDir 'report.md'
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
- Repo root: $repoRoot
"@
$report | Out-File -LiteralPath $reportPath -Encoding utf8

Write-Host "Smoke complete."
Write-Host "Evidence directory: $outputDir"
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
