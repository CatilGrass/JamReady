using System.Windows;
using System.Windows.Controls;
using System.Windows.Input;

namespace JamReadyGui.AppWindows.StyleBehaviours;

public class TemplateWindowBehaviour
{
     // 最小化
    public static readonly DependencyProperty MinimizeProperty =
        DependencyProperty.RegisterAttached("Minimize", typeof(bool), typeof(TemplateWindowBehaviour),
            new PropertyMetadata(false, OnMinimizeChanged));

    public static bool GetMinimize(DependencyObject obj) => (bool)obj.GetValue(MinimizeProperty);
    public static void SetMinimize(DependencyObject obj, bool value) => obj.SetValue(MinimizeProperty, value);

    private static void OnMinimizeChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
    {
        if (d is Button button)
        {
            button.Click += (_, _) =>
            {
                var window = Window.GetWindow(button);
                if (window != null) window.WindowState = WindowState.Minimized;
            };
        }
    }

    // 最大化
    public static readonly DependencyProperty MaximizeRestoreProperty =
        DependencyProperty.RegisterAttached("MaximizeRestore", typeof(bool), typeof(TemplateWindowBehaviour),
            new PropertyMetadata(false, OnMaximizeRestoreChanged));

    public static bool GetMaximizeRestore(DependencyObject obj) => (bool)obj.GetValue(MaximizeRestoreProperty);
    public static void SetMaximizeRestore(DependencyObject obj, bool value) => obj.SetValue(MaximizeRestoreProperty, value);

    private static void OnMaximizeRestoreChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
    {
        if (d is Button button)
        {
            button.Click += (_, _) =>
            {
                var window = Window.GetWindow(button);
                if (window != null)
                {
                    window.WindowState = window.WindowState == WindowState.Maximized 
                        ? WindowState.Normal 
                        : WindowState.Maximized;
                }
            };
        }
    }

    // 关闭
    public static readonly DependencyProperty CloseProperty =
        DependencyProperty.RegisterAttached("Close", typeof(bool), typeof(TemplateWindowBehaviour),
            new PropertyMetadata(false, OnCloseChanged));

    public static bool GetClose(DependencyObject obj) => (bool)obj.GetValue(CloseProperty);
    public static void SetClose(DependencyObject obj, bool value) => obj.SetValue(CloseProperty, value);

    private static void OnCloseChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
    {
        if (d is Button button)
        {
            button.Click += (_, _) =>
            {
                var window = Window.GetWindow(button);
                window?.Close();
            };
        }
    }

    // 拖拽
    public static readonly DependencyProperty DragProperty =
        DependencyProperty.RegisterAttached("Drag", typeof(bool), typeof(TemplateWindowBehaviour),
            new PropertyMetadata(false, OnDragChanged));

    public static bool GetDrag(DependencyObject obj) => (bool)obj.GetValue(DragProperty);
    public static void SetDrag(DependencyObject obj, bool value) => obj.SetValue(DragProperty, value);

    private static void OnDragChanged(DependencyObject d, DependencyPropertyChangedEventArgs e)
    {
        if (d is UIElement element)
        {
            element.MouseDown += (_, args) =>
            {
                if (args.LeftButton == MouseButtonState.Pressed)
                {
                    Window.GetWindow(element)?.DragMove();
                }
            };
        }
    }
}