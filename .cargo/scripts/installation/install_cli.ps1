$scriptPath = Split-Path -Parent $MyInvocation.MyCommand.Definition
$targetBinPath = Resolve-Path (Join-Path $scriptPath "..\bin") -ErrorAction Stop

$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
$pathList = $userPath -split ';' | Where-Object { $_ }  # 过滤空值

$exists = $pathList | Where-Object {
    $normalizedCurrent = $_.TrimEnd('\') -replace '\\+$'
    $normalizedTarget = $targetBinPath.Path.TrimEnd('\') -replace '\\+$'
    return $normalizedCurrent -eq $normalizedTarget
}

if (-not $exists) {
    $newPathList = $pathList + $targetBinPath.Path
    $newPath = $newPathList -join ';'

    [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
    Write-Host "Installed." -ForegroundColor Green
} else {
    Write-Host "Already installed!" -ForegroundColor Cyan
}