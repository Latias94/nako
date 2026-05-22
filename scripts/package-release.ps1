#!/usr/bin/env pwsh
[CmdletBinding(SupportsShouldProcess = $true)]
param(
    [string]$OutputDir = 'target/package-release',

    [string]$Version,

    [switch]$SkipBuild
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot '..')
Set-Location $RepoRoot

function Invoke-Step {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Name,

        [Parameter(Mandatory = $true)]
        [scriptblock]$Action
    )

    Write-Host ""
    Write-Host "==> $Name"
    & $Action
    if ($LASTEXITCODE -ne 0) {
        throw "$Name failed with exit code $LASTEXITCODE."
    }
}

function Get-WorkspaceVersion {
    if (-not [string]::IsNullOrWhiteSpace($Version)) {
        return $Version
    }

    foreach ($line in Get-Content -Path 'Cargo.toml') {
        if ($line -match '^\s*version\s*=\s*"([^"]+)"') {
            return $Matches[1]
        }
    }

    throw 'Failed to find workspace package version in Cargo.toml.'
}

function Get-HostTargetTriple {
    $rustcVersion = & rustc -vV
    if ($LASTEXITCODE -ne 0) {
        throw 'rustc -vV failed.'
    }

    foreach ($line in $rustcVersion) {
        if ($line -match '^host:\s+(.+)$') {
            return $Matches[1]
        }
    }

    throw 'Failed to read host target triple from rustc -vV.'
}

function Get-GitRevision {
    $revision = (& git rev-parse HEAD 2>$null)
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($revision)) {
        return 'unknown'
    }
    return $revision.Trim()
}

function Test-GitDirty {
    & git diff --quiet --ignore-submodules -- 2>$null
    $worktreeDirty = $LASTEXITCODE -ne 0
    & git diff --cached --quiet --ignore-submodules -- 2>$null
    $indexDirty = $LASTEXITCODE -ne 0
    return $worktreeDirty -or $indexDirty
}

function Assert-SubPath {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Child,

        [Parameter(Mandatory = $true)]
        [string]$Parent
    )

    $childFull = [System.IO.Path]::GetFullPath($Child)
    $parentFull = [System.IO.Path]::GetFullPath($Parent)
    if (-not $parentFull.EndsWith([System.IO.Path]::DirectorySeparatorChar)) {
        $parentFull = $parentFull + [System.IO.Path]::DirectorySeparatorChar
    }
    if (-not $childFull.StartsWith($parentFull, [System.StringComparison]::OrdinalIgnoreCase)) {
        throw "Refusing to operate outside package output directory: $childFull"
    }
}

function Copy-ReleasePath {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Source,

        [Parameter(Mandatory = $true)]
        [string]$Destination
    )

    if (-not (Test-Path -LiteralPath $Source)) {
        throw "Required release input does not exist: $Source"
    }

    $destinationParent = Split-Path -Parent $Destination
    New-Item -ItemType Directory -Force -Path $destinationParent | Out-Null
    Copy-Item -LiteralPath $Source -Destination $Destination -Recurse -Force
}

function Get-RelativeFileList {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Root
    )

    $rootFull = [System.IO.Path]::GetFullPath($Root)
    if (-not $rootFull.EndsWith([System.IO.Path]::DirectorySeparatorChar)) {
        $rootFull = $rootFull + [System.IO.Path]::DirectorySeparatorChar
    }

    Get-ChildItem -LiteralPath $Root -File -Recurse |
        ForEach-Object {
            [System.IO.Path]::GetFullPath($_.FullName).Substring($rootFull.Length).Replace('\', '/')
        } |
        Sort-Object
}

$packageVersion = Get-WorkspaceVersion
$targetTriple = Get-HostTargetTriple
$gitRevision = Get-GitRevision
$shortRevision = if ($gitRevision.Length -ge 12) { $gitRevision.Substring(0, 12) } else { $gitRevision }
$packageId = "nako-server-v$packageVersion-$targetTriple-$shortRevision"

$outputRoot = Join-Path $RepoRoot $OutputDir
$stagingParent = Join-Path $outputRoot 'staging'
$stagingRoot = Join-Path $stagingParent $packageId
$archivePath = Join-Path $outputRoot "$packageId.zip"
$manifestOutputPath = Join-Path $outputRoot "$packageId.release-manifest.json"
$checksumsPath = Join-Path $outputRoot 'SHA256SUMS'
$binaryName = if ($IsWindows) { 'nako-server.exe' } else { 'nako-server' }
$binaryPath = Join-Path $RepoRoot "target/release/$binaryName"

