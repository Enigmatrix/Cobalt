using System.Linq.Expressions;
using System.Reactive.Linq;
using Cobalt.Common.Data.Entities;
using Cobalt.Common.Util;
using CommunityToolkit.Mvvm.ComponentModel;
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
    [ObservableProperty] private TriggerAction? _inner;
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

        this.WhenAnyValue(
                self => self.Tag,
                self => self.MessageContent,
                self => self.DimDuration, (tag, messageContent, dimDuration) =>
                {
                    return tag switch
                    {
                        TriggerAction.KillTag => new TriggerAction(TriggerAction.KillTag),
                        TriggerAction.MessageTag => string.IsNullOrWhiteSpace(messageContent)
                            ? null
                            : new TriggerAction(TriggerAction.MessageTag, messageContent),
                        TriggerAction.DimTag => dimDuration == null
                            ? null
                            : new TriggerAction(TriggerAction.DimTag, DimDuration: dimDuration),
                        null => null,
                        _ => throw new DiscriminatedUnionException<long?>(nameof(Tag), Tag)
                    };
                })
            // This is a workaround for #117
            .ObserveOn(RxApp.TaskpoolScheduler)
            .Subscribe(inner => Inner = inner);

        this.ValidationRule(
            this.WhenAnyValue(self => self.Inner),
            inner => inner != null,
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
                .SkipWhile((v, idx) => tag == tagMatch && idx == 0 && v == null)
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
}