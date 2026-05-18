[CmdletBinding()]
param(
    [string]$Serial,
    [string]$OutputRoot,
    [ValidateSet('current-state', 'empty-setup', 'profile-missing-token')]
    [string]$FixtureState = 'current-state',
    [switch]$SkipBuild,
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

        if ($RequestedFixtureState -ne 'empty-setup') {
            throw '-ResetAppData can only be combined with the default fixture state or -FixtureState empty-setup.'
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
    $remoteDump = "/sdcard/taru-android-smoke-$safeName.xml"
    $localDump = Join-Path $OutputDir "$safeName.uiautomator.xml"
    $lastError = $null

    for ($attempt = 1; $attempt -le 5; $attempt += 1) {
        try {
            & $AdbPath -s $DeviceSerial shell rm $remoteDump *> $null
            Invoke-Adb -AdbPath $AdbPath -Arguments @('-s', $DeviceSerial, 'shell', 'uiautomator', 'dump', $remoteDump) -FailureMessage "adb UI dump failed for '$Name'."
            Invoke-Adb -AdbPath $AdbPath -Arguments @('-s', $DeviceSerial, 'pull', $remoteDump, $localDump) -FailureMessage "adb pull UI dump failed for '$Name'."
            Invoke-Adb -AdbPath $AdbPath -Arguments @('-s', $DeviceSerial, 'shell', 'rm', $remoteDump) -FailureMessage "adb cleanup UI dump failed for '$Name'."
            return $localDump
        } catch {
            $lastError = $_.Exception.Message
            Start-Sleep -Milliseconds 750
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
        [string[]]$RequiredText = @()
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

    $criteriaLines.Add("Result: $(if ($missing.Count -eq 0) { 'PASS' } else { 'FAIL' })")
    $criteriaLines | Out-File -LiteralPath $criteriaPath -Encoding utf8

    if ($missing.Count -gt 0) {
        throw "Surface '$Name' missing expected UI text/content descriptions: $($missing -join ', ')."
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
$clearsAppData = $stateMode -in @('empty-setup', 'profile-missing-token')

if ([string]::IsNullOrWhiteSpace($OutputRoot)) {
    $OutputRoot = Join-Path $androidRoot 'build\smoke'
}

$timestamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$outputDir = Join-Path $OutputRoot "$timestamp-$stateMode-$deviceSerial"
New-Item -ItemType Directory -Force -Path $outputDir | Out-Null

if (-not $SkipBuild) {
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
if ($stateMode -eq 'profile-missing-token') {
    Install-SmokeProfileFixture -AdbPath $adb -DeviceSerial $deviceSerial -OutputDir $outputDir
}
Invoke-Adb -AdbPath $adb -Arguments @('-s', $deviceSerial, 'shell', 'am', 'force-stop', 'dev.taru.android') -FailureMessage 'adb force-stop failed.'
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
- Build step: $(if ($SkipBuild) { 'skipped' } else { 'assembleDebug' })
- State mode: $stateMode
- Reset app data: $clearsAppData
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
