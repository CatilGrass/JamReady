using System.IO;
using System.Text;
using YamlDotNet.Serialization;
using YamlDotNet.Serialization.NamingConventions;

namespace JamReadyGui.Data.Base;

public class DataFile<TData> : DataFileBase where TData : DataFileBase, new()
{
    public void Save()
    {
        Directory.CreateDirectory(ConfigPath.FullName);
        File.WriteAllText(DataFilePath.FullName, ToYaml(this), Encoding.UTF8);
    }

    public static TData Read()
    {
        var file = GetDataPath(typeof(TData));
        if (file.Exists)
        {
            return FromYaml(File.ReadAllText(file.FullName, Encoding.UTF8));
        }
        return new TData();
    }

    public static string ToYaml(object data)
    {
        ISerializer serializer = new SerializerBuilder()
            .WithNamingConvention(CamelCaseNamingConvention.Instance)
            .Build();
        return serializer.Serialize(data);
    }
    
    public static TData FromYaml(string yamlContent)
    {
        IDeserializer deserializer = new DeserializerBuilder()
            .WithNamingConvention(CamelCaseNamingConvention.Instance)
            .Build();
        return deserializer.Deserialize<TData>(yamlContent);
    }
}