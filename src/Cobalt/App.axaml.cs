using System.Linq;
using Avalonia;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Data.Core.Plugins;
using Avalonia.Markup.Xaml;
using Cobalt.Common.Infrastructure;
using Cobalt.Common.ViewModels;
using Cobalt.Views;
using Microsoft.Extensions.DependencyInjection;

namespace Cobalt;

public class App : Application
{
    public override void Initialize()
    {
        AvaloniaXamlLoader.Load(this);
    }

    public override void OnFrameworkInitializationCompleted()
    {
        var validationPlugins = BindingPlugins.DataValidators.Where(x => x is DataAnnotationsValidationPlugin).ToList();
        foreach (var plugin in validationPlugins)
            BindingPlugins.DataValidators.Remove(plugin);

        if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
            desktop.MainWindow = new MainWindow
            {
                DataContext = ServiceManager.Services.GetRequiredService<MainWindowViewModel>()
            };

        base.OnFrameworkInitializationCompleted();
    }
}