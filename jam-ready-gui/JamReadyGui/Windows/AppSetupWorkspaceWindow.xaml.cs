using System;
using System.Windows;
using JamReadyGui.Data;

namespace JamReadyGui.Windows;

public partial class AppSetupWorkspaceWindow : Window
{
    public AppSetupWorkspaceWindow(string workingDirectory)
    {
        var workingDirectory1 = workingDirectory;
        
        InitializeComponent();
        
        // 部署服务端
        ServerWorkspaceSetupButton.Click += (_, _) =>
        {
            var serverWorkspaceName = ServerWorkspaceNameInput.Text.Trim();
            
            AppCoreInvoker.Execute(new[] { "server", serverWorkspaceName });
            AppServerWorkspace workspaceWindow = new AppServerWorkspace(workingDirectory1);
            workspaceWindow.Show();
            Close();
        };
        
        // 部署客户端
        ClientWorkspaceSetupButton.Click += (_, _) =>
        {
            var clientJoinWorkspaceName = ClientWorkspaceNameInput.Text.Trim();
            var clientLoginCode = ClientWorkspaceLoginCodeInput.Text.Trim();
            var clientTargetAddress = ClientWorkspaceAddressInput.Text.Trim();
            
            AppCoreInvoker.Execute(new[] { "client", clientLoginCode, "--target", clientTargetAddress, "--workspace", clientJoinWorkspaceName });
            AppClientWorkspace workspaceWindow = new AppClientWorkspace();
            workspaceWindow.Show();
            Close();
        };
        
        // 退出时清理目录
        Closed += (_, _) =>
        {
            var workspaceType = AppCoreInvoker.Execute(new[] { "type" })?.Output;
            
            // 如果退出时没有创建任何工作区，则无视当前打开的目录
            if (workspaceType == null || workspaceType == "null")
            {
                AppPreference.WorkspacePreference.ExitWorkspace();
            }
        };
    }
}