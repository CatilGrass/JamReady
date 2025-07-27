using System.Windows;
using System.Windows.Media;
using JamReadyGui.AppData;
using JamReadyGui.AppData.Explorer;
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
            
            // 初始化
            ExplorerRuntime.InitializeExplorerRuntime();
            
            // 应用颜色
            preference.Theme.Apply();
            
            // 查询所有插件文件
            foreach (var dllFile in AppConstants.GetPluginDllFiles())
            {
                // 加载插件
                PluginLoader.LoadPluginByPath(dllFile);
                
                // 加载其下所有语言文件
                ExplorerRegistry.LoadLanguages(dllFile.Name.Replace(".dll", "").Trim());
            }
            
            // 创建窗口
            var explorer = new Explorer();

            // 设置记录的位置
            if (!(preference.WindowPosition.Left == 0 || preference.WindowPosition.Top == 0 ||
                preference.WindowPosition.Width == 0 || preference.WindowPosition.Height == 0))
            {
                explorer.Left = preference.WindowPosition.Left;
                explorer.Top = preference.WindowPosition.Top;
                explorer.Width = preference.WindowPosition.Width;
                explorer.Height = preference.WindowPosition.Height;
            }

            // 显示窗口
            explorer.Show();
        }
    }
}