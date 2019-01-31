using System;
using System.Collections.Generic;
using System.Linq;
using System.Runtime.CompilerServices;
using System.Text;
using System.Threading.Tasks;

namespace Cobalt.Common.Util
{
    public static class Throw
    {
        public static T Unreachable<T>(
            [CallerMemberName]string memberName = null,
            [CallerFilePath]string filePath = null,
            [CallerLineNumber]int lineNo = 0)
        {
            throw new Exception($"Unreachable code in {filePath}, {lineNo} ({memberName}) was somehow reached.");
        }
    }
}
