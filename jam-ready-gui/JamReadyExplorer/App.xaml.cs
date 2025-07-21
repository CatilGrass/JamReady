using System.Windows;
using JamReadyGui.AppData;
using JamReadyGui.AppData.Utils;
using JamReadyGui.AppWindows.AppExplorer;

namespace JamReadyGui
{
    public partial class App
    {
        protected override void OnStartup(StartupEventArgs e)
        {
            // 初始化首选项
            var preference = AppPreference.LoadPreference();
            if (preference == null) return;
            
            // 加载插件
            foreach (var dllFile in AppConstants.GetPluginDllFiles())
            {
                PluginLoader.LoadPluginByPath(dllFile);
            }
            
            // 显示窗口
            new Explorer().Show();
        }
    }
}