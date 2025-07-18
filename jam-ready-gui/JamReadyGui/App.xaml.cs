using System.Windows;
using JamReadyGui.AppData;
using JamReadyGui.AppWindows.AppEntry;

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