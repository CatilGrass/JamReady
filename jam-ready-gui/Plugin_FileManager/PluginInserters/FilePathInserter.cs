using System.Collections.Generic;
using JamReadyGui.AppData.Explorer;
using Plugin_FileManager.PluginAdapters;

namespace Plugin_FileManager.PluginInserters;

public class FilePathInserter : ItemInserter
{
    public override List<ItemAdapter?> GetAdapters(string path)
    {
        var result = new List<ItemAdapter?>();
        if (ExplorerRuntime.CurrentPath.StartsWith("homepage://"))
        {
            result.Add(AdapterFactory.Create<FolderAdapter>(""));
        }
        return result;
    }
}