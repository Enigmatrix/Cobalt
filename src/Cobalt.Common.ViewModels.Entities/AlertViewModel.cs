using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using CommunityToolkit.Mvvm.ComponentModel;
using Microsoft.EntityFrameworkCore;
using ActionEntity = Cobalt.Common.Data.Entities.Action;
using TargetEntity = Cobalt.Common.Data.Entities.Target;

namespace Cobalt.Common.ViewModels.Entities;

public abstract record TargetViewModel
{
    public sealed record AppTarget(AppViewModel App) : TargetViewModel;
    public sealed record TagTarget(TagViewModel Tag) : TargetViewModel;
}

public partial class AlertViewModel : EditableEntityViewModel<Alert>
{
    [ObservableProperty] private TargetViewModel _target = default!;
    [ObservableProperty] private TimeSpan _usageLimit = default!;
    [ObservableProperty] private TimeFrame _timeFrame = default!;
    [ObservableProperty] private ActionEntity _action = default!;

    public override void InitializeWith(Alert alert)
    {
        base.InitializeWith(alert);
    }

    public AlertViewModel(IEntityViewModelCache cache, IDbContextFactory<CobaltContext> conn) : base(cache, conn)
    {
    }

    public override void Save()
    {
        using var ctx = Conn.CreateDbContext();
        ctx.Attach(Entity);
        Entity.Target = Target switch
        {
            TargetViewModel.AppTarget app => new TargetEntity.AppTarget(app.App.Entity),
            TargetViewModel.TagTarget tag => new TargetEntity.TagTarget(tag.Tag.Entity),
            _ => throw new ArgumentOutOfRangeException() // TODO
        };
        Entity.UsageLimit = UsageLimit;
        Entity.TimeFrame = TimeFrame;
        Entity.Action = Action;
        ctx.SaveChanges();
    }
}
