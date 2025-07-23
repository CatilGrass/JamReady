using System;
using System.Collections.Generic;
using System.IO;
using JamReadyGui.AppData;
using JamReadyGui.AppData.Explorer;
using JamReadyGui.AppData.Utils;
using Plugin_FileManager.PluginAdapters;

namespace Plugin_FileManager.PluginInserters;

public class DirectorySelectInserter : ItemInserter
{
    public override List<ItemAdapter?> GetAdapters(ExplorerPath path)
    {
        var result = new List<ItemAdapter?>();
        if (
            path.Prefix != Plugin.PluginPrefix || // 路径前缀不匹配
            string.IsNullOrWhiteSpace(path.Path) // 路径内地址为空
            ) 
            return result;
        
        // 获得目录
        var directory = new DirectoryInfo(path.Path);
        if (directory.Exists)
        {
            // 插入返回上一级按钮
            if (directory.Parent != null) 
                result.Add(AdapterFactory.Create<ParentDirectoryAdapter>(directory.Parent));
            else // 若上级目录不存在 （说明文件根或文件丢失）
                result.Add(AdapterFactory.Create<ParentDirectoryAdapter>("")); // 返回磁盘选择

            // 插入所有文件夹
            foreach (var directoryInfo in directory.GetDirectories())
            {
                result.Add(AdapterFactory.Create<DirectoryAdapter>(directoryInfo));
            }

            // 插入所有文件
            foreach (var fileInfo in directory.GetFiles())
            {
                result.Add(AdapterFactory.Create<FileAdapter>(fileInfo));
            }
        }
        
        return result;
    }
}