using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using Cobalt.Common.ViewModels.Analysis;
using CommunityToolkit.Mvvm.ComponentModel;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Common.ViewModels.Entities;

/// <summary>
///     ViewModel for the <see cref="TriggerAction" /> owned object. Exists mainly to provide
///     <see cref="ObservableObject" /> to the <see cref="TriggerAction" />
/// </summary>
public partial class TriggerActionViewModel : ObservableObject
{
    [ObservableProperty] private TimeSpan? _dimDuration;
    [ObservableProperty] private string? _messageContent;
    [ObservableProperty] private long _tag;

    public TriggerActionViewModel(TriggerAction triggerAction)
    {
        _tag = triggerAction.Tag;
        _messageContent = triggerAction.MessageContent;
        _dimDuration = triggerAction.DimDuration;
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