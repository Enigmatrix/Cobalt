using System;
using System.Reactive.Disposables;
using Avalonia.ReactiveUI;
using Cobalt.Common.ViewModels.Dialogs;
using ReactiveUI;

namespace Cobalt.Views.Dialogs;

public partial class AddAlertDialog : ReactiveUserControl<AddAlertDialogViewModel>
{
    public AddAlertDialog()
    {
        InitializeComponent();

        // Hide the TargetPicker Flyout when we have an selected Target
        this.WhenActivated(dis =>
        {
            ViewModel!.WhenAnyValue(self => self.ChooseTargetDialog.Target)
                .WhereNotNull()
                .Subscribe(_ => { TargetPicker.Flyout?.Hide(); }).DisposeWith(dis);
        });
    }
}