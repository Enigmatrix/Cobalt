using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Cobalt.Common.Analysis.OutputTypes
{
    public struct Usage<T>
    {
        public bool JustStarted { get; }
        public T Value { get; }

        public Usage(T value = default(T), bool justStarted = false)
        {
            Value = value;
            JustStarted = justStarted;
        }
    }
}
