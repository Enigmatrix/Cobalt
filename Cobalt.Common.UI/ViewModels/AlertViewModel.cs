using System;
using Cobalt.Common.Data;

namespace Cobalt.Common.UI.ViewModels
{
    public class AlertViewModel : EntityViewModel
    {
        private AlertAction _alertAction;
        private bool _isEnabled;
        private TimeSpan _maxDuration;
        private AlertRange _range;
        private TimeSpan _reminderOffset;

        public AlertViewModel(Alert alert) : base(alert)
        {
            AlertAction = alert.AlertAction;
            MaxDuration = alert.MaxDuration;
            IsEnabled = alert.IsEnabled;
            ReminderOffset = alert.ReminderOffset;
            Range = alert.Range;
        }

        public AlertAction AlertAction
        {
            get => _alertAction;
            set => Set(ref _alertAction, value);
        }

        public TimeSpan MaxDuration
        {
            get => _maxDuration;
            set => Set(ref _maxDuration, value);
        }

        public bool IsEnabled
        {
            get => _isEnabled;
            set => Set(ref _isEnabled, value);
        }

        public TimeSpan ReminderOffset
        {
            get => _reminderOffset;
            set => Set(ref _reminderOffset, value);
        }

        public AlertRange Range
        {
            get => _range;
            set => Set(ref _range, value);
        }
    }
}