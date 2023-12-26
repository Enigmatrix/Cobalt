using System.Reactive;
using System.Reactive.Disposables;
using System.Reactive.Linq;
using Cobalt.Common.Data;
using Cobalt.Common.ViewModels.Analysis;
using Cobalt.Common.ViewModels.Entities;
using CommunityToolkit.Mvvm.Input;
using Microsoft.EntityFrameworkCore;
using ReactiveUI;

namespace Cobalt.Common.ViewModels.Pages;

/// <summary>
///     ViewModel for the Alerts Page
/// </summary>
public partial class AlertsPageViewModel : PageViewModelBase
{
    public AlertsPageViewModel(IEntityViewModelCache entityCache, IDbContextFactory<QueryContext> contexts) :
        base(contexts)
    {
        Alerts = new Query<List<WithDuration<AlertViewModel>>>(Contexts,
            async context => (await context.AlertDurations().ToListAsync())
                .Select(alertDur => alertDur.Map(entityCache.Alert)).ToList());

        this.WhenActivated((CompositeDisposable dis) =>
        {
            // Getting all Alerts
            Alerts.Refresh();
        });
    }

    public Query<List<WithDuration<AlertViewModel>>> Alerts { get; }

    public Interaction<Unit, AlertViewModel> AddAlertInteraction { get; } = new();

    public override string Name => "Alerts";

    [RelayCommand]
    public async Task AddAlert()
    {
        await AddAlertInteraction.Handle(Unit.Default); // TODO handle this?
        await Alerts.Refresh();
    }
}