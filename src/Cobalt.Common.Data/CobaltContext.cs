using Cobalt.Common.Data.Entities;
using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Storage.ValueConversion;
using System.Reflection.Emit;

namespace Cobalt.Common.Data;

public class CobaltContext : DbContext
{
    private readonly string _dbPath;

    public CobaltContext(string dbPath)
    {
        // TODO enable WAL
        _dbPath = dbPath;
    }

    public DbSet<App> Apps { get; set; } = null!;
    public DbSet<Session> Sessions { get; set; } = null!;
    public DbSet<Usage> Usages { get; set; } = null!;
    public DbSet<InteractionPeriod> InteractionPeriods { get; set; } = null!;
    public DbSet<Tag> Tags { get; set; } = null!;
    /*
    public DbSet<Alert> Alerts { get; set; } = null!;
    public DbSet<Reminder> Reminders { get; set; } = null!;
    */

    protected override void OnModelCreating(ModelBuilder model)
    {
        // TODO setup all the stupid properties

        model.Entity<App>(app =>
        {
            app.Property("_identityTag").HasColumnName("identity_tag");
            app.Property("_identityText0").HasColumnName("identity_text0");
            app.HasMany(e => e.Tags)
                .WithMany(tag => tag.Apps)
                .UsingEntity(
                    "_app_tag",
                    l => l.HasOne(typeof(Tag)).WithMany().HasForeignKey("tag").HasPrincipalKey(nameof(Tag.Id)),
                    r => r.HasOne(typeof(App)).WithMany().HasForeignKey("app").HasPrincipalKey(nameof(App.Id)),
                    j => j.HasKey("app", "tag"));
        });

        model.Entity<Usage>(usage =>
        {
            usage.Property(x => x.Start).HasConversion<DateTimeToTicksConverter>();
            usage.Property(x => x.End).HasConversion<DateTimeToTicksConverter>();
        });

        model.Entity<InteractionPeriod>(ip =>
        {
            ip.Property(x => x.Start).HasConversion<DateTimeToTicksConverter>();
            ip.Property(x => x.End).HasConversion<DateTimeToTicksConverter>();
        });
    }

    protected override void OnConfiguring(DbContextOptionsBuilder opts) =>
        opts.UseSqlite($"Data Source={_dbPath}");
}
