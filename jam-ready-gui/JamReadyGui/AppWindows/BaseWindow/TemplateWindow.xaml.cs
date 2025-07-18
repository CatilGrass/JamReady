using System.Windows;
using System.Windows.Input;

namespace JamReadyGui.AppWindows.BaseWindow;

public partial class TemplateWindow : Window
{
    public TemplateWindow()
    {
        InitializeComponent();
    }

    private void Title_OnMouseDown(object sender, MouseButtonEventArgs e)
    {
        if (e.LeftButton == MouseButtonState.Pressed)
        {
            DragMove();
        }
    }

    private void MinimizeButton(object sender, RoutedEventArgs e)
    {
        WindowState = WindowState.Minimized;
    }

    private void MaximizeButton(object sender, RoutedEventArgs e)
    {
        WindowState = WindowState == WindowState.Maximized ? WindowState.Normal : WindowState.Maximized;
    }

    private void CloseButton(object sender, RoutedEventArgs e)
    {
        Close();
    }
}