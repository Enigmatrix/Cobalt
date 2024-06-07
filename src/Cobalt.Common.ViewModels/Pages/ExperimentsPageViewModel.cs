using System.Reactive.Disposables;
using Cobalt.Common.Data;
using Cobalt.Common.ViewModels.Analysis;
using Cobalt.Common.ViewModels.Entities;
using CommunityToolkit.Mvvm.Input;
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
    public ExperimentsPageViewModel(IEntityViewModelCache entityCache, IDbContextFactory<QueryContext> contexts) :
        base(contexts)
    {
        AppUsagesPerDay = Query(context => context.AppDurations(context.Apps, DateTime.Today).ToListAsync(),
            appDur => appDur.Map(entityCache.App));

        this.WhenActivated((CompositeDisposable dis) => { AppUsagesPerDay.Refresh(); });
    }

    public Query<List<WithDuration<AppViewModel>>> AppUsagesPerDay { get; }

    public override string Name => "Experiments";

    [RelayCommand]
    public async Task UpdateAllUsageEndsAsync()
    {
        await using var context = await Contexts.CreateDbContextAsync();
        await context.UpdateAllUsageEndsAsync(DateTime.Now);
    }
}