using System;
using System.Diagnostics;
using Newtonsoft.Json.Serialization;
using Serilog;

namespace Cobalt.Common.Transmission.Util
{
    public class DebugTraceWriter : ITraceWriter
    {
        public void Trace(TraceLevel level, string message, Exception ex)
        {
            Log.Information(message);
        }

        public TraceLevel LevelFilter { get; } = TraceLevel.Verbose;
    }
}