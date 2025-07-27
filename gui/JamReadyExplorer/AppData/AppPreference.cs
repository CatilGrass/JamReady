using System;
using System.IO;
using System.Windows;
using System.Windows.Media;
using Newtonsoft.Json;

namespace JamReadyGui.AppData;

/// <summary>
/// App 首选项
/// </summary>
[Serializable]
public class AppPreference
{
    /// <summary>
    /// 当前所在的地址
    /// </summary>
    public string CurrentPath = "HOME://";

    /// <summary>
    /// 当前语言标识
    /// </summary>
    public string Language = "en_us";

    /// <summary>
    /// 当前主题
    /// </summary>
    public AppTheme Theme = new();

    /// <summary>
    /// 窗口位置
    /// </summary>
    public AppWindowPosition WindowPosition = new();

    public class AppTheme
    {
        public Color MainColor = Color.FromArgb(255,31,31,31);
        public Color MainColorDark = Color.FromArgb(255,28,28,28);
        public Color MainColorLight = Color.FromArgb(255,46,46,46);
        
        public Color AccentColor = Color.FromArgb(255,76,175,80);
        public Color AccentColorDark = Color.FromArgb(255,60,150,62);
        public Color AccentColorLight = Color.FromArgb(255,103,182,95);
        
        public Color ForegroundColor = Colors.LightGray;
        public Color ForegroundColorDark = Color.FromArgb(255,126,126,126);
        public Color ForegroundColorLight = Colors.White;
        
        public Color BackgroundColor = Color.FromArgb(255,20,20,20);
        public Color BackgroundColorDark = Colors.Black;
        public Color BackgroundColorLight = Color.FromArgb(255,30,30,30);
        
        public Color WidgetForegroundColor = Color.FromArgb(255,255,255,255);

        public void Apply()
        {
            Application.Current.Resources["AppMainColor"] = new SolidColorBrush(MainColor);
            Application.Current.Resources["AppMainColorDark"] = new SolidColorBrush(MainColorDark);
            Application.Current.Resources["AppMainColorLight"] = new SolidColorBrush(MainColorLight);
            
            Application.Current.Resources["AccentColor"] = new SolidColorBrush(AccentColor);
            Application.Current.Resources["AccentColorDark"] = new SolidColorBrush(AccentColorDark);
            Application.Current.Resources["AccentColorLight"] = new SolidColorBrush(AccentColorLight);
            
            Application.Current.Resources["ForegroundColor"] = new SolidColorBrush(ForegroundColor);
            Application.Current.Resources["ForegroundColorDark"] = new SolidColorBrush(ForegroundColorDark);
            Application.Current.Resources["ForegroundColorLight"] = new SolidColorBrush(ForegroundColorLight);
            
            Application.Current.Resources["BackgroundColor"] = new SolidColorBrush(BackgroundColor);
            Application.Current.Resources["BackgroundColorDark"] = new SolidColorBrush(BackgroundColorDark);
            Application.Current.Resources["BackgroundColorLight"] = new SolidColorBrush(BackgroundColorLight);
                
            Application.Current.Resources["WidgetForegroundColor"] = new SolidColorBrush(WidgetForegroundColor);
        }
    }
    
    /// <summary>
    /// 窗口位置
    /// </summary>
    public class AppWindowPosition
    {
        public double Top = 0;
        public double Left = 0;
        public double Width = 0;
        public double Height = 0;
    }
    
    /// <summary>
    /// 加载 App 首选项
    /// </summary>
    /// <returns></returns>
    public static AppPreference? LoadPreference()
    {
        var jsonFile = new FileInfo(AppConstants.PreferenceConfigureFile);
        if (! jsonFile.Exists)
        {
            jsonFile.Directory?.Create();
            File.WriteAllText(jsonFile.FullName, JsonConvert.SerializeObject(new AppPreference()));
        }
        return JsonConvert.DeserializeObject<AppPreference>(File.ReadAllText(AppConstants.PreferenceConfigureFile));
    }
    
    /// <summary>
    /// 保存 App 首选项
    /// </summary>
    /// <param name="preference"></param>
    public static void WritePreference(AppPreference preference)
    {
        var jsonFile = new FileInfo(AppConstants.PreferenceConfigureFile);
        File.WriteAllText(jsonFile.FullName, JsonConvert.SerializeObject(preference, Formatting.Indented));
    }

    /// <summary>
    /// 操作首选项
    /// </summary>
    /// <param name="action"></param>
    public static void OperatePreference(Action<AppPreference> action)
    {
        var preference = LoadPreference();
        if (preference != null)
        {
            action(preference);
            WritePreference(preference);
        }
    }
}
