using System.IO;
using JamReadyGui.AppData.Utils;

namespace JamReadyGui.AppData;

public static class AppCoreInvoker
{
    public static ProcessRunnerResult? Execute(string command)
    {
        return Execute(new[] { command });
    }
    
    public static ProcessRunnerResult? Execute(string[] commands)
    {
        var preference = AppPreference.LoadPreference();
        if (preference != null && AppRuntimeData.CurrentDirectory != null)
        {
            return ProcessRunner.Run(
                AppRuntimeData.CurrentDirectory,
                new FileInfo(AppConstants.CoreExecutableFile), 
                commands);
        }
        return null;
    }
}