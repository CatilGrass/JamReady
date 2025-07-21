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
        ItemAdapters.Clear();
        foreach (var inserter in ExplorerRegistry.Inserters)
        {
            Console.WriteLine($"Generating adapter by inserter: {inserter.GetType().Name}");
            int i = 0;
            foreach (var adapter in inserter.GetAdapters(ExplorerPath.FromString(pathString)))
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