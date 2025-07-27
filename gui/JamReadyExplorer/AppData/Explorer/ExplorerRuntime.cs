using System;
using System.Collections.Generic;
using JamReadyGui.AppData.Utils;

namespace JamReadyGui.AppData.Explorer;

/// <summary>
/// 浏览器运行时数据
/// </summary>
public static class ExplorerRuntime
{
    /// <summary>
    /// 当前浏览器窗口
    /// </summary>
    public static AppWindows.AppExplorer.Explorer? CurrentExplorer;
    
    /// <summary>
    /// 当前语言标识
    /// </summary>
    public static string Language
    {
        get => _language;
        set
        {
            // 设置语言
            _language = value.Trim().ToLower();
            
            // 保存首选项
            AppPreference.OperatePreference(p =>
            {
                p.Language = _language;
            });
        }
    }
    
    /// <summary>
    /// 正在搜索的内容
    /// </summary>
    public static string SearchContent = "";
    
    /// <summary>
    /// 当前在的目录
    /// </summary>
    public static string CurrentPath
    {
        get => _currentPath;
        set => UpdatePath(value);
    }
    
    /// <summary>
    /// 当前在的目录 (Setter 为强制更新)
    /// </summary>
    public static string Path
    {
        set => UpdatePath(value, true);
    }

    /// <summary>
    /// 当前浏览器下所有的适配器实例
    /// </summary>
    public static List<ItemAdapter?> CurrentAdapters => ItemAdapters;
    
    // 私有数据
    private static string _currentPath = "HOME://";
    private static readonly List<string> HistoryPaths = new();
    private static readonly List<ItemAdapter?> ItemAdapters = new();
    private static string _language = "en_us";

    /// <summary>
    /// 构建运行时
    /// </summary>
    public static void InitializeExplorerRuntime()
    {
        // 加载语言
        AppPreference.OperatePreference(p => _language = p.Language);
    }
    
    /// <summary>
    /// 刷新目录
    /// </summary>
    /// <param name="path"> 进入目录 </param>
    /// <param name="force"> 强制刷新 </param>
    private static void UpdatePath(string path, bool force = false)
    {
        var trimPath = path.Trim();
        if (trimPath != _currentPath.Trim() || force)
        {
            HistoryPaths.Add(_currentPath);
            if (HistoryPaths.Count > 15)
            {
                HistoryPaths.RemoveAt(0);
            }
            _currentPath = trimPath;
            RegenerateAdaptersByPath(trimPath);
        }
    }

    /// <summary>
    /// 使用路径重新生成适配器
    /// </summary>
    /// <param name="pathString"></param>
    private static void RegenerateAdaptersByPath(string pathString)
    {
        Console.WriteLine("Regenerating adapters...");
        ItemAdapters.Clear();
        foreach (var inserter in ExplorerRegistry.Inserters)
        {
            int i = 0;
            var path = ExplorerPath.FromString(pathString);
            if (path != null)
            {
                foreach (var adapter in inserter.GetAdapters(path.Value))
                {
                    if (adapter != null)
                    {
                        ItemAdapters.Add(adapter);
                        i++;
                    }
                }
            }

            if (i > 0)
            {
                Console.WriteLine($"Generated {i} adapter{(i > 1 ? "s" : "")} by {inserter.GetType().Name}");
            }
        }
        Console.WriteLine("Regenerate adapter complete.");
    }

    /// <summary>
    /// 前往上一个目录
    /// </summary>
    public static void ToLastPath()
    {
        if (HistoryPaths.Count > 0)
        {
            CurrentPath = HistoryPaths[^1];
            HistoryPaths.RemoveAt(HistoryPaths.Count - 1);
        }
    }

    /// <summary>
    /// 获得语言内容
    /// </summary>
    /// <param name="pluginName"></param>
    /// <param name="key"></param>
    /// <returns></returns>
    public static string Lang(string pluginName, string key)
    {
        return ExplorerRegistry.Lang(pluginName, Language, key);
    }
}