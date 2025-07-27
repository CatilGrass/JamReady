using System;
using System.Collections.Generic;
using System.IO;
using JamReadyGui.AppData.Explorer;
using JamReadyGui.AppData.Utils;
using Plugin_FileManager.PluginAdapters;

namespace Plugin_FileManager.PluginInserters;

public class DiskSelectInserter : ItemInserter
{
    public override List<ItemAdapter?> GetAdapters(ExplorerPath path)
    {
        var result = new List<ItemAdapter?>();
        if (
            path.Prefix != Plugin.PluginPrefix || // 路径前缀不匹配
            path.Path.Trim() != "_" // 路径内地址不为 "_"
        ) 
            return result;

        foreach (var driveLetter in GetAllDriveLetters())
        {
            result.Add(AdapterFactory.Create<DiskAdapter>(driveLetter));
        }
        
        return result;
    }
    
    /// <summary>
    /// 获得系统盘符
    /// </summary>
    /// <returns></returns>
    private static List<string> GetAllDriveLetters()
    {
        List<string> drives = new List<string>();
        
        foreach (DriveInfo drive in DriveInfo.GetDrives())
        {
            try
            {
                if (drive.DriveType == DriveType.Fixed || 
                    drive.DriveType == DriveType.Network)
                {
                    // 获取盘符
                    string root = drive.RootDirectory.ToString();
                    if (root.EndsWith("\\"))
                    {
                        drives.Add(root.Substring(0, root.Length - 1));
                    }
                    else
                    {
                        drives.Add(root);
                    }
                }
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Failed {drive.Name}: {ex.Message}");
            }
        }
        
        return drives;
    }
}