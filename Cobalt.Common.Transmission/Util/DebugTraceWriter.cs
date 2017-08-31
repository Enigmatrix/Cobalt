using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Newtonsoft.Json.Serialization;

namespace Cobalt.Common.Transmission.Util
{
    public class DebugTraceWriter : ITraceWriter
    {
        public void Trace(TraceLevel level, string message, Exception ex)
        {
            Debug.Write(message);
        }

        public TraceLevel LevelFilter { get; } = TraceLevel.Verbose;
    }
}
