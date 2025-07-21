using System.Windows.Controls;
using System.Windows.Controls.Primitives;
using System.Windows.Media.Imaging;
using JamReadyGui.AppData.Explorer;
using Microsoft.Xaml.Behaviors.Core;

namespace JamReadyGui.AppWindows.AppExplorer.ExplorerData;

/// <summary>
/// 浏览器中的行为
/// </summary>
public static class ExplorerActions
{
    /// <summary>
    /// 右键事件
    /// </summary>
    public static void RightClick(Explorer explorer)
    {
        // 上下文菜单
        var menu = new ContextMenu();
        
        // 目录
        var path = ExplorerRuntime.CurrentPath;
        
        // 遍历所有菜单
        foreach (var pathMenu in ExplorerRegistry.PathMenus)
        {
            var pathMenuItems = pathMenu.OnRegisterOperation(path);
            if (pathMenuItems != null)
            {
                // 建立一级菜单
                var menuItemMain = new MenuItem
                {
                    Header = pathMenu.GetMenuName().Trim(), Command = new ActionCommand(() =>
                    {
                        // 建立二级菜单
                        var menuSub = new ContextMenu();
                        int i = 0;
                        foreach (var pathMenuItem in pathMenuItems)
                        {
                            int insert = i;
                            var menuItemSub = new MenuItem
                            {
                                Header = pathMenuItem.Trim(), Command = new ActionCommand(() =>
                                {
                                    if (pathMenu.OnOperate(path, insert))
                                    {
                                        explorer.RefreshExplorerItems();
                                    }
                                })
                            };
                            var iconSub = pathMenu.GetOperationIcon(path, i);
                            if (iconSub != null)
                            {
                                menuItemSub.Icon = new Image
                                {
                                    Source = new BitmapImage(iconSub.Path)
                                };
                            }
                            menuSub.Items.Add(menuItemSub);
                            i++;
                        }
                        menuSub.Placement = PlacementMode.Mouse;
                        menuSub.IsOpen = true;
                    })
                };

                var icon = pathMenu.GetIcon();
                if (icon != null)
                {
                    menuItemMain.Icon = new Image
                    {
                        Source = new BitmapImage(icon.Path)
                    };
                }
                menu.Items.Add(menuItemMain);
            }
        }
        
        if (menu.Items.Count > 0) {
            menu.Placement = PlacementMode.Mouse;
            menu.IsOpen = true;
        }
    }
}