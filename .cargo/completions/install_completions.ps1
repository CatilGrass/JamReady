$ErrorActionPreference = "Stop"

try {
    # Check if jam.exe exists in the current directory
    if (-not (Test-Path -Path ".\jam.exe" -PathType Leaf)) {
        throw "Error: jam.exe program not found in the current directory"
    }

    # Generate completion script
    Write-Host "Generating jam.ps1 file..."
    .\jam.exe generate-client-completions powershell | Out-File jam.ps1 -Encoding utf8 -Force

    # Get target profile file path
    $targetProfile = $PROFILE.CurrentUserAllHosts

    # Ensure the profile directory exists
    $profileDir = Split-Path $targetProfile -Parent
    if (-not (Test-Path $profileDir)) {
        New-Item -ItemType Directory -Path $profileDir -Force | Out-Null
    }

    # Copy the generated script to the profile file
    Write-Host "Installing to user profile file..."
    Copy-Item jam.ps1 $targetProfile -Force

    # Clean up temporary files (optional)
    # Remove-Item jam.ps1 -Force

    Write-Host "`nComplete! Auto-completion has been successfully installed to $targetProfile" -ForegroundColor Green
    Write-Host "Effective in new terminal sessions (restart PowerShell to use)"
}
catch {
    Write-Host "`nError: $_" -ForegroundColor Red
    exit 1
}

# Pause the script
Write-Host "`nPress any key to continue..."
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")