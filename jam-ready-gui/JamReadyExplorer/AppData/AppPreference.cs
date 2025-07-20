using System;
using System.Collections.Generic;
using System.IO;
using Newtonsoft.Json;

namespace JamReadyGui.AppData;

/// <summary>
/// App 首选项
/// </summary>
[Serializable]
public class AppPreference
{
    /// <summary>
    /// 加载 App 首选项
    /// </summary>
    /// <returns></returns>
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
    
    /// <summary>
    /// 保存 App 首选项
    /// </summary>
    /// <param name="preference"></param>
    public static void WritePreference(AppPreference preference)
    {
        var jsonFile = new FileInfo(AppConstants.PreferenceConfigureFile);
        File.WriteAllText(jsonFile.FullName, JsonConvert.SerializeObject(preference));
    }
}
