using System.Collections.Generic;
using JamReadyGui.AppData.Utils;

namespace JamReadyGui.AppData.Explorer;

/// <summary>
/// 浏览器项插入器
/// </summary>
public abstract class ItemInserter
{
    /// <summary>
    /// 获得适配器
    /// </summary>
    /// <param name="path"></param>
    /// <returns></returns>
    public abstract List<ItemAdapter?> GetAdapters(ExplorerPath path);
}