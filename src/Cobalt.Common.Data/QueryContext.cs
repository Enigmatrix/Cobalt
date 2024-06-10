using Cobalt.Common.Data.Entities;
using Microsoft.Data.Sqlite;
using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.ChangeTracking;
using Microsoft.EntityFrameworkCore.Storage.ValueConversion;
using Microsoft.EntityFrameworkCore.ValueGeneration;

namespace Cobalt.Common.Data;

/// <summary>
///     Database Query Context
/// </summary>
public class QueryContext : DbContext
{
    /// <summary>
    ///     Constructor needed for using with AddPooledDbContextFactory
    /// </summary>
    /// <param name="options">OnConfiguring options sent to the constructor</param>
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

    // These searches might benefit from FTS or at least a case-insensitive collation

    /// <summary>
    ///     Search Apps using a query
    /// </summary>
    public IQueryable<App> SearchApps(string query)
    {
        return Apps.Where(app =>
            app.Name.ToLower().Contains(query.ToLower()) ||
            app.Description.ToLower().Contains(query.ToLower()) ||
            app.Company.ToLower().Contains(query.ToLower()));
    }

    /// <summary>
    ///     Search Tags using a query
    /// </summary>
    public IQueryable<Tag> SearchTags(string query)
    {
        return Tags.Where(tag =>
            tag.Name.ToLower().Contains(query.ToLower()));
    }

    /// <summary>
    ///     Get the Icon of an <see cref="App" />
    /// </summary>
    /// <param name="app"></param>
    /// <returns>Icon as bytes</returns>
    public async Task<byte[]?> GetAppIconBytes(App app)
    {
        return await Database.SqlQuery<byte[]?>($"SELECT icon AS Value FROM apps WHERE id={app.Id}").FirstAsync();
    }

    // TODO document this
    private static IQueryable<WithTicks<App>> AppDurationTicksOnly(IQueryable<App> apps, DateTime? start = null,
        DateTime? end = null)
    {
        var startTicks = (start ?? DateTime.MinValue).Ticks;
        var endTicks = (end ?? DateTime.MaxValue).Ticks;

        return apps.Select(app => new
                {
                    App = app,
                    Ticks = app.Sessions.AsQueryable()
                        .SelectMany(session => session.Usages)
                        .Where(usage => usage.EndTicks > startTicks && endTicks >= usage.StartTicks)
                        .Select(usage => Math.Min(endTicks, usage.EndTicks) - Math.Max(startTicks, usage.StartTicks))
                        .Sum()
                })
                .Select(appTicks => new WithTicks<App> { Inner = appTicks.App, Ticks = appTicks.Ticks })
            ;
    }

    /// <summary>
    ///     Gets <see cref="App" /> and their Durations
    /// </summary>
    /// <param name="apps">Pre-initialized <see cref="App" /> with e.g. Include queries or filters</param>
    /// <param name="start">Start range time. A null value implies no limit on the start time</param>
    /// <param name="end">End range time. A null value implies no limit on the end time</param>
    public IQueryable<WithDuration<App>> AppDurations(IQueryable<App>? apps = null, DateTime? start = null,
        DateTime? end = null)
    {
        return AppDurationTicksOnly(apps ?? Apps, start, end)
            .Where(appTicks => appTicks.Ticks > 0)
            .Select(appTicks => new WithDuration<App>(appTicks.Inner, new TimeSpan(appTicks.Ticks)));
    }

    // TODO document this
    public async Task<TimeSpan> UsagesBetween(IQueryable<Usage>? usages = null, DateTime? start = null,
        DateTime? end = null)
    {
        var startTicks = (start ?? DateTime.MinValue).Ticks;
        var endTicks = (end ?? DateTime.MaxValue).Ticks;

        var dur = await (usages ?? Usages)
            .Where(usage => usage.EndTicks > startTicks && endTicks >= usage.StartTicks)
            .Select(usage => Math.Min(endTicks, usage.EndTicks) - Math.Max(startTicks, usage.StartTicks))
            .SumAsync();
        return new TimeSpan(dur);
    }


