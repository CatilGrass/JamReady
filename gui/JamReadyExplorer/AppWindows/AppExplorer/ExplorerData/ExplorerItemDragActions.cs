using System;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using JamReadyGui.AppData.Explorer;

namespace JamReadyGui.AppWindows.AppExplorer.ExplorerData;

/// <summary>
/// 浏览器中，列表项的拖拽行为
/// </summary>
public abstract class ExplorerItemDragActions
{
    /// <summary>
    /// 拖拽开始
    /// </summary>
    /// <param name="panel"></param>
    /// <param name="item"></param>
    public static void DragStart(VirtualizingStackPanel panel, ExplorerItem item)
    {
        // 记录拖拽的位置
        item.DragStartPosition = Mouse.GetPosition(null);
        
        // 记录自身元素
        item.SelfElement = panel.Children[item.Index];
    }
    
    /// <summary>
    /// 拖拽结束
    /// </summary>
    /// <param name="item"></param>
    public static void DragEnd(ExplorerItem item)
    {
        // 清除位置
        item.DragStartPosition = null;
    }
    
    /// <summary>
    /// 拖拽移动
    /// </summary>
    /// <param name="item"></param>
    public static void DragMove(ExplorerItem item)
    {
        // 判空
        if (item.DragStartPosition == null || item.ItemAdapter == null) return;
        
        // 判断是否存在拖拽器
        if (item.ItemAdapter is IDragGenerator drag)
        {
            // 获得拖拽的数据
            var dragData = drag.OnDrag();
            
            // 判空
            if (dragData == null) return;
            
            // 判断当前鼠标位置
            Point currentPos = Mouse.GetPosition(null);
            Vector delta = currentPos - item.DragStartPosition.Value;
        
            // 达到最小拖动距离
            if (Math.Abs(delta.X) > SystemParameters.MinimumHorizontalDragDistance || 
                Math.Abs(delta.Y) > SystemParameters.MinimumVerticalDragDistance)
            {
                // 启动拖拽操作
                item.DragStartPosition = null;
                if (item.SelfElement != null)
                {
                    DragDrop.DoDragDrop(item.SelfElement, dragData.Value.Data, dragData.Value.Effect);
                }
            }
        }
    }
}