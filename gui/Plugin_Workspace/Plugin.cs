using JamReadyGui.AppData.Explorer;
using Plugin_Workspace.PluginPathMenu;

namespace Plugin_Workspace;

public class Plugin
{
    public const string PluginName = "Plugin_Workspace";
    public const string PluginPrefix = "WS";
    
    /// <summary>
    /// 注册插件
    /// </summary>
    public static void Register()
    {
        ExplorerRegistry.PathMenus.Add(new CreateWorkspaceMenu());
        ExplorerRegistry.PathMenus.Add(new EnterWorkspaceMenu());
    }
}