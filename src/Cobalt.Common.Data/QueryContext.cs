using Cobalt.Common.Data.Entities;
using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Storage.ValueConversion;

namespace Cobalt.Common.Data;

public class QueryContext : DbContext
{
    public QueryContext(DbContextOptions<QueryContext> options) : base(options)
    {
    }

    public DbSet<App> Apps { get; set; } = null!;
    public DbSet<Session> Sessions { get; set; } = null!;
    public DbSet<Usage> Usages { get; set; } = null!;
    public DbSet<Tag> Tags { get; set; } = null!;
    public DbSet<InteractionPeriod> InteractionPeriods { get; set; } = null!;
    public DbSet<Alert> Alerts { get; set; } = null!;
    public DbSet<Reminder> Reminders { get; set; } = null!;
    public DbSet<AlertEvent> AlertEvents { get; set; } = null!;
    public DbSet<ReminderEvent> ReminderEvents { get; set; } = null!;

    public IQueryable<(App App, DateTime Duration)> AppDurations()
    {
        return Apps.Select(app => ValueTuple.Create(app,
            new DateTime(app.Sessions.SelectMany(session =>
                    session.Usages.Select(usage =>
                        // IEnumerable<DateTime>.Sum() cannot be translated to a SQL query by EntityFramework.
                        // ref: https://github.com/dotnet/efcore/issues/27103 and https://github.com/dotnet/efcore/issues/10434 

                        // Instead we access the backing `long` value, and summation of that is valid. Surprisingly, converting
                        // `long` to `TimeSpan` using the constructor is perfectly valid!
                        EF.Property<long>(usage, nameof(usage.End)) - EF.Property<long>(usage, nameof(usage.Start))))
                .Sum())
        ));
    }

    public void UpdateAlert(Alert alert)
    {
        /*
         * If there is any AlertEvent or ReminderEvent associated with this Alert, then we must create a new Alert with higher Version
         * and duplicate the Reminders, with empty AlertEvents and ReminderEvents. Otherwise, we can just update.
         * 
         * Note that an AlertEvent or ReminderEvent can be generated after we check for their existence; this causes a race condition.
         * The effect is that the event is added to an Alert that does not match the initial Alert that triggered it, but has the same
         * identity. This is an rare occurrence, and we will not bother fixing it. For reference, the fix would be to have field called
         * LastUpdate (set on update) in Alert. The event needs to be conditionally inserted if the Timestamp of the event is after this
         * LastUpdate, else no insert should occur. Additionally, a system-wide mutex needs to be held from the time of the check till
         * the time of the update that prevents insertion of events. This is so that LastUpdated is accurately set after the comparison.
         */
        var anyAlertEvents = alert.AlertEvents.Count != 0 || AlertEvents.Any(alertEvent => alertEvent.Alert == alert);
        var anyReminderEvents = alert.Reminders.Any(reminder => reminder.ReminderEvents.Count != 0) ||
                                ReminderEvents.Any(reminderEvent => reminderEvent.Reminder.Alert == alert);

        if (anyAlertEvents || anyReminderEvents)
        {
            var newAlert = alert.Clone();
            newAlert.Version++;
            var newReminders =
                (alert.Reminders.Count == 0 ? Reminders.Where(x => x.Alert == alert).ToList() : alert.Reminders).Select(
                    reminder =>
                    {
                        var newReminder = reminder.Clone();
                        newReminder.Guid = Guid.NewGuid();
                        newReminder.Version = 1;
                        newReminder.Alert = newAlert;
                        newReminder.ReminderEvents.Clear();
                        return newReminder;
                    });
            newAlert.AlertEvents.Clear();
            newAlert.Reminders.Clear();
            newAlert.Reminders.AddRange(newReminders);
            Add(newAlert);
        }
        else
        {
            UpdateAlert(alert);
        }

        SaveChanges();
    }

    public void UpdateReminder(Reminder reminder)
    {
        /*
         * If there is any ReminderEvent associated with this Reminder, then we must create a new Reminder with higher Version
         * with empty ReminderEvents. Otherwise, we can just update.
         * 
         * Note that an ReminderEvent can be generated after we check for their existence; this causes a race condition.
         * The effect is that the event is added to an Reminder that does not match the initial Reminder that triggered it, but has the same
         * identity. This is an rare occurrence, and we will not bother fixing it. The fix is similar to the one in UpdateAlert, with a field
         * called LastUpdate in this Reminder as well.
         */

        var anyReminderEvents = reminder.ReminderEvents.Count != 0 ||
                                ReminderEvents.Any(reminderEvent => reminderEvent.Reminder == reminder);
        if (anyReminderEvents)
        {
            var newReminder = reminder.Clone();
            newReminder.Version++;
            Add(newReminder);
        }
        else
        {
            Update(reminder);
        }

        SaveChanges();
    }

    public static void ConfigureFor(DbContextOptionsBuilder optionsBuilder, string connectionString)
    {
        optionsBuilder
            .UseSqlite(connectionString)
            // We will be doing massive reads, and very little writes
            .UseQueryTrackingBehavior(QueryTrackingBehavior.NoTracking)
            .UseSnakeCaseNamingConvention();
    }

    protected override void ConfigureConventions(ModelConfigurationBuilder configurationBuilder)
    {
        configurationBuilder.Properties<DateTime>().HaveConversion<DateTimeToTicksConverter>();
        configurationBuilder.Properties<TimeSpan>().HaveConversion<TimeSpanToTicksConverter>();
    }

    protected override void OnModelCreating(ModelBuilder modelBuilder)
    {
        // The many-to-many relation table should be called _app_tags.
        // Leaving this out generates a table called app_tag
        modelBuilder.Entity<App>()
            .HasMany(e => e.Tags)
            .WithMany(e => e.Apps)
            .UsingEntity("_app_tags",
                l => l.HasOne(typeof(Tag)).WithMany().HasForeignKey("tag_id").HasPrincipalKey(nameof(Tag.Id)),
                r => r.HasOne(typeof(App)).WithMany().HasForeignKey("app_id").HasPrincipalKey(nameof(App.Id)),
                j => j.HasKey("app_id", "tag_id")
            );

        // Only take the Alert with the highest Versions
        modelBuilder.Entity<Alert>().HasQueryFilter(alert =>
            alert.Version == Alerts
                .Where(otherAlert => otherAlert.Guid == alert.Guid)
                .Max(otherAlert => otherAlert.Version));

        // Only take the Reminder with the highest Versions
        modelBuilder.Entity<Reminder>().HasQueryFilter(reminder =>
            reminder.Version == Reminders
                .Where(otherReminder => otherReminder.Guid == reminder.Guid)
                .Max(otherReminder => otherReminder.Version));
    }
}