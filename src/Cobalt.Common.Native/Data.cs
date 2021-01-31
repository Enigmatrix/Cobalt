
using System;
using System.Runtime.InteropServices;

namespace Cobalt.Common.Native
{
    public class Data
    {
        [DllImport(Ffi.Library)]
        public static extern void migrate(IntPtr handle, ref Ffi.String error);
    }
}
