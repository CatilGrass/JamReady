using System.Collections.Generic;
using System.IO;
using System.Linq;
using JamReadyGui.AppData.Explorer;
using JamReadyGui.AppData.Utils;

namespace Plugin_Workspace.PluginPathMenu;

public class EnterWorkspaceMenu : PathMenu
{
    public override string GetMenuName()
    {
        return ExplorerRuntime.Lang(Plugin.PluginName, "Menu_Enter_Workspace_Name");
    }

    public override List<string>? OnRegisterOperation(ExplorerPath path)
    {
        var dir = new DirectoryInfo(path.Path);
        if (path.Prefix != "FS" || !dir.Exists)
            return new List<string>();

        var jamDir = new DirectoryInfo(dir.FullName + "\\.jam\\");
        if (!jamDir.Exists)
            return new List<string>();

        return new List<string>
        {
            ExplorerRuntime.Lang(Plugin.PluginName, "MenuItem_EnterWorkspace_Name"),
        };
    }

    public override bool OnOperate(ExplorerPath path, int operationIndex)
    {
        var newPath = new ExplorerPath
        {
            Prefix = Plugin.PluginPrefix,
            Path = "main/",
            ["local"] = path.Path
        };
        ExplorerRuntime.Path = newPath.ToString();
        return true;
    }
}