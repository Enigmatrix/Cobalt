using Cobalt.Common.ViewModels.Pages;
using CommunityToolkit.Mvvm.ComponentModel;

namespace Cobalt.Common.ViewModels;

public partial class MainWindowViewModel : ViewModelBase
{
    private readonly AlertsPageViewModel _alertsPage = new();
    private readonly AppsPageViewModel _appsPage = new();
    private readonly HistoryPageViewModel _historyPage = new();
    private readonly HomePageViewModel _homePage = new();
    private readonly SettingsPageViewModel _settingsPage = new();
    private readonly TagsPageViewModel _tagsPage = new();
    [ObservableProperty] private PageViewModelBase _currentPage;

    public MainWindowViewModel()
    {
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