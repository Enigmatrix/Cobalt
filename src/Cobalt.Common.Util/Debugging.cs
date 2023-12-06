namespace Cobalt.Common.Util;

/// <summary>
///     Utility to find the DEBUG mode at runtime.
/// </summary>
public class Debugging
{
#if DEBUG
    public static bool IsDebug => true;
#else
    public static bool IsDebug => false;
#endif
}