using Avalonia.Controls;
using Avalonia.ReactiveUI;
using Cobalt.Common.ViewModels.Dialogs;
using Cobalt.Common.ViewModels.Entities;

namespace Cobalt.Views.Dialogs;

public partial class ChooseTargetDialogView : ReactiveUserControl<ChooseTargetDialogViewModel>
{
    public ChooseTargetDialogView()
    {
        InitializeComponent();
    }

    private void Tags_OnSelectionChanged(object? sender, SelectionChangedEventArgs e)
    {
        if (e.AddedItems.Count == 1) ViewModel!.Target = e.AddedItems[0] as EntityViewModelBase;
    }

    private void Apps_OnSelectionChanged(object? sender, SelectionChangedEventArgs e)
    {
        if (e.AddedItems.Count == 1) ViewModel!.Target = e.AddedItems[0] as EntityViewModelBase;
    }
}