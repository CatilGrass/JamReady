using System.Collections.Generic;
using System.IO;

namespace JamReadyGui.AppData;

/// <summary>
/// App 常量
/// </summary>
public static class AppConstants
{
    /// <summary>
    /// 首选项配置文件
    /// </summary>
    public static readonly string PreferenceConfigureFile = Directory.GetCurrentDirectory() + "\\config\\preferences.json";
    
    /// <summary>
    /// 核心的可执行文件
    /// </summary>
    public static readonly string CoreExecutableFile = Directory.GetCurrentDirectory() + "\\bin\\jam.exe";
    
    /// <summary>
    /// 核心的可执行文件
    /// </summary>
    public static readonly string PluginDirectory = Directory.GetCurrentDirectory() + "\\plugins\\";

    /// <summary>
    /// 获得所有插件的Dll文件
    /// </summary>
    /// <returns></returns>
    public static List<FileInfo> GetPluginDllFiles()
    {
        List<FileInfo> files = new List<FileInfo>();
        foreach (var directory in new DirectoryInfo(PluginDirectory).GetDirectories())
        {
            var directoryName = directory.Name;
            foreach (var fileInfo in directory.GetFiles())
            {
                if (fileInfo.Name.Equals(directoryName + ".dll"))
                {
                    files.Add(fileInfo);
                    break;
                }
            }
        }

        return files;
    }

    /// <summary>
    /// 通过插件名称获得资源目录
    /// </summary>
    /// <param name="name"></param>
    /// <returns></returns>
    public static DirectoryInfo? GetResourceDirectoryOfPluginName(string name)
    {
        var targetDirectory = new DirectoryInfo(PluginDirectory + name + "\\Res\\");
        return targetDirectory.Exists ? targetDirectory : null;
    }
}