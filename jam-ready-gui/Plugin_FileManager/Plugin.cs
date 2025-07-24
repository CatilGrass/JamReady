using JamReadyGui.AppData.Explorer;
using Plugin_FileManager.PluginInserters;
using Plugin_FileManager.PluginPathMenu;

namespace Plugin_FileManager;

public class Plugin
{
    public const string PluginName = "Plugin_FileManager";
    public const string PluginPrefix = "FS";
    
    /// <summary>
    /// 注册插件
    /// </summary>
    public static void Register()
    {
        ExplorerRegistry.Inserters.Add(new DirectorySelectInserter());
        ExplorerRegistry.Inserters.Add(new DiskSelectInserter());
        
        ExplorerRegistry.PathMenus.Add(new FolderManageMenu());
    }
}