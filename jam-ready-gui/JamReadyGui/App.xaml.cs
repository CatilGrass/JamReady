using System;
using System.IO;
using System.Windows;
using JamReadyGui.AppConfigure;
using JamReadyGui.Utils;
using Microsoft.WindowsAPICodePack.Dialogs;

namespace JamReadyGui
{
    public partial class App
    {
        protected override void OnStartup(StartupEventArgs e)
        {
            var preference = AppPreference.LoadPreference();
            if (preference == null) return;

            // 判断工作区目录是否存在
            var workspaceDirectory = new DirectoryInfo(preference.Workspace.CurrentWorkspace);
            if (!workspaceDirectory.Exists)
            {
                var openDirectoryDialog = new CommonOpenFileDialog
                {
                    IsFolderPicker = true,
                    Title = "选择工作区目录",
                    InitialDirectory = Environment.GetFolderPath(Environment.SpecialFolder.Personal)
                };
                
                // 打开文件夹
                if (openDirectoryDialog.ShowDialog() == CommonFileDialogResult.Ok)
                {
                    if (new DirectoryInfo(openDirectoryDialog.FileName).Exists)
                    {
                        // 修改首选项设置
                        preference.Workspace.CurrentWorkspace = openDirectoryDialog.FileName;
                        
                        // 保存首选项
                        AppPreference.WritePreference(preference);
                        
                        EntryApp();
                    }
                }
                else
                {
                    Current.Shutdown();
                }
            }
            else
            {
                // 存在则直接进入入口
                EntryApp();
            }
        }

        private void EntryApp()
        {
            var workspaceTypeResult = JamExecute.Execute("type");
            if (workspaceTypeResult != null)
            {
                var resultType = workspaceTypeResult.Value.Output;
                switch (resultType)
                {
                    // 未初始化的工作区
                    case "null": 
                        
                        break;
                    
                    // 客户端工作区
                    case "client": 
                        
                        break;
                    
                    // 服务端工作区
                    case "server": 
                        
                        break;
                }
            }
        }
    }
}