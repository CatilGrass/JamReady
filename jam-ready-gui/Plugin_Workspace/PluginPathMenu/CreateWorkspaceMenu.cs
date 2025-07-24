using System.Collections.Generic;
using System.IO;
using JamReadyGui.AppData.Explorer;
using JamReadyGui.AppData.Utils;
using Plugin_Workspace.PluginWindows;

namespace Plugin_Workspace.PluginPathMenu;

public class CreateWorkspaceMenu : PathMenu
{
    public override string GetMenuName()
    {
        return ExplorerRuntime.Lang(Plugin.PluginName, "Menu_Workspace_Name");
    }

    public override List<string> OnRegisterOperation(ExplorerPath path)
    {
        if (! new DirectoryInfo(path.Path).Exists)
            return new List<string>();
        
        return new List<string>
        {
            ExplorerRuntime.Lang(Plugin.PluginName, "MenuItem_JoinWorkspace_Name"),
            ExplorerRuntime.Lang(Plugin.PluginName, "MenuItem_CreateWorkspace_Name")
        };
    }

    public override bool OnOperate(ExplorerPath path, int operationIndex)
    {
        switch (operationIndex)
        {
            case 0: 
                CreateClientWorkspace(path);
                break;
            case 1:
                CreateServerWorkspace(path);
                break;
        }
        return false;
    }

    private void CreateClientWorkspace(ExplorerPath path)
    {
        new CreateClientWorkspaceWindow().Show();
    }
    
    private void CreateServerWorkspace(ExplorerPath path)
    {
        
    }
}