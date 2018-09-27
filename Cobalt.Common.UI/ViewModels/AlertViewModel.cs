using System;
using Cobalt.Common.Data;

namespace Cobalt.Common.UI.ViewModels
{
    public class AlertViewModel : EntityViewModel
    {
        public AlertViewModel(Alert alert) : base(alert)
        {
            AlertAction = alert.AlertAction;
            MaxDuration = alert.MaxDuration;
            IsEnabled = alert.IsEnabled;
            ReminderOffset = alert.ReminderOffset;
            Range = alert.Range;
        }

        public AlertAction AlertAction { get; set; }

        public TimeSpan MaxDuration { get; set; }

        public bool IsEnabled { get; set; }

        public TimeSpan ReminderOffset { get; set; }

        public AlertRange Range { get; set; }
    }
}