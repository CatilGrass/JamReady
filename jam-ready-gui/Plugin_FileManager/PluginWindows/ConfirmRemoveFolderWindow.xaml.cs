using System.IO;
using System.Windows;
using JamReadyGui.AppData.Explorer;

namespace Plugin_FileManager.PluginWindows;

public partial class ConfirmRemoveFolderWindow : Window
{
    private readonly DirectoryInfo _directoryInfo;
    
    public ConfirmRemoveFolderWindow(DirectoryInfo directoryInfo)
    {
        _directoryInfo = directoryInfo;
        
        InitializeComponent();

        Title = ExplorerRuntime.Lang(Plugin.PluginName, "remove_folder");
        
        ConfirmText.Content = ExplorerRuntime.Lang(Plugin.PluginName, "remove_folder_hint")
            .Replace("[folder]", _directoryInfo.Name);
        ConfirmButton.Content = ExplorerRuntime.Lang(Plugin.PluginName, "confirm");
        CancelButton.Content = ExplorerRuntime.Lang(Plugin.PluginName, "cancel");
    }

    private void ConfirmButton_OnClick(object sender, RoutedEventArgs e)
    {
        Application.Current.Dispatcher.InvokeAsync(() =>
        {
            Directory.Delete(_directoryInfo.FullName, true);
            ExplorerRuntime.Path = ExplorerRuntime.CurrentPath;
            ExplorerRuntime.CurrentExplorer?.RefreshExplorerItems();
        });
        
        Close();
    }

    private void CancelButton_OnClick(object sender, RoutedEventArgs e)
    {
        Close();
    }
}