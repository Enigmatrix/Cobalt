using Cobalt.Common.Data.Entities;
using Microsoft.Data.Sqlite;
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

#if DEBUG

    private const string SeedDbFile = "seed.db";

    public void MigrateFromSeed(bool force = false, DateTime? usageEndAt = null)
    {
        var connStr = new SqliteConnectionStringBuilder(Database.GetConnectionString());
        if (!force && usageEndAt == null && File.Exists(connStr.DataSource)) return;
        var username = Environment.UserName;

        File.Copy(SeedDbFile, connStr.DataSource, true);
        Apps.ExecuteUpdate(appSet => appSet.SetProperty(app => app.Identity.PathOrAumid,
            app => app.Identity.PathOrAumid.Replace("|user|", username)));


        if (usageEndAt is { } usageEndAtValue)
        {
            var usageLastEnd = Usages.Max(usage => usage.End);
            var deltaTicks = (usageEndAtValue - usageLastEnd).Ticks;
            Database.ExecuteSql($"UPDATE usages SET start = start + {deltaTicks}, end = end + {deltaTicks}");
        }

        SaveChanges();
    }
#endif
}