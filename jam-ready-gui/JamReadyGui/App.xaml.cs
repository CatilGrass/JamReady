using System.Windows;
using JamReadyGui.AppData;
using JamReadyExplorer = JamReadyGui.AppWindows.AppExplorer.JamReadyExplorer;

namespace JamReadyGui
{
    public partial class App
    {
        protected override void OnStartup(StartupEventArgs e)
        {
            var preference = AppPreference.LoadPreference();
            if (preference == null) return;
            
            new JamReadyExplorer().Show();
        }
    }
}