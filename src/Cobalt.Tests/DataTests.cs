using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Tests;

public class DataTests : IDisposable
{
    private static int _dbCount;
    private readonly int _count;
    private QueryContext _context;

    public DataTests()
    {
        _count = Interlocked.Increment(ref _dbCount);
        _context = CreateContext(_count);
    }

    public void Dispose()
    {
        var connection = _context.Database;
        _context.Dispose();
        connection.EnsureDeleted();
    }

    public QueryContext CreateContext(int count)
    {
        var optionsBuilder = new DbContextOptionsBuilder<QueryContext>();
        QueryContext.ConfigureFor(optionsBuilder, $"Data Source=test-{count}.db");
        var context = new QueryContext(optionsBuilder.Options);
        context.Database.EnsureCreated();
        return context;
    }

    [Fact]
    public void QueryContext_InitializationSucceeds_OnSetup()
    {
        Assert.True(true);
    }

    [Fact]
    public void UpdateAlert_CreatesNewAlertWithDuplicatedRemindersAndEmptyEvents_OnNonEmptyEvents()
    {
        var alert = new Alert
        {
            Version = 1,
            App = null,
            Tag = null,
            UsageLimit = TimeSpan.FromSeconds(5),
            TimeFrame = TimeFrame.Daily,
            TriggerAction = new TriggerAction.Message("Hi")
        };
        alert.AlertEvents.Add(new AlertEvent
        {
            Alert = alert,
            Timestamp = new DateTime(2)
        });
        alert.AlertEvents.Add(new AlertEvent
        {
            Alert = alert,
            Timestamp = new DateTime(3)
        });

        var reminder1 = new Reminder
        {
            Alert = alert,
            Threshold = 0.2,
            Message = "NO1",
            Version = 1
        };
        var reminder2 = new Reminder
        {
            Alert = alert,
            Threshold = 0.3,
            Message = "NO2",
            Version = 1
        };
        alert.Reminders.Add(reminder1);
        alert.Reminders.Add(reminder2);

        reminder1.ReminderEvents.Add(new ReminderEvent
        {
            Reminder = reminder1,
            Timestamp = new DateTime(3)
        });
        reminder2.ReminderEvents.Add(new ReminderEvent
        {
            Reminder = reminder2,
            Timestamp = new DateTime(4)
        });

        _context.Add(alert);
        _context.SaveChanges();
        _context.Dispose();

        _context = CreateContext(_count);

        Assert.Equal(1, _context.Alerts.Count());
        alert = _context.Alerts.Include(x => x.Reminders).ThenInclude(x => x.ReminderEvents).Include(x => x.AlertEvents)
            .First();
        Assert.Equal(2, alert.Reminders.Count);
        Assert.Equal(2, alert.AlertEvents.Count);
        Assert.Single(alert.Reminders[0].ReminderEvents);
        Assert.Single(alert.Reminders[1].ReminderEvents);

        alert.TimeFrame = TimeFrame.Monthly;
        _context.UpdateAlert(alert);
        Assert.Single(_context.Alerts);

        /*alert = _context.Alerts.Include(x => x.Reminders).ThenInclude(x => x.ReminderEvents).Include(x => x.AlertEvents)
            .First();
        Assert.Equal(TimeFrame.Daily, alert.TimeFrame);
        Assert.Equal(2, alert.Reminders.Count);
        Assert.Equal(2, alert.AlertEvents.Count);
        Assert.Single(alert.Reminders[0].ReminderEvents);
        Assert.Single(alert.Reminders[1].ReminderEvents);*/

        alert = _context.Alerts.Include(x => x.Reminders).ThenInclude(x => x.ReminderEvents).Include(x => x.AlertEvents)
            .First();
        Assert.Equal(TimeFrame.Monthly, alert.TimeFrame);
        Assert.Equal(2, alert.Reminders.Count);
        Assert.Empty(alert.AlertEvents);
        Assert.Empty(alert.Reminders[0].ReminderEvents);
        Assert.Empty(alert.Reminders[1].ReminderEvents);
    }
}