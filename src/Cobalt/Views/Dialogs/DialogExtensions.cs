using System.Threading.Tasks;
using Cobalt.Common.ViewModels.Dialogs;
using FluentAvalonia.UI.Controls;
using ReactiveUI;

namespace Cobalt.Views.Dialogs;

/// <summary>
///     Extensions for common Dialog operations
/// </summary>
public static class DialogExtensions
{
    /// <summary>
    ///     Show a <see cref="ContentDialog" /> with Title, Primary and Close buttons.
    /// </summary>
    /// <typeparam name="TResult">Expected output type</typeparam>
    /// <param name="vm">ViewModel of the dialog</param>
    /// <param name="view">View of the dialog</param>
    /// <returns>Dialog result</returns>
    public static async Task<TResult?> ShowDialog<TResult>(this DialogViewModelBase<TResult> vm, IViewFor view)
    {
        view.ViewModel = vm;

        var dialog = new ContentDialog
        {
            DefaultButton = ContentDialogButton.Primary,
            PrimaryButtonText = vm.PrimaryButtonText,
            CloseButtonText = vm.CloseButtonText,
            Content = view
        };
        var result = await dialog.ShowAsync();
        return result == ContentDialogResult.Primary ? await vm.GetResultAsync() : default;
    }
}