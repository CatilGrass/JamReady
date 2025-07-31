# 进入调试目录
Set-Location ..\
New-Item -Path ".\debug" -ItemType Directory -Force | Out-Null
Set-Location ".\debug"

# 启动客户端命令行
Set-Location ".\client"
Start-Process cmd.exe

# 启动服务端
Set-Location "..\server"
jam run