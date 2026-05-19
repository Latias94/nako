[CmdletBinding()]
param(
    [string[]]$States = @('empty-setup', 'profile-missing-token', 'profile-with-media'),
    [string]$Serial,
    [string]$OutputRoot,
    [int]$FixtureServerPort = 3018,
    [string]$FixtureAccessToken = 'demo-fixture-token',
    [switch]$SkipBuild,
    [switch]$SkipFixtureServerBuild,
    [int]$RetriesPerState = 1,
    [switch]$ContinueOnFailure
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

function Invoke-AndroidBuild {
    param(
        [string]$AndroidRoot
    )

    $gradlew = Join-Path $AndroidRoot 'gradlew.bat'
    if (-not (Test-Path -LiteralPath $gradlew)) {
        throw "Gradle wrapper was not found at '$gradlew'."
    }

    Push-Location $AndroidRoot
    try {
        & $gradlew :app:assembleDebug --no-daemon
        if ($LASTEXITCODE -ne 0) {
            throw 'Gradle assembleDebug failed.'
        }
    } finally {
        Pop-Location
    }
}

function Get-LatestEvidenceDirectory {
    param(
        [string]$StateRoot
    )

    if (-not (Test-Path -LiteralPath $StateRoot)) {
        return $null
    }

    return Get-ChildItem -LiteralPath $StateRoot -Directory |
        Sort-Object LastWriteTimeUtc -Descending |
        Select-Object -First 1
}

function Convert-ToReportPath {
    param(
        [string]$Path
    )

    if ([string]::IsNullOrWhiteSpace($Path)) {
        return 'n/a'
    }

    return $Path.Replace('\', '/')
}

function Resolve-SmokeStates {
    param(
        [string[]]$RequestedStates
    )

    $allowedStates = @('current-state', 'empty-setup', 'profile-missing-token', 'profile-with-media')
    $resolvedStates = New-Object System.Collections.Generic.List[string]
    foreach ($entry in $RequestedStates) {
        if ([string]::IsNullOrWhiteSpace($entry)) {
            continue
        }

        foreach ($state in ($entry -split ',')) {
            $trimmed = $state.Trim()
            if ([string]::IsNullOrWhiteSpace($trimmed)) {
                continue
            }

            if ($allowedStates -notcontains $trimmed) {
                throw "Unknown smoke fixture state '$trimmed'. Allowed states: $($allowedStates -join ', ')."
            }

            $resolvedStates.Add($trimmed)
        }
    }

    if ($resolvedStates.Count -eq 0) {
        throw 'At least one smoke fixture state is required.'
    }

    return $resolvedStates.ToArray()
}

$scriptDir = $PSScriptRoot
$androidRoot = (Resolve-Path -LiteralPath (Join-Path $scriptDir '..')).Path
$smokeScript = Join-Path $scriptDir 'Smoke-Emulator.ps1'
$resolvedStates = Resolve-SmokeStates -RequestedStates $States

if (-not (Test-Path -LiteralPath $smokeScript)) {
    throw "Smoke script was not found at '$smokeScript'."
}

if ([string]::IsNullOrWhiteSpace($OutputRoot)) {
    $OutputRoot = Join-Path $androidRoot 'build\smoke-regression'
}

$timestamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$runRoot = Join-Path $OutputRoot $timestamp
$statesRoot = Join-Path $runRoot 'states'
New-Item -ItemType Directory -Force -Path $statesRoot | Out-Null

$results = New-Object System.Collections.Generic.List[object]
$startedAt = Get-Date

try {
    if (-not $SkipBuild) {
        Invoke-AndroidBuild -AndroidRoot $androidRoot
    }

    foreach ($state in $resolvedStates) {
        $stateRoot = Join-Path $statesRoot $state
        New-Item -ItemType Directory -Force -Path $stateRoot | Out-Null

        $stateLog = Join-Path $runRoot "$state.log"
        $smokeArgs = @{
            FixtureState = $state
            OutputRoot = $stateRoot
            FixtureServerPort = $FixtureServerPort
            FixtureAccessToken = $FixtureAccessToken
            SkipAppBuild = $true
        }

        if (-not [string]::IsNullOrWhiteSpace($Serial)) {
            $smokeArgs.Serial = $Serial
        }

        if ($SkipFixtureServerBuild) {
            $smokeArgs.SkipFixtureServerBuild = $true
        }

        $status = 'FAIL'
        $errorMessage = $null
        $attempts = [Math]::Max(1, $RetriesPerState + 1)

        for ($attempt = 1; $attempt -le $attempts; $attempt += 1) {
            if ($attempt -gt 1) {
                Add-Content -LiteralPath $stateLog -Encoding utf8 -Value ''
                Add-Content -LiteralPath $stateLog -Encoding utf8 -Value "Retrying state '$state' after failed attempt $($attempt - 1)."
                Start-Sleep -Seconds 3
            }

            try {
                & $smokeScript @smokeArgs *>&1 | Tee-Object -FilePath $stateLog -Append
                $status = 'PASS'
                $errorMessage = $null
                break
            } catch {
                $status = 'FAIL'
                $errorMessage = $_.Exception.Message
                Add-Content -LiteralPath $stateLog -Encoding utf8 -Value ''
                Add-Content -LiteralPath $stateLog -Encoding utf8 -Value "ERROR attempt ${attempt}: $errorMessage"
            }
        }

        $latestEvidence = Get-LatestEvidenceDirectory -StateRoot $stateRoot
        $results.Add([pscustomobject]@{
            State = $state
            Status = $status
            EvidenceDirectory = if ($latestEvidence) { $latestEvidence.FullName } else { $null }
            Log = $stateLog
            Error = $errorMessage
            Attempts = if ($status -eq 'PASS') { $attempt } else { $attempts }
        })

        if ($status -ne 'PASS' -and -not $ContinueOnFailure) {
            break
        }
    }
} finally {
    $finishedAt = Get-Date
    $failed = @($results | Where-Object { $_.Status -ne 'PASS' })
    $reportPath = Join-Path $runRoot 'report.md'
    $lines = New-Object System.Collections.Generic.List[string]
    $lines.Add('# Taru Android Smoke Regression')
    $lines.Add('')
    $lines.Add("- Started: $($startedAt.ToString('o'))")
    $lines.Add("- Finished: $($finishedAt.ToString('o'))")
    $lines.Add("- Android build step: $(if ($SkipBuild) { 'skipped' } else { 'assembleDebug' })")
    $lines.Add("- Requested serial: $(if ([string]::IsNullOrWhiteSpace($Serial)) { 'auto' } else { $Serial })")
    $lines.Add("- Fixture server port: $FixtureServerPort")
    $lines.Add("- Fixture server build: $(if ($SkipFixtureServerBuild) { 'skipped when needed' } else { 'enabled when needed' })")
    $lines.Add("- Retries per state: $RetriesPerState")
    $lines.Add("- Result: $(if ($failed.Count -eq 0 -and $results.Count -eq $resolvedStates.Count) { 'PASS' } else { 'FAIL' })")
    $lines.Add('')
    $lines.Add('## States')
    $lines.Add('')
    $lines.Add('| State | Status | Attempts | Evidence | Log |')
    $lines.Add('| --- | --- | --- | --- | --- |')

    foreach ($result in $results) {
        $lines.Add("| $($result.State) | $($result.Status) | $($result.Attempts) | $(Convert-ToReportPath -Path $result.EvidenceDirectory) | $(Convert-ToReportPath -Path $result.Log) |")
    }

    $notRun = @($resolvedStates | Where-Object { $state = $_; -not ($results | Where-Object { $_.State -eq $state }) })
    foreach ($state in $notRun) {
        $lines.Add("| $state | NOT_RUN | 0 | n/a | n/a |")
    }

    $errors = @($results | Where-Object { -not [string]::IsNullOrWhiteSpace($_.Error) })
    if ($errors.Count -gt 0) {
        $lines.Add('')
        $lines.Add('## Errors')
        $lines.Add('')
        foreach ($result in $errors) {
            $lines.Add("- $($result.State): $($result.Error)")
        }
    }

    Write-Utf8File -Path $reportPath -Content ($lines -join [Environment]::NewLine)

    Write-Host "Smoke regression complete."
    Write-Host "Result: $(if ($failed.Count -eq 0 -and $results.Count -eq $resolvedStates.Count) { 'PASS' } else { 'FAIL' })"
    Write-Host "Report: $reportPath"
}

if (@($results | Where-Object { $_.Status -ne 'PASS' }).Count -gt 0) {
    throw "Smoke regression failed. See report: $reportPath"
}

if ($results.Count -ne $resolvedStates.Count) {
    throw "Smoke regression did not run every requested state. See report: $reportPath"
}
