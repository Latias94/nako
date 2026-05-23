param(
    [ValidateSet('dry-run', 'publish')]
    [string] $Mode = 'dry-run',

    [switch] $AllowDirty
)

$ErrorActionPreference = 'Stop'

$Script = Join-Path $PSScriptRoot 'publish_crates.py'
$Arguments = @($Script, '--mode', $Mode)
if ($AllowDirty) {
    $Arguments += '--allow-dirty'
}

python @Arguments
