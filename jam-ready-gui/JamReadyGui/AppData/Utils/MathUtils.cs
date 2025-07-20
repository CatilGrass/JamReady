namespace JamReadyGui.AppData.Utils;

public static class MathUtils
{
    public static float Lerp(float start, float end, float t)
    {
        return start + (end - start) * t;
    }

    public static double Lerp(double start, double end, double t)
    {
        return start + (end - start) * t;
    }
}