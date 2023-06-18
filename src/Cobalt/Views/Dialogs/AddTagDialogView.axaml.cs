using Avalonia;
using Avalonia.Controls;
using Avalonia.Input;
using Avalonia.Interactivity;
using Cobalt.Common.ViewModels.Dialogs;
using Cobalt.Common.ViewModels.Entities;

namespace Cobalt.Views.Dialogs;

public partial class AddTagDialogView : DialogViewBase
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

    private void Button_OnClick(object? sender, RoutedEventArgs e)
    {
        AddSelectedApp();
    }
}