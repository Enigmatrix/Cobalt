using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

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
