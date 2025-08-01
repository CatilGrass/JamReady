try {
    Write-Host "Installing cli..."
    & "$PSScriptRoot\install_cli.ps1"
    Write-Host "Installed cli."
} catch {
    Write-Host "Install cli failed."
    Write-Host "`nPress any key to continue..."
    $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
    exit 1
}

try {
    Write-Host "Installing completion script..."
    & "$PSScriptRoot\install_completions.ps1"
    Write-Host "Installed completion script."
} catch {
    Write-Host "Install completion script failed."
    Write-Host "`nPress any key to continue..."
    $null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")
    exit 1
}

Write-Host "Done."
Write-Host "`nPress any key to continue..."
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")