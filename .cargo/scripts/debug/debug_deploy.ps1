# 进入调试目录
Set-Location ..\
$debugPath = ".\debug"
New-Item $debugPath -ItemType Directory -Force | Out-Null
Set-Location $debugPath

# 删除 server 和 client 文件夹（如果存在）
if (Test-Path ".\server") { Remove-Item ".\server" -Recurse -Force }
if (Test-Path ".\client") { Remove-Item ".\client" -Recurse -Force }

# 创建新文件夹
New-Item "server" -ItemType Directory | Out-Null
New-Item "client" -ItemType Directory | Out-Null

# Server 部分
Push-Location "server"

# 配置工作区
jam setup MyWorkspace *>$null

# 添加管理员成员和职责
jam add member admin *>$null
jam add duty leader admin *>$null

# 获取登录码并存储到变量
$login_code = jam query login-code admin 2>&1 | ForEach-Object {
    if ($_ -is [System.Management.Automation.ErrorRecord]) { $_.Exception.Message }
    else { $_ }
}

# 启动服务器（在新窗口）
Write-Host "Starting server..."
Start-Process cmd.exe -ArgumentList "/c", "title Jams & jam run"

# 等待服务器初始化
Write-Host "Waiting for server initialization (2 seconds)..."
Start-Sleep -Seconds 2
Pop-Location

# Client 部分
Push-Location "client"

# 使用记录的登录码认证
Write-Host "Performing client login..."
jam login "$login_code" --workspace MyWorkspace

# 清理和暂停
Pop-Location
Write-Host "Stopping server..."
Get-Process | Where-Object { $_.MainWindowTitle -eq "Jams" } | Stop-Process -Force -ErrorAction SilentlyContinue
Write-Host "All operations completed. Press any key to exit..."
$null = $Host.UI.RawUI.ReadKey("NoEcho,IncludeKeyDown")