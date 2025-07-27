using System.ComponentModel;
using System.Windows;
using JamReadyGui.AppData.Explorer;

namespace JamReadyGui.AppWindows.AppExplorer.ExplorerData;

// 列表项
public class ExplorerItem : INotifyPropertyChanged
{
    /// <summary>
    /// 自身的元素
    /// </summary>
    public UIElement? SelfElement;
    
    /// <summary>
    /// 自身的适配器
    /// </summary>
    public readonly ItemAdapter? ItemAdapter;

    /// <summary>
    /// 该列表项的显示名称
    /// </summary>
    public string ItemName
    {
        get => _itemName;
        set
        {
            _itemName = value;
            OnPropertyChanged(nameof(ItemName));
        }
    }
    
    /// <summary>
    /// 该列表项的图标目录
    /// </summary>
    public string IconPath
    {
        get => _iconPath;
        set
        {
            _iconPath = value;
            OnPropertyChanged(nameof(IconPath));
        }
    }
    
    /// <summary>
    /// 该列表项的下角标图标目录
    /// </summary>
    public string SubscriptIconPath
    {
        get => _subscriptIconPath;
        set
        {
            _subscriptIconPath = value;
            OnPropertyChanged(nameof(SubscriptIconPath));
        }
    }

    /// <summary>
    /// 该列表项的索引值
    /// </summary>
    public int Index { get; }

    /// <summary>
    /// 列表项拖拽开始的位置 (null 表示未开始拖拽)
    /// </summary>
    public Point? DragStartPosition = null;

    private string _itemName;
    private string _iconPath;
    private string _subscriptIconPath;
    
    public ExplorerItem(int index, ItemAdapter? itemAdapter)
    {
        _itemName = itemAdapter?.Name ?? "Unknown";
        _iconPath = itemAdapter?.Icon?.Path.ToString() ?? "";
        _subscriptIconPath = itemAdapter?.SubscriptIcon?.Path.ToString() ?? "";
        Index = index;
        ItemAdapter = itemAdapter;
    }
    
    public event PropertyChangedEventHandler? PropertyChanged;

    private void OnPropertyChanged(string propertyName)
    {
        PropertyChanged?.Invoke(this, new PropertyChangedEventArgs(propertyName));
    }
}
