using Avalonia.Controls;
using Cobalt.Common.ViewModels;
using FluentAvalonia.UI.Controls;

namespace Cobalt.Views;

public partial class MainWindow : Window
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