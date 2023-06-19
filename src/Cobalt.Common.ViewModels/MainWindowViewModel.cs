using Cobalt.Common.ViewModels.Pages;
using CommunityToolkit.Mvvm.ComponentModel;

namespace Cobalt.Common.ViewModels;

public partial class MainWindowViewModel : ViewModelBase
{
    private readonly AlertsPageViewModel _alertsPage;
    private readonly AppsPageViewModel _appsPage;
    private readonly HistoryPageViewModel _historyPage;
    private readonly HomePageViewModel _homePage;
    private readonly SettingsPageViewModel _settingsPage;
    private readonly TagsPageViewModel _tagsPage;

    [ObservableProperty] private PageViewModelBase _currentPage;

    public MainWindowViewModel(
        AlertsPageViewModel alertsPage,
        AppsPageViewModel appsPage,
        HistoryPageViewModel historyPage,
        HomePageViewModel homePage,
        SettingsPageViewModel settingsPage,
        TagsPageViewModel tagsPage
    )
    {
        _alertsPage = alertsPage;
        _appsPage = appsPage;
        _historyPage = historyPage;
        _homePage = homePage;
        _settingsPage = settingsPage;
        _tagsPage = tagsPage;

        CurrentPage = _homePage;
    }

    public string Greeting => "Welcome to Avalonia!";

    public void NavigateTo(string page)
    {
        CurrentPage = page switch
        {
            "Home" => _homePage,
            "Apps" => _appsPage,
            "Tags" => _tagsPage,
            "Alerts" => _alertsPage,
            "History" => _historyPage,
            "Settings" => _settingsPage,
            _ => throw new InvalidOperationException("Unknown page")
        };
    }
}