using System.Reactive.Disposables;
using Cobalt.Common.Data;
using Cobalt.Common.ViewModels.Analysis;
using Cobalt.Common.ViewModels.Entities;
using Microsoft.EntityFrameworkCore;
using ReactiveUI;

namespace Cobalt.Common.ViewModels.Pages;

/// <summary>
///     ViewModel for the Home Page
/// </summary>
public class HomePageViewModel : PageViewModelBase
{
    public HomePageViewModel(IEntityViewModelCache entityCache, IDbContextFactory<QueryContext> contexts) :
        base(contexts)
    {
        AppUsagesPerDay = Query(context => context.AppDurations(context.Apps, DateTime.Today).ToListAsync(),
            appDur => appDur.Map(entityCache.App));

        AppUsagesPerWeek = Query(
            context => context.AppDurations(context.Apps, DateTime.Today.AddDays(-(int)DateTime.Today.DayOfWeek))
                .ToListAsync(),
            appDur => appDur.Map(entityCache.App));

        AppUsagesPerMonth = Query(
            context => context.AppDurations(context.Apps, DateTime.Today.AddDays(-DateTime.Today.Day + 1))
                .ToListAsync(),
            appDur => appDur.Map(entityCache.App));

        this.WhenActivated((CompositeDisposable dis) =>
        {
            AppUsagesPerDay.Refresh();
            AppUsagesPerWeek.Refresh();
            AppUsagesPerMonth.Refresh();
        });
    }

    public Query<List<WithDuration<AppViewModel>>> AppUsagesPerDay { get; }
    public Query<List<WithDuration<AppViewModel>>> AppUsagesPerWeek { get; }
    public Query<List<WithDuration<AppViewModel>>> AppUsagesPerMonth { get; }

    public override string Name => "Home";
}