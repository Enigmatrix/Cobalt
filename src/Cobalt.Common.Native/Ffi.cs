using System;
using System.Runtime.InteropServices;

namespace Cobalt.Common.Native
{
    public static class Ffi
    {
        public const string Library = "engine.exe";

        [StructLayout(LayoutKind.Sequential)]
        public readonly struct String
        {
            public readonly IntPtr Buffer;
            public readonly IntPtr Capacity;
            public readonly IntPtr Length;

            public override string ToString()
            {
                return Marshal.PtrToStringUni(Buffer, Length.ToInt32());
            }
        }
    }
}
