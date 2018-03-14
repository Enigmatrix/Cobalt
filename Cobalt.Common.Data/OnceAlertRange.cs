using System;

namespace Cobalt.Common.Data
{
    public class OnceAlertRange : AlertRange
    {
        public DateTime StartTimestamp { get; set; }
        public DateTime EndTimestamp { get; set; }
    }
}