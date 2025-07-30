# 添加当前项目bin目录到用户PATH环境变量
$scriptPath = Split-Path -Parent $MyInvocation.MyCommand.Definition
$targetBinPath = Resolve-Path (Join-Path $scriptPath "..\bin") -ErrorAction Stop

# 获取当前用户PATH并转为列表
$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
$pathList = $userPath -split ';' | Where-Object { $_ }  # 过滤空值

# 检查是否已存在（大小写不敏感）
$exists = $pathList | Where-Object {
    $normalizedCurrent = $_.TrimEnd('\') -replace '\\+$'
    $normalizedTarget = $targetBinPath.Path.TrimEnd('\') -replace '\\+$'
    return $normalizedCurrent -eq $normalizedTarget
}

if (-not $exists) {
    # 添加新路径到PATH列表
    $newPathList = $pathList + $targetBinPath.Path
    $newPath = $newPathList -join ';'
    
    # 更新用户环境变量
    [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
    Write-Host "Installed." -ForegroundColor Green
} else {
    Write-Host "Already installed!" -ForegroundColor Cyan
}

Write-Host "`nPress any key to continue..."
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")