using System.Linq.Expressions;
using System.Reactive.Linq;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using Cobalt.Common.ViewModels.Analysis;
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

    // TODO commonalize the 1 and 2
    public TriggerActionViewModel()
    {
        // Reset properties after switching to other enums
        // TODO dispose ...
        this.WhenAnyValue(self => self.Tag).Where(tag => tag != 1).Subscribe(_ => MessageContent = null);
        this.WhenAnyValue(self => self.Tag).Where(tag => tag != 2).Subscribe(_ => DimDuration = null);

        this.ValidationRule(self => self.Tag,
            tag => tag != null,
            "Trigger Action must be selected");

        this.ValidationRule(
            self => self.MessageContent,
            WhenTagAndPropertyValid(1, self => self.MessageContent,
                messageContent => !string.IsNullOrWhiteSpace(messageContent)),
            "Message Content is empty");

        this.ValidationRule(
            self => self.DimDuration,
            WhenTagAndPropertyValid(2, self => self.DimDuration, dimDuration => dimDuration != null),
            "Dim Duration is empty");

        this.ValidationRule(self => self.DimDuration, dimDuration => dimDuration == null || dimDuration > TimeSpan.Zero,
            "Dim Duration cannot be negative");
    }

    /// <inheritdoc />
    public ValidationContext ValidationContext { get; } = new();

    /// <summary>
    ///     Real usability validation status as an <see cref="IObservable{Boolean}" />
    /// </summary>
    public IObservable<bool> UsableValid()
    {
        return this.WhenAnyValue(
            self => self.Tag,
            self => self.MessageContent,
            self => self.DimDuration
            , (tag, messageContent, dimDuration) => tag switch
            {
                0 => true,
                1 => !string.IsNullOrWhiteSpace(messageContent),
                2 => dimDuration != null,
                null => false,
                _ => throw new Exception() // TODO actual exception
            });
    }

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
    public TriggerAction ToTriggerAction()
    {
        TriggerAction triggerAction = Tag switch
        {
            0 => new TriggerAction.Kill(),
            1 =>
                // TODO make bad MessageContent throw a validation error
                new TriggerAction.Message(MessageContent ?? ""),
            2 =>
                // TODO make bad DimDuration throw a validation error
                new TriggerAction.Dim(DimDuration ?? TimeSpan.Zero),
            _ => throw new InvalidOperationException() // TODO better exception
        };

        return triggerAction;
    }
}

/// <summary>
///     ViewModel for the App Entity
/// </summary>
public partial class AppViewModel : EditableEntityViewModelBase<App>
{
    [ObservableProperty] private string _color;
    [ObservableProperty] private string _company;
    [ObservableProperty] private string _description;
    [ObservableProperty] private AppIdentity _identity;
    [ObservableProperty] private bool _initialized;
    [ObservableProperty] private string _name;

    public AppViewModel(App entity, IEntityViewModelCache entityCache, IDbContextFactory<QueryContext> contexts) : base(
        entity, entityCache, contexts)
    {
        _initialized = entity.Initialized;
        _name = entity.Name;
        _description = entity.Description;
        _company = entity.Company;
        _color = entity.Color;
        _identity = entity.Identity;

        Image = Query(async context => await context.GetAppIconBytes(Entity), false);
    }

    /// <summary>
    ///     Icon for the <see cref="App" />
    /// </summary>
    public Query<byte[]?> Image { get; }

    public override void UpdateEntity()
    {
        Entity.Initialized = true; // cannot be set to false as a user
        Entity.Name = Name;
        Entity.Description = Description;
        Entity.Company = Company;
        Entity.Color = Color;
        Entity.Identity = Identity;
    }
}