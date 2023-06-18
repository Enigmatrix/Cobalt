using System.Diagnostics;
using Cobalt.Common.Data.Entities;
using Microsoft.Data.Sqlite;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.Configuration;

namespace Cobalt.Common.Data;

public class CobaltContext : DbContext
{
    private readonly string _connStr;

    public CobaltContext(IConfiguration config)
    {
        var dbPath = config.GetConnectionString("DatabasePath")
                     ?? throw new InvalidOperationException("DatabasePath not found");
        _connStr = $"Data Source={dbPath}";
    }

    public DbSet<App> Apps { get; set; } = default!;
    public DbSet<Session> Sessions { get; set; } = default!;
    public DbSet<Usage> Usages { get; set; } = default!;
    public DbSet<InteractionPeriod> InteractionPeriods { get; set; } = default!;
    public DbSet<Tag> Tags { get; set; } = default!;
    public DbSet<Alert> Alerts { get; set; } = default!;
    public DbSet<Reminder> Reminders { get; set; } = default!;

    protected override void OnModelCreating(ModelBuilder model)
    {
        // TODO maybe include app initialized field, as well as a HasQueryInclude

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

        model.Entity<Alert>(alerts =>
        {
            alerts.Property("_targetIsApp").HasColumnName("target_is_app");
            alerts.HasOne("_app").WithMany().HasForeignKey("app");
            alerts.HasOne("_tag").WithMany().HasForeignKey("tag");
            alerts.Navigation("_app").AutoInclude();
            alerts.Navigation("_tag").AutoInclude();
            alerts.Property("_actionTag").HasColumnName("action_tag");
            alerts.Property("_actionInt0").HasColumnName("action_int0");
            alerts.Property("_actionText0").HasColumnName("action_text0");
        });
    }

    protected override void OnConfiguring(DbContextOptionsBuilder opts)
    {
        // var conn = new SqliteConnection(_connStr);
        // conn.Open();
        // using var cmd = new SqliteCommand("PRAGMA journal_mode=WAL", conn);
        // cmd.ExecuteNonQuery();
        // opts.UseSqlite(conn);

        opts.UseSqlite(_connStr);

        opts.UseQueryTrackingBehavior(QueryTrackingBehavior.NoTracking);

        // TODO for debugging only! Remove in prod
        opts.LogTo(log => Trace.WriteLine(log));
    }

    public Stream AppIcon(long appId)
    {
        using var conn = new SqliteConnection(_connStr);
        conn.Open();
        using var cmd = new SqliteCommand("PRAGMA journal_mode=WAL", conn);
        cmd.ExecuteNonQuery();

        var ms = new MemoryStream();
        using var blob = new SqliteBlob(conn, "app", "icon", readOnly: true,
            rowid: appId);
        blob.CopyTo(ms);

        return ms;
    }
}