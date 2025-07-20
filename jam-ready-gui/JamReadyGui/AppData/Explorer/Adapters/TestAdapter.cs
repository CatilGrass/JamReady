using System;
using System.Collections.Generic;
using System.Windows.Media.Imaging;

namespace JamReadyGui.AppData.Explorer.Adapters;

public class TestAdapter : ItemAdapter
{
    public override ImagePath OnInit(object value)
    {
        if (value is string valueStr)
        {
            Name = valueStr;
        }
        SubscriptIcon = new ImagePath(new Uri("pack://application:,,,/AppResources/Assets/Icons/poor_folder_icon.png"));
        return new ImagePath(new Uri("pack://application:,,,/AppResources/Assets/Icons/poor_folder_icon.png"));
    }

    public override List<string> OnRegisterOperation()
    {
        return new()
        {
            "File Button 1",
            "File Button 2",
            "File Button 3",
            "File Button 4",
            "File Button 5"
        };
    }

    public override void OnEnter()
    {
        Console.WriteLine("Entered");
    }

    public override void OnOperate(int operationIndex)
    {
        Console.WriteLine($"Press Button: {operationIndex}");
    }

    public override ImagePath? GetOperationIcon(int operationIndex)
    {
        return new ImagePath(new Uri("pack://application:,,,/AppResources/Assets/Icons/poor_folder_icon.png"));
    }
}