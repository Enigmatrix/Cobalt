using System;
using System.Collections.Generic;
using System.Text;

namespace Cobalt.Common.Data.Entities
{

    public enum AppUsageStartReason : long
    {
        Switch = 0L,
        Start = 1L,
        Resume = 2L,
        MonitorOn = 3L
    }

    public enum AppUsageEndReason : long
    {
        Switch = 0L,
        Shutdown = 1L,
        Logoff = 2L,
        Suspend = 3L,
        MonitorOff = 4L
    }

    public enum AppUsageType : long
    {
        Focus = 0L,
        InView = 1L
    }

    public class AppUsage : Entity
    {
        public App App { get; set; }
        public DateTime Start { get; set; }
        public DateTime End { get; set; }
        public AppUsageStartReason StartReason { get; set; }
        public AppUsageEndReason EndReason { get; set; }
        public AppUsageType UsageType { get; set; }
    }
}
