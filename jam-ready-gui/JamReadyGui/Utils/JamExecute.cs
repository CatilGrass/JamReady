using System.IO;
using JamReadyGui.AppConfigure;

namespace JamReadyGui.Utils;

public class JamExecute
{
    public static ProcessRunnerResult? Execute(string command)
    {
        return Execute(new[] { command });
    }
    
    public static ProcessRunnerResult? Execute(string[] commands)
    {
        var preference = AppPreference.LoadPreference();
        if (preference != null)
        {
            return ProcessRunner.Run(
                new DirectoryInfo(preference.Workspace.CurrentWorkspace),
                new FileInfo(AppPreference.JamReadyExeFile), 
                commands);
        }
        return null;
    }
}