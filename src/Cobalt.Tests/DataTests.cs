using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using Microsoft.EntityFrameworkCore;
using Xunit.Abstractions;

namespace Cobalt.Tests;

public class DataTests : IDisposable
{
    private static int _dbCount;
    private readonly int _count;
    private readonly ITestOutputHelper _output;
    private QueryContext _context;

    public DataTests(ITestOutputHelper output)
    {
        _count = Interlocked.Increment(ref _dbCount);
        _context = CreateContext();
        _output = output;
    }

    public void Dispose()
    {
        var connection = _context.Database;
        _context.Dispose();
        connection.EnsureDeleted();
    }

    public QueryContext CreateContext()
    {
        var optionsBuilder = new DbContextOptionsBuilder<QueryContext>();
        QueryContext.ConfigureFor(optionsBuilder, $"Data Source=test-{_count}.db");
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
    public async Task UpdateAlert_CreatesNewAlertWithDuplicatedRemindersAndEmptyEvents_OnNonEmptyEvents()
    {
        var alert = new Alert
        {
            Guid = Guid.NewGuid(),
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
            Guid = Guid.NewGuid(),
            Alert = alert,
            Threshold = 0.2,
            Message = "NO1",
            Version = 1
        };
        var reminder2 = new Reminder
        {
            Guid = Guid.NewGuid(),
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

        _context = CreateContext();

        Assert.Equal(1, _context.Alerts.Count());
        alert = _context.Alerts.Include(x => x.Reminders).ThenInclude(x => x.ReminderEvents).Include(x => x.AlertEvents)
            .First();
        Assert.Equal(2, alert.Reminders.Count);
        Assert.Equal(2, alert.AlertEvents.Count);
        Assert.Single(alert.Reminders[0].ReminderEvents);
        Assert.Single(alert.Reminders[1].ReminderEvents);

        alert.TimeFrame = TimeFrame.Monthly;
        await _context.UpdateAlertAsync(alert);
        Assert.Single(_context.Alerts);

        alert = _context.Alerts.Include(x => x.Reminders).ThenInclude(x => x.ReminderEvents).Include(x => x.AlertEvents)
            .First();
        Assert.Equal(TimeFrame.Monthly, alert.TimeFrame);
        Assert.Equal(2, alert.Reminders.Count);
        Assert.Empty(alert.AlertEvents);
        Assert.Empty(alert.Reminders[0].ReminderEvents);
        Assert.Empty(alert.Reminders[1].ReminderEvents);
    }

    /*
    [Fact]
    public void _PrintGeneratedQuery()
    {
        var query = _context.AlertDurations(
            _context.Alerts
                .Include(alert => alert.App)
                .Include(alert => alert.Tag)
                .Include(alert => alert.AlertEvents)
                .Include(alert => alert.Reminders)
                .ThenInclude(reminder => reminder.ReminderEvents));
        var sql = query.ToQueryString();
        _output.WriteLine(sql);
    }
    */
}