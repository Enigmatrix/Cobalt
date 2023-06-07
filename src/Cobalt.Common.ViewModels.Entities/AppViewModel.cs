using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using CommunityToolkit.Mvvm.ComponentModel;

namespace Cobalt.Common.ViewModels.Entities;

public partial class AppViewModel : EditableEntityViewModel<App>
{
    [ObservableProperty] private string _name = default!;
    [ObservableProperty] private string _description = default!;
    [ObservableProperty] private string _company = default!;
    [ObservableProperty] private string _color = default!;
    [ObservableProperty] private AppIdentity _identity = default!;

    public AppViewModel(IEntityViewModelCache cache, CobaltContext context) : base(cache, context)
    {
    }

    public override void InitializeWith(App app)
    {
        Name = app.Name;
        Description = app.Description;
        Company = app.Company;
        Color = app.Color;
        Identity = app.Identity;

        base.InitializeWith(app);
    }

    public override void Save()
    {
        using var ctx = Conn.CreateDbContext();
        ctx.Attach(Entity);
        Entity.Name = Name;
        Entity.Description = Description;
        Entity.Company = Company;
        Entity.Color = Color;
        Entity.Identity = Identity;
        ctx.SaveChanges();
    }
}
