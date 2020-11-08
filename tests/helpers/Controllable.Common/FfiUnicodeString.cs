using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Vanara.Extensions;
using Vanara.PInvoke;

namespace Controllable.Common
{
    public class FfiUnicodeString
    {
        public static NtDll.UNICODE_STRING FromString(string value)
        {
            var buffer = StringHelper.AllocString(value);
            return new NtDll.UNICODE_STRING
            {
                Buffer = buffer,
                Length = (ushort)value.Length,
                MaximumLength = (ushort)value.Length
            };
        }
    }
}