    /// <summary>
    ///     Gets <see cref="Alert" /> and their Durations
    /// </summary>
    /// <param name="alerts">Pre-initialized <see cref="Alert" /> with e.g. Include queries or filters</param>
    public IQueryable<WithDuration<Alert>> AlertDurations(IQueryable<Alert>? alerts = null)
    {
        var today = DateTime.Today;

        var todayStart = today;
        var todayEnd = todayStart.AddDays(1);
        var weekStart = today.AddDays(-(int)today.DayOfWeek);
        var weekEnd = weekStart.AddDays(7);
        var monthStart = today.AddDays(1 - today.Day);
        var monthEnd = monthStart.AddMonths(1);

        return (alerts ?? Alerts).Select(alert => new
        {
            Alert = alert,
            // Nasty way to get correct (start,end) bounds
            LimitStartTicks = alert.TimeFrame == TimeFrame.Daily ? todayStart.Ticks :
                alert.TimeFrame == TimeFrame.Weekly ? weekStart.Ticks : monthStart.Ticks,
            LimitEndTicks = alert.TimeFrame == TimeFrame.Daily ? todayEnd.Ticks :
                alert.TimeFrame == TimeFrame.Weekly ? weekEnd.Ticks : monthEnd.Ticks
        }).Select(alertInfo =>
            new WithDuration<Alert>(alertInfo.Alert,
                new TimeSpan(Apps
                    .Where(app =>
                        alertInfo.Alert.App == null
                            ? alertInfo.Alert.Tag!.Apps.Contains(app)
                            : alertInfo.Alert.App == app)
                    // Copied from AppDurationsTickOnly - can't seem to use the start/end as expressions
                    .SelectMany(app => app.Sessions
                        .SelectMany(session => session.Usages)
                        .Where(usage =>
                            usage.EndTicks > alertInfo.LimitStartTicks && alertInfo.LimitEndTicks >= usage.StartTicks)
                        .Select(usage =>
                            Math.Min(alertInfo.LimitEndTicks, usage.EndTicks) -
                            Math.Max(alertInfo.LimitStartTicks, usage.StartTicks))
                    ).Sum())));
    }

    /// <summary>
    ///     Updates an <see cref="Alert" />, creating a new version of the <see cref="Alert" /> if necessary.
    /// </summary>
    public async Task UpdateAlertAsync(Alert alert)
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
        var anyAlertEvents = alert.AlertEvents.Count != 0 ||
                             await AlertEvents.AnyAsync(alertEvent => alertEvent.Alert == alert);
        var anyReminderEvents = alert.Reminders.Any(reminder => reminder.ReminderEvents.Count != 0) ||
                                await ReminderEvents.AnyAsync(reminderEvent => reminderEvent.Reminder.Alert == alert);

