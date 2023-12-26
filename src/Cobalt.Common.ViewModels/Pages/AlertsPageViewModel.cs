using System.Reactive.Disposables;
using System.Reactive.Linq;
using Cobalt.Common.Data;
using Cobalt.Common.ViewModels.Analysis;
using Cobalt.Common.ViewModels.Dialogs;
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
    private readonly AddAlertDialogViewModel _addAlertDialog;

    public AlertsPageViewModel(IEntityViewModelCache entityCache, IDbContextFactory<QueryContext> contexts,
        AddAlertDialogViewModel addAlertDialog) :
        base(contexts)
    {
        Alerts = new Query<List<WithDuration<AlertViewModel>>>(Contexts,
            async context => (await context.AlertDurations().ToListAsync())
                .Select(alertDur => alertDur.Map(entityCache.Alert)).ToList());

        _addAlertDialog = addAlertDialog;

        this.WhenActivated((CompositeDisposable dis) =>
        {
            // Getting all Alerts
            Alerts.Refresh();
        });
    }

    public Query<List<WithDuration<AlertViewModel>>> Alerts { get; }

    public Interaction<AddAlertDialogViewModel, AlertViewModel?> AddAlertInteraction { get; } = new();

    public override string Name => "Alerts";

    [RelayCommand]
    public async Task AddAlert()
    {
        var alert = await AddAlertInteraction.Handle(_addAlertDialog);
        if (alert == null) return;

        await Alerts.Refresh();
    }
}