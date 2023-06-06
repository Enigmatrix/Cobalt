using Cobalt.Common.ViewModels;
using FluentAvalonia.UI.Controls;
using FluentAvalonia.UI.Windowing;

namespace Cobalt.Views;

public partial class MainWindow : AppWindow
{
    public MainWindow()
    {
        InitializeComponent();
    }

    private void NavigationItemInvoked(object? sender, NavigationViewItemInvokedEventArgs e)
    {
        ((MainWindowViewModel)DataContext!).NavigateTo((string)e.InvokedItem);
    }
}