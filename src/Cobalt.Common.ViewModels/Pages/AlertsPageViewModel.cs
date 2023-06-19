using Cobalt.Common.ViewModels.Dialogs;
using Cobalt.Common.ViewModels.Entities;
using Cobalt.Common.ViewModels.Interactions;
using CommunityToolkit.Mvvm.Input;

namespace Cobalt.Common.ViewModels.Pages;

public partial class AlertsPageViewModel : PageViewModelBase
{
    private readonly Func<AddAlertDialogViewModel> _addAlertDialogFactory;

    public AlertsPageViewModel(Func<AddAlertDialogViewModel> addAlertDialogFactory)
    {
        _addAlertDialogFactory = addAlertDialogFactory;
        AddAlertInteraction = new Interaction<AddAlertDialogViewModel, AlertViewModel>();
    }

    public Interaction<AddAlertDialogViewModel, AlertViewModel> AddAlertInteraction { get; }


    [RelayCommand]
    public async Task AddAlert()
    {
        var alert = await AddAlertInteraction.Handle(_addAlertDialogFactory());
        // TODO add alert to list
    }
}