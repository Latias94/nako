[CmdletBinding()]
param(
    [string]$Serial,
    [string]$OutputRoot,
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

    & $AdbPath @Arguments
    if ($LASTEXITCODE -ne 0) {
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

$scriptDir = $PSScriptRoot
$androidRoot = (Resolve-Path -LiteralPath (Join-Path $scriptDir '..')).Path
$repoRoot = Split-Path -Parent (Split-Path -Parent $androidRoot)
$gradlew = Join-Path $androidRoot 'gradlew.bat'
$adb = Resolve-AdbPath
$deviceSerial = Get-ConnectedDeviceSerial -AdbPath $adb -RequestedSerial $Serial

if ([string]::IsNullOrWhiteSpace($OutputRoot)) {
    $OutputRoot = Join-Path $androidRoot 'build\smoke'
}

$timestamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$stateMode = if ($ResetAppData) { 'empty-setup' } else { 'current-state' }
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
if ($ResetAppData) {
    Invoke-Adb -AdbPath $adb -Arguments @('-s', $deviceSerial, 'shell', 'pm', 'clear', 'dev.taru.android') -FailureMessage 'adb app data reset failed.'
}
Invoke-Adb -AdbPath $adb -Arguments @('-s', $deviceSerial, 'shell', 'am', 'force-stop', 'dev.taru.android') -FailureMessage 'adb force-stop failed.'

$launchPath = Join-Path $outputDir 'launch.txt'
$launchOutput = & $adb -s $deviceSerial shell am start -W -n 'dev.taru.android/.MainActivity' -a android.intent.action.MAIN -c android.intent.category.LAUNCHER 2>&1 | Tee-Object -FilePath $launchPath
if ($LASTEXITCODE -ne 0) {
    throw 'adb am start failed.'
}

Start-Sleep -Seconds 6

$remoteShot = "/sdcard/taru-android-smoke-$timestamp.png"
$localShot = Join-Path $outputDir 'launch.png'
Invoke-Adb -AdbPath $adb -Arguments @('-s', $deviceSerial, 'shell', 'screencap', '-p', $remoteShot) -FailureMessage 'adb screencap failed.'
Invoke-Adb -AdbPath $adb -Arguments @('-s', $deviceSerial, 'pull', $remoteShot, $localShot) -FailureMessage 'adb pull screenshot failed.'
Invoke-Adb -AdbPath $adb -Arguments @('-s', $deviceSerial, 'shell', 'rm', $remoteShot) -FailureMessage 'adb cleanup of remote screenshot failed.'

$reportPath = Join-Path $outputDir 'report.md'
$report = @"
# Taru Android Smoke Evidence

- Timestamp: $(Get-Date -Format o)
- Device: $deviceSerial
- APK: $apkPath
- Build step: $(if ($SkipBuild) { 'skipped' } else { 'assembleDebug' })
- State mode: $stateMode
- Reset app data: $([bool]$ResetAppData)
- Launch activity: dev.taru.android/.MainActivity
- Launch output: launch.txt
- Screenshot: launch.png
- Repo root: $repoRoot
"@
$report | Out-File -LiteralPath $reportPath -Encoding utf8

Write-Host "Smoke complete."
Write-Host "Evidence directory: $outputDir"
Write-Host "Launch output: $launchPath"
Write-Host "Screenshot: $localShot"
