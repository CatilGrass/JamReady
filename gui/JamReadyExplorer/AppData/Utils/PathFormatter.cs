namespace JamReadyGui.AppData.Utils;

using System;
using System.IO;
using System.Text;
using System.Text.RegularExpressions;

public static class PathFormatter
{
    /// <summary>
    /// 无效的路径字符
    /// </summary>
    private static readonly char[] InvalidPathChars = Path.GetInvalidPathChars();
    
    /// <summary>
    /// 无效的文件字符
    /// </summary>
    private static readonly char[] InvalidFileNameChars = Path.GetInvalidFileNameChars();
    
    /// <summary>
    /// 格式化路径字符串
    /// </summary>
    /// <param name="inputPath"> 输入的路径字符串 </param>
    /// <returns></returns>
    public static string FormatPath(string inputPath)
    {
        if (string.IsNullOrWhiteSpace(inputPath))
            return string.Empty;
        
        // 处理盘符
        bool hasDriveLetter = inputPath.Length >= 2 && char.IsLetter(inputPath[0]) && inputPath[1] == ':';
        
        // 处理不友好字符
        var builder = new StringBuilder();
        for (int i = 0; i < inputPath.Length; i++)
        {
            char c = inputPath[i];
            
            // 保留盘符
            if (i == 0 && hasDriveLetter)
            {
                builder.Append(char.ToUpper(c));
                continue;
            }
            if (i == 1 && hasDriveLetter && c == ':')
            {
                builder.Append(c);
                continue;
            }
            
            // 替换分隔符
            if (c == '\\')
            {
                builder.Append('/');
                continue;
            }
            
            // 去除字符
            if (c != '/' && (Array.IndexOf(InvalidPathChars, c) >= 0 || Array.IndexOf(InvalidFileNameChars, c) >= 0))
            {
                continue;
            }
            
            builder.Append(c);
        }
        
        // 合并连续斜杠
        string result = Regex.Replace(builder.ToString(), "/+", "/");
        
        return result.Trim();
    }
}