Write-Host "Nako release package"
Write-Host "Package: $packageId"
Write-Host "Output: $outputRoot"
Write-Host "SkipBuild: $SkipBuild"

if ($WhatIfPreference) {
    Write-Host ""
    Write-Host 'WhatIf: would build/copy release files, write manifest, archive, and SHA256SUMS.'
    exit 0
}

if (-not $SkipBuild) {
    Invoke-Step 'cargo build --locked --release -p nako-server' {
        cargo build --locked --release -p nako-server
    }
}

if (-not (Test-Path -LiteralPath $binaryPath)) {
    throw "Release binary does not exist: $binaryPath"
}

New-Item -ItemType Directory -Force -Path $outputRoot | Out-Null
Assert-SubPath -Child $stagingRoot -Parent $outputRoot
if (Test-Path -LiteralPath $stagingRoot) {
    Remove-Item -LiteralPath $stagingRoot -Recurse -Force
}
New-Item -ItemType Directory -Force -Path (Join-Path $stagingRoot 'bin') | Out-Null

Copy-ReleasePath -Source $binaryPath -Destination (Join-Path $stagingRoot "bin/$binaryName")

$releaseInputs = @(
    @{ Source = 'LICENSE'; Destination = 'LICENSE' },
    @{ Source = 'README.md'; Destination = 'README.md' },
    @{ Source = 'Dockerfile'; Destination = 'Dockerfile' },
    @{ Source = '.dockerignore'; Destination = '.dockerignore' },
    @{ Source = 'deploy/sqlite/nako.toml'; Destination = 'deploy/sqlite/nako.toml' },
    @{ Source = 'deploy/postgres/nako.toml'; Destination = 'deploy/postgres/nako.toml' },
    @{ Source = 'deploy/container'; Destination = 'deploy/container' },
    @{ Source = 'deploy/compose/.env.example'; Destination = 'deploy/compose/.env.example' },
    @{ Source = 'deploy/compose/nako-sqlite.yml'; Destination = 'deploy/compose/nako-sqlite.yml' },
    @{ Source = 'deploy/compose/nako-postgres.yml'; Destination = 'deploy/compose/nako-postgres.yml' },
    @{ Source = 'docs/deployment/SELF_HOSTED.md'; Destination = 'docs/deployment/SELF_HOSTED.md' },
    @{ Source = 'docs/deployment/RELEASE_ARTIFACTS.md'; Destination = 'docs/deployment/RELEASE_ARTIFACTS.md' },
    @{ Source = 'docs/deployment/BACKUP_RESTORE_UPGRADE.md'; Destination = 'docs/deployment/BACKUP_RESTORE_UPGRADE.md' }
)

foreach ($entry in $releaseInputs) {
    Copy-ReleasePath `
        -Source (Join-Path $RepoRoot $entry.Source) `
        -Destination (Join-Path $stagingRoot $entry.Destination)
}

$manifestPath = Join-Path $stagingRoot 'release-manifest.json'
$manifest = [ordered]@{
    schema_version = 1
    package = 'nako-server'
    version = $packageVersion
    git_revision = $gitRevision
    git_dirty = Test-GitDirty
    target_triple = $targetTriple
    built_at_utc = (Get-Date).ToUniversalTime().ToString('o')
    archive_file = (Split-Path -Leaf $archivePath)
    binary = "bin/$binaryName"
    build_command = if ($SkipBuild) { 'skipped; existing target/release binary was packaged' } else { 'cargo build --locked --release -p nako-server' }
    preflight_command = 'nako-server --config /config/nako.toml config-check --create-dirs'
    included_files = @()
}
$manifest | ConvertTo-Json -Depth 8 | Set-Content -Path $manifestPath -Encoding utf8
$manifest.included_files = @(Get-RelativeFileList -Root $stagingRoot)
$manifest | ConvertTo-Json -Depth 8 | Set-Content -Path $manifestPath -Encoding utf8
$manifest | ConvertTo-Json -Depth 8 | Set-Content -Path $manifestOutputPath -Encoding utf8

if (Test-Path -LiteralPath $archivePath) {
    Remove-Item -LiteralPath $archivePath -Force
}
Compress-Archive -Path $stagingRoot -DestinationPath $archivePath -Force

$checksumLines = foreach ($path in @($archivePath, $manifestOutputPath)) {
    $hash = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
    "$hash  $(Split-Path -Leaf $path)"
}
$checksumLines | Set-Content -Path $checksumsPath -Encoding ascii

Write-Host ""
Write-Host "Archive: $archivePath"
Write-Host "Manifest: $manifestOutputPath"
Write-Host "Checksums: $checksumsPath"
