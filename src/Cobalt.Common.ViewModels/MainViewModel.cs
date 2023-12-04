using Cobalt.Common.ViewModels.Pages;

namespace Cobalt.Common.ViewModels;

public class MainViewModel : ViewModelBase
{
    public MainViewModel(
#if DEBUG
        ExperimentsPageViewModel experiments,
#endif
        HomePageViewModel home, AppsPageViewModel apps, TagsPageViewModel tags,
        AlertsPageViewModel alerts, HistoryPageViewModel history)
    {
#if DEBUG
        Experiments = experiments;
#endif
        Home = home;
        Apps = apps;
        Tags = tags;
        Alerts = alerts;
        History = history;
    }

#if DEBUG
    public ExperimentsPageViewModel Experiments { get; }
#endif
    public HomePageViewModel Home { get; }
    public AppsPageViewModel Apps { get; }
    public TagsPageViewModel Tags { get; }
    public AlertsPageViewModel Alerts { get; }
    public HistoryPageViewModel History { get; }
}