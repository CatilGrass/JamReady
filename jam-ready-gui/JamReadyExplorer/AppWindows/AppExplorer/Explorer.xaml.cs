using System;
using System.Collections.ObjectModel;
using System.ComponentModel;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;
using JamReadyGui.AppData.Explorer;
using JamReadyGui.AppData.Utils;
using JamReadyGui.AppWindows.AppExplorer.ExplorerData;
using JamReadyGui.AppWindows.AppExplorer.ExplorerData.Commands;
using Microsoft.Xaml.Behaviors.Core;

namespace JamReadyGui.AppWindows.AppExplorer;

public sealed partial class Explorer : INotifyPropertyChanged
{
    /// <summary>
    /// 浏览器中的所有列表数据
    /// </summary>
    public ObservableCollection<ExplorerItem> ExplorerItems { get; } = new();

    /// <summary>
    /// 浏览器的列表项大小百分比
    /// </summary>
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

    /// <summary>
    /// 浏览器中的列表项大小
    /// </summary>
    public double ExplorerItemSize => MathUtils.Lerp(_explorerItemSizeMin, _explorerItemSizeMax, Math.Clamp(ExplorerItemSizePercent, 0, 1));

    /// <summary>
    /// 浏览器中的列表项文本大小
    /// </summary>
    public double ExplorerItemFontSize => MathUtils.Lerp(_explorerItemFontSizeMin, _explorerItemFontSizeMax, Math.Clamp(ExplorerItemSizePercent, 0, 1));

    /// <summary>
    /// 属性修改事件
    /// </summary>
    public event PropertyChangedEventHandler? PropertyChanged;
    
    private double _explorerItemSizePercent = 0.2;
    private readonly double _explorerItemSizeMin = 75;
    private readonly double _explorerItemSizeMax = 260;
    private readonly double _explorerItemFontSizeMin = 11;
    private readonly double _explorerItemFontSizeMax = 16;
    private WrapPanel? _explorerWrapPanel;
    
    public Explorer()
    {
        InitializeComponent();
        
        // 设置数据上下文
        DataContext = this; 

        // 设置地址栏的文本为 App 默认内容
        PathBox.Text = ExplorerRuntime.CurrentPath;
        
        // 初次刷新浏览器列表项
        RefreshExplorerItems();
    }
    
    // -----------------------------------------------------------------------------------
    // 浏览器操作

    /// <summary>
    /// 刷新浏览器列表项
    /// </summary>
    public void RefreshExplorerItems()
    {
        ExplorerRuntime.Path = PathBox.Text.Trim();
        ExplorerItems.Clear();
        int i = 0;
        foreach (var adapter in ExplorerRuntime.CurrentAdapters)
        {
            ExplorerItems.Add(new ExplorerItem(i, adapter));
            i ++;
        }
    }
    
    // -----------------------------------------------------------------------------------
    // 底部地址栏、搜索框事件

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
    // 底部导航栏按钮事件

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
    // 浏览器监听事件、列表事件
    
    /// <summary>
    /// 列表视图大小更改时
    /// </summary>
    /// <param name="sender"></param>
    /// <param name="e"></param>
    private void ExplorerWrapPanel_OnSizeChanged(object sender, SizeChangedEventArgs e)
    {
        if (e.Source is WrapPanel panel)
        {
            // 在 WrapPanel 大小变更时，刷新器引用
            _explorerWrapPanel = panel;
            
            // 根据布局高度计算对齐模式
            panel.HorizontalAlignment = panel.ActualHeight > ExplorerItemSize
                ? HorizontalAlignment.Center
                : HorizontalAlignment.Left;
        }
    }
    
    /// <summary>
    /// 浏览器中，在当前路径右键时
    /// </summary>
    public ICommand ExplorerPathRightClickCommand => new ActionCommand(_ =>
    {
        // 跳转执行
        ExplorerActions.RightClick(this);
    });
    
    /// <summary>
    /// 浏览器中，在选定列表项上左键时
    /// </summary>
    public ICommand ExplorerItemLeftClickCommand => new RelayCommand<ExplorerItem>(item =>
    {
        ExplorerItemClickActions.LeftClick(item, this);
    });
    
    /// <summary>
    /// 浏览器中，在选定列表项上右键时
    /// </summary>
    public ICommand ExplorerItemRightClickCommand => new RelayCommand<ExplorerItem>(item =>
    {
        ExplorerItemClickActions.RightClick(item, this);
    });
    
    /// <summary>
    /// 浏览器中，在选定列表项上开始拖拽时
    /// </summary>
    public ICommand ExplorerItemDragStartCommand => new RelayCommand<ExplorerItem>(item =>
    {
        if (_explorerWrapPanel != null) ExplorerItemDragActions.DragStart(_explorerWrapPanel, item);
    });
    
    /// <summary>
    /// 浏览器中，在选定列表项上结束拖拽时
    /// </summary>
    public ICommand ExplorerItemDragEndCommand => new RelayCommand<ExplorerItem>(ExplorerItemDragActions.DragEnd);

    /// <summary>
    /// 浏览器中，在选定列表项上拖拽时
    /// </summary>
    public ICommand ExplorerItemDragMoveCommand => new RelayCommand<ExplorerItem>(ExplorerItemDragActions.DragMove);

    /// <summary>
    /// 浏览器中，当数据在列表项上释放时
    /// </summary>
    /// <param name="sender"></param>
    /// <param name="e"></param>
    private void Item_OnDrop(object sender, DragEventArgs e)
    {
        if (_explorerWrapPanel == null) return;
        
        if (sender is ContentPresenter itemElement)
        {
            int targetIndex = _explorerWrapPanel.Children.IndexOf(itemElement);

            if (targetIndex > -1)
            {
                // 获得数据
                var item = ExplorerItemsControl.Items[targetIndex] as ExplorerItem;

                // 包含接收 Drop 的接口
                if (item?.ItemAdapter is IDropExtractor extractor)
                {
                    // 执行
                    extractor.OnExtract(e.Data);
                }
            }
        }

        e.Handled = true;
    }

    private void OnPropertyChanged(string propertyName)
    {
        PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
    }
}