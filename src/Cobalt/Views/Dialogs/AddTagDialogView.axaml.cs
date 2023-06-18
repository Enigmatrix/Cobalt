using System;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using Avalonia.Styling;
using Cobalt.Common.ViewModels.Dialogs;
using Cobalt.Common.ViewModels.Entities;
using FluentAvalonia.UI.Controls;

namespace Cobalt.Views.Dialogs;

public partial class AddTagDialogView : ContentDialog, IStyleable
{
    public static readonly StyledProperty<AppViewModel?> SelectedAppProperty =
        AvaloniaProperty.Register<AutoCompleteBox, AppViewModel?>(
            nameof(SelectedApp));

    public AddTagDialogView()
    {
        InitializeComponent();
    }

    public AppViewModel? SelectedApp
    {
        get => GetValue(SelectedAppProperty);
        set => SetValue(SelectedAppProperty, value);
    }

    public AutoCompleteSelector<object?> AppTextSelector => (search, item) =>
        (item as AppViewModel)?.Name!;

    Type IStyleable.StyleKey => typeof(ContentDialog);

    private void Search_OnKeyDown(object? sender, KeyEventArgs e)
    {
        if (e.Key != Key.Enter || SelectedApp == null) return;

        AddSelectedApp();
    }

    private void AddSelectedApp()
    {
        var toAdd = SelectedApp;
        ((AddTagDialogViewModel)DataContext!).AddApp(toAdd);

        SearchApps.Text = ""; // this also clears SelectedApp ...
    }

    private void Button_OnClick(object? sender, RoutedEventArgs e)
    {
        AddSelectedApp();
    }
}