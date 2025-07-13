using System;
using System.Windows;

namespace JamReadyGui
{
    public partial class App
    {
        protected override void OnStartup(StartupEventArgs e)
        {
            MainWindow window = new MainWindow();
            // window.Show();
            
            Console.WriteLine(AppDomain.CurrentDomain.BaseDirectory);
            
        }
    }
}