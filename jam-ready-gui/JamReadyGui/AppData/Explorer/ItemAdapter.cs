using System;
using System.Collections.Generic;

namespace JamReadyGui.AppData.Explorer;

/// <summary>
/// 项目适配器
/// </summary>
public abstract class ItemAdapter
{
    /// <summary>
    /// 下角标
    /// </summary>
    public ImagePath? SubscriptIcon { get; set; }
    
    /// <summary>
    /// 图标
    /// </summary>
    public ImagePath? Icon { get; set; }

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
    public abstract List<string> OnRegisterOperation();

    /// <summary>
    /// 获得操作的图标
    /// </summary>
    /// <param name="operationIndex"> 索引号 </param>
    /// <returns></returns>
    public virtual ImagePath? GetOperationIcon(int operationIndex) => null;
    
    /// <summary>
    /// 进入此适配器时执行
    /// </summary>
    public abstract void OnEnter();

    /// <summary>
    /// 操作某个事件
    /// </summary>
    /// <param name="operationIndex"> 操作索引 </param>
    public abstract void OnOperate(int operationIndex);
}

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

public class ImagePath
{
    public ImagePath(Uri path)
    {
        Path = path;
    }

    public Uri Path { get; set; }
}