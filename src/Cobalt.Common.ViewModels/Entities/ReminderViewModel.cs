using System.Reactive;
using System.Reactive.Linq;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using Cobalt.Common.ViewModels.Dialogs;
using CommunityToolkit.Mvvm.ComponentModel;
using Microsoft.EntityFrameworkCore;
using ReactiveUI;
using ReactiveUI.Validation.Abstractions;
using ReactiveUI.Validation.Contexts;
using ReactiveUI.Validation.Extensions;

namespace Cobalt.Common.ViewModels.Entities;

/// <summary>
///     ViewModel for an editable Reminder Entity
/// </summary>
public partial class EditableReminderViewModel : ReactiveObservableObject, IValidatableViewModel
{
    private readonly AlertDialogViewModelBase _alert;
    [ObservableProperty] private string? _commitMessage;
    [ObservableProperty] private double _commitThreshold;
    [ObservableProperty] private bool _editing;
    [ObservableProperty] private bool _isDirty;
    [ObservableProperty] private string? _message;
    [ObservableProperty] private double _threshold = double.NaN;

    public EditableReminderViewModel(AlertDialogViewModelBase alert, Reminder? reminder = null, bool editing = false)
    {
        _alert = alert;
        Reminder = reminder;
        if (reminder != null)
        {
            Message = reminder.Message;
            Threshold = reminder.Threshold;
        }

        Editing = editing;
        CommitMessage = Message;
        CommitThreshold = Threshold;

        var commitsValid = this.WhenAnyValue(self => self.CommitMessage, self => self.CommitThreshold,
            (message, threshold) => !string.IsNullOrWhiteSpace(message) && !double.IsNaN(threshold));

        StartEditingCommand = ReactiveCommand.Create(() =>
        {
            CommitMessage = Message;
            CommitThreshold = Threshold;
            Editing = true;
        });
        StopEditingCommand = ReactiveCommand.Create(() =>
        {
            Message = CommitMessage;
            Threshold = CommitThreshold;
            Editing = false;
        }, commitsValid);

        // Setup IsDirty
        this.WhenAnyValue(self => self.Message, self => self.Threshold,
                // ReSharper disable once CompareOfFloatsByEqualityOperator
                (message, threshold) =>
                    Reminder == null || Reminder.Message != message || Reminder.Threshold != threshold)
            .Subscribe(isDirty => IsDirty = isDirty);

        // Mark the properties as valid at start for the properties
        this.ValidationRule(self => self.CommitMessage,
            this.WhenAnyValue(self => self.CommitMessage,
                message => !string.IsNullOrWhiteSpace(message)).Skip(1).StartWith(true), "Message is empty");
        this.ValidationRule(self => self.CommitThreshold,
            this.WhenAnyValue(self => self.CommitThreshold,
                threshold => !double.IsNaN(threshold)).Skip(1).StartWith(true), "Threshold is empty");

        // Validate the whole model at the start
        this.ValidationRule(this.WhenAnyValue(self => self.Message,
            message => !string.IsNullOrWhiteSpace(message)), "Message is empty");
        this.ValidationRule(this.WhenAnyValue(self => self.Threshold,
            threshold => !double.IsNaN(threshold)), "Threshold is empty");
    }

    public Reminder? Reminder { get; set; }

    public ReactiveCommand<Unit, Unit> StopEditingCommand { get; }
    public ReactiveCommand<Unit, Unit> StartEditingCommand { get; }

    public IObservable<TimeSpan?> ThresholdUsageLimit => this.WhenAnyValue(self => self.Threshold,
        self => self._alert.UsageLimit,
        (threshold, usageLimit) => double.IsNaN(threshold) || usageLimit == null
            ? new TimeSpan?()
            : new TimeSpan((long)(threshold * usageLimit.Value.Ticks)));

    public ValidationContext ValidationContext { get; } = new();
}

/// <summary>
///     ViewModel for a read-only Reminder Entity
/// </summary>
public partial class ReminderViewModel : EditableEntityViewModelBase<Reminder>
{
    [ObservableProperty] private string _message;
    [ObservableProperty] private double _threshold;

    public ReminderViewModel(Reminder entity, IEntityViewModelCache entityCache,
        IDbContextFactory<QueryContext> contexts) : base(entity, entityCache,
        contexts)
    {
        Message = entity.Message;
        Threshold = entity.Threshold;
    }

    public AlertViewModel Alert => EntityCache.Alert(Entity.Alert);

    public IObservable<TimeSpan> ThresholdUsageLimit => this.WhenAnyValue(self => self.Threshold,
        self => self.Alert.UsageLimit, (threshold, usageLimit) => new TimeSpan((long)(threshold * usageLimit.Ticks)));

    public override void UpdateEntity()
    {
        // We ignore updating Alert here, since that will never get updated through this view model
        Entity.Message = Message ?? throw new InvalidOperationException(nameof(Message));
        Entity.Threshold = Threshold;
    }
}