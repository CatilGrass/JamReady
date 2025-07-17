using System;
using System.Collections.Generic;
using System.IO;
using Newtonsoft.Json;

namespace JamReadyGui.AppData;

[Serializable]
public class AppPreference
{
    public WorkspacePreference Workspace = new();

    [Serializable]
    public class WorkspacePreference
    {
        public List<string> HistoryWorkspaces = new();
    }
    
    public static AppPreference? LoadPreference()
    {
        var jsonFile = new FileInfo(AppConstants.PreferenceConfigureFile);
        if (! jsonFile.Exists)
        {
            jsonFile.Directory?.Create();
            File.WriteAllText(jsonFile.FullName, JsonConvert.SerializeObject(new AppPreference()));
        }
        return JsonConvert.DeserializeObject<AppPreference>(File.ReadAllText(AppConstants.PreferenceConfigureFile));
    }
    
    public static void WritePreference(AppPreference preference)
    {
        var jsonFile = new FileInfo(AppConstants.PreferenceConfigureFile);
        File.WriteAllText(jsonFile.FullName, JsonConvert.SerializeObject(preference));
    }
}
