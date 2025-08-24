using System;
using JamReadyGui.Models.Base;

namespace JamReadyGui.Models;

public class AppConfig : DataFile<AppConfig>
{
    public string Workspace = Environment.CurrentDirectory;
}