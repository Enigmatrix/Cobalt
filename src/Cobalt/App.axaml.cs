using Avalonia;
using Avalonia.Controls.ApplicationLifetimes;
using Avalonia.Markup.Xaml;
using Cobalt.Common.Data;
using Cobalt.Common.Infrastructure;
using Cobalt.Common.ViewModels;
using Cobalt.Common.ViewModels.Pages;
using Cobalt.Views;
using Cobalt.Views.Pages;
using Microsoft.EntityFrameworkCore;
using Microsoft.Extensions.DependencyInjection;
using ReactiveUI;

namespace Cobalt;

public class App : Application
{
    public override void Initialize()
    {
        AvaloniaXamlLoader.Load(this);
    }

    public override async void OnFrameworkInitializationCompleted()
    {
        var serviceCollection = new ServiceCollection();
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
        var contexts = services.Resolve<IDbContextFactory<QueryContext>>();
        await using (var context = await contexts.CreateDbContextAsync())
        {
            await context.MigrateFromSeed();
        }
#endif

        var mainVm = services.Resolve<MainViewModel>();

        if (ApplicationLifetime is IClassicDesktopStyleApplicationLifetime desktop)
            desktop.MainWindow = new MainWindow
            {
                ViewModel = mainVm
            };

        base.OnFrameworkInitializationCompleted();
    }
}