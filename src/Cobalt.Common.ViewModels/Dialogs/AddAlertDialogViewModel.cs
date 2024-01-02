﻿using System.Collections.ObjectModel;
using System.Reactive;
using System.Reactive.Disposables;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using Cobalt.Common.Util;
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
    [ObservableProperty] private TimeSpan? _usageLimit;


    // TODO count how many refreshes are done, try to get rid of the assumeRefreshIsCalled parameter
    // TODO too many ^ bindings to Apps/Tags, try reduce?

    public AddAlertDialogViewModel(IEntityViewModelCache entityCache, IDbContextFactory<QueryContext> contexts) :
        base(contexts)
    {
        _entityCache = entityCache;
        var searches = this.WhenAnyValue(self => self.TargetSearch);

        Apps = Query(searches,
            async (context, search) => await context.SearchApps(search).ToListAsync(),
            _entityCache.App, false);

        Tags = Query(searches,
            async (context, search) => await context.SearchTags(search).ToListAsync(),
            _entityCache.Tag, false);

        // This is validation context composition
        ValidationContext.Add(TriggerAction.ValidationContext);

        var validUsageLimitAndTimeFrame =
            this.WhenAnyValue(self => self.UsageLimit, self => self.TimeFrame, ValidUsageLimitAndTimeFrame);

        // Additionally, validation that all our properties are set.
        // This isn't a validation rule we don't want to display "X is unset" errors,
        // that would just mean the entire form is red.
        this.ValidationRule(this.WhenAnyValue(
                self => self.SelectedTarget,
                self => self.UsageLimit,
                self => self.TimeFrame),
            props => props is { Item1: not null, Item2: not null, Item3: not null },
            _ => "Fields are empty");

        PrimaryButtonCommand =
            ReactiveCommand.CreateFromTask(ProduceAlert, this.IsValid());

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


            this.ValidationRule(self => self.TimeFrame, validUsageLimitAndTimeFrame,
                "Time Frame smaller than Usage Limit").DisposeWith(dis);

            this.ValidationRule(self => self.UsageLimit, validUsageLimitAndTimeFrame,
                "Usage Limit larger than Time Frame").DisposeWith(dis);
            this.ValidationRule(self => self.UsageLimit, usageLimit => usageLimit == null || usageLimit > TimeSpan.Zero,
                "Usage Limit cannot be negative").DisposeWith(dis);


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

    public ObservableCollection<ReminderViewModel> Reminders { get; } = new();

    public override ReactiveCommand<Unit, Unit> PrimaryButtonCommand { get; set; }

    public Query<string, List<AppViewModel>> Apps { get; }
    public Query<string, List<TagViewModel>> Tags { get; }

    public override string Title => "Add Alert";

    private AlertViewModel? Result { get; set; }

    public ValidationContext ValidationContext { get; } = new();

    public void AddReminder()
    {
        // TODO Create a new ReminderViewModel type that's editable in place
        // e.g. it has a EditMode field that's set to true when you just add it, with validation rules
        Reminders.Add(new ReminderViewModel(new Reminder
        {
            Guid = Guid.NewGuid(),
            Version = 1,
            Alert = null!, // this is actually fine!
            Message = "Go to work!", // TODO for testing
            Threshold = 0.10 // TODO for testing
        }, _entityCache, Contexts));
    }

    private bool ValidUsageLimitAndTimeFrame(TimeSpan? usageLimit, TimeFrame? timeFrame)
    {
        if (usageLimit == null || timeFrame == null) return true;

        return timeFrame switch
        {
            Data.Entities.TimeFrame.Daily => usageLimit <= TimeSpan.FromDays(1),
            Data.Entities.TimeFrame.Weekly => usageLimit <= TimeSpan.FromDays(7),
            Data.Entities.TimeFrame.Monthly => usageLimit <= TimeSpan.FromDays(31),
            _ => throw new DiscriminatedUnionException<TimeFrame?>(nameof(timeFrame), timeFrame)
        };
    }

    public async Task ProduceAlert()
    {
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

        Result = new AlertViewModel(alert, _entityCache, Contexts);
    }

    public override AlertViewModel GetResult()
    {
        return Result ?? throw new InvalidOperationException();
    }
}