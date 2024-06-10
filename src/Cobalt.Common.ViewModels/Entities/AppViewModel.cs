using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using Cobalt.Common.ViewModels.Analysis;
using CommunityToolkit.Mvvm.ComponentModel;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Common.ViewModels.Entities;

/// <summary>
///     ViewModel for the App Entity
/// </summary>
public partial class AppViewModel : EditableEntityViewModelBase<App>, IHasColor, IHasName, IHasIcon
{
    [ObservableProperty] private string _color;
    [ObservableProperty] private string _company;
    [ObservableProperty] private string _description;
    private byte[]? _icon;
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

        Image = Query(async context => _icon ??= await context.GetAppIconBytes(Entity), false);
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