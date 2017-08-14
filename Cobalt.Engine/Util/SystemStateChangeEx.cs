using System;
using System.Linq;
using Cobalt.Common.Data;

namespace Cobalt.Engine.Util
{
    public static class SystemStateChangeEx
    {
        private static readonly SystemStateChange[] StartRecordingEvents =
        {
            SystemStateChange.MonitorOn,
            SystemStateChange.Resume
        };

        public static bool IsStartRecordingEvent(this SystemStateChange ssc)
        {
            return StartRecordingEvents.Contains(ssc);
        }

        public static AppUsageStartReason ToStartReason(this SystemStateChange ssc)
        {
            switch (ssc)
            {
                case SystemStateChange.MonitorOn:
                    return AppUsageStartReason.MonitorOn;
                case SystemStateChange.Resume:
                    return AppUsageStartReason.Resume;
                default:
                    throw new Exception();
            }
        }

        public static AppUsageEndReason ToEndReason(this SystemStateChange ssc)
        {
            switch (ssc)
            {
                case SystemStateChange.MonitorOff:
                    return AppUsageEndReason.MonitorOff;
                case SystemStateChange.Suspend:
                    return AppUsageEndReason.Suspend;
                case SystemStateChange.Logoff:
                    return AppUsageEndReason.Logoff;
                case SystemStateChange.Shutdown:
                    return AppUsageEndReason.Shutdown;
                default:
                    throw new Exception();
            }
        }
    }
}