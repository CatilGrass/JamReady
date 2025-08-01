$scriptPath = "$PSScriptRoot\completion_script_jam.ps1"
$profilePath = $PROFILE

if (-not (Test-Path $profilePath)) {
    New-Item -ItemType File -Path $profilePath -Force | Out-Null
}

$profileContent = Get-Content $profilePath -Raw
$newEntry = "# jam.exe completion script`n. $scriptPath"

$pattern = '(?ms)# jam\.exe completion script\s*\.\s*".*?completion_script_jam\.ps1"'

if ($profileContent -match $pattern) {
    $updatedContent = [Regex]::Replace($profileContent, $pattern, $newEntry)
    Set-Content -Path $profilePath -Value $updatedContent
    Write-Host "Updated jam completion path in profile to: $scriptPath"
} else {
    Add-Content -Path $profilePath -Value "`n$newEntry"
    Write-Host "Added jam completion script to profile: $scriptPath"
}

Write-Host "Installation complete! Please restart PowerShell or run the following command to take effect immediately:"
Write-Host "`n. $scriptPath"