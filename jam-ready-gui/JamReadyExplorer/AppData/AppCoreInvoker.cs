using System.IO;
using JamReadyGui.AppData.Utils;

namespace JamReadyGui.AppData;

/// <summary>
/// 核心程序调用器
/// </summary>
public static class AppCoreInvoker
{
    /// <summary>
    /// 运行单参数命令
    /// </summary>
    /// <param name="command"> 命令 </param>
    /// <param name="workingDirectory"> 当前工作目录 </param>
    /// <returns></returns>
    public static ProcessRunnerResult? Execute(string command, DirectoryInfo workingDirectory)
    {
        return Execute(new[] { command }, workingDirectory);
    }

    /// <summary>
    /// 运行多参数命令
    /// </summary>
    /// <param name="commands"> 命令 </param>
    /// <param name="workingDirectory"> 当前工作目录 </param>
    /// <returns></returns>
    public static ProcessRunnerResult? Execute(string[] commands, DirectoryInfo workingDirectory)
    {
        var preference = AppPreference.LoadPreference();
        if (preference != null)
        {
            return ProcessRunner.Run(
                workingDirectory,
                new FileInfo(AppConstants.CoreExecutableFile), 
                commands);
        }
        return null;
    }
}