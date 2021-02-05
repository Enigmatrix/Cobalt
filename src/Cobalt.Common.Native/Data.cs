
using System;
using System.Runtime.InteropServices;

namespace Cobalt.Common.Native
{
    public class Data
    {
        [DllImport(Ffi.Library)]
        public static extern Ffi.Result migrate(string conn);
        [DllImport(Ffi.Library)]
        public static extern long add(long a, long b, ref Ffi.Result result);
    }
}
