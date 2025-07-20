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
}