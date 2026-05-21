[CmdletBinding()]
param(
    [string]$Serial,
    [ValidateSet('arm64-v8a', 'armeabi-v7a', 'x86', 'x86_64')]
    [string]$Abi = 'arm64-v8a',
    [string]$TestClass = 'dev.taru.android.uniffi.TaruUniFfiNativeSmokeTest',
    [switch]$SkipBuild,
    [switch]$SkipInstall
)

$ErrorActionPreference = 'Stop'

function Resolve-RepoRoot {
    $scriptPath = $PSScriptRoot
    return (Resolve-Path (Join-Path $scriptPath '../../..')).Path
}

function Invoke-CheckedCommand {
    param(
        [string]$Label,
        [scriptblock]$Command
    )

    Write-Host "==> $Label"
    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "$Label failed with exit code $LASTEXITCODE."
    }
}

function Get-AdbPath {
    $adb = Get-Command adb -ErrorAction SilentlyContinue
    if ($adb) {
        return $adb.Source
    }
    throw 'adb was not found on PATH.'
}

function Resolve-DeviceSerial {
    param(
        [string]$Adb,
        [string]$RequestedSerial
    )

    if (-not [string]::IsNullOrWhiteSpace($RequestedSerial)) {
        $null = Invoke-CheckedCommand "adb device state for $RequestedSerial" {
            & $Adb -s $RequestedSerial get-state
        }
        return $RequestedSerial
    }

    $devices = & $Adb devices | Select-Object -Skip 1 | Where-Object { $_ -match '\tdevice$' }
    if ($devices.Count -eq 0) {
        throw 'No connected Android device is in device state. Pass -Serial after connecting a device.'
    }
    if ($devices.Count -gt 1) {
        throw "Multiple Android devices are connected. Pass -Serial. Devices: $($devices -join '; ')"
    }
    return (($devices[0] -split '\s+')[0])
}

function Assert-DeviceSupportsAbi {
    param(
        [string]$Adb,
        [string]$DeviceSerial,
        [string]$ExpectedAbi
    )

    $primaryAbi = (& $Adb -s $DeviceSerial shell getprop ro.product.cpu.abi).Trim()
    $abiList = (& $Adb -s $DeviceSerial shell getprop ro.product.cpu.abilist).Trim()
    Write-Host "Device serial: $DeviceSerial"
    Write-Host "Device primary ABI: $primaryAbi"
    Write-Host "Device ABI list: $abiList"

    $supportedAbis = @($abiList -split ',' | Where-Object { -not [string]::IsNullOrWhiteSpace($_) })
    if ($supportedAbis.Count -eq 0) {
        $supportedAbis = @($primaryAbi)
    }

    if ($ExpectedAbi -notin $supportedAbis) {
        throw "Device '$DeviceSerial' does not report ABI '$ExpectedAbi'. Reported ABI list: '$abiList'."
    }
}

$repoRoot = Resolve-RepoRoot
$androidRoot = Join-Path $repoRoot 'apps/android'
$gradlew = Join-Path $androidRoot 'gradlew.bat'
$appApk = Join-Path $androidRoot 'app/build/outputs/apk/debug/app-debug.apk'
$testApk = Join-Path $androidRoot 'app/build/outputs/apk/androidTest/debug/app-debug-androidTest.apk'
$adb = Get-AdbPath
$deviceSerial = Resolve-DeviceSerial -Adb $adb -RequestedSerial $Serial

Assert-DeviceSupportsAbi -Adb $adb -DeviceSerial $deviceSerial -ExpectedAbi $Abi

if (-not $SkipBuild) {
    Push-Location $androidRoot
    try {
        Invoke-CheckedCommand "Gradle assemble debug and androidTest for $Abi" {
            & $gradlew :app:assembleDebug :app:assembleDebugAndroidTest "-PtaruRustAndroidAbis=$Abi" --no-daemon
        }
    } finally {
        Pop-Location
    }
}

if (-not (Test-Path -LiteralPath $appApk)) {
    throw "App APK was not found at '$appApk'."
}
if (-not (Test-Path -LiteralPath $testApk)) {
    throw "AndroidTest APK was not found at '$testApk'."
}

if (-not $SkipInstall) {
    Invoke-CheckedCommand 'Install debug APK' {
        & $adb -s $deviceSerial install -r $appApk
    }
    Invoke-CheckedCommand 'Install androidTest APK' {
        & $adb -s $deviceSerial install -r $testApk
    }
}

Invoke-CheckedCommand "Run $TestClass" {
    & $adb -s $deviceSerial shell am instrument -w -r -e class $TestClass dev.taru.android.test/androidx.test.runner.AndroidJUnitRunner
}

[pscustomobject]@{
    status = 'PASS'
    serial = $deviceSerial
    abi = $Abi
    test_class = $TestClass
    app_apk = $appApk.Replace('\', '/')
    android_test_apk = $testApk.Replace('\', '/')
} | ConvertTo-Json -Depth 3
