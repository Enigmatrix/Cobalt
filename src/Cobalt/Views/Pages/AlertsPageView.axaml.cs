using Avalonia.Controls;
using Avalonia.Interactivity;
using Cobalt.Common.ViewModels.Dialogs;
using Cobalt.Views.Dialogs;

namespace Cobalt.Views.Pages;

public partial class AlertsPageView : UserControl
{
    public AlertsPageView()
    {
        InitializeComponent();
    }

    private async void Button_OnClick(object? sender, RoutedEventArgs e)
    {
        var dialog = new AddAlertDialogView
        {
            DataContext = new AddAlertDialogViewModel(null)
        };
        var result = await dialog.ShowAsync();
    }
}