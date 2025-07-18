using System.Windows;
using System.Windows.Input;

namespace JamReadyGui.AppWindows.BaseWindow;

public partial class BaseWindow : Window
{
    public BaseWindow()
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
}