using Avalonia.Controls;
using Avalonia.Interactivity;
using Cobalt.Common.Data;
using Cobalt.Common.Infrastructure;
using Cobalt.Common.Utils;
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
        var cache = resolve.GetRequiredService<IEntityViewModelCache>();
        var dialog =
            new AddAlertDialogViewModel(
                cache,
                conn, new AddTagDialogViewModel(cache, conn));

        var res = await dialog.ShowDialog<Unit, AlertViewModel, AddAlertDialogView>();
    }
}