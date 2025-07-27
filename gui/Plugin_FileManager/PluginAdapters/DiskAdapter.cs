using System;
using System.IO;
using JamReadyGui.AppData;
using JamReadyGui.AppData.Explorer;
using JamReadyGui.AppData.Utils;

namespace Plugin_FileManager.PluginAdapters;

public class DiskAdapter : ItemAdapter
{
    private string? _diskChar;
    
    public override ImagePath OnInit(object value)
    {
        var iconFile = AppConstants.GetPluginResourceFile(Plugin.PluginName, "FileSystem_Disk.png");
        if (value is string diskChar && iconFile != null)
        {
            Name = diskChar.ToUpper() + "\\";
            _diskChar = diskChar;
            return new ImagePath(new Uri(iconFile.FullName));
        }
        return ImagePath.Empty;
    }

    public override bool OnEnter()
    {
        if (_diskChar == null) return false;
        
        var path = new ExplorerPath("FS")
        {
            Path = _diskChar?.ToUpper() + "/"
        };
        ExplorerRuntime.Path = path.ToString();
        return true;
    }
}