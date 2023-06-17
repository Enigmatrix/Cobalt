using System;
using Avalonia.Styling;
using FluentAvalonia.UI.Controls;

namespace Cobalt.Views.Dialogs;

public partial class AddAlertDialogView : ContentDialog, IStyleable
{
    public AddAlertDialogView()
    {
        InitializeComponent();
    }

    Type IStyleable.StyleKey => typeof(ContentDialog);
}