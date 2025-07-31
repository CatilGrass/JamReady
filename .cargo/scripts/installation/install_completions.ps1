$ErrorActionPreference = "Stop"

try {
    # 检查 jam.exe 是否存在
    if (-not (Test-Path -Path ".\..\bin\jam.exe" -PathType Leaf)) {
        throw "Error: jam.exe program not found in the current directory"
    }

    # 生成补全脚本
    Write-Host "Generating jam.ps1 file..."
    .\..\bin\jam.exe generate-client-completions powershell | Out-File jam.ps1 -Encoding utf8 -Force

    $targetProfile = $PROFILE.CurrentUserAllHosts
    $profileDir = Split-Path $targetProfile -Parent
    if (-not (Test-Path $profileDir)) {
        New-Item -ItemType Directory -Path $profileDir -Force | Out-Null
    }

    Write-Host "Installing to user profile file..."
    Copy-Item jam.ps1 $targetProfile -Force

    Write-Host "`nComplete! Auto-completion has been successfully installed to $targetProfile" -ForegroundColor Green
    Write-Host "Effective in new terminal sessions (restart PowerShell to use)"
}
catch {
    Write-Host "`nError: $_" -ForegroundColor Red
    exit 1
}

Write-Host "`nPress any key to continue..."
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")