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
    private readonly IEntityViewModelCache _entityCache;

    public AlertsPageViewModel(IEntityViewModelCache entityCache, IDbContextFactory<QueryContext> contexts,
        AddAlertDialogViewModel addAlertDialog) :
        base(contexts)
    {
        Alerts = Query(async context => await context.AlertDurations().ToListAsync(),
            alertDur => alertDur.Map(entityCache.Alert));

        _entityCache = entityCache;
        _addAlertDialog = addAlertDialog;

        this.WhenActivated((CompositeDisposable dis) =>
        {
            // Getting all Alerts
            Alerts.Refresh();
        });
    }

    public Query<List<WithDuration<AlertViewModel>>> Alerts { get; }

    public Interaction<AddAlertDialogViewModel, AlertViewModel?> AddAlertInteraction { get; } = new();
    public Interaction<EditAlertDialogViewModel, AlertViewModel?> EditAlertInteraction { get; } = new();

    public override string Name => "Alerts";

    [RelayCommand]
    public async Task EditAlert(AlertViewModel alertVm)
    {
        var editAlertDialog = new EditAlertDialogViewModel(alertVm, _entityCache, Contexts);
        var alert = await EditAlertInteraction.Handle(editAlertDialog);
        if (alert == null) return;

        await Alerts.Refresh();
    }

    [RelayCommand]
    public async Task DeleteAlert(AlertViewModel alertVm)
    {
        await using var context = await Contexts.CreateDbContextAsync();
        var guid = alertVm.Entity.Guid;
        // no need to do anything special to delete Reminders, AlertEvent and ReminderEvent,
        // they are cascade delete (App and Tag are not since they are nullable)
        await context.Alerts
            .IgnoreAutoIncludes() // otherwise it errors out
            .IgnoreQueryFilters() // ignore max by version constraint
            .Where(alert => alert.Guid == guid)
            .ExecuteDeleteAsync();

        await Alerts.Refresh();
    }

    [RelayCommand]
    public async Task AddAlert()
    {
        var alert = await AddAlertInteraction.Handle(_addAlertDialog);
        if (alert == null) return;

        await Alerts.Refresh();
    }
}