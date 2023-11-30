using Avalonia;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Markup.Xaml;
using Cobalt.Common.Infrastructure;
using Cobalt.Common.ViewModels;
using Cobalt.Views;

namespace Cobalt;

public class App : Application
{
    public override void Initialize()
    {
        AvaloniaXamlLoader.Load(this);
    }

    public override void OnFrameworkInitializationCompleted()
    {
        var services = new ServiceInjector();
        var mainVm = services.Resolve<MainViewModel>();

        if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
            desktop.MainWindow = new MainWindow
            {
                ViewModel = mainVm
            };

        base.OnFrameworkInitializationCompleted();
    }
}