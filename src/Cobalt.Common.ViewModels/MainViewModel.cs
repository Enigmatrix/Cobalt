using Cobalt.Common.Data;
using Cobalt.Common.ViewModels.Pages;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Common.ViewModels;

/// <summary>
///     Main View Model that is loaded first.
/// </summary>
public class MainViewModel : ViewModelBase
{
    public MainViewModel(
        IServiceProvider provider,
#if DEBUG
        ExperimentsPageViewModel experiments,
#endif
        HomePageViewModel home, AppsPageViewModel apps, TagsPageViewModel tags,
        AlertsPageViewModel alerts, HistoryPageViewModel history,
        IDbContextFactory<QueryContext> contexts) : base(contexts)
    {
#if DEBUG
        Experiments = experiments;
#endif
        Home = home;
        Apps = apps;
        Tags = tags;
        Alerts = alerts;
        History = history;
        Provider = provider;

        Pages = new Dictionary<string, PageViewModelBase>
        {
#if DEBUG
            [Experiments.Name] = experiments,
#endif
            [Home.Name] = home,
            [Apps.Name] = apps,
            [Tags.Name] = tags,
            [Alerts.Name] = alerts,
            [History.Name] = history
        };
    }

    /// <summary>
    ///     IoC container
    /// </summary>
    public IServiceProvider Provider { get; }

    /// <summary>
    ///     Mapping between Page names and the <see cref="PageViewModelBase" />
    /// </summary>
    public Dictionary<string, PageViewModelBase> Pages { get; }

#if DEBUG
    public ExperimentsPageViewModel Experiments { get; }
#endif
    public HomePageViewModel Home { get; }
    public AppsPageViewModel Apps { get; }
    public TagsPageViewModel Tags { get; }
    public AlertsPageViewModel Alerts { get; }
    public HistoryPageViewModel History { get; }
}