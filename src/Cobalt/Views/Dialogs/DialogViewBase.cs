using System;
using Avalonia.Input;
using FluentAvalonia.UI.Controls;

namespace Cobalt.Views.Dialogs;

public class DialogViewBase : ContentDialog
{
    protected override Type StyleKeyOverride { get; } = typeof(ContentDialog);


    // Override the ContentDialog PrimaryButton trigger on Enter key
    protected override void OnKeyUp(KeyEventArgs e)
    {
        if (e is { Handled: false, Key: Key.Enter }) return;

        base.OnKeyUp(e);
    }
}