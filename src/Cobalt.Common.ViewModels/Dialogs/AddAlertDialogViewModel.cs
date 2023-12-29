using System.Reactive.Disposables;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using Cobalt.Common.ViewModels.Analysis;
using Cobalt.Common.ViewModels.Entities;
using CommunityToolkit.Mvvm.ComponentModel;
using Microsoft.EntityFrameworkCore;
using ReactiveUI;
using ReactiveUI.Validation.Abstractions;
using ReactiveUI.Validation.Contexts;
using ReactiveUI.Validation.Extensions;

namespace Cobalt.Common.ViewModels.Dialogs;

/// <summary>
///     Dialog ViewModel to Add Alert
/// </summary>
public partial class AddAlertDialogViewModel : DialogViewModelBase<AlertViewModel>, IValidatableViewModel
{
    private readonly IEntityViewModelCache _entityCache;
    [ObservableProperty] private AppViewModel? _selectedApp;
    [ObservableProperty] private TagViewModel? _selectedTag;
    [ObservableProperty] private EntityViewModelBase? _selectedTarget;
    [ObservableProperty] private string _targetSearch = "";
    [ObservableProperty] private TimeFrame? _timeFrame;
    [ObservableProperty] private TriggerActionViewModel _triggerAction = new();

    // TODO find some validation library!!!!
    // TODO count how many refreshes are done, try to get rid of the assumeRefreshIsCalled parameter
    // TODO too many bindings to Apps/Tags, try reduce?

    [ObservableProperty] private TimeSpan? _usageLimit;

    public AddAlertDialogViewModel(IEntityViewModelCache entityCache, IDbContextFactory<QueryContext> contexts) :
        base(contexts)
    {
        _entityCache = entityCache;
        var searches = this.WhenAnyValue(self => self.TargetSearch);

        // TODO commonalize the search expression 
        // TODO use some case-ignore string search instead of tolower

        Apps = Query(searches,
            async (context, search) => await context.Apps.Where(app =>
                app.Name.ToLower().Contains(search.ToLower()) ||
                app.Description.ToLower().Contains(search.ToLower()) ||
                app.Company.ToLower().Contains(search.ToLower())).ToListAsync(),
            _entityCache.App, false);

        Tags = Query(searches,
            async (context, search) => await context.Tags.Where(tag =>
                tag.Name.ToLower().Contains(search.ToLower())).ToListAsync(),
            _entityCache.Tag, false);

        // TODO INHERITABLEASDASDSADASD
        ValidationContext.Add(TriggerAction.ValidationContext);

        var validUsageLimitAndTimeFrame =
            this.WhenAnyValue(self => self.UsageLimit, self => self.TimeFrame, ValidUsageLimitAndTimeFrame);

        this.ValidationRule(self => self.TimeFrame, validUsageLimitAndTimeFrame,
            "Time Frame cannot contain Usage Limit");

        this.ValidationRule(self => self.UsageLimit, validUsageLimitAndTimeFrame,
            "Usage Limit cannot contain Time Frame");
        this.WhenActivated(dis =>
        {
            TargetSearch = "";
            SelectedTarget = null;
            SelectedApp = null;
            SelectedTag = null;
            UsageLimit = null;
            TimeFrame = null;
            TriggerAction.Clear();
            Apps.Refresh();
            Tags.Refresh();


            // Reset SelectedTarget based on the two selection properties, SelectedApp and SelectedTag
            this.WhenAnyValue(self => self.SelectedApp)
                .WhereNotNull()
                .Subscribe(app =>
                {
                    SelectedTag = null;
                    SelectedTarget = app;
                })
                .DisposeWith(dis);

            this.WhenAnyValue(self => self.SelectedTag)
                .WhereNotNull()
                .Subscribe(tag =>
                {
                    SelectedApp = null;
                    SelectedTarget = tag;
                })
                .DisposeWith(dis);
        });
    }

    public Query<string, List<AppViewModel>> Apps { get; }
    public Query<string, List<TagViewModel>> Tags { get; }

    public override string Title => "Add Alert";

    public ValidationContext ValidationContext { get; } = new();

    private bool ValidUsageLimitAndTimeFrame(TimeSpan? usageLimit, TimeFrame? timeFrame)
    {
        if (usageLimit == null || timeFrame == null) return true;

        return timeFrame switch
        {
            Data.Entities.TimeFrame.Daily => usageLimit <= TimeSpan.FromDays(0.1),
            Data.Entities.TimeFrame.Weekly => usageLimit <= TimeSpan.FromDays(0.7),
            Data.Entities.TimeFrame.Monthly => usageLimit <=
                                               TimeSpan.FromDays(3.1), // maximum number of days in a month
            _ => throw new ArgumentOutOfRangeException(nameof(timeFrame), timeFrame, null) // TODO better exception
        };
    }

    public override async Task<AlertViewModel> GetResultAsync()
    {
        // TODO validate
        var alert = new Alert
        {
            Guid = Guid.NewGuid(),
            Version = 1,
            TimeFrame = TimeFrame!.Value,
            TriggerAction = TriggerAction.ToTriggerAction(),
            UsageLimit = UsageLimit!.Value
        };
        switch (SelectedTarget)
        {
            case AppViewModel app:
                alert.App = app.Entity;
                break;
            case TagViewModel tag:
                alert.Tag = tag.Entity;
                break;
        }

        await using var context = await Contexts.CreateDbContextAsync();
        context.Attach(alert);
        await context.Alerts.AddAsync(alert);
        await context.SaveChangesAsync();

        return new AlertViewModel(alert, _entityCache, Contexts);
    }
}