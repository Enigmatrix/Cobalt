using Avalonia.Controls;
using Avalonia.Interactivity;
using Cobalt.Common.Data;
using Cobalt.Common.Infrastructure;
using Cobalt.Common.ViewModels.Dialogs;
using Cobalt.Common.ViewModels.Entities;
using Cobalt.Views.Dialogs;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.DependencyInjection;

namespace Cobalt.Views.Pages;

public partial class AlertsPageView : UserControl
{
    public AlertsPageView()
    {
        InitializeComponent();
    }

    private async void Button_OnClick(object? sender, RoutedEventArgs e)
    {
        var resolve = ServiceManager.Services;
        var conn = resolve.GetRequiredService<IDbContextFactory<CobaltContext>>();
        var dialog = new AddAlertDialogView
        {
            DataContext = new AddAlertDialogViewModel(
                resolve.GetRequiredService<IEntityViewModelCache>(),
                conn)
        };
        var result = await dialog.ShowAsync();
    }
}