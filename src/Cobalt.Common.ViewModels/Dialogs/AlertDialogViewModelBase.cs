﻿using System.Reactive.Linq;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using Cobalt.Common.Util;
using Cobalt.Common.ViewModels.Entities;
using CommunityToolkit.Mvvm.ComponentModel;
using CommunityToolkit.Mvvm.Input;
using DynamicData;
using DynamicData.Binding;
using Microsoft.EntityFrameworkCore;
using ReactiveUI;
using ReactiveUI.Validation.Abstractions;
using ReactiveUI.Validation.Contexts;
using ReactiveUI.Validation.Extensions;

namespace Cobalt.Common.ViewModels.Dialogs;

/// <summary>
///     Dialog ViewModel to modify Alerts
/// </summary>
public abstract partial class AlertDialogViewModelBase : DialogViewModelBase<AlertViewModel>, IValidatableViewModel
{
    protected readonly IEntityViewModelCache EntityCache;
    protected readonly SourceList<EditableReminderViewModel> RemindersSource = new();

    [ObservableProperty] private ChooseTargetDialogViewModel? _chooseTargetDialog;
    [ObservableProperty] private TimeFrame? _timeFrame;
    [ObservableProperty] private TriggerActionViewModel? _triggerAction;
    [ObservableProperty] private TimeSpan? _usageLimit;


    // TODO count how many refreshes are done, try to get rid of the assumeRefreshIsCalled parameter
    // TODO too many ^ bindings to Apps/Tags, try reduce?


    protected AlertDialogViewModelBase(IEntityViewModelCache entityCache, IDbContextFactory<QueryContext> contexts) :
        base(contexts)
    {
        EntityCache = entityCache;

        var validUsageLimitAndTimeFrame =
            this.WhenAnyValue(self => self.UsageLimit, self => self.TimeFrame, ValidUsageLimitAndTimeFrame);

        this.ValidationRule(self => self.TimeFrame, validUsageLimitAndTimeFrame,
            "Time Frame smaller than Usage Limit");

        this.ValidationRule(self => self.UsageLimit, validUsageLimitAndTimeFrame,
            "Usage Limit larger than Time Frame");
        this.ValidationRule(self => self.UsageLimit, usageLimit => usageLimit == null || usageLimit > TimeSpan.Zero,
            "Usage Limit cannot be negative");

        // Additionally, validation that all our properties are set.
        // This isn't a validation rule we don't want to display "X is unset" errors,
        // that would just mean the entire form is red.
        // This does not need to be disposed, and exists here so that at the point before activation,
        // the Primary Button is already disabled
        this.ValidationRule(this.WhenAnyValue(
                self => self.ChooseTargetDialog!.Target,
                self => self.UsageLimit,
                self => self.TimeFrame),
            props => props is { Item1: not null, Item2: not null, Item3: not null },
            _ => "Fields are empty");

        RemindersSource
            .Connect()
            .AutoRefreshOnObservable(reminder => reminder.WhenAnyValue(self => self.Threshold))
            .Sort(SortExpressionComparer<EditableReminderViewModel>.Ascending(reminder => reminder.Threshold))
            .Bind(Reminders)
            .Subscribe();

        this.ValidationRule(RemindersSource
                .Connect()
                .AddKey(reminder =>
                    reminder) // This is just to change this to a SourceCache-model since TrueForAll only exists for this
                .TrueForAll(reminder => reminder.IsValid()
                        .CombineLatest(reminder.WhenAnyValue(self => self.Editing)),
                    static prop => prop is { First: true, Second: false })
                .StartWith(true), // Reminders are empty at start
            "Reminders are invalid");
    }

    public ObservableCollectionExtended<EditableReminderViewModel> Reminders { get; } = new();

    protected AlertViewModel? Result { get; set; }

    public ValidationContext ValidationContext { get; } = new();

    public void AddReminder()
    {
        RemindersSource.Add(new EditableReminderViewModel(this, editing: true));
    }

    [RelayCommand]
    public void DeleteReminder(EditableReminderViewModel reminder)
    {
        RemindersSource.Remove(reminder);
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

    public override AlertViewModel GetResult()
    {
        return Result ?? throw new InvalidOperationException();
    }
}