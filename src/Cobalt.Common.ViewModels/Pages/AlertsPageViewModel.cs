using System.Reactive.Disposables;
using System.Reactive.Linq;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using Cobalt.Common.ViewModels.Entities;
using CommunityToolkit.Mvvm.ComponentModel;
using Microsoft.EntityFrameworkCore;
using ReactiveUI;

namespace Cobalt.Common.ViewModels.Pages;

/// <summary>
///     ViewModel for the Alerts Page
/// </summary>
public partial class AlertsPageViewModel : PageViewModelBase
{
    [ObservableProperty] private List<WithDuration<AlertViewModel>> _alerts = default!;

    public AlertsPageViewModel(IEntityViewModelCache entityCache, IDbContextFactory<QueryContext> contexts) :
        base(contexts)
    {
        this.WhenActivated(dis =>
        {
            // Getting all Alerts
            // TODO make this WithDuration cleaner
            Observable.FromAsync(GetAlerts)
                .Select(alerts => alerts.Select(alertWithDur =>
                        new WithDuration<AlertViewModel>(entityCache.Alert(alertWithDur.Inner), alertWithDur.Duration))
                    .ToList())
                .SubscribeOn(RxApp.TaskpoolScheduler)
                .BindTo(this, self => self.Alerts)
                .DisposeWith(dis);
        });
    }

    public override string Name => "Alerts";

    private async Task<List<WithDuration<Alert>>> GetAlerts()
    {
        await using var context = await Contexts.CreateDbContextAsync();
        return await context.AlertDurations().ToListAsync();
    }
}