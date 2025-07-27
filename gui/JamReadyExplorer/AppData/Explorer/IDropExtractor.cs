using System.Windows;

namespace JamReadyGui.AppData.Explorer;

/// <summary>
/// 数据释放器
/// </summary>
public interface IDropExtractor
{
    /// <summary>
    /// 释放数据
    /// </summary>
    /// <param name="item"> 释放的数据 </param>
    public void OnExtract(IDataObject item);
}