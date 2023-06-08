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
    public DbSet<Session> Sessions { get; set; } = null!;
    public DbSet<Usage> Usages { get; set; } = null!;
    public DbSet<InteractionPeriod> InteractionPeriods { get; set; } = null!;
    public DbSet<Tag> Tags { get; set; } = null!;
    public DbSet<Alert> Alerts { get; set; } = null!;
    public DbSet<Reminder> Reminders { get; set; } = null!;

    // 1. Can i use something other than anonymous types (so that I can return a WithDuration<Alert> from AlertDuritions)
    // 2. How can I map a IQueryable<Usages>, Start, End => Duration using expressions???

    public IQueryable<Alert> TriggeredAlerts(DateTime now)
    {
        var today = now.Date;
        var week = today - TimeSpan.FromDays((int)today.DayOfWeek);
        var month = today - TimeSpan.FromDays(today.Day);

        return Alerts
            .Select(alert => new
            {
                Alert = alert,
                // no switch statement in Linq Expressions
                Start = alert.TimeFrame == TimeFrame.Daily ? today.Ticks :
                    alert.TimeFrame == TimeFrame.Weekly ? week.Ticks : month.Ticks,
                End = alert.TimeFrame == TimeFrame.Daily ? today.AddDays(1).Ticks :
                    alert.TimeFrame == TimeFrame.Weekly ? week.AddDays(7).Ticks : month.AddMonths(1).Ticks
            })
            .Select(als => new
            {
                als.Alert,
                Expired = Apps.Where(app => EF.Property<bool>(als.Alert, "_targetIsApp")
                        ? EF.Property<App?>(als.Alert, "_app")! == app
                        : EF.Property<Tag?>(als.Alert, "_tag")!.Apps.Contains(app))
                    .SelectMany(x => x.Sessions)
                    .SelectMany(x => x.Usages)
                    .Where(x => x.EndTicks > als.Start && x.StartTicks < als.End)
                    .Select(x => Math.Min(x.EndTicks, als.End) - Math.Max(x.StartTicks, als.Start))
                    .Sum() >= als.Alert.UsageLimitTicks
            })
            .Where(x => x.Expired)
            .Select(x => x.Alert);
    }

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
        opts.UseSqlite($"Data Source={_dbPath}");
        opts.UseQueryTrackingBehavior(QueryTrackingBehavior.NoTracking);
    }
}