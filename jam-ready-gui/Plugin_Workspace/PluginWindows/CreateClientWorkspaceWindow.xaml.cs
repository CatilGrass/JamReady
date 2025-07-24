using System.Windows;
using JamReadyGui.AppData.Explorer;

namespace Plugin_Workspace.PluginWindows;

public partial class CreateClientWorkspaceWindow : Window
{
    public CreateClientWorkspaceWindow()
    {
        InitializeComponent();
            
        Title = ExplorerRuntime.Lang(Plugin.PluginName, "Window_CreateClientWorkspaceWindow_Title");
    }
}