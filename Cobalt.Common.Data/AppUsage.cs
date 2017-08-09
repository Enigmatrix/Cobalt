using System;

namespace Cobalt.Common.Data
{
    public class AppUsage : Entity
    {
        public App App { get; set; }
        public AppUsageType UsageType { get; set; }
        public DateTime StartTimestamp { get; set; }
        public DateTime EndTimestamp { get; set; }
        public AppUsageEndReason UsageEndReason { get; set; }
        public AppUsageStartReason UsageStartReason { get; set; }
        public TimeSpan Duration => EndTimestamp - StartTimestamp;
    }
}