using System;
using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using Cobalt.Common.ViewModels.Dialogs;
using Cobalt.Common.ViewModels.Entities;
using FluentAvalonia.UI.Controls;

namespace Cobalt.Views.Dialogs;

public partial class AddTagDialogView : ContentDialog
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

    protected override Type StyleKeyOverride => typeof(ContentDialog);

    private void Search_OnKeyUp(object? sender, KeyEventArgs e)
    {
        if (e.Key != Key.Enter) return;
        AddSelectedApp();
    }

    private void AddSelectedApp()
    {
        var toAdd = SelectedApp;
        if (toAdd == null) return;
        ((AddTagDialogViewModel)DataContext!).AddApp(toAdd);

        SearchApps.Text = ""; // this also clears SelectedApp ...
    }

    // TODO create a common class, put it there
    protected override void OnKeyUp(KeyEventArgs e)
    {
        // Override the ContentDialog PrimaryButton trigger on Enter key
        if (e is { Handled: false, Key: Key.Enter }) return;

        base.OnKeyUp(e);
    }

    private void Button_OnClick(object? sender, RoutedEventArgs e)
    {
        AddSelectedApp();
    }
}