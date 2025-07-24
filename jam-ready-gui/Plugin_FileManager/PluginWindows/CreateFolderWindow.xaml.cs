using System.IO;
using System.Windows;
using JamReadyGui.AppData.Explorer;

namespace Plugin_FileManager.PluginWindows;

public partial class CreateFolderWindow
{
    private readonly DirectoryInfo _directoryInfo;
    
    public CreateFolderWindow(DirectoryInfo directoryInfo)
    {
        _directoryInfo = directoryInfo;
        
        InitializeComponent();

        Title = ExplorerRuntime.Lang(Plugin.PluginName, "create_folder");
        
        NameInput.Text = ExplorerRuntime.Lang(Plugin.PluginName, "new_folder_name");
        CreateButton.Content = ExplorerRuntime.Lang(Plugin.PluginName, "create");
    }

    private void CreateButton_OnClick(object sender, RoutedEventArgs e)
    {
        if (!string.IsNullOrWhiteSpace(NameInput.Text))
        {
            var targetFolder = new DirectoryInfo(_directoryInfo.FullName + "\\" + NameInput.Text);
            if (! targetFolder.Exists)
            {
                Directory.CreateDirectory(targetFolder.FullName);
                if (ExplorerRuntime.CurrentExplorer != null)
                {
                    ExplorerRuntime.Path = ExplorerRuntime.CurrentPath;
                    ExplorerRuntime.CurrentExplorer.RefreshExplorerItems();
                }
                Close();
            }
        }
    }
}