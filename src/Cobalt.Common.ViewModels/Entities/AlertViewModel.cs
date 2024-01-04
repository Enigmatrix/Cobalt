using System.Linq.Expressions;
using System.Reactive.Linq;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using Cobalt.Common.Util;
using CommunityToolkit.Mvvm.ComponentModel;
using Microsoft.EntityFrameworkCore;
using ReactiveUI;
using ReactiveUI.Validation.Abstractions;
using ReactiveUI.Validation.Contexts;
using ReactiveUI.Validation.Extensions;

namespace Cobalt.Common.ViewModels.Entities;

/// <summary>
///     ViewModel for the <see cref="TriggerAction" /> owned object. Exists mainly to provide
///     <see cref="ObservableObject" /> to the <see cref="TriggerAction" />
/// </summary>
public partial class TriggerActionViewModel : ReactiveObservableObject, IValidatableViewModel
{
    [ObservableProperty] private TimeSpan? _dimDuration;
    [ObservableProperty] private string? _messageContent;
    [ObservableProperty] private long? _tag;

    public TriggerActionViewModel(TriggerAction? action = null)
    {
        if (action != null)
        {
            Tag = action.Tag;
            MessageContent = action.MessageContent;
            DimDuration = action.DimDuration;
        }

        // Reset properties after switching to other enums
        // These should be disposed, but we don't really have a good timing for it ...
        this.WhenAnyValue(self => self.Tag).Where(tag => tag != TriggerAction.MessageTag)
            .Subscribe(_ => MessageContent = null);
        this.WhenAnyValue(self => self.Tag).Where(tag => tag != TriggerAction.DimTag)
            .Subscribe(_ => DimDuration = null);

        this.ValidationRule(self => self.Tag,
            tag => tag != null,
            "Trigger Action must be selected");

        this.ValidationRule(
            self => self.MessageContent,
            WhenTagAndPropertyValid(TriggerAction.MessageTag, self => self.MessageContent,
                messageContent => !string.IsNullOrWhiteSpace(messageContent)),
            "Message Content is empty");

        this.ValidationRule(
            self => self.DimDuration,
            WhenTagAndPropertyValid(TriggerAction.DimTag, self => self.DimDuration, dimDuration => dimDuration != null),
            "Dim Duration is empty");

        this.ValidationRule(self => self.DimDuration, dimDuration => dimDuration == null || dimDuration > TimeSpan.Zero,
            "Dim Duration cannot be negative");

        this.ValidationRule(
            this.WhenAnyValue(
                self => self.Tag,
                self => self.MessageContent,
                self => self.DimDuration),
            props => props.Item1 switch
            {
                TriggerAction.KillTag => true,
                TriggerAction.MessageTag => !string.IsNullOrWhiteSpace(props.Item2),
                TriggerAction.DimTag => props.Item3 != null,
                null => false,
                _ => throw new DiscriminatedUnionException<long?>(nameof(props.Item1), props.Item1)
            },
            _ => "Fields are empty");
    }

    /// <inheritdoc />
    public ValidationContext ValidationContext { get; } = new();

    /// <summary>
    ///     When <see cref="Tag" /> matches <paramref name="tagMatch" />, validate <paramref name="prop" /> according to
    ///     <paramref name="validate" /> and send the result, skipping the first validation since that value is always null.
    ///     Otherwise send true by default.
    /// </summary>
    /// <typeparam name="T">Type of <paramref name="prop" /></typeparam>
    /// <param name="tagMatch">Matching <see cref="Tag" /> value</param>
    /// <param name="prop">Property extractor</param>
    /// <param name="validate">The actual validation logic</param>
    private IObservable<bool> WhenTagAndPropertyValid<T>(long tagMatch,
        Expression<Func<TriggerActionViewModel, T>> prop,
        Func<T, bool> validate)
    {
        return this.WhenAnyValue(self => self.Tag)
            .Select(tag => this
                .WhenAnyValue(prop)
                // don't validate at the start, value is intentionally null at that point.
                .SkipWhile((_, idx) => tag == tagMatch && idx == 0)
                .Select(v => tag != tagMatch || validate(v)))
            .Switch();
    }

    /// <summary>
    ///     Reset all properties
    /// </summary>
    public void Clear()
    {
        Tag = null;
        MessageContent = null;
        DimDuration = null;
    }

    /// <summary>
    ///     Convert back to a <see cref="TriggerAction" />
    /// </summary>
    public TriggerAction? ToTriggerAction()
    {
        if (!ValidationContext.IsValid) return null;
        var triggerAction = Tag switch
        {
            /*0 => new TriggerAction.Kill(),
            1 => new TriggerAction.Message(MessageContent!),
            2 => new TriggerAction.Dim(DimDuration!.Value),*/
            // Until DbContexts can recognize DUs and load them properly from the db, we gotta resort to this
            0 => new TriggerAction(0),
            1 => new TriggerAction(1, MessageContent!),
            2 => new TriggerAction(2, DimDuration: DimDuration ?? TimeSpan.MinValue),
            _ => throw new DiscriminatedUnionException<long?>(nameof(Tag), Tag)
        };

        return triggerAction;
    }
}

/// <summary>
///     ViewModel for the Alert Entity
/// </summary>
public partial class AlertViewModel : EditableEntityViewModelBase<Alert>
{
    [ObservableProperty] private AppViewModel? _app;
    [ObservableProperty] private List<ReminderViewModel> _reminders;
    [ObservableProperty] private TagViewModel? _tag;
    [ObservableProperty] private TimeFrame _timeFrame;
    [ObservableProperty] private TriggerAction _triggerAction;
    [ObservableProperty] private TimeSpan _usageLimit;

    public AlertViewModel(Alert entity, IEntityViewModelCache entityCache, IDbContextFactory<QueryContext> contexts) :
        base(entity, entityCache, contexts)
    {
        TimeFrame = entity.TimeFrame;
        UsageLimit = entity.UsageLimit;
        TriggerAction = entity.TriggerAction;
        if (entity.App != null) App = EntityCache.App(entity.App);
        else if (entity.Tag != null) Tag = EntityCache.Tag(entity.Tag);
        Reminders = entity.Reminders.Select(EntityCache.Reminder).ToList();
    }

    public override void UpdateEntity()
    {
        Entity.TimeFrame = TimeFrame;
        Entity.UsageLimit = UsageLimit;
        Entity.TriggerAction = TriggerAction;
        if (App != null) Entity.App = App.Entity;
        else if (Tag != null) Entity.Tag = Tag.Entity;
        Entity.Reminders.Clear();
        Entity.Reminders.AddRange(Reminders.Select(reminder => reminder.Entity));
    }
}