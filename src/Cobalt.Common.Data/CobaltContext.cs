using Cobalt.Common.Data.Entities;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Common.Data;

public class CobaltContext : DbContext
{
    public DbSet<App> Apps { get; set; }
    public DbSet<Session> Sessions { get; set; }
    public DbSet<Usage> Usages { get; set; }
    public DbSet<InteractionPeriod> InteractionPeriods { get; set; }
    public DbSet<Tag> Tags { get; set; }
    public DbSet<Alert> Alerts { get; set; }
    public DbSet<Reminder> Reminders { get; set; }

    protected override void OnModelCreating(ModelBuilder model)
    {
        // TODO
    }
}
