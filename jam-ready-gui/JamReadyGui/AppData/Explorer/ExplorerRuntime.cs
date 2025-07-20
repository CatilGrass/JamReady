using System;
using System.Collections.Generic;

namespace JamReadyGui.AppData.Explorer;

/// <summary>
/// 浏览器运行时
/// </summary>
public static class ExplorerRuntime
{
    public static string SearchContent = "";
    
    public static string CurrentPath
    {
        get => _currentPath;
        set => UpdatePath(value);
    }
    
    public static string Path
    {
        get => _currentPath;
        set => UpdatePath(value, true);
    }

    public static List<ItemAdapter?> CurrentAdapters => ItemAdapters;
    
    // 当前的目录
    private static string _currentPath = "file://C:/";
    
    // 历史的目录
    private static readonly List<string> HistoryPaths = new();
    
    // 当前的 Adapter
    private static readonly List<ItemAdapter?> ItemAdapters = new();

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
    /// <param name="path"></param>
    private static void RegenerateAdaptersByPath(string path)
    {
        ItemAdapters.Clear();
        foreach (var inserter in Registry.Inserters)
        {
            Console.WriteLine($"Generating adapter by inserter: {inserter.GetType().Name}");
            int i = 0;
            foreach (var adapter in inserter.GetAdapters(path))
            {
                if (adapter != null)
                {
                    ItemAdapters.Add(adapter);
                    i++;
                }
            }
            Console.WriteLine($"Generated {i} Adapters");
        }
    }
}