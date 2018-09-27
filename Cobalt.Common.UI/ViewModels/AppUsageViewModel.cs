using System;
using System.Reactive.Linq;
using Cobalt.Common.Data;
using Cobalt.Common.IoC;
using Cobalt.Common.Util;

namespace Cobalt.Common.UI.ViewModels
{
    public class AppUsageViewModel : EntityViewModel, IHasDuration
    {

        public AppUsageViewModel(AppUsage au) : base(au)
        {
            App = new AppViewModel(au.App);
            EndTimestamp = au.EndTimestamp;
            StartTimestamp = au.StartTimestamp;
            UsageStartReason = au.UsageStartReason;
            UsageEndReason = au.UsageEndReason;
            UsageType = au.UsageType;
        }

        public AppViewModel App { get; set; }

        public AppUsageType UsageType { get; set; }

        public AppUsageEndReason UsageEndReason { get; set; }

        public AppUsageStartReason UsageStartReason { get;set; }

        public DateTime StartTimestamp { get; set; }

        public DateTime EndTimestamp { get; set; }

        public TimeSpan Duration
        {
            get => EndTimestamp - StartTimestamp;
            set => EndTimestamp = StartTimestamp + value;
        }
    }
}