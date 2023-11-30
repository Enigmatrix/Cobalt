using Cobalt.Common.ViewModels.Pages;

namespace Cobalt.Common.ViewModels;

public class MainViewModel : ViewModelBase
{
    public MainViewModel(HomePageViewModel home, AppsPageViewModel apps, TagsPageViewModel tags,
        AlertsPageViewModel alerts, HistoryPageViewModel history)
    {
        Home = home;
        Apps = apps;
        Tags = tags;
        Alerts = alerts;
        History = history;
    }

    public HomePageViewModel Home { get; }
    public AppsPageViewModel Apps { get; }
    public TagsPageViewModel Tags { get; }
    public AlertsPageViewModel Alerts { get; }
    public HistoryPageViewModel History { get; }
}