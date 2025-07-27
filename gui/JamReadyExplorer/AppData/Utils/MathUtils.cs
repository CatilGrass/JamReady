namespace JamReadyGui.AppData.Utils;

/// <summary>
/// 数学相关工具
/// </summary>
public static class MathUtils
{
    /// <summary>
    /// 线性插值 (Float)
    /// </summary>
    /// <param name="start"></param>
    /// <param name="end"></param>
    /// <param name="t"></param>
    /// <returns></returns>
    public static float Lerp(float start, float end, float t)
    {
        return start + (end - start) * t;
    }
    
    /// <summary>
    /// 线性插值 (Double)
    /// </summary>
    /// <param name="start"></param>
    /// <param name="end"></param>
    /// <param name="t"></param>
    /// <returns></returns>
    public static double Lerp(double start, double end, double t)
    {
        return start + (end - start) * t;
    }
}