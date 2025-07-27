using System;
using System.Collections.Generic;
using System.IO;
using JamReadyGui.AppData;
using JamReadyGui.AppData.Explorer;
using JamReadyGui.AppData.Utils;
using Plugin_FileManager.PluginWindows;

namespace Plugin_FileManager.PluginAdapters;

public class DirectoryAdapter : ItemAdapter
{
    private DirectoryInfo? _jumpTo;
    
    public override ImagePath OnInit(object value)
    {
        var iconFile = AppConstants.GetPluginResourceFile(Plugin.PluginName, "FileSystem_Folder.png");
        if (value is DirectoryInfo directory && iconFile != null)
        {
            Name = directory.Name;
            _jumpTo = directory;
            return new ImagePath(new Uri(iconFile.FullName));
        }
        return ImagePath.Empty;
    }

    public override bool OnEnter()
    {
        if (_jumpTo is { Exists: true })
        {
            var path = new ExplorerPath("FS")
            {
                Path = _jumpTo.FullName
            };
            ExplorerRuntime.Path = path.ToString();
            return true;
        }
        return false;
    }

    public override List<string> OnRegisterOperation()
    {
        return new List<string>
        {
            ExplorerRuntime.Lang(Plugin.PluginName, "Adapter_DirectoryAdapter_MenuItem_RemoveFolder_Name")
        };
    }

    public override bool OnOperate(int operationIndex)
    {
        // 删除文件夹
        if (operationIndex == 0)
        {
            if (_jumpTo != null) new ConfirmRemoveFolderWindow(_jumpTo).Show();
        }
        return false;
    }
}