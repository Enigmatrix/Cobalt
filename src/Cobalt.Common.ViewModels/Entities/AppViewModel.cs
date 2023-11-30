using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using CommunityToolkit.Mvvm.ComponentModel;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Common.ViewModels.Entities;

public partial class AppViewModel : EditableEntityViewModelBase<App>
{
    [ObservableProperty] private string _color;
    [ObservableProperty] private string _company;
    [ObservableProperty] private string _description;
    [ObservableProperty] private AppIdentity _identity;
    [ObservableProperty] private bool _initialized;
    [ObservableProperty] private string _name;

    public AppViewModel(App entity, IDbContextFactory<QueryContext> contexts) : base(entity, contexts)
    {
        _initialized = entity.Initialized;
        _name = entity.Name;
        _description = entity.Description;
        _company = entity.Company;
        _color = entity.Color;
        _identity = entity.Identity;
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