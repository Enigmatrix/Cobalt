using System;

namespace Cobalt.Common.Util
{
    public static class StringEx
    {
        public static bool StrContains(this string n1, string n2)
        {
            return n1?.IndexOf(n2, StringComparison.OrdinalIgnoreCase) >= 0;
        }
    }
}