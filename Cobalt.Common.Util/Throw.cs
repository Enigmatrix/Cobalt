using System;
using System.Runtime.CompilerServices;

namespace Cobalt.Common.Util
{
    public static class Throw
    {
        public static T Unreachable<T>(
            [CallerMemberName] string memberName = null,
            [CallerFilePath] string filePath = null,
            [CallerLineNumber] int lineNo = 0)
        {
            throw new Exception($"Unreachable code in {filePath}, {lineNo} ({memberName}) was somehow reached.");
        }
    }
}