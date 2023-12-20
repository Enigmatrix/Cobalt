using System.Collections.ObjectModel;
using System.Reactive.Disposables;
using System.Reactive.Linq;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using DynamicData;
using Microsoft.EntityFrameworkCore;
using ReactiveUI;

namespace Cobalt.Common.ViewModels.Pages;

/// <summary>
///     ViewModel for the Experiments Page
/// </summary>
/// <remarks>
///     This class does not exist during production.
/// </remarks>
public partial class ExperimentsPageViewModel : PageViewModelBase
{
    [ObservableProperty] private string _search = "";

    public ExperimentsPageViewModel(IDbContextFactory<QueryContext> contexts) : base(contexts)
    {
        this.WhenActivated(dis =>
        {
            this.WhenAnyValue(x => x.Search)
                .SelectMany(async search => await GetApps(search))
                .BindTo(this, x => x.Apps)
                .DisposeWith(dis);

            this.WhenAnyValue(x => x.Search)
                .SelectMany(async search => await GetTags(search))
                .BindTo(this, x => x.Tags)
                .DisposeWith(dis);

            this.WhenAnyValue(x => x.SelectedApp)
                .Where(x => x != null)
                .Subscribe(x =>
                {
                    SelectedTag = null;
                    Selected = x!;
                })
                .DisposeWith(dis);

            this.WhenAnyValue(x => x.SelectedTag)
                .Where(x => x != null)
                .Subscribe(x =>
                {
                    SelectedApp = null;
                    Selected = x!;
                })
                .DisposeWith(dis);

            this.WhenAnyValue(x => x.SelectedTag, x => x.SelectedApp)
                .Where(x => x.Item1 == null && x.Item2 == null)
                .Subscribe(x =>
                { Selected = null;
                })
                .DisposeWith(dis);
        });
    }

    public override string Name => "Experiments";

    public async Task<List<App>> GetApps(string search)
    {
        await using var context = await Contexts.CreateDbContextAsync();
        var a = await context.Apps.Where(x => x.Name.ToLower().Contains(search.ToLower()) || x.Description.Contains(search) || x.Company.Contains(search)).ToListAsync();
        return a;
    }

    public async Task<List<Tag>> GetTags(string search)
    {
        await using var context = await Contexts.CreateDbContextAsync();

        var a = await context.Tags.Where(x => x.Name.ToLower().Contains(search.ToLower())).ToListAsync();
        return a;
    }

    //[ObservableProperty] private ReadOnlyObservableCollection<App> _apps;
    [ObservableProperty] private List<App> _apps;
    [ObservableProperty] private List<Tag> _tags;
    [ObservableProperty] private object? _selected;
    [ObservableProperty] private App? _selectedApp;
    [ObservableProperty] private Tag? _selectedTag;

    [RelayCommand]
    public async Task UpdateAllUsageEndsAsync()
    {
        await using var context = await Contexts.CreateDbContextAsync();
        await context.UpdateAllUsageEndsAsync(DateTime.Now);
    }
}