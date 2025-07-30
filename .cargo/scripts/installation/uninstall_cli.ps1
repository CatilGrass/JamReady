# 从用户PATH环境变量移除当前项目bin目录
$scriptPath = Split-Path -Parent $MyInvocation.MyCommand.Definition
$targetBinPath = Resolve-Path (Join-Path $scriptPath "..\bin") -ErrorAction Stop

# 获取当前用户PATH并转为列表
$userPath = [Environment]::GetEnvironmentVariable("PATH", "User")
$pathList = $userPath -split ';' | Where-Object { $_ }  # 过滤空值

# 创建标准化的目标路径（移除尾部反斜杠）
$normalizedTarget = $targetBinPath.Path.TrimEnd('\') -replace '\\+$'

# 过滤掉匹配路径（大小写不敏感）
$newPathList = $pathList | Where-Object {
    $normalizedCurrent = $_.TrimEnd('\') -replace '\\+$'
    return $normalizedCurrent -ne $normalizedTarget
}

if ($newPathList.Count -lt $pathList.Count) {
    # 更新用户环境变量
    $newPath = $newPathList -join ';'
    [Environment]::SetEnvironmentVariable("PATH", $newPath, "User")
    Write-Host "Uninstalled" -ForegroundColor Green
}

Write-Host "`nPress any key to continue..."
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")