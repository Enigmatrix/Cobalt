using System;
using System.Runtime.InteropServices;

namespace Cobalt.Common.Native
{
    public static class Ffi
    {
        public const string Library = "exports.dll";

        [StructLayout(LayoutKind.Explicit)]
        public struct Result
        {
            [FieldOffset(0)] public long Tag;
            [FieldOffset(8)] public long Ok;
            [FieldOffset(8)] public String Error;

            public long Value()
            {
                if (Tag != 0)
                {
                    throw new SEHException(Error.ToString());
                }

                return Ok;
            }
        }

        [StructLayout(LayoutKind.Sequential)]
        public readonly struct String
        {
            public readonly IntPtr Buffer;
            public readonly IntPtr Capacity;
            public readonly IntPtr Length;

            public override string ToString()
            {
                return Marshal.PtrToStringUTF8(Buffer, Length.ToInt32());
            }
        }
    }
}
