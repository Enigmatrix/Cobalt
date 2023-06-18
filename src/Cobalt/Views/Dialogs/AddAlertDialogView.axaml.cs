using System;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Styling;
using Cobalt.Common.Data;
using Cobalt.Common.Infrastructure;
using Cobalt.Common.ViewModels.Dialogs;
using Cobalt.Common.ViewModels.Entities;
using FluentAvalonia.UI.Controls;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.DependencyInjection;

namespace Cobalt.Views.Dialogs;

public partial class AddAlertDialogView : ContentDialog, IStyleable
{
    public AddAlertDialogView()
    {
        InitializeComponent();
    }

    Type IStyleable.StyleKey => typeof(ContentDialog);

    private async void AddTag_OnClick(object? sender, RoutedEventArgs e)
    {
        // TODO cleanup this, make it return the added tag
        var resolve = ServiceManager.Services;
        var conn = resolve.GetRequiredService<IDbContextFactory<CobaltContext>>();
        var dialog = new AddTagDialogView
        {
            DataContext = new AddTagDialogViewModel(
                resolve.GetRequiredService<IEntityViewModelCache>(),
                conn)
        };
        var result = await dialog.ShowAsync();
    }

    protected override void OnKeyUp(KeyEventArgs e)
    {
        // Override the ContentDialog PrimaryButton trigger on Enter key
        if (e is { Handled: false, Key: Key.Enter }) return;

        base.OnKeyUp(e);
    }
}