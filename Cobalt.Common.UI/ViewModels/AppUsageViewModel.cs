using System;
using Cobalt.Common.Data;

namespace Cobalt.Common.UI.ViewModels
{
    public class AppUsageViewModel : EntityViewModel, IHasDuration
    {
        private AppViewModel _app;
        private DateTime _endTimestamp;
        private DateTime _startTimestamp;
        private AppUsageEndReason _usageEndReason;
        private AppUsageStartReason _usageStartReason;
        private AppUsageType _usageType;

        public AppUsageViewModel(AppUsage au) : base(au)
        {
            _app = new AppViewModel(au.App);
            EndTimestamp = au.EndTimestamp;
            StartTimestamp = au.StartTimestamp;
            UsageStartReason = au.UsageStartReason;
            UsageEndReason = au.UsageEndReason;
            UsageType = au.UsageType;
        }

        public AppViewModel App
        {
            get => _app;
            set => Set(ref _app, value);
        }

        public AppUsageType UsageType
        {
            get => _usageType;
            set => Set(ref _usageType, value);
        }

        public AppUsageEndReason UsageEndReason
        {
            get => _usageEndReason;
            set => Set(ref _usageEndReason, value);
        }

        public AppUsageStartReason UsageStartReason
        {
            get => _usageStartReason;
            set => Set(ref _usageStartReason, value);
        }

        public DateTime StartTimestamp
        {
            get => _startTimestamp;
            set
            {
                Set(ref _startTimestamp, value);
                NotifyOfPropertyChange(() => Duration);
            }
        }

        public DateTime EndTimestamp
        {
            get => _endTimestamp;
            set
            {
                Set(ref _endTimestamp, value);
                NotifyOfPropertyChange(() => Duration);
            }
        }

        public TimeSpan Duration
        {
            get => EndTimestamp - StartTimestamp;
            set => EndTimestamp = StartTimestamp + value;
        }
    }
}