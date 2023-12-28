using System.Diagnostics;
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

    public TriggerActionViewModel()
    {
        ValidationContext = new ValidationContext();
        this.WhenAnyValue(self => self.Tag).Where(tag => tag != 1).Subscribe(_ => MessageContent = null);
        this.WhenAnyValue(self => self.Tag).Where(tag => tag != 2).Subscribe(_ => DimDuration = null);

        this.WhenAnyValue(self => self.MessageContent).Subscribe(nani => Debug.WriteLine($"send help: {nani}"));

        var messageContentValid = this.WhenAnyValue(self => self.Tag)
            .Select(tag => this
                .WhenAnyValue(self => self.MessageContent)
                // don't validate at the start, MessageContent is intentionally null at that point.
                .SkipWhile((_, idx) => tag == 1 && idx == 0)
                .Select(messageContent => tag != 1 || !string.IsNullOrWhiteSpace(messageContent)))
            .Switch();

        var dimDurationValid = this.WhenAnyValue(self => self.Tag)
            .Select(tag => this
                .WhenAnyValue(self => self.DimDuration)
                // don't validate at the start, DimDuration is intentionally null at that point.
                .SkipWhile((_, idx) => tag == 2 && idx == 0)
                .Select(dimDuration => tag != 2 || dimDuration != null))
            .Switch();

        // TODO shall we just use fucking .BindValidation?

        this.ValidationRule(
            self => self.MessageContent,
            messageContentValid.Do(x => Debug.WriteLine($"Message content is {(x ? "valid" : "invalid")}")),
            "Message Content is empty");

        this.ValidationRule(
            self => self.DimDuration,
            dimDurationValid,
            "Dim Duration is empty");
    }

    /// <inheritdoc />
    public ValidationContext ValidationContext { get; } = new();

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