using System.Threading.Tasks;
using Cobalt.Common.ViewModels.Dialogs;
using FluentAvalonia.UI.Controls;

namespace Cobalt.Views.Dialogs;

public static class DialogViewModelExtensions
{
    public static async ValueTask<TOutput?> ShowDialog<TInput, TOutput, TView>(
        this DialogViewModelBase<TInput, TOutput> vm)
        where TView : DialogViewBase, new()
    {
        var view = new TView
        {
            DataContext = vm
        };
        var res = await view.ShowAsync();
        var ret = res != ContentDialogResult.Primary ? default : vm.GetOutput();
        view.DataContext = null;
        return ret;
    }
}