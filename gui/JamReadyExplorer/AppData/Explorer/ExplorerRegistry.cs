using System;
using System.Collections.Generic;
using System.IO;
using JamReadyGui.AppData.Utils;

namespace JamReadyGui.AppData.Explorer;

/// <summary>
/// 浏览器插件信息注册表
/// </summary>
public static class ExplorerRegistry
{
    /// <summary>
    /// 插入器
    /// </summary>
    public static List<ItemInserter> Inserters = new();
    
    /// <summary>
    /// 目录菜单
    /// </summary>
    public static List<PathMenu> PathMenus = new();
    
    /// <summary>
    /// 环境中加载的语言 (插件名语言标签摘要, (语言键, 值))
    /// </summary>
    private static Dictionary<string, Dictionary<string, string>> _loadedLanguages = new();

    /// <summary>
    /// 加载插件的语言
    /// </summary>
    /// <param name="pluginName"></param>
    public static void LoadLanguages(string pluginName)
    {
        // 查询该插件下所有资源文件
        foreach (var resourceFile in AppConstants.GetPluginResourceFiles(pluginName))
        {
            // 语言文件
            if (resourceFile.Name.EndsWith(".yaml") ||
                resourceFile.Name.EndsWith(".yml"))
            {
                // 处理后的语言文本
                var language = resourceFile.Name
                    .Replace(".yaml", "")
                    .Replace(".yml", "")
                    .Trim().ToLower();

                // 查询符号
                var langKey = GetLanguageKey(pluginName, language);
                
                // 创建字典
                var langDic = LangTextParser.ParseText(File.ReadAllText(resourceFile.FullName));
                
                Console.WriteLine($"Loaded {langDic.Count} language texts from: {resourceFile.Name}");
                
                // 添加进加载项
                _loadedLanguages.Add(langKey, langDic);
            }
        }
    }

    /// <summary>
    /// 获得语言文本
    /// </summary>
    /// <param name="pluginName"></param>
    /// <param name="lang"></param>
    /// <param name="key"></param>
    /// <returns></returns>
    public static string Lang(string pluginName, string lang, string key)
    {
        var processedKey = LangTextParser.SanitizeKey(key);
        
        // 尝试获得当前语言下的文本
        if (_loadedLanguages.TryGetValue(GetLanguageKey(pluginName, lang), out var langDic))
            if (langDic.TryGetValue(processedKey, out var value))
                return value;
        
        // 失败后，尝试获得英文
        if (_loadedLanguages.TryGetValue(GetLanguageKey(pluginName, "en_us"), out var defaultLangDic))
            if (defaultLangDic.TryGetValue(processedKey, out var value))
                return value;
        
        // 再次失败后，返回空内容
        return "";
    }

    /// <summary>
    /// 获得插件的语言 Key
    /// </summary>
    /// <param name="pluginName"></param>
    /// <param name="lang"></param>
    /// <returns></returns>
    private static string GetLanguageKey(string pluginName, string lang)
    {
        return $"{pluginName.ToLower().Trim()}_{lang.ToLower().Trim()}";
    }
}