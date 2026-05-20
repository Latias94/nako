function Add-JUnitProperty {
    param(
        [System.Xml.XmlDocument]$Document,
        [System.Xml.XmlElement]$Properties,
        [string]$Name,
        [string]$Value
    )

    if ([string]::IsNullOrWhiteSpace($Value)) {
        return
    }

    $property = $Document.CreateElement('property')
    $property.SetAttribute('name', $Name)
    $property.SetAttribute('value', $Value)
    [void]$Properties.AppendChild($property)
}

function Add-JUnitTestCase {
    param(
        [System.Xml.XmlDocument]$Document,
        [System.Xml.XmlElement]$Suite,
        [string]$ClassName,
        [string]$Name,
        [string]$Outcome,
        [string]$Type,
        [string]$Message,
        [string[]]$Details = @()
    )

    $testcase = $Document.CreateElement('testcase')
    $testcase.SetAttribute('classname', $ClassName)
    $testcase.SetAttribute('name', $Name)
    $testcase.SetAttribute('time', '0')

    if ($Outcome -eq 'failure') {
        $failure = $Document.CreateElement('failure')
        $failure.SetAttribute('type', $(if ([string]::IsNullOrWhiteSpace($Type)) { 'unknown' } else { $Type }))
        $failure.SetAttribute('message', $(if ([string]::IsNullOrWhiteSpace($Message)) { 'Validation failed.' } else { $Message }))
        if ($Details.Count -gt 0) {
            $failure.InnerText = ($Details | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }) -join [Environment]::NewLine
        }
        [void]$testcase.AppendChild($failure)
    } elseif ($Outcome -eq 'skipped') {
        $skipped = $Document.CreateElement('skipped')
        $skipped.SetAttribute('message', $(if ([string]::IsNullOrWhiteSpace($Message)) { 'Skipped.' } else { $Message }))
        if ($Details.Count -gt 0) {
            $skipped.InnerText = ($Details | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }) -join [Environment]::NewLine
        }
        [void]$testcase.AppendChild($skipped)
    }

    [void]$Suite.AppendChild($testcase)
}

function Write-JUnitXmlFile {
    param(
        [System.Xml.XmlDocument]$Document,
        [string]$Path
    )

    $settings = [System.Xml.XmlWriterSettings]::new()
    $settings.Encoding = [System.Text.UTF8Encoding]::new($false)
    $settings.Indent = $true

    $writer = [System.Xml.XmlWriter]::Create($Path, $settings)
    try {
        $Document.Save($writer)
    } finally {
        $writer.Close()
    }
}
