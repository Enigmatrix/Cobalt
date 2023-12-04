namespace Cobalt.Common.Util;

public class Debugging
{
#if DEBUG
    public static bool IsDebug => true;
#else
    public static bool IsDebug => false;
#endif
}