[CmdletBinding()]
param(
    [string]$Serial,
    [string]$OutputRoot,
    [string[]]$SmokeStates = @('empty-setup', 'profile-missing-token', 'profile-with-media'),
    [switch]$SkipUnitTests,
    [switch]$SkipAssemble,
    [switch]$SkipSmoke,
    [switch]$SkipFixtureServerBuild,
    [int]$RetriesPerState = 1
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

function Convert-ToReportPath {
    param(
        [string]$Path
    )

    if ([string]::IsNullOrWhiteSpace($Path)) {
        return 'n/a'
    }

    return $Path.Replace('\', '/')
}

function Invoke-ValidationStep {
    param(
        [string]$Name,
        [string]$LogPath,
        [scriptblock]$Command
    )

    $startedAt = Get-Date
    $status = 'PASS'
    $errorMessage = $null

    try {
        & $Command *>&1 | Tee-Object -FilePath $LogPath
    } catch {
        $status = 'FAIL'
        $errorMessage = $_.Exception.Message
        Add-Content -LiteralPath $LogPath -Encoding utf8 -Value ''
        Add-Content -LiteralPath $LogPath -Encoding utf8 -Value "ERROR: $errorMessage"
    }

    [pscustomobject]@{
        Name = $Name
        Status = $status
        StartedAt = $startedAt
        FinishedAt = Get-Date
        Log = $LogPath
        Error = $errorMessage
    }
}

function Invoke-GradleTask {
    param(
        [string]$AndroidRoot,
        [string]$TaskName
    )

    $gradlew = Join-Path $AndroidRoot 'gradlew.bat'
    if (-not (Test-Path -LiteralPath $gradlew)) {
        throw "Gradle wrapper was not found at '$gradlew'."
    }

    Push-Location $AndroidRoot
    try {
        & $gradlew $TaskName --no-daemon
        if ($LASTEXITCODE -ne 0) {
            throw "Gradle $TaskName failed."
        }
    } finally {
        Pop-Location
    }
}

function Get-SmokeReportPath {
    param(
        [string]$LogPath
    )

    if (-not (Test-Path -LiteralPath $LogPath)) {
        return $null
    }

    $reportLine = Get-Content -LiteralPath $LogPath |
        Where-Object { $_ -match '^Report:\s+' } |
        Select-Object -Last 1
    if (-not $reportLine) {
        return $null
    }

    return ($reportLine -replace '^Report:\s+', '').Trim()
}

$scriptDir = $PSScriptRoot
$androidRoot = (Resolve-Path -LiteralPath (Join-Path $scriptDir '..')).Path
$smokeRegressionScript = Join-Path $scriptDir 'Smoke-Regression.ps1'

if (-not (Test-Path -LiteralPath $smokeRegressionScript)) {
    throw "Smoke regression script was not found at '$smokeRegressionScript'."
}

if ([string]::IsNullOrWhiteSpace($OutputRoot)) {
    $OutputRoot = Join-Path $androidRoot 'build\validation'
}

$timestamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$runRoot = Join-Path $OutputRoot $timestamp
New-Item -ItemType Directory -Force -Path $runRoot | Out-Null

$startedAt = Get-Date
$results = New-Object System.Collections.Generic.List[object]
$smokeReportPath = $null

if ($SkipUnitTests) {
    $results.Add([pscustomobject]@{
        Name = 'Android JVM tests'
        Status = 'SKIPPED'
        StartedAt = $startedAt
        FinishedAt = $startedAt
        Log = $null
        Error = $null
    })
} else {
    $results.Add((Invoke-ValidationStep `
        -Name 'Android JVM tests' `
        -LogPath (Join-Path $runRoot 'android-unit-tests.log') `
        -Command { Invoke-GradleTask -AndroidRoot $androidRoot -TaskName ':app:testDebugUnitTest' }))
}

if ($SkipAssemble) {
    $results.Add([pscustomobject]@{
        Name = 'Android debug assemble'
        Status = 'SKIPPED'
        StartedAt = Get-Date
        FinishedAt = Get-Date
        Log = $null
        Error = $null
    })
} else {
    $results.Add((Invoke-ValidationStep `
        -Name 'Android debug assemble' `
        -LogPath (Join-Path $runRoot 'android-assemble-debug.log') `
        -Command { Invoke-GradleTask -AndroidRoot $androidRoot -TaskName ':app:assembleDebug' }))
}

$failedBeforeSmoke = @($results | Where-Object { $_.Status -eq 'FAIL' }).Count -gt 0
if ($SkipSmoke -or $failedBeforeSmoke) {
    $results.Add([pscustomobject]@{
        Name = 'Android smoke regression'
        Status = if ($SkipSmoke) { 'SKIPPED' } else { 'NOT_RUN' }
        StartedAt = Get-Date
        FinishedAt = Get-Date
        Log = $null
        Error = if ($failedBeforeSmoke) { 'Skipped because an earlier validation step failed.' } else { $null }
    })
} else {
    $smokeLogPath = Join-Path $runRoot 'android-smoke-regression.log'
    $smokeResult = Invoke-ValidationStep `
        -Name 'Android smoke regression' `
        -LogPath $smokeLogPath `
        -Command {
            $smokeArgs = @{
                States = $SmokeStates
                RetriesPerState = $RetriesPerState
            }

            if (-not [string]::IsNullOrWhiteSpace($Serial)) {
                $smokeArgs.Serial = $Serial
            }

            if ($SkipFixtureServerBuild) {
                $smokeArgs.SkipFixtureServerBuild = $true
            }

            if (-not $SkipAssemble) {
                $smokeArgs.SkipBuild = $true
            }

            & $smokeRegressionScript @smokeArgs
        }
    $results.Add($smokeResult)
    $smokeReportPath = Get-SmokeReportPath -LogPath $smokeLogPath
}

$finishedAt = Get-Date
$failed = @($results | Where-Object { $_.Status -eq 'FAIL' })
$overallPass = $failed.Count -eq 0
$reportPath = Join-Path $runRoot 'report.md'
$lines = New-Object System.Collections.Generic.List[string]

$lines.Add('# Taru Android Local Validation')
$lines.Add('')
$lines.Add("- Started: $($startedAt.ToString('o'))")
$lines.Add("- Finished: $($finishedAt.ToString('o'))")
$lines.Add("- Requested serial: $(if ([string]::IsNullOrWhiteSpace($Serial)) { 'auto' } else { $Serial })")
$lines.Add("- Smoke states: $($SmokeStates -join ', ')")
$lines.Add("- Retries per smoke state: $RetriesPerState")
$lines.Add("- Result: $(if ($overallPass) { 'PASS' } else { 'FAIL' })")
$lines.Add('')
$lines.Add('## Steps')
$lines.Add('')
$lines.Add('| Step | Status | Log |')
$lines.Add('| --- | --- | --- |')
foreach ($result in $results) {
    $lines.Add("| $($result.Name) | $($result.Status) | $(Convert-ToReportPath -Path $result.Log) |")
}

if (-not [string]::IsNullOrWhiteSpace($smokeReportPath)) {
    $lines.Add('')
    $lines.Add('## Smoke Regression')
    $lines.Add('')
    $lines.Add("- Report: $(Convert-ToReportPath -Path $smokeReportPath)")
}

if ($failed.Count -gt 0) {
    $lines.Add('')
    $lines.Add('## Errors')
    foreach ($result in $failed) {
        $lines.Add('')
        $lines.Add("### $($result.Name)")
        $lines.Add('')
        $lines.Add("- Log: $(Convert-ToReportPath -Path $result.Log)")
        $lines.Add('')
        $lines.Add('```text')
        $lines.Add($result.Error)
        $lines.Add('```')
    }
}

Write-Utf8File -Path $reportPath -Content ($lines -join [Environment]::NewLine)

Write-Host 'Android local validation complete.'
Write-Host "Result: $(if ($overallPass) { 'PASS' } else { 'FAIL' })"
Write-Host "Report: $reportPath"

if (-not $overallPass) {
    throw "Android local validation failed. See report: $reportPath"
}
