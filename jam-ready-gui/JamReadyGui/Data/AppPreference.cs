using System;
using System.IO;
using Newtonsoft.Json;

namespace JamReadyGui.Data;

/// <summary>
/// App 首选项
/// </summary>
[Serializable]
public class AppPreference
{
    // 首选项 Json 路径
    public static string PreferenceJsonFile = Directory.GetCurrentDirectory() + "\\config\\preferences.json";
    
    // 核心路径
    public static string JamReadyExeFile = Directory.GetCurrentDirectory() + "\\bin\\jam.exe";
    
    // 工作区首选项
    public WorkspacePreference Workspace = new();

    /// <summary>
    /// 工作区相关首选项
    /// </summary>
    [Serializable]
    public class WorkspacePreference
    {
        // 当前打开的工作区
        public string CurrentWorkspace = Environment.GetFolderPath(Environment.SpecialFolder.Personal) + "\\JamReady\\";

        // 退出工作区
        public static void ExitWorkspace()
        {
            var preference = LoadPreference();
            if (preference == null) return;
            preference.Workspace.CurrentWorkspace = "";
            WritePreference(preference);
        }
    }
    
    // 从本地加载 App 首选项
    public static AppPreference? LoadPreference()
    {
        var jsonFile = new FileInfo(PreferenceJsonFile);
        if (! jsonFile.Exists)
        {
            jsonFile.Directory?.Create();
            File.WriteAllText(jsonFile.FullName, JsonConvert.SerializeObject(new AppPreference()));
        }
        return JsonConvert.DeserializeObject<AppPreference>(File.ReadAllText(PreferenceJsonFile));
    }
    
    // 保存 App 首选项 到本地
    public static void WritePreference(AppPreference preference)
    {
        var jsonFile = new FileInfo(PreferenceJsonFile);
        File.WriteAllText(jsonFile.FullName, JsonConvert.SerializeObject(preference));
    }
}