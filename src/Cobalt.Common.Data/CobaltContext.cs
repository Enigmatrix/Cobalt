using Cobalt.Common.Data.Entities;
using Microsoft.EntityFrameworkCore;

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
    /*
    public DbSet<Session> Sessions { get; set; } = null!;
    public DbSet<Usage> Usages { get; set; } = null!;
    public DbSet<InteractionPeriod> InteractionPeriods { get; set; } = null!;
    public DbSet<Tag> Tags { get; set; } = null!;
    public DbSet<Alert> Alerts { get; set; } = null!;
    public DbSet<Reminder> Reminders { get; set; } = null!;
    */

    protected override void OnModelCreating(ModelBuilder model)
    {
        // TODO setup all the stupid properties


        var app = model.Entity<App>();
        app.Property("_identityTag").HasColumnName("identity_tag");
        app.Property("_identityText0").HasColumnName("identity_text0");
    }

    protected override void OnConfiguring(DbContextOptionsBuilder opts) =>
        opts.UseSqlite($"Data Source={_dbPath}");
}
