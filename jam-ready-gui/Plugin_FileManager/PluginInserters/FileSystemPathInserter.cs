using System.Collections.Generic;
using JamReadyGui.AppData.Explorer;
using JamReadyGui.AppData.Utils;

namespace Plugin_FileManager.PluginInserters;

public class FileSystemPathInserter : ItemInserter
{
    private const string PluginPrefix = "FS";

    public override List<ItemAdapter?> GetAdapters(ExplorerPath path)
    {
        var result = new List<ItemAdapter?>();
        if (path.IsNone()) return result;

        // 确认前缀为 FS
        if (path.Prefix == PluginPrefix)
        {
            
        }
        
        return result;
    }
}