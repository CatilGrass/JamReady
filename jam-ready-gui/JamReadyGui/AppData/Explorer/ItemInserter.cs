using System.Collections.Generic;

namespace JamReadyGui.AppData.Explorer;

/// <summary>
/// 项目插入器
/// </summary>
public abstract class ItemInserter
{
    /// <summary>
    /// 获得适配器
    /// </summary>
    /// <param name="path"></param>
    /// <returns></returns>
    public abstract List<ItemAdapter?> GetAdapters(string path);
}