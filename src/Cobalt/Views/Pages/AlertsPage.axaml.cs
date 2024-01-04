using System.Reactive.Disposables;
using Avalonia.ReactiveUI;
using Cobalt.Common.ViewModels.Pages;
using Cobalt.Views.Dialogs;
using ReactiveUI;

namespace Cobalt.Views.Pages;

public partial class AlertsPage : ReactiveUserControl<AlertsPageViewModel>
{
    public AlertsPage()
    {
        InitializeComponent();

        this.WhenActivated(dis =>
        {
            ViewModel!.AddAlertInteraction.RegisterHandler(async context =>
                    context.SetOutput(await context.Input.ShowDialog(new AlertDialogBase())))
                .DisposeWith(dis);
            ViewModel!.EditAlertInteraction.RegisterHandler(async context =>
                    context.SetOutput(await context.Input.ShowDialog(new AlertDialogBase())))
                .DisposeWith(dis);
        });
    }
}