using System;
using System.IO;
using JamReadyGui.AppData;
using JamReadyGui.AppData.Explorer;

namespace Plugin_FileManager.PluginAdapters;

public class FileAdapter : ItemAdapter
{
    public override ImagePath OnInit(object value)
    {
        var iconFile = AppConstants.GetPluginResourceFile(Plugin.PluginName, "FileSystem_File.png");
        if (value is FileInfo file && iconFile != null)
        {
            var fileName = file.Name.ToLower().Trim();
            Name = file.Name;
         
            // 图片文件判断，如果是图片直接加载该图片内容
            if (fileName.EndsWith(".bmp") || fileName.EndsWith(".jpg") || fileName.EndsWith(".png"))
                if (fileName.Length < 1024 * 1024 * 2)
                {
                    return new ImagePath(new Uri(file.FullName));
                }
            
            return new ImagePath(new Uri(iconFile.FullName));
        }
        return ImagePath.Empty;
    }
}