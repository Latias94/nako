[CmdletBinding()]
param(
    [string]$Serial,
    [int]$AdbServerPort,
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

function Convert-ToJsonPath {
    param(
        [string]$Path
    )

    if ([string]::IsNullOrWhiteSpace($Path)) {
        return $null
    }

    return $Path.Replace('\', '/')
}

function Get-ValidationJUnitCaseName {
    param(
        [string]$StepName
    )

    switch ($StepName) {
        'Android debug assemble' { return 'step.android-build' }
        'Android JVM tests' { return 'step.android-unit-tests' }
        'Android smoke regression' { return 'step.smoke-regression' }
        default {
            $slug = ($StepName.ToLowerInvariant() -replace '[^a-z0-9]+', '-').Trim('-')
            return "step.$slug"
        }
    }
}

function Write-LocalValidationJUnitReport {
    param(
        [string]$Path,
        [datetime]$StartedAt,
        [datetime]$FinishedAt,
        [string]$ReportPath,
        [string]$JsonPath,
        [string]$SmokeReportPath,
        [string]$SmokeJsonReportPath,
        [string]$SmokeJUnitReportPath,
        [object[]]$Results
    )

    $failureCount = @($Results | Where-Object { $_.Status -eq 'FAIL' }).Count
    $skippedCount = @($Results | Where-Object { $_.Status -in @('SKIPPED', 'NOT_RUN') }).Count
    $duration = [Math]::Max(0, ($FinishedAt - $StartedAt).TotalSeconds)
    $durationText = $duration.ToString('0.###', [System.Globalization.CultureInfo]::InvariantCulture)

    $document = [System.Xml.XmlDocument]::new()
    [void]$document.AppendChild($document.CreateXmlDeclaration('1.0', 'utf-8', $null))
    $root = $document.CreateElement('testsuites')
    $root.SetAttribute('tests', $Results.Count.ToString())
    $root.SetAttribute('failures', $failureCount.ToString())
    $root.SetAttribute('errors', '0')
    $root.SetAttribute('skipped', $skippedCount.ToString())
    $root.SetAttribute('time', $durationText)
    [void]$document.AppendChild($root)

    $suite = $document.CreateElement('testsuite')
    $suite.SetAttribute('name', 'taru.android.local-validation')
    $suite.SetAttribute('tests', $Results.Count.ToString())
    $suite.SetAttribute('failures', $failureCount.ToString())
    $suite.SetAttribute('errors', '0')
    $suite.SetAttribute('skipped', $skippedCount.ToString())
    $suite.SetAttribute('time', $durationText)
    $suite.SetAttribute('timestamp', $StartedAt.ToString('o'))
    [void]$root.AppendChild($suite)

    $properties = $document.CreateElement('properties')
    Add-JUnitProperty -Document $document -Properties $properties -Name 'report.markdown' -Value (Convert-ToJsonPath -Path $ReportPath)
    Add-JUnitProperty -Document $document -Properties $properties -Name 'report.json' -Value (Convert-ToJsonPath -Path $JsonPath)
    Add-JUnitProperty -Document $document -Properties $properties -Name 'started_at' -Value $StartedAt.ToString('o')
    Add-JUnitProperty -Document $document -Properties $properties -Name 'finished_at' -Value $FinishedAt.ToString('o')
    Add-JUnitProperty -Document $document -Properties $properties -Name 'smoke.report.markdown' -Value (Convert-ToJsonPath -Path $SmokeReportPath)
    Add-JUnitProperty -Document $document -Properties $properties -Name 'smoke.report.json' -Value (Convert-ToJsonPath -Path $SmokeJsonReportPath)
    Add-JUnitProperty -Document $document -Properties $properties -Name 'smoke.report.junit' -Value (Convert-ToJsonPath -Path $SmokeJUnitReportPath)
    [void]$suite.AppendChild($properties)

    foreach ($result in $Results) {
        $caseName = Get-ValidationJUnitCaseName -StepName $result.Name
        $details = @(
            "step=$($result.Name)",
            "status=$($result.Status)",
            "log=$(Convert-ToJsonPath -Path $result.Log)"
        )
        if ($result.Name -eq 'Android smoke regression') {
            $details += "smoke_report_markdown=$(Convert-ToJsonPath -Path $SmokeReportPath)"
            $details += "smoke_report_json=$(Convert-ToJsonPath -Path $SmokeJsonReportPath)"
            $details += "smoke_report_junit=$(Convert-ToJsonPath -Path $SmokeJUnitReportPath)"
        }
        if (-not [string]::IsNullOrWhiteSpace($result.Error)) {
            $details += "error=$($result.Error)"
        }

        if ($result.Status -eq 'FAIL') {
            Add-JUnitTestCase `
                -Document $document `
                -Suite $suite `
                -ClassName 'taru.android.validation' `
                -Name $caseName `
                -Outcome 'failure' `
                -Type $result.Status `
                -Message "Validation step '$($result.Name)' failed." `
                -Details $details
        } elseif ($result.Status -in @('SKIPPED', 'NOT_RUN')) {
            $message = if ($result.Name -eq 'Android smoke regression' -and $result.Status -eq 'SKIPPED') {
                'SkipSmoke'
            } elseif (-not [string]::IsNullOrWhiteSpace($result.Error)) {
                $result.Error
            } else {
                "Validation step '$($result.Name)' was skipped."
            }
            Add-JUnitTestCase `
                -Document $document `
                -Suite $suite `
                -ClassName 'taru.android.validation' `
                -Name $caseName `
                -Outcome 'skipped' `
                -Type $result.Status `
                -Message $message `
                -Details $details
        } else {
            Add-JUnitTestCase `
                -Document $document `
                -Suite $suite `
                -ClassName 'taru.android.validation' `
                -Name $caseName `
                -Outcome 'pass' `
                -Type $null `
                -Message $null `
                -Details @()
        }
    }

    Write-JUnitXmlFile -Document $document -Path $Path
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

function Get-SmokeJsonReportPath {
    param(
        [string]$LogPath
    )

    if (-not (Test-Path -LiteralPath $LogPath)) {
        return $null
    }

    $reportLine = Get-Content -LiteralPath $LogPath |
        Where-Object { $_ -match '^Structured report:\s+' } |
        Select-Object -Last 1
    if (-not $reportLine) {
        return $null
    }

    return ($reportLine -replace '^Structured report:\s+', '').Trim()
}

function Get-SmokeJUnitReportPath {
    param(
        [string]$LogPath
    )

    if (-not (Test-Path -LiteralPath $LogPath)) {
        return $null
    }

    $reportLine = Get-Content -LiteralPath $LogPath |
        Where-Object { $_ -match '^JUnit report:\s+' } |
        Select-Object -Last 1
    if (-not $reportLine) {
        return $null
    }

    return ($reportLine -replace '^JUnit report:\s+', '').Trim()
}

$scriptDir = $PSScriptRoot
. (Join-Path $scriptDir 'Android-JUnitReport.ps1')
$androidRoot = (Resolve-Path -LiteralPath (Join-Path $scriptDir '..')).Path
$smokeRegressionScript = Join-Path $scriptDir 'Smoke-Regression.ps1'

if (-not (Test-Path -LiteralPath $smokeRegressionScript)) {
    throw "Smoke regression script was not found at '$smokeRegressionScript'."
}

if ([string]::IsNullOrWhiteSpace($OutputRoot)) {
    $OutputRoot = Join-Path $androidRoot 'build\validation'
}
$OutputRoot = Resolve-OutputRootPath -Path $OutputRoot

$timestamp = Get-Date -Format 'yyyyMMdd-HHmmss'
$runRoot = Join-Path $OutputRoot $timestamp
New-Item -ItemType Directory -Force -Path $runRoot | Out-Null

$startedAt = Get-Date
$results = New-Object System.Collections.Generic.List[object]
$smokeReportPath = $null
$smokeJsonReportPath = $null
$smokeJUnitReportPath = $null

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

            if ($AdbServerPort -gt 0) {
                $smokeArgs.AdbServerPort = $AdbServerPort
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
    $smokeJsonReportPath = Get-SmokeJsonReportPath -LogPath $smokeLogPath
    $smokeJUnitReportPath = Get-SmokeJUnitReportPath -LogPath $smokeLogPath
}

$finishedAt = Get-Date
$failed = @($results | Where-Object { $_.Status -eq 'FAIL' })
$overallPass = $failed.Count -eq 0
$reportPath = Join-Path $runRoot 'report.md'
$jsonPath = Join-Path $runRoot 'report.json'
$junitPath = Join-Path $runRoot 'report.junit.xml'
$stepResults = @(
    foreach ($result in $results) {
        [pscustomobject]@{
            name = $result.Name
            status = $result.Status
            started_at = $result.StartedAt.ToString('o')
            finished_at = $result.FinishedAt.ToString('o')
            log = Convert-ToJsonPath -Path $result.Log
            error = $result.Error
        }
    }
)
$jsonReport = [ordered]@{
    schema_version = 1
    kind = 'taru_android_local_validation'
    started_at = $startedAt.ToString('o')
    finished_at = $finishedAt.ToString('o')
    result = if ($overallPass) { 'PASS' } else { 'FAIL' }
    report_markdown = Convert-ToJsonPath -Path $reportPath
    report_json = Convert-ToJsonPath -Path $jsonPath
    report_junit = Convert-ToJsonPath -Path $junitPath
    options = [ordered]@{
        requested_serial = if ([string]::IsNullOrWhiteSpace($Serial)) { $null } else { $Serial }
        smoke_states = @($SmokeStates)
        skip_unit_tests = [bool]$SkipUnitTests
        skip_assemble = [bool]$SkipAssemble
        skip_smoke = [bool]$SkipSmoke
        skip_fixture_server_build = [bool]$SkipFixtureServerBuild
        retries_per_state = $RetriesPerState
    }
    steps = $stepResults
    delegated_reports = [ordered]@{
        smoke_markdown = Convert-ToJsonPath -Path $smokeReportPath
        smoke_json = Convert-ToJsonPath -Path $smokeJsonReportPath
        smoke_junit = Convert-ToJsonPath -Path $smokeJUnitReportPath
    }
}
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
    if (-not [string]::IsNullOrWhiteSpace($smokeJsonReportPath)) {
        $lines.Add("- Structured report: $(Convert-ToReportPath -Path $smokeJsonReportPath)")
    }
    if (-not [string]::IsNullOrWhiteSpace($smokeJUnitReportPath)) {
        $lines.Add("- JUnit report: $(Convert-ToReportPath -Path $smokeJUnitReportPath)")
    }
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
Write-Utf8File -Path $jsonPath -Content ($jsonReport | ConvertTo-Json -Depth 12)
Write-LocalValidationJUnitReport `
    -Path $junitPath `
    -StartedAt $startedAt `
    -FinishedAt $finishedAt `
    -ReportPath $reportPath `
    -JsonPath $jsonPath `
    -SmokeReportPath $smokeReportPath `
    -SmokeJsonReportPath $smokeJsonReportPath `
    -SmokeJUnitReportPath $smokeJUnitReportPath `
    -Results $results

Write-Host 'Android local validation complete.'
Write-Host "Result: $(if ($overallPass) { 'PASS' } else { 'FAIL' })"
Write-Host "Report: $reportPath"
Write-Host "Structured report: $jsonPath"
Write-Host "JUnit report: $junitPath"

if (-not $overallPass) {
    throw "Android local validation failed. See report: $reportPath"
}
