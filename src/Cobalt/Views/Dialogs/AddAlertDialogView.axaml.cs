using Avalonia.Interactivity;
using Cobalt.Common.Utils;
using Cobalt.Common.ViewModels.Dialogs;
using Cobalt.Common.ViewModels.Entities;

namespace Cobalt.Views.Dialogs;

public partial class AddAlertDialogView : DialogViewBase
{
    public AddAlertDialogView()
    {
        InitializeComponent();
    }

    private async void AddTag_OnClick(object? sender, RoutedEventArgs e)
    {
        var result = await ((AddAlertDialogViewModel)DataContext!).AddTagDialogViewModel
            .ShowDialog<Unit, TagViewModel, AddTagDialogView>();
    }
}