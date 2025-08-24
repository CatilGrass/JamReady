using System;
using System.IO;
using YamlDotNet.Serialization;

namespace JamReadyGui.Models.Base;

public abstract class DataFileBase
{
    [YamlIgnore] 
    public static DirectoryInfo ConfigPath => new(Environment.ProcessPath + "/../Configs/");
        
    [YamlIgnore]
    public FileInfo DataFilePath => GetDataPath(GetType());

    public static FileInfo GetDataPath(Type type)
    {
        return new(ConfigPath.FullName + type.Name + ".yaml");
    }
}