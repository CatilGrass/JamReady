using System;
using System.IO;
using System.Reflection;

namespace JamReadyGui.AppData.Utils;

public class PluginLoader
{
    /// <summary>
    /// 通过 dll 文件加载插件
    /// </summary>
    /// <param name="file"></param>
    /// <exception cref="InvalidOperationException"></exception>
    public static void LoadPluginByPath(FileInfo file)
    {
        var dllPath = file.FullName;
        try
        {
            Assembly assembly = Assembly.LoadFrom(dllPath);

            Type? pluginType = assembly.GetType($"{file.Name.Trim().Replace(".dll", "")}.Plugin");
            if (pluginType == null)
            {
                throw new InvalidOperationException("Class Plugin could not be found");
            }
            
            object? pluginInstance = Activator.CreateInstance(pluginType);
            
            MethodInfo? registerMethod = pluginType.GetMethod("Register");
            if (registerMethod == null)
            {
                throw new InvalidOperationException("Method Register could not be found");
            }
            
            registerMethod.Invoke(pluginInstance, null);
            Console.WriteLine($"Plugin {file.Name} loaded successfully.");
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Plugin {file.Name} load failed: {ex.Message}");
        }
    }
}