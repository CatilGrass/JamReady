using System;
using System.IO;
using JamReadyGui.AppData;
using JamReadyGui.AppData.Explorer;
using JamReadyGui.AppData.Utils;

namespace Plugin_FileManager.PluginAdapters;

public class ParentDirectoryAdapter : ItemAdapter
{
    private DirectoryInfo? _jumpTo;
    
    public override ImagePath OnInit(object value)
    {
        var iconFile = AppConstants.GetPluginResourceFile(Plugin.PluginName, "FileSystem_Folder_Back.png");
        
        // 上一级文件
        if (value is DirectoryInfo directory && iconFile != null)
        {
            Name = "..";
            _jumpTo = directory;
            return new ImagePath(new Uri(iconFile.FullName));
        }
        
        // 彩蛋
        if (value is (DirectoryInfo dir, string easterEggWord))
        {
            if (easterEggWord == "Safety")
            {
                iconFile = AppConstants.GetPluginResourceFile(Plugin.PluginName, "FileSystem_Folder_Back_Pete.png");
                if (iconFile != null)
                {
                    Name = "Exit here :)";
                    _jumpTo = dir;
                    return new ImagePath(new Uri(iconFile.FullName));
                }
            }
        }
        
        // 磁盘选择
        if (value is string && iconFile != null)
        {
            Name = "..";
            return new ImagePath(new Uri(iconFile.FullName));
        }
        
        return ImagePath.Empty;
    }

    public override bool OnEnter()
    {
        // 上一级文件
        if (_jumpTo is { Exists: true })
        {
            var path = new ExplorerPath("FS")
            {
                Path = _jumpTo.FullName
            };
            ExplorerRuntime.Path = path.ToString();
            return true;
        }
        
        // 磁盘选择
        if (_jumpTo == null)
        {
            var path = new ExplorerPath("FS")
            {
                Path = "_"
            };
            ExplorerRuntime.Path = path.ToString();
            return true;
        }
        
        return false;
    }
}