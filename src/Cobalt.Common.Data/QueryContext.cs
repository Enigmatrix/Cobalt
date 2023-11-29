using Cobalt.Common.Data.Entities;
using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Storage.ValueConversion;

namespace Cobalt.Common.Data;

public class QueryContext : DbContext
{
    public DbSet<App> Apps { get; set; } = null!;
    public DbSet<Session> Session { get; set; } = null!;
    public DbSet<Usage> Usages { get; set; } = null!;
    public DbSet<Tag> Tags { get; set; } = null!;
    public DbSet<InteractionPeriod> InteractionPeriods { get; set; } = null!;
    public DbSet<Alert> Alerts { get; set; } = null!;
    public DbSet<Reminder> Reminders { get; set; } = null!;
    public DbSet<AlertEvent> AlertEvents { get; set; } = null!;
    public DbSet<ReminderEvent> ReminderEvents { get; set; } = null!;

    protected override void OnConfiguring(DbContextOptionsBuilder optionsBuilder)
    {
        optionsBuilder.UseSqlite("Data Source=file.db").UseSnakeCaseNamingConvention();
    }

    protected override void ConfigureConventions(ModelConfigurationBuilder configurationBuilder)
    {
        configurationBuilder.Properties<DateTime>().HaveConversion<DateTimeToTicksConverter>();
        configurationBuilder.Properties<TimeSpan>().HaveConversion<TimeSpanToTicksConverter>();
    }

    protected override void OnModelCreating(ModelBuilder modelBuilder)
    {
    }
}