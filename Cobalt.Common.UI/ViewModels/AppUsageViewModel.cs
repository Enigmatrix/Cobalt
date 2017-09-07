using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;
using Cobalt.Common.Data;

namespace Cobalt.Common.UI.ViewModels
{
    public interface IAppUsageViewModel : IViewModel, IHasDuration
    {
        App App { get; }
        AppUsageType UsageType { get; }
        AppUsageStartReason UsageStartReason { get; }
        AppUsageEndReason UsageEndReason { get; }
        DateTime StartTimestamp { get; }
        DateTime EndTimestamp { get; }
    }
    public class AppUsageViewModel : ViewModelBase, IAppUsageViewModel
    {
        private App _app;
        private AppUsageType _usageType;
        private AppUsageEndReason _usageEndReason;
        private AppUsageStartReason _usageStartReason;
        private DateTime _startTimestamp;
        private TimeSpan _duration;
        private DateTime _endTimestamp;

        public AppUsageViewModel(AppUsage au)
        {
            _app = au.App;
            EndTimestamp = au.EndTimestamp;
            StartTimestamp = au.StartTimestamp;
            UsageStartReason = au.UsageStartReason;
            UsageEndReason = au.UsageEndReason;
            UsageType = au.UsageType;
        }

        public App App
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
            set { Set(ref _startTimestamp, value); NotifyOfPropertyChange(() => Duration); }
        }

        public DateTime EndTimestamp
        {
            get => _endTimestamp;
            set { Set(ref _endTimestamp, value); NotifyOfPropertyChange(() => Duration); }
        }

        public TimeSpan Duration
        {
            get => EndTimestamp - StartTimestamp;
            set => EndTimestamp = StartTimestamp + value;
        }
    }
}
