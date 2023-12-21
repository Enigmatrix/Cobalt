using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using CommunityToolkit.Mvvm.ComponentModel;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Common.ViewModels.Entities;

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
    }

    /// <summary>
    ///     Icon for the <see cref="App" />
    /// </summary>
    public Task<byte[]?> Image => GetImage();

    private async Task<byte[]?> GetImage()
    {
        return await Task.Run(async () =>
        {
            await using var context = await Contexts.CreateDbContextAsync();
            return await context.GetAppIconBytes(Entity);
        });
    }

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