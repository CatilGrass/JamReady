using System.Windows;

namespace JamReadyGui.AppData.Explorer;

/// <summary>
/// 拖拽事件生成器
/// </summary>
public interface IDragGenerator
{
    /// <summary>
    /// 拖拽事件
    /// </summary>
    /// <returns> 拖拽的数据 </returns>
    public DragInformation? OnDrag();
}

/// <summary>
/// 拖拽信息
/// </summary>
public struct DragInformation
{
    /// <summary>
    /// 拖拽的数据
    /// </summary>
    public readonly DataObject Data;
    
    /// <summary>
    /// 拖拽的效果
    /// </summary>
    public readonly DragDropEffects Effect;

    /// <summary>
    /// 拖拽信息
    /// </summary>
    /// <param name="data"> 数据 </param>
    /// <param name="effect"> 效果 </param>
    public DragInformation(DataObject data, DragDropEffects effect)
    {
        Data = data;
        Effect = effect;
    }
}