        if (anyAlertEvents || anyReminderEvents)
        {
            var newAlert = alert.Clone();
            newAlert.Version++;
            var newReminders =
                (alert.Reminders.Count == 0
                    ? await Reminders.Where(x => x.Alert == alert).ToListAsync()
                    : alert.Reminders).Select(
                    reminder =>
                    {
                        var newReminder = reminder.Clone();
                        newReminder.Id = 0;
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
            Update(alert);
        }

        await SaveChangesAsync();
    }

    /// <summary>
    ///     Updates an <see cref="Reminder" />, creating a new version of the <see cref="Reminder" /> if necessary.
    /// </summary>
    public async Task UpdateReminderAsync(Reminder reminder)
    {
        /*
         * If there is any ReminderEvent associated with this Reminder, then we must create a new Reminder with higher Version
         * with empty ReminderEvents. Otherwise, we can just update.
         *
         * Note that an ReminderEvent can be generated after we check for their existence; this causes a race condition.
         * The effect is that the event is added to an Reminder that does not match the initial Reminder that triggered it, but has the same
         * identity. This is an rare occurrence, and we will not bother fixing it. The fix is similar to the one in UpdateAlertAsync, with a field
         * called LastUpdate in this Reminder as well.
         */

        var anyReminderEvents = reminder.ReminderEvents.Count != 0 ||
                                await ReminderEvents.AnyAsync(reminderEvent => reminderEvent.Reminder == reminder);
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

        await SaveChangesAsync();
    }

    /// <summary>
    ///     Configure the DbContext options
    /// </summary>
    /// <param name="optionsBuilder">DbContext Options Builder</param>
    /// <param name="connectionString">Connection String from Configuration</param>
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


    protected void CreateSeq(ModelBuilder modelBuilder, string table)
    {
        var seq = modelBuilder.Entity(table);
        seq.Property<long>("Id");
        seq.HasData(new { Id = 1L });
    }

    protected override void OnModelCreating(ModelBuilder modelBuilder)
    {
        CreateSeq(modelBuilder, "alert_id_seq");
        CreateSeq(modelBuilder, "reminder_id_seq");

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
                .Where(otherAlert => otherAlert.Id == alert.Id)
                .Max(otherAlert => otherAlert.Version));

        // Auto-generate Alert Ids
        modelBuilder.Entity<Alert>()
            .Property(alert => alert.Id)
            .HasValueGenerator((_, _) => new IdSeqGenerator("alert_id_seq"))
            .ValueGeneratedOnAdd();

        // Auto-include Tag relationships
        modelBuilder.Entity<Tag>()
            .Navigation(tag => tag.Apps).AutoInclude();

        // Auto-include Alert relationships
        modelBuilder.Entity<Alert>()
            .Navigation(alert => alert.App).AutoInclude();
        modelBuilder.Entity<Alert>()
            .Navigation(alert => alert.Tag).AutoInclude();
        modelBuilder.Entity<Alert>()
            .Navigation(alert => alert.Reminders).AutoInclude();


        // Auto-generate Reminder Ids
        modelBuilder.Entity<Reminder>()
            .Property(reminder => reminder.Id)
            .HasValueGenerator((_, _) => new IdSeqGenerator("reminder_id_seq"))
            .ValueGeneratedOnAdd();

        // Only take the Reminder with the highest Versions
        modelBuilder.Entity<Reminder>().HasQueryFilter(reminder =>
            reminder.Version == Reminders
                .Where(otherReminder => otherReminder.Id == reminder.Id)
                .Max(otherReminder => otherReminder.Version));
    }

    public record struct WithTicks<T>
    {
        public T Inner { get; set; }
        public long Ticks { get; set; }
    }

    private class IdSeqGenerator(string table) : ValueGenerator
    {
        public override bool GeneratesTemporaryValues => false;

        protected override object? NextValue(EntityEntry entry)
        {
            var insertSql = $"UPDATE {table} SET id = id + 1 RETURNING id - 1";
            var id = entry.Context.Database.SqlQueryRaw<long>(insertSql).AsEnumerable().First();
            return id;
        }
    }

#if DEBUG

    private const string SeedDbFile = "dbg/seed.db";

    /// <summary>
    ///     Migrate the seed database to the actual database.
    /// </summary>
    /// <param name="force">Overwrite the existing actual database</param>
    public async Task MigrateFromSeedAsync(bool force = false)
    {
        var connStr = new SqliteConnectionStringBuilder(Database.GetConnectionString());
        if (!force && File.Exists(connStr.DataSource)) return;
        var username = Environment.UserName;

        {
            await using var seedFile = File.OpenRead(SeedDbFile);
            await using var dbFile = File.Create(connStr.DataSource);
            await seedFile.CopyToAsync(dbFile).ConfigureAwait(false);
        }
        await Apps.ExecuteUpdateAsync(appSet => appSet.SetProperty(app => app.Identity.PathOrAumid,
            app => app.Identity.PathOrAumid.Replace("|user|", username))).ConfigureAwait(false);

        await SaveChangesAsync().ConfigureAwait(false);
    }

    /// <summary>
    ///     Update all the <see cref="Usage" /> Start and End times such that the last <see cref="Usage" /> End is equal to
    ///     <paramref name="newUsageEnd" />,
    ///     while preserving the deltas between all <see cref="Usage" />.
    /// </summary>
    /// <param name="newUsageEnd">New last <see cref="Usage" /> End</param>
    public async Task UpdateAllUsageEndsAsync(DateTime newUsageEnd)
    {
        var usageLastEnd = await Usages.MaxAsync(usage => usage.End);
        var deltaTicks = (newUsageEnd - usageLastEnd).Ticks;
        await Database.ExecuteSqlAsync($"UPDATE usages SET start = start + {deltaTicks}, end = end + {deltaTicks}");
        await SaveChangesAsync();
    }
#endif
}