using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using Cobalt.Common.Utils;
using CommunityToolkit.Mvvm.ComponentModel;
using Microsoft.EntityFrameworkCore;
using ActionEntity = Cobalt.Common.Data.Entities.Action;
using TargetEntity = Cobalt.Common.Data.Entities.Target;

namespace Cobalt.Common.ViewModels.Entities;

// TODO inherit the HasColor, HasName, HasIcon traits here
public abstract record TargetViewModel
{
    public sealed record AppTarget(AppViewModel App) : TargetViewModel;

    public sealed record TagTarget(TagViewModel Tag) : TargetViewModel;
}

public partial class AlertViewModel : EditableEntityViewModel<Alert>
{
    [ObservableProperty] private ActionEntity _action = default!;
    [ObservableProperty] private TargetViewModel _target = default!;
    [ObservableProperty] private TimeFrame _timeFrame;
    [ObservableProperty] private TimeSpan _usageLimit;

    public AlertViewModel(IEntityViewModelCache cache, IDbContextFactory<CobaltContext> conn) : base(cache, conn)
    {
    }

    public override void InitializeWith(Alert alert)
    {
        base.InitializeWith(alert);
    }

    public override void Save()
    {
        using var ctx = Conn.CreateDbContext();
        ctx.Attach(Entity);
        Entity.Target = Target switch
        {
            TargetViewModel.AppTarget app => new Target.AppTarget(app.App.Entity),
            TargetViewModel.TagTarget tag => new Target.TagTarget(tag.Tag.Entity),
            _ => throw new DiscriminatedUnionException<TargetViewModel>(nameof(Target))
        };
        Entity.UsageLimit = UsageLimit;
        Entity.TimeFrame = TimeFrame;
        Entity.Action = Action;
        ctx.SaveChanges();
    }
}