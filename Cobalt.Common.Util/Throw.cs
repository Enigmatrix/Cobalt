using System;
using System.Security;

namespace Cobalt.Common.Util
{
    public static class Throw
    {
        public static void InvalidOperation(string message)
        {
            throw new InvalidOperationException(message);
        }

        public static void SecurityException(string s)
        {
            throw new SecurityException(s);
        }
    }
}