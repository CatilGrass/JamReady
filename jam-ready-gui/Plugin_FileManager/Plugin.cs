using JamReadyGui.AppData.Explorer;
using Plugin_FileManager.PluginInserters;

namespace Plugin_FileManager;

public class Plugin
{
    /// <summary>
    /// 注册插件
    /// </summary>
    public static void Register()
    {
        ExplorerRegistry.Inserters.Add(new FilePathInserter());
    }
}