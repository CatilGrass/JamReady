using System.Windows.Controls;
using System.Windows.Controls.Primitives;
using System.Windows.Media.Imaging;
using Microsoft.Xaml.Behaviors.Core;

namespace JamReadyGui.AppWindows.AppExplorer.ExplorerData;

/// <summary>
/// 浏览器中，列表项的点击行为
/// </summary>
public static class ExplorerItemClickActions
{
    /// <summary>
    /// 右键事件
    /// </summary>
    public static void RightClick(ExplorerItem item, Explorer explorer)
    {
        if (item.ItemAdapter == null) return;
        var operations = item.ItemAdapter.OnRegisterOperation();
        if (operations.Count > 0)
        {
            // 上下文菜单
            var menu = new ContextMenu();
            int i = 0;
            foreach (var operate in operations)
            {
                var insertIndex = i;
                var menuItem = new MenuItem
                {
                    Header = operate.Trim(), Command = new ActionCommand(() =>
                    {
                        if (item.ItemAdapter?.OnOperate(insertIndex) == true)
                        {
                            explorer.RefreshExplorerItems();
                        }
                    })
                };
                var icon = item.ItemAdapter.GetOperationIcon(i);
                if (icon != null)
                {
                    menuItem.Icon = new Image
                    {
                        Source = new BitmapImage(icon.Path)
                    };
                }
                menu.Items.Add(menuItem);
                i++;
            }
            menu.Placement = PlacementMode.Mouse;
            menu.IsOpen = true;
        }
    }

    /// <summary>
    /// 左键事件
    /// </summary>
    public static void LeftClick(ExplorerItem item, Explorer explorer)
    {
        // 按下左键，执行 Adapter 的 OnEnter
        if (item.ItemAdapter?.OnEnter() == true)
        {
            // 若返回 true 则执行刷新
            explorer.RefreshExplorerItems();
        }
    }
}