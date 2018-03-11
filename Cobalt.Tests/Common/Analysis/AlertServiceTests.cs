using System;
using Cobalt.Common.Data;
using Cobalt.Common.Util;
using Cobalt.TaskbarNotifier;
using Xunit;

namespace Cobalt.Tests.Common.Analysis
{
    public class AlertServiceTests : IDisposable
    {
        public AlertServiceTests()
        {
            Alerts = new AlertService(null, null, null);
        }

        public void Dispose()
        {
            Alerts = null;
        }

        private AlertService Alerts;

        [Fact]
        public void TestRangeMonthly()
        {
            var current = DateTime.Today.StartOfMonth();
            var range = new RepeatingAlertRange
            {
                RepeatType = RepeatType.Monthly
            };
            Assert.Equal(Alerts.GetEffectiveTimeRange(current.AddDays(4), range),
                new (DateTime, DateTime?)[]
                {
                    (current.AddDays(0), current.AddDays(1)),
                    (current.AddDays(1), current.AddDays(2)),
                    (current.AddDays(2), current.AddDays(3)),
                    (current.AddDays(3), current.AddDays(4)),
                    (current.AddDays(4), current.AddDays(5))
                });
        }

        [Fact]
        public void TestRangeOnce()
        {
            var start = DateTime.Today.AddHours(1);
            DateTime? end = DateTime.Today.AddHours(2);
            var range = new OnceAlertRange
            {
                StartTimestamp = start,
                EndTimestamp = end.Value
            };
            Assert.Equal(Alerts.GetEffectiveTimeRange(DateTime.Today, range),
                new[] {(start, end)});
        }

        [Fact]
        public void TestRangeOnceOutOfToday()
        {
            var start = DateTime.Today.AddHours(-2);
            DateTime? end = DateTime.Today.AddHours(-1);
            var range = new OnceAlertRange
            {
                StartTimestamp = start,
                EndTimestamp = end.Value
            };
            Assert.Equal(Alerts.GetEffectiveTimeRange(DateTime.Today, range),
                new (DateTime, DateTime?)[] { });
        }

        [Fact]
        public void TestRangeWeekdayOnWeekDay()
        {
            var current = DateTime.Today.StartOfWeek();
            var range = new RepeatingAlertRange
            {
                RepeatType = RepeatType.Weekday
            };
            Assert.Equal(Alerts.GetEffectiveTimeRange(current.AddDays((int) DayOfWeek.Friday), range),
                new (DateTime, DateTime?)[]
                {
                    (current.AddDays(1), current.AddDays(2)),
                    (current.AddDays(2), current.AddDays(3)),
                    (current.AddDays(3), current.AddDays(4)),
                    (current.AddDays(4), current.AddDays(5)),
                    (current.AddDays(5), current.AddDays(6))
                });
        }

        [Fact]
        public void TestRangeWeekdayOnWeekEnd()
        {
            var current = DateTime.Today.StartOfWeek();
            var range = new RepeatingAlertRange
            {
                RepeatType = RepeatType.Weekday
            };
            Assert.Equal(Alerts.GetEffectiveTimeRange(current.AddDays(0), range),
                new (DateTime, DateTime?)[] { });
        }

        [Fact]
        public void TestRangeWeekendOnSunday()
        {
            var current = DateTime.Today.StartOfWeek().AddDays((int) DayOfWeek.Sunday);
            var range = new RepeatingAlertRange
            {
                RepeatType = RepeatType.Weekend
            };
            Assert.Equal(Alerts.GetEffectiveTimeRange(current, range),
                new (DateTime, DateTime?)[]
                {
                    (current.AddDays(-1), current.AddDays(0)),
                    (current.AddDays(0), current.AddDays(1))
                });
        }

        [Fact]
        public void TestRangeWeekendOnWeekDay()
        {
            var current = DateTime.Today.StartOfWeek().AddDays(1);
            var range = new RepeatingAlertRange
            {
                RepeatType = RepeatType.Weekend
            };
            Assert.Equal(Alerts.GetEffectiveTimeRange(current, range),
                new (DateTime, DateTime?)[] { });
        }

        [Fact]
        public void TestRangeWeekendOnWeekEnd()
        {
            var current = DateTime.Today.StartOfWeek().AddDays((int) DayOfWeek.Saturday);
            var range = new RepeatingAlertRange
            {
                RepeatType = RepeatType.Weekend
            };
            Assert.Equal(Alerts.GetEffectiveTimeRange(current, range),
                new (DateTime, DateTime?)[] {(current, current.AddDays(1))});
        }
    }
}