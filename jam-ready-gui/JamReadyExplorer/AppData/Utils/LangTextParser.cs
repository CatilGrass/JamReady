using System;
using System.Collections.Generic;
using System.Text;
using System.Text.RegularExpressions;

namespace JamReadyGui.AppData.Utils;

public class LangTextParser
{
    /// <summary>
    /// 转换文本
    /// </summary>
    /// <param name="input"></param>
    /// <returns></returns>
    public static Dictionary<string, string> ParseText(string input)
    {
        var dictionary = new Dictionary<string, string>(StringComparer.OrdinalIgnoreCase);
        var lines = input.Split(new[] { '\n' }, StringSplitOptions.RemoveEmptyEntries);

        foreach (var line in lines)
        {
            // 跳过注释行和空行
            if (string.IsNullOrWhiteSpace(line) || line.TrimStart().StartsWith("#"))
            {
                continue;
            }

            // key: "value"
            var match = Regex.Match(line, @"^\s*([^:]+)\s*:\s*""([^""]*)""\s*$");
            if (match.Success)
            {
                var key = SanitizeKey(match.Groups[1].Value);
                var value = UnescapeValue(match.Groups[2].Value);
                
                if (!string.IsNullOrEmpty(key))
                {
                    dictionary[key] = value;
                }
            }
        }

        return dictionary;
    }

    public static string SanitizeKey(string key)
    {
        // 移除特殊字符
        return Regex.Replace(key.ToLower(), @"[^a-z0-9]", "");
    }

    private static string UnescapeValue(string value)
    {
        // 处理转义符
        var sb = new StringBuilder(value);
        sb.Replace("\\n", "\n")
            .Replace("\\t", "\t")
            .Replace("\\\"", "\"")
            .Replace("\\r", "\r");
        
        return sb.ToString();
    }
}