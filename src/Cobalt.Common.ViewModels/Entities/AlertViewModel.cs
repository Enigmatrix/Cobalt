using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using CommunityToolkit.Mvvm.ComponentModel;

namespace Cobalt.Common.ViewModels.Entities;

public partial class AlertViewModel : EditableEntityViewModel<Alert>
{
    [ObservableProperty] private AppViewModel? _app;

    [ObservableProperty] private ExceedAction _exceedAction;

    [ObservableProperty] private TagViewModel? _tag;

    [ObservableProperty] private bool _targetIsApp;

    [ObservableProperty] private TimeFrame _timeFrame;

    [ObservableProperty] private TimeSpan _usageLimit;

    public AlertViewModel(CobaltContext ctx, EntityViewModelCache cache) : base(ctx, cache)
    {
    }

    public override void InitializeEntity(Alert alert)
    {
        // alert...
        TargetIsApp = alert.TargetIsApp;
        if (TargetIsApp)
            App = Cache.GetForApp(alert.App!);
        else
            Tag = Cache.GetForTag(alert.Tag!);
        UsageLimit = alert.UsageLimit;
        TimeFrame = alert.TimeFrame;
        ExceedAction = alert.ExceedAction;

        Inner = alert;
    }

    public override void SaveChanges()
    {
        Inner.TargetIsApp = TargetIsApp;
        if (Inner.TargetIsApp)
            Inner.App = App!.Inner;
        else
            Inner.Tag = Tag!.Inner;
        Inner.UsageLimit = UsageLimit;
        Inner.TimeFrame = TimeFrame;
        Inner.ExceedAction = ExceedAction;

        Context.SaveChanges();
    }
}