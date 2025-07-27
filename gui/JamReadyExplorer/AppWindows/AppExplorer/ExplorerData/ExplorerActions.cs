using System;
using System.Windows.Controls;
using System.Windows.Controls.Primitives;
using System.Windows.Media.Imaging;
using JamReadyGui.AppData.Explorer;
using JamReadyGui.AppData.Utils;
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
            var explorerPath = ExplorerPath.FromString(path);
            if (explorerPath != null)
            {
                var pathMenuItems = pathMenu.OnRegisterOperation(explorerPath.Value);
                if (pathMenuItems != null)
                {
                    // 建立一级菜单
                    var menuItemMain = new MenuItem
                    {
                        Header = pathMenu.GetMenuName().Trim(), Command = new ActionCommand(() =>
                        {
                            // 子项数量
                            int subItemCount = 0;
                            
                            // 建立二级菜单
                            var menuSub = new ContextMenu();
                            foreach (var pathMenuItem in pathMenuItems)
                            {
                                int insert = subItemCount;
                                var menuItemSub = new MenuItem
                                {
                                    Header = pathMenuItem.Trim(), Command = new ActionCommand(() =>
                                    {
                                        if (pathMenu.OnOperate(explorerPath.Value, insert))
                                            explorer.RefreshExplorerItems();
                                    })
                                };
                                var iconSub = pathMenu.GetOperationIcon(explorerPath.Value, subItemCount);
                                if (iconSub != null)
                                {
                                    menuItemSub.Icon = new Image
                                    {
                                        Source = new BitmapImage(iconSub.Path)
                                    };
                                }
                                
                                // 添加子项，增加子项计数
                                menuSub.Items.Add(menuItemSub);
                                subItemCount++;
                            }
                            menuSub.Placement = PlacementMode.Mouse;
                            menuSub.IsOpen = true;
                            menuSub.HorizontalOffset = menuSub.ActualWidth;
                            menuSub.VerticalOffset = - 16;
                        })
                    };

                    if (pathMenuItems.Count > 0)
                    {
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
            }
        }
        
        if (menu.Items.Count > 0) {
            menu.Placement = PlacementMode.Mouse;
            menu.IsOpen = true;
            menu.HorizontalOffset = menu.ActualWidth;
            menu.VerticalOffset = - 16;
        }
    }
}