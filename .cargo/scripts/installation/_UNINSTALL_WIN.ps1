$scriptPath = Split-Path -Parent $MyInvocation.MyCommand.Definition
$targetBinPath = Resolve-Path (Join-Path $scriptPath "..\bin") -ErrorAction Stop

$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
$pathList = $userPath -split ';' | Where-Object { $_ }  # 过滤空值

$normalizedTarget = $targetBinPath.Path.TrimEnd('\') -replace '\\+$'

$newPathList = $pathList | Where-Object {
    $normalizedCurrent = $_.TrimEnd('\') -replace '\\+$'
    return $normalizedCurrent -ne $normalizedTarget
}

if ($newPathList.Count -lt $pathList.Count) {
    $newPath = $newPathList -join ';'
    [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
    Write-Host "Uninstalled" -ForegroundColor Green
}

Write-Host "`nPress any key to continue..."
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")