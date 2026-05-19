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

function Convert-ToJsonPath {
    param(
        [string]$Path
    )

    if ([string]::IsNullOrWhiteSpace($Path)) {
        return $null
    }

    return $Path.Replace('\', '/')
}

function Resolve-OutputRootPath {
    param(
        [string]$Path
    )

    if ([System.IO.Path]::IsPathRooted($Path)) {
        return $Path
    }

    return (Join-Path (Get-Location).Path $Path)
}

function Convert-ToCommandValue {
    param(
        [string]$Value
    )

    if ([string]::IsNullOrWhiteSpace($Value)) {
        return ''
    }

    if ($Value -match '[\s,;''"]') {
        return "'$($Value -replace "'", "''")'"
    }

    return $Value
}

function Get-FailureCategory {
    param(
        [string]$State,
        [string]$ErrorMessage
    )

    if ([string]::IsNullOrWhiteSpace($ErrorMessage)) {
        return 'n/a'
    }

    $message = $ErrorMessage.ToLowerInvariant()
    if ($message.Contains('gradle') -or $message.Contains('assembledebug')) {
        return 'android-build'
    }

    if ($message.Contains('no connected android devices') -or
        $message.Contains('requested device') -or
        $message.Contains('adb ') -or
        $message.Contains('uiautomator') -or
        $message.Contains('ui dump') -or
        $message.Contains('null root node')) {
        return 'device-automation'
    }

    if ($message.Contains('demo fixture') -or
        $message.Contains('fixture server') -or
        $message.Contains('health check') -or
        $message.Contains('adb reverse')) {
        return 'fixture-server'
    }

    if ($message.Contains('timed out waiting for ui text') -or
        $message.Contains('missing expected ui text') -or
        $message.Contains('cannot tap ui text')) {
        return 'surface-criteria'
    }

    if ($State -eq 'profile-with-media') {
        return 'media-smoke'
    }

    return 'unknown'
}

function Get-RerunCommand {
    param(
        [string]$State,
        [string]$Serial,
        [int]$FixtureServerPort,
        [bool]$SkipFixtureServerBuild
    )

    $parts = New-Object System.Collections.Generic.List[string]
    $parts.Add('pwsh')
    $parts.Add('-NoProfile')
    $parts.Add('-File')
    $parts.Add('apps/android/scripts/Smoke-Emulator.ps1')
    $parts.Add('-FixtureState')
    $parts.Add($State)
    $parts.Add('-SkipAppBuild')

    if (-not [string]::IsNullOrWhiteSpace($Serial)) {
        $parts.Add('-Serial')
        $parts.Add((Convert-ToCommandValue -Value $Serial))
    }

    if ($FixtureServerPort -ne 3018) {
        $parts.Add('-FixtureServerPort')
        $parts.Add($FixtureServerPort.ToString())
    }

    if ($SkipFixtureServerBuild) {
        $parts.Add('-SkipFixtureServerBuild')
    }

    return ($parts -join ' ')
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
$OutputRoot = Resolve-OutputRootPath -Path $OutputRoot

$timestamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$runRoot = Join-Path $OutputRoot $timestamp
$statesRoot = Join-Path $runRoot 'states'
New-Item -ItemType Directory -Force -Path $statesRoot | Out-Null

$results = New-Object System.Collections.Generic.List[object]
$startedAt = Get-Date
$buildStatus = 'SKIPPED'
$buildError = $null

try {
    if (-not $SkipBuild) {
        $buildStatus = 'PASS'
        try {
            Invoke-AndroidBuild -AndroidRoot $androidRoot
        } catch {
            $buildStatus = 'FAIL'
            $buildError = $_.Exception.Message
            throw
        }
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
            Category = Get-FailureCategory -State $state -ErrorMessage $errorMessage
            EvidenceDirectory = if ($latestEvidence) { $latestEvidence.FullName } else { $null }
            Log = $stateLog
            Error = $errorMessage
            Attempts = if ($status -eq 'PASS') { $attempt } else { $attempts }
            RerunCommand = Get-RerunCommand -State $state -Serial $Serial -FixtureServerPort $FixtureServerPort -SkipFixtureServerBuild ([bool]$SkipFixtureServerBuild)
        })

        if ($status -ne 'PASS' -and -not $ContinueOnFailure) {
            break
        }
    }
} finally {
    $finishedAt = Get-Date
    $failed = @($results | Where-Object { $_.Status -ne 'PASS' })
    $reportPath = Join-Path $runRoot 'report.md'
    $jsonPath = Join-Path $runRoot 'report.json'
    $overallPass = $buildStatus -ne 'FAIL' -and $failed.Count -eq 0 -and $results.Count -eq $resolvedStates.Count
    $notRun = @($resolvedStates | Where-Object { $state = $_; -not ($results | Where-Object { $_.State -eq $state }) })
    $notRunStates = @(
        foreach ($state in $notRun) {
            [pscustomobject]@{
                state = $state
                status = 'NOT_RUN'
                category = if ($buildStatus -eq 'FAIL') { 'android-build' } else { 'blocked-by-earlier-state' }
                attempts = 0
                evidence_directory = $null
                log = $null
                error = $null
                rerun_command = $null
            }
        }
    )
    $stateResults = @(
        foreach ($result in $results) {
            [pscustomobject]@{
                state = $result.State
                status = $result.Status
                category = $result.Category
                attempts = $result.Attempts
                evidence_directory = Convert-ToJsonPath -Path $result.EvidenceDirectory
                log = Convert-ToJsonPath -Path $result.Log
                error = $result.Error
                rerun_command = $result.RerunCommand
            }
        }
        foreach ($state in $notRunStates) {
            $state
        }
    )
    $jsonReport = [ordered]@{
        schema_version = 1
        kind = 'taru_android_smoke_regression'
        started_at = $startedAt.ToString('o')
        finished_at = $finishedAt.ToString('o')
        result = if ($overallPass) { 'PASS' } else { 'FAIL' }
        report_markdown = Convert-ToJsonPath -Path $reportPath
        report_json = Convert-ToJsonPath -Path $jsonPath
        options = [ordered]@{
            requested_serial = if ([string]::IsNullOrWhiteSpace($Serial)) { $null } else { $Serial }
            states = @($resolvedStates)
            fixture_server_port = $FixtureServerPort
            skip_build = [bool]$SkipBuild
            skip_fixture_server_build = [bool]$SkipFixtureServerBuild
            retries_per_state = $RetriesPerState
            continue_on_failure = [bool]$ContinueOnFailure
        }
        android_build = [ordered]@{
            step = if ($SkipBuild) { 'skipped' } else { 'assembleDebug' }
            status = $buildStatus
            error = $buildError
        }
        states = $stateResults
    }
    $lines = New-Object System.Collections.Generic.List[string]
    $lines.Add('# Taru Android Smoke Regression')
    $lines.Add('')
    $lines.Add("- Started: $($startedAt.ToString('o'))")
    $lines.Add("- Finished: $($finishedAt.ToString('o'))")
    $lines.Add("- Android build step: $(if ($SkipBuild) { 'skipped' } else { 'assembleDebug' })")
    $lines.Add("- Android build status: $buildStatus")
    $lines.Add("- Requested serial: $(if ([string]::IsNullOrWhiteSpace($Serial)) { 'auto' } else { $Serial })")
    $lines.Add("- Fixture server port: $FixtureServerPort")
    $lines.Add("- Fixture server build: $(if ($SkipFixtureServerBuild) { 'skipped when needed' } else { 'enabled when needed' })")
    $lines.Add("- Retries per state: $RetriesPerState")
    $lines.Add("- Result: $(if ($overallPass) { 'PASS' } else { 'FAIL' })")
    $lines.Add('')
    $lines.Add('## States')
    $lines.Add('')
    $lines.Add('| State | Status | Attempts | Category | Evidence | Log |')
    $lines.Add('| --- | --- | --- | --- | --- | --- |')

    foreach ($result in $results) {
        $lines.Add("| $($result.State) | $($result.Status) | $($result.Attempts) | $($result.Category) | $(Convert-ToReportPath -Path $result.EvidenceDirectory) | $(Convert-ToReportPath -Path $result.Log) |")
    }

    foreach ($state in $notRun) {
        $reason = if ($buildStatus -eq 'FAIL') { 'android-build' } else { 'blocked-by-earlier-state' }
        $lines.Add("| $state | NOT_RUN | 0 | $reason | n/a | n/a |")
    }

    if ($buildStatus -eq 'FAIL') {
        $lines.Add('')
        $lines.Add('## Build Failure')
        $lines.Add('')
        $lines.Add("Category: android-build")
        $lines.Add('')
        $lines.Add('Rerun:')
        $lines.Add('')
        $lines.Add('```powershell')
        $lines.Add('apps/android/gradlew.bat -p apps/android :app:assembleDebug --no-daemon')
        $lines.Add('```')
        $lines.Add('')
        $lines.Add('Error:')
        $lines.Add('')
        $lines.Add('```text')
        $lines.Add($buildError)
        $lines.Add('```')
    }

    $errors = @($results | Where-Object { -not [string]::IsNullOrWhiteSpace($_.Error) })
    if ($errors.Count -gt 0) {
        $lines.Add('')
        $lines.Add('## Errors')
        $lines.Add('')
        foreach ($result in $errors) {
            $lines.Add("### $($result.State)")
            $lines.Add('')
            $lines.Add("- Category: $($result.Category)")
            $lines.Add("- Attempts: $($result.Attempts)")
            $lines.Add("- Evidence: $(Convert-ToReportPath -Path $result.EvidenceDirectory)")
            $lines.Add("- Log: $(Convert-ToReportPath -Path $result.Log)")
            $lines.Add('')
            $lines.Add('Rerun:')
            $lines.Add('')
            $lines.Add('```powershell')
            $lines.Add($result.RerunCommand)
            $lines.Add('```')
            $lines.Add('')
            $lines.Add('Error:')
            $lines.Add('')
            $lines.Add('```text')
            $lines.Add($result.Error)
            $lines.Add('```')
            $lines.Add('')
        }
    }

    Write-Utf8File -Path $reportPath -Content ($lines -join [Environment]::NewLine)
    Write-Utf8File -Path $jsonPath -Content ($jsonReport | ConvertTo-Json -Depth 12)

    Write-Host "Smoke regression complete."
    Write-Host "Result: $(if ($overallPass) { 'PASS' } else { 'FAIL' })"
    Write-Host "Report: $reportPath"
    Write-Host "Structured report: $jsonPath"
}

if (@($results | Where-Object { $_.Status -ne 'PASS' }).Count -gt 0) {
    throw "Smoke regression failed. See report: $reportPath"
}

if ($results.Count -ne $resolvedStates.Count) {
    throw "Smoke regression did not run every requested state. See report: $reportPath"
}
