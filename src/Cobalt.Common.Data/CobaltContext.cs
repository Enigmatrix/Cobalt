using Cobalt.Common.Data.Models;
using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Storage.ValueConversion;

namespace Cobalt.Common.Data;

public class CobaltContext : DbContext
{
    private readonly string _path;

    /// <summary>
    ///     Initialize CobaltContext with a connection string file path
    /// </summary>
    /// <param name="path">file path</param>
    public CobaltContext(string path)
    {
        _path = path;
    }

    public DbSet<App> Apps { get; init; } = default!;
    public DbSet<Session> Sessions { get; init; } = default!;
    public DbSet<Usage> Usages { get; init; } = default!;
    public DbSet<InteractionPeriod> InteractionPeriods { get; init; } = default!;
    public DbSet<Tag> Tags { get; init; } = default!;

    public IQueryable<App> InitializedApps => Apps.Where(app => EF.Property<bool>(app, "initialized"));

    protected override void OnConfiguring(DbContextOptionsBuilder optionsBuilder)
    {
        optionsBuilder.UseSqlite($"Data Source={_path}");
    }

    protected override void OnModelCreating(ModelBuilder modelBuilder)
    {
        var dateTimeConverter = new ValueConverter<DateTime, long>(
            dt => dt.ToFileTime(),
            ticks => DateTime.FromFileTime(ticks));

        var app = modelBuilder.Entity<App>();
        app.Property<bool>("initialized");
        app.Property<int>("_identityTag");
        app.Property<string>("_identityText0");

        var usage = modelBuilder.Entity<Usage>();
        usage.Property(u => u.Start).HasConversion(dateTimeConverter);
        usage.Property(u => u.End).HasConversion(dateTimeConverter);

        var interactionPeriod = modelBuilder.Entity<InteractionPeriod>();
        interactionPeriod.Property(ip => ip.Start).HasConversion(dateTimeConverter);
        interactionPeriod.Property(ip => ip.End).HasConversion(dateTimeConverter);

        app.HasMany(a => a.Tags).WithMany(t => t.Apps)
            .UsingEntity("_app_tag", 
                at => at.HasOne(typeof(Tag)).WithMany().HasForeignKey("tag"),
                at => at.HasOne(typeof(App)).WithMany().HasForeignKey("app"));
    }
}