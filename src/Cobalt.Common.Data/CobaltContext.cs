using Cobalt.Common.Data.Entities;
using Microsoft.EntityFrameworkCore;

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
    public DbSet<Alert> Alerts { get; init; } = default!;

    public IQueryable<App> InitializedApps => Apps.Where(app => EF.Property<bool>(app, "initialized"));

    public IQueryable<Alert> GetExpiredAlerts()
    {
        var today = DateTime.Today;
        var week = today - TimeSpan.FromDays((int)today.DayOfWeek);
        var month = today - TimeSpan.FromDays(today.Day);

        return Alerts
            .Select(alert => new
            {
                Alert = alert,
                // no switch statement in Linq Expressions
                Start = alert.TimeFrame == TimeFrame.Daily ? today.ToFileTime() :
                    alert.TimeFrame == TimeFrame.Weekly ? week.ToFileTime() : month.ToFileTime()
            })
            .Where(v =>
                // The cursed things I have to do to get this to be a SQL query ...
                Apps.Where(app => v.Alert.TargetIsApp ? app.Id == v.Alert.App!.Id : v.Alert.Tag!.Apps.Contains(app))
                    .SelectMany(app => app.Sessions.SelectMany(sess =>
                        sess.Usages.Where(usage => usage.EndTicks > v.Start).Select(usage =>
                            usage.EndTicks - Math.Max(usage.StartTicks, v.Start))))
                    .Sum()
                >
                v.Alert.UsageLimitTicks)
            .Select(v => v.Alert);
    }

    protected override void OnConfiguring(DbContextOptionsBuilder optionsBuilder)
    {
        optionsBuilder.UseSqlite($"Data Source={_path}");
    }

    protected override void OnModelCreating(ModelBuilder modelBuilder)
    {
        var app = modelBuilder.Entity<App>();
        app.Property<bool>("initialized");
        app.Property<int>("_identityTag");
        app.Property<string>("_identityText0");

        var usage = modelBuilder.Entity<Usage>();
        usage.Property(u => u.StartTicks);
        usage.Property(u => u.EndTicks);

        var interactionPeriod = modelBuilder.Entity<InteractionPeriod>();
        interactionPeriod.Property(ip => ip.StartTicks);
        interactionPeriod.Property(ip => ip.EndTicks);

        app.HasMany(a => a.Tags).WithMany(t => t.Apps)
            .UsingEntity("_app_tag",
                at => at.HasOne(typeof(Tag)).WithMany().HasForeignKey("tag"),
                at => at.HasOne(typeof(App)).WithMany().HasForeignKey("app"));

        var alert = modelBuilder.Entity<Alert>();
        alert.Property<int>("_exceedActionTag");
        alert.Property<string>("_exceedActionText0");
        alert.Property(a => a.UsageLimitTicks);
    }
}