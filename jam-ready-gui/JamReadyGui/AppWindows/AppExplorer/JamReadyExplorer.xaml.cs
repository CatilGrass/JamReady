using System;
using System.Collections.ObjectModel;
using System.ComponentModel;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Controls.Primitives;
using System.Windows.Input;
using System.Windows.Media.Imaging;
using JamReadyGui.AppData.Explorer;
using JamReadyGui.AppData.Utils;
using Microsoft.Xaml.Behaviors.Core;

namespace JamReadyGui.AppWindows.AppExplorer;

public sealed partial class JamReadyExplorer : INotifyPropertyChanged
{
    /// <summary>
    /// 项目大小百分比
    /// </summary>
    private double _explorerItemSizePercent = 0.2;
    
    /// <summary>
    /// 浏览器项大小
    /// </summary>
    private readonly double _explorerItemSizeMin = 75;
    private readonly double _explorerItemSizeMax = 260;
    
    /// <summary>
    /// 浏览器项文字大小
    /// </summary>
    private readonly double _explorerItemFontSizeMin = 11;
    private readonly double _explorerItemFontSizeMax = 16;

    // 浏览器列表项
    public ObservableCollection<ExplorerItem> ExplorerItems { get; } = new();

    public double ExplorerItemSizePercent
    {
        get => _explorerItemSizePercent;
        set
        {
            _explorerItemSizePercent = Math.Clamp(value, 0, 1.0);
            OnPropertyChanged(nameof(ExplorerItemSizePercent));
            OnPropertyChanged(nameof(ExplorerItemSize));
            OnPropertyChanged(nameof(ExplorerItemFontSize));
        }
    }

    public double ExplorerItemSize => MathUtils.Lerp(_explorerItemSizeMin, _explorerItemSizeMax, Math.Clamp(ExplorerItemSizePercent, 0, 1));

    public double ExplorerItemFontSize => MathUtils.Lerp(_explorerItemFontSizeMin, _explorerItemFontSizeMax, Math.Clamp(ExplorerItemSizePercent, 0, 1));

    public event PropertyChangedEventHandler? PropertyChanged;
    
    public JamReadyExplorer()
    {
        InitializeComponent();

        // 设置地址栏的文本
        PathBox.Text = ExplorerRuntime.CurrentPath;
        
        // 设置数据上下文
        DataContext = this; 
        
        // 刷新内容
        RefreshExplorerItems();
    }
    
    // -----------------------------------------------------------------------------------
    // 页面操作

    /// <summary>
    /// 刷新浏览器项目
    /// </summary>
    public void RefreshExplorerItems()
    {
        ExplorerRuntime.Path = PathBox.Text.Trim();
        ExplorerItems.Clear();
        foreach (var adapter in ExplorerRuntime.CurrentAdapters)
        {
            ExplorerItems.Add(new ExplorerItem(adapter));
        }
    }
    
    
    // -----------------------------------------------------------------------------------
    // 底部文字输入框

    /// <summary>
    /// 地址栏文本变更
    /// </summary>
    /// <param name="sender"></param>
    /// <param name="e"></param>
    private void PathBox_OnPreviewKeyDown(object sender, KeyEventArgs e)
    {
        if (e.Key == Key.Enter)
        {
            if (ExplorerRuntime.CurrentPath != PathBox.Text)
            {
                ExplorerRuntime.CurrentPath = PathBox.Text;
                
                RefreshExplorerItems();
                Console.WriteLine($"Path changed to : {PathBox.Text}");
            }
        }
    }

    /// <summary>
    /// 搜索栏文本变更
    /// </summary>
    /// <param name="sender"></param>
    /// <param name="e"></param>
    /// <exception cref="NotImplementedException"></exception>
    private void SearchBox_OnPreviewKeyDown(object sender, KeyEventArgs e)
    {
        if (e.Key == Key.Enter)
        {
            if (ExplorerRuntime.SearchContent != SearchBox.Text)
            {
                ExplorerRuntime.SearchContent = SearchBox.Text;
                Console.WriteLine($"Search text changed to : {SearchBox.Text}");
            }
        }
    }
    
    // -----------------------------------------------------------------------------------
    // 底部按钮

    /// <summary>
    /// 资源管理器画面放大
    /// </summary>
    /// <param name="sender"></param>
    /// <param name="e"></param>
    /// <exception cref="NotImplementedException"></exception>
    private void Explorer_Zoom_In(object sender, RoutedEventArgs e)
    {
        ExplorerItemSizePercent += 0.08;
    }

    /// <summary>
    /// 资源管理器画面缩小
    /// </summary>
    /// <param name="sender"></param>
    /// <param name="e"></param>
    /// <exception cref="NotImplementedException"></exception>
    private void Explorer_Zoom_Out(object sender, RoutedEventArgs e)
    {
        ExplorerItemSizePercent -= 0.08;
    }

    /// <summary>
    /// 浏览器刷新
    /// </summary>
    /// <param name="sender"></param>
    /// <param name="e"></param>
    /// <exception cref="NotImplementedException"></exception>
    private void Explorer_Refresh(object sender, RoutedEventArgs e)
    {
        RefreshExplorerItems();
    }
    
    
    // -----------------------------------------------------------------------------------
    // 资源管理器相关
    
    /// <summary>
    /// 当前路径右键
    /// </summary>
    public ICommand ExplorerPathRightClickCommand => new ActionCommand(_ =>
    {
        // 上下文菜单
        var menu = new ContextMenu();
        
        // 目录
        var path = ExplorerRuntime.CurrentPath;
        
        // 遍历所有菜单
        foreach (var pathMenu in Registry.PathMenus)
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
                                    pathMenu.OnOperate(path, insert);
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
        
        menu.Placement = PlacementMode.Mouse;
        menu.IsOpen = true;
    });
    
    /// <summary>
    /// 项左键
    /// </summary>
    public ICommand ExplorerItemLeftClickCommand => new RelayCommand<ExplorerItem>(item =>
    {
        // 按下左键，执行 Adapter 的 OnEnter
        item.ItemAdapter?.OnEnter();
    });
    
    /// <summary>
    /// 项左键
    /// </summary>
    public ICommand ExplorerItemRightClickCommand => new RelayCommand<ExplorerItem>(item =>
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
                        item.ItemAdapter?.OnOperate(insertIndex);
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
    });

    private void OnPropertyChanged(string propertyName)
    {
        PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
    }
}
    
// -----------------------------------------------------------------------------------
// 资源管理器相关数据

// 列表项
public class ExplorerItem : INotifyPropertyChanged
{
    private string _itemName;
    private string _iconPath;
    private string _subscriptIconPath;
    
    public readonly ItemAdapter? ItemAdapter;

    public string ItemName
    {
        get => _itemName;
        set
        {
            _itemName = value;
            OnPropertyChanged(nameof(ItemName));
        }
    }
    
    public string IconPath
    {
        get => _iconPath;
        set
        {
            _iconPath = value;
            OnPropertyChanged(nameof(IconPath));
        }
    }
    
    public string SubscriptIconPath
    {
        get => _subscriptIconPath;
        set
        {
            _subscriptIconPath = value;
            OnPropertyChanged(nameof(SubscriptIconPath));
        }
    }

    public ExplorerItem(ItemAdapter? itemAdapter)
    {
        _itemName = itemAdapter?.Name ?? "Unknown";
        _iconPath = itemAdapter?.Icon?.Path.ToString() ?? "";
        _subscriptIconPath = itemAdapter?.SubscriptIcon?.Path.ToString() ?? "";
        ItemAdapter = itemAdapter;
    }
    
    public event PropertyChangedEventHandler? PropertyChanged;

    private void OnPropertyChanged(string propertyName)
    {
        PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
    }
}

public class RelayCommand<T> : ICommand
{
    private readonly Action<T> _execute;
    
    public RelayCommand(Action<T> execute) => _execute = execute;
    
    public bool CanExecute(object? parameter) => true;
    
    public void Execute(object? parameter)
    {
        if (parameter is T typedParam)
            _execute(typedParam);
    }
    
    public event EventHandler? CanExecuteChanged;
}