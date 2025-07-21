using System.IO;
using JamReadyGui.AppData.Explorer;

namespace Plugin_FileManager.PluginAdapters;

public class FolderAdapter : ItemAdapter
{
    public override ImagePath OnInit(object value)
    {
        if (value is FileInfo file)
        {
            Name = file.Name;
        }
        return ImagePath.Empty;
    }
}