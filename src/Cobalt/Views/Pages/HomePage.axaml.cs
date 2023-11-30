using Avalonia.ReactiveUI;
using Cobalt.Common.ViewModels.Pages;

namespace Cobalt.Views.Pages;

public partial class HomePage : ReactiveUserControl<HomePageViewModel>
{
    public HomePage()
    {
        InitializeComponent();
    }
}