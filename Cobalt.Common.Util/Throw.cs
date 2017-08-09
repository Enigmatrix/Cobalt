using System;

namespace Cobalt.Common.Util
{
    public static class Throw
    {
        public static void InvalidOperation(string message)
        {
            throw new InvalidOperationException(message);
        }
    }
}