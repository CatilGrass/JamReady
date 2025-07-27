using System;
using System.Collections.Generic;
using System.IO;
using JamReadyGui.AppData.Explorer;
using JamReadyGui.AppData.Utils;
using Plugin_FileManager.PluginWindows;

namespace Plugin_FileManager.PluginPathMenu;

public class FolderManageMenu : PathMenu
{
    public override string GetMenuName()
    {
        return ExplorerRuntime.Lang(Plugin.PluginName, "Menu_File_Name");
    }

    public override List<string>? OnRegisterOperation(ExplorerPath path)
    {
        List<string> menuItemList = new();
        Do(path, _ =>
        {
            menuItemList.Add(ExplorerRuntime.Lang(Plugin.
                PluginName, "MenuItem_CreateFolder_Name"));
        });
        return menuItemList;
    }

    public override bool OnOperate(ExplorerPath path, int operationIndex)
    {
        var changed = false;
        Do(path, d =>
        {
            if (operationIndex == 0)
            {
                new CreateFolderWindow(d).Show();
            }
        });
        return changed;
    }

    private void Do(ExplorerPath path, Action<DirectoryInfo> doOperation)
    {
        if (path.Prefix == Plugin.PluginPrefix && path.Path.Trim() != "_")
        {
            var dir = new DirectoryInfo(path.Path);
            if (dir.Exists)
            {
                doOperation.Invoke(dir);
            }
        }
    }
}