using System;
using System.Collections.Generic;

namespace JamReadyGui.AppData.Explorer;

/// <summary>
/// 浏览器项适配器
/// </summary>
public abstract class ItemAdapter
{
    /// <summary>
    /// 图标
    /// </summary>
    public ImagePath? Icon { get; set; }
    
    /// <summary>
    /// 下角标图标
    /// </summary>
    public ImagePath? SubscriptIcon { get; set; }

    /// <summary>
    /// 此适配器显示的名称
    /// </summary>
    public string Name = "_";
    
    /// <summary>
    /// 创建此适配器时执行
    /// </summary>
    /// <param name="value"> 用于创建此 Adapter 的参数 </param>
    public abstract ImagePath OnInit(object value);

    /// <summary>
    /// 注册此适配器的操作
    /// </summary>
    /// <returns> 注册的按钮名称 </returns>
    public virtual List<string> OnRegisterOperation()
    {
        return new List<string>();
    }

    /// <summary>
    /// 获得操作的图标
    /// </summary>
    /// <param name="operationIndex"> 索引号 </param>
    /// <returns></returns>
    public virtual ImagePath? GetOperationIcon(int operationIndex) => null;
    
    /// <summary>
    /// 进入此适配器时执行
    /// </summary>
    /// <returns> 是否需要更新页面 </returns>
    public virtual bool OnEnter() { return false; }

    /// <summary>
    /// 操作某个事件
    /// </summary>
    /// <param name="operationIndex"> 操作索引 </param>
    public virtual void OnOperate(int operationIndex) { }
}

/// <summary>
/// 适配器创建器
/// </summary>
public static class AdapterFactory
{
    /// <summary>
    /// 创建适配器
    /// </summary>
    /// <param name="value"></param>
    /// <typeparam name="TAdapter"></typeparam>
    /// <returns></returns>
    public static ItemAdapter Create<TAdapter>(object value)
        where TAdapter : ItemAdapter, new()
    {
        var adapter = new TAdapter();
        var icon = adapter.OnInit(value);
        adapter.Icon = icon;

        return adapter;
    }
}

/// <summary>
/// 图像地址容器
/// </summary>
public class ImagePath
{
    public static ImagePath Empty => new ImagePath(new Uri(""));
    
    public ImagePath(Uri path)
    {
        Path = path;
    }

    public Uri Path { get; set; }
}