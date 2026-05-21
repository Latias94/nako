[CmdletBinding()]
param(
    [string]$Package = 'taru-client-uniffi',
    [string[]]$AllowedDirectDependencies = @('taru-client-core', 'uniffi'),
    [string[]]$ForbiddenTransitiveDependencies = @(
        'reqwest',
        'tokio',
        'hyper',
        'hyper-util',
        'tower',
        'axum',
        'sqlx',
        'rusqlite',
        'jni',
        'ndk',
        'android-activity'
    )
)

$ErrorActionPreference = 'Stop'

function Invoke-CargoJson {
    param([string[]]$Arguments)

    $output = & cargo @Arguments 2>&1
    if ($LASTEXITCODE -ne 0) {
        throw "cargo $($Arguments -join ' ') failed:`n$($output -join [Environment]::NewLine)"
    }
    return ($output -join [Environment]::NewLine)
}

$metadataJson = Invoke-CargoJson -Arguments @('metadata', '--no-deps', '--format-version', '1')
$metadata = $metadataJson | ConvertFrom-Json
$packageMetadata = @($metadata.packages | Where-Object { $_.name -eq $Package }) | Select-Object -First 1
if ($null -eq $packageMetadata) {
    throw "Cargo package '$Package' was not found in workspace metadata."
}

$directDependencies = @($packageMetadata.dependencies | ForEach-Object { $_.name } | Sort-Object -Unique)
$unexpectedDirectDependencies = @($directDependencies | Where-Object { $_ -notin $AllowedDirectDependencies })
if ($unexpectedDirectDependencies.Count -gt 0) {
    throw "Package '$Package' has unexpected direct dependency/dependencies: $($unexpectedDirectDependencies -join ', '). Allowed: $($AllowedDirectDependencies -join ', ')."
}

$treeText = Invoke-CargoJson -Arguments @('tree', '-p', $Package, '--prefix', 'none')
$treeDependencyNames = New-Object System.Collections.Generic.HashSet[string]
foreach ($line in ($treeText -split "`r?`n")) {
    $trimmed = $line.Trim()
    if ([string]::IsNullOrWhiteSpace($trimmed)) {
        continue
    }

    $name = ($trimmed -split '\s+')[0]
    if (-not [string]::IsNullOrWhiteSpace($name)) {
        [void]$treeDependencyNames.Add($name)
    }
}

$forbiddenFound = @($ForbiddenTransitiveDependencies | Where-Object { $treeDependencyNames.Contains($_) })
if ($forbiddenFound.Count -gt 0) {
    throw "Package '$Package' dependency tree contains forbidden runtime/platform dependency/dependencies: $($forbiddenFound -join ', ')."
}

$summary = [pscustomobject]@{
    package = $Package
    allowed_direct_dependencies = $AllowedDirectDependencies
    direct_dependencies = $directDependencies
    forbidden_transitive_dependencies_checked = $ForbiddenTransitiveDependencies
    dependency_tree_package_count = $treeDependencyNames.Count
    status = 'PASS'
}

$summary | ConvertTo-Json -Depth 4
