using System.Collections.Generic;

namespace JamReadyGui.AppData.Explorer;

/// <summary>
/// 浏览器插件注册表
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
}