param(
    [ValidateSet('dry-run', 'publish')]
    [string] $Mode = 'dry-run',

    [switch] $AllowDirty,

    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]] $ExtraArgs = @()
)

$ErrorActionPreference = 'Stop'

$Script = Join-Path $PSScriptRoot 'publish_crates.py'
$Arguments = @($Script, '--mode', $Mode)
if ($AllowDirty) {
    $Arguments += '--allow-dirty'
}
$Arguments += $ExtraArgs

python @Arguments
