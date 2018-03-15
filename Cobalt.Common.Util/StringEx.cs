using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

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
