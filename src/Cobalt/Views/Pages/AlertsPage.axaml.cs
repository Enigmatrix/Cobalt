using System.Reactive.Disposables;
using Avalonia.ReactiveUI;
using Cobalt.Common.ViewModels.Pages;
using Cobalt.Views.Dialogs;
using FluentAvalonia.UI.Controls;
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
                {
                    var dialog = new ContentDialog
                    {
                        Title = "Add Alert",
                        DefaultButton = ContentDialogButton.Primary,
                        PrimaryButtonText = "Add",
                        CloseButtonText = "Cancel",
                        Content = new AddAlertDialog { ViewModel = ViewModel!.AddAlertDialog }
                    };
                    await dialog.ShowAsync();
                    context.SetOutput(null!);
                })
                .DisposeWith(dis);
        });
    }
}