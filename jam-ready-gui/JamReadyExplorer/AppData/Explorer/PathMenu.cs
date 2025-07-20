using System.Collections.Generic;

namespace JamReadyGui.AppData.Explorer;

/// <summary>
/// 浏览器目录菜单项
/// </summary>
public abstract class PathMenu
{
    /// <summary>
    /// 获得菜单名称
    /// </summary>
    /// <returns></returns>
    public abstract string GetMenuName();
    
    /// <summary>
    /// 获得自身的图标
    /// </summary>
    /// <returns></returns>
    public virtual ImagePath? GetIcon() => null;
    
    /// <summary>
    /// 注册此目录的操作
    /// </summary>
    /// <returns> 注册的按钮名称 </returns>
    public abstract List<string>? OnRegisterOperation(string path);

    /// <summary>
    /// 获得操作的图标
    /// </summary>
    /// <param name="path"> 按下此按钮的目录 </param>
    /// <param name="operationIndex"> 索引号 </param>
    /// <returns></returns>
    public virtual ImagePath? GetOperationIcon(string path, int operationIndex) => null;

    /// <summary>
    /// 此目录的按钮按下时执行
    /// </summary>
    /// <param name="path"> 按下此按钮的目录 </param>
    /// <param name="operationIndex"> 操作索引 </param>
    public abstract void OnOperate(string path, int operationIndex);
}