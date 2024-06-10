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
        var dayStart = DateTime.Today;
        var weekStart = dayStart.AddDays(-(int)dayStart.DayOfWeek);
        var monthStart = dayStart.AddDays(-dayStart.Day + 1);

        AppUsagesPerDay = Query(context => context.AppDurations(start: dayStart).ToListAsync(),
            appDur => appDur.Map(entityCache.App));

        AppUsagesPerWeek = Query(context => context.AppDurations(start: weekStart).ToListAsync(),
            appDur => appDur.Map(entityCache.App));

        AppUsagesPerMonth = Query(context => context.AppDurations(start: monthStart).ToListAsync(),
            appDur => appDur.Map(entityCache.App));

        DailyUsage = Query(context => context.UsagesBetween(start: dayStart));
        WeeklyUsage = Query(context => context.UsagesBetween(start: weekStart));
        MonthlyUsage = Query(context => context.UsagesBetween(start: monthStart));

        this.WhenActivated((CompositeDisposable dis) =>
        {
            AppUsagesPerDay.Refresh();
            AppUsagesPerWeek.Refresh();
            AppUsagesPerMonth.Refresh();

            DailyUsage.Refresh();
            WeeklyUsage.Refresh();
            MonthlyUsage.Refresh();
        });
    }

    public Query<List<WithDuration<AppViewModel>>> AppUsagesPerDay { get; }
    public Query<List<WithDuration<AppViewModel>>> AppUsagesPerWeek { get; }
    public Query<List<WithDuration<AppViewModel>>> AppUsagesPerMonth { get; }
    public Query<TimeSpan> DailyUsage { get; }
    public Query<TimeSpan> WeeklyUsage { get; }
    public Query<TimeSpan> MonthlyUsage { get; }

    public override string Name => "Home";
}