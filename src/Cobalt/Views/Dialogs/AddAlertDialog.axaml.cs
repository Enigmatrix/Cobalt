using Avalonia.ReactiveUI;
using Cobalt.Common.ViewModels.Dialogs;

namespace Cobalt.Views.Dialogs;

public partial class AddAlertDialog : ReactiveUserControl<AddAlertDialogViewModel>
{
    public AddAlertDialog()
    {
        InitializeComponent();
    }
}