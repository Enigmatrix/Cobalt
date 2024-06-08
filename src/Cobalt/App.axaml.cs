using Avalonia;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Data.Core.Plugins;
using Avalonia.Markup.Xaml;
using Cobalt.Common.Data;
using Cobalt.Common.Infrastructure;
using Cobalt.Common.ViewModels;
using Cobalt.Common.ViewModels.Pages;
using Cobalt.Extensions;
using Cobalt.Views;
using Cobalt.Views.Pages;
using LiveChartsCore;
using LiveChartsCore.SkiaSharpView;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.DependencyInjection;
using ReactiveUI;

namespace Cobalt;

public class App : Application
{
    public override void Initialize()
    {
        AvaloniaXamlLoader.Load(this);

        LiveCharts.Configure(config =>
                config
                    .AddDarkTheme()
            // .HasMap<City>((city, index) => new(index, city.Population)) 
        );
    }

    public override void OnFrameworkInitializationCompleted()
    {
        // Add our IValidatorViewModel-based validation plugin

        // This exists since we have view models not inheriting from ReactiveValidationObject
        // but instead from IValidatorViewModel, which only has a ValidationContext.
        BindingPlugins.DataValidators.Add(new ValidatorViewModelValidationPlugin());

        var serviceCollection = new ServiceCollection();

        // Add views
#if DEBUG
        serviceCollection.AddSingleton<IViewFor<ExperimentsPageViewModel>, ExperimentsPage>();
#endif
        serviceCollection.AddSingleton<IViewFor<HomePageViewModel>, HomePage>();
        serviceCollection.AddSingleton<IViewFor<AppsPageViewModel>, AppsPage>();
        serviceCollection.AddSingleton<IViewFor<TagsPageViewModel>, TagsPage>();
        serviceCollection.AddSingleton<IViewFor<AlertsPageViewModel>, AlertsPage>();
        serviceCollection.AddSingleton<IViewFor<HistoryPageViewModel>, HistoryPage>();

        var services = new ServiceInjector(serviceCollection);

#if DEBUG
        // initial migration, since there is no Cobalt.Engine doing it for us.
        var contexts = services.Resolve<IDbContextFactory<QueryContext>>();
        using (var context = contexts.CreateDbContext())
        {
            context.MigrateFromSeedAsync().Wait();
        }
#endif

        var mainVm = services.Resolve<MainViewModel>();

        if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
            desktop.MainWindow = new MainWindow
            {
                ViewModel = mainVm
            };
    }
}