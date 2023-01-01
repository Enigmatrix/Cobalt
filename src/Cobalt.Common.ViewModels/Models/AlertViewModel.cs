using System.Collections.ObjectModel;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Models;
using CommunityToolkit.Mvvm.ComponentModel;

namespace Cobalt.Common.ViewModels.Models;

public partial class AlertViewModel : EditableEntityViewModel<Alert>
{
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

    [ObservableProperty]
    private bool _targetIsApp;

    [ObservableProperty]
    private AppViewModel? _app;

    [ObservableProperty]
    private TagViewModel? _tag;

    [ObservableProperty]
    private TimeSpan _usageLimit;

    [ObservableProperty]
    private TimeFrame _timeFrame;

    [ObservableProperty]
    private ExceedAction _exceedAction;

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
