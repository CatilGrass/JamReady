using System;
using System.IO;
using System.Windows;
using JamReadyGui.Data;
using JamReadyGui.Windows;
using Microsoft.WindowsAPICodePack.Dialogs;

namespace JamReadyGui
{
    public partial class App
    {
        protected override void OnStartup(StartupEventArgs e)
        {
            var preference = AppPreference.LoadPreference();
            if (preference == null) return;

            var workspaceDirectory = new DirectoryInfo(preference.Workspace.CurrentWorkspace);
            if (!workspaceDirectory.Exists)
            {
                var openDirectoryDialog = new CommonOpenFileDialog
                {
                    IsFolderPicker = true,
                    Title = "Select workspace directory",
                    InitialDirectory = Environment.GetFolderPath(Environment.SpecialFolder.Personal)
                };
                
                if (openDirectoryDialog.ShowDialog() == CommonFileDialogResult.Ok)
                {
                    if (new DirectoryInfo(openDirectoryDialog.FileName).Exists)
                    {
                        preference.Workspace.CurrentWorkspace = openDirectoryDialog.FileName;
                        AppPreference.WritePreference(preference);
                        
                        OpenWorkspace();
                    }
                }
                else
                {
                    Current.Shutdown();
                }
            }
            else
            {
                OpenWorkspace();
            }
        }

        private void OpenWorkspace()
        {
            var workspaceTypeResult = AppCoreInvoker.Execute("type");
            if (workspaceTypeResult != null)
            {
                var resultType = workspaceTypeResult.Value.Output;
                switch (resultType)
                {
                    case "null": 
                        AppSetupWorkspaceWindow window = new AppSetupWorkspaceWindow();
                        window.Show();
                        break;
                    
                    case "client": 
                        
                        break;
                    
                    case "server": 
                        
                        break;
                }
            }
        }
    }
}