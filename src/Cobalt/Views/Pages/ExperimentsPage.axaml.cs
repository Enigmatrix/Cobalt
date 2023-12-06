#if DEBUG
using Avalonia.ReactiveUI;
using Cobalt.Common.ViewModels.Pages;

namespace Cobalt.Views.Pages;

public partial class ExperimentsPage : ReactiveUserControl<ExperimentsPageViewModel>
{
    public ExperimentsPage()
    {
        InitializeComponent();
    }
}
#endif