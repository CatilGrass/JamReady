using System;
using System.IO;
using System.Reflection;

namespace JamReadyGui.AppData.Utils;

public class PluginLoader
{
    public static void LoadPluginByPath(FileInfo file)
    {
        var dllPath = file.FullName;
        try
        {
            Console.WriteLine($"Loading plugin from: {dllPath}");
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
            Console.WriteLine($"Plugin: {dllPath} loaded successfully.");
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Load plugin failed: {ex.Message}");
        }
    }
}