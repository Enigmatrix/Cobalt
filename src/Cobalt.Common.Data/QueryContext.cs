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

    public void UpdateAlert(Alert alert)
    {
        var newAlert = alert.Clone();
        newAlert.Version++;
        var newReminders =
            (alert.Reminders.Count == 0 ? Reminders.Where(x => x.Alert == alert).ToList() : alert.Reminders).Select(
                reminder =>
                {
                    var newReminder = reminder.Clone();
                    newReminder.Alert = newAlert;
                    newReminder.Guid = Guid.NewGuid();
                    newReminder.ReminderEvents.Clear();
                    return newReminder;
                });
        newAlert.AlertEvents.Clear();
        newAlert.Reminders.Clear();
        newAlert.Reminders.AddRange(newReminders);
        Add(newAlert);
        SaveChanges();
    }

    public void UpdateReminder(Reminder reminder)
    {
        var newReminder = reminder.Clone();
        newReminder.Version++;
        Add(newReminder);
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
    }
}