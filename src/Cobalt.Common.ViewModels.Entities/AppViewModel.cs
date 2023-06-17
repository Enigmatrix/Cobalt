using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using CommunityToolkit.Mvvm.ComponentModel;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Common.ViewModels.Entities;

public partial class AppViewModel : EditableEntityViewModel<App>, IHasColor, IHasName, IHasIcon
{
    [ObservableProperty] private string _color = default!;
    [ObservableProperty] private string _company = default!;
    [ObservableProperty] private string _description = default!;
    [ObservableProperty] private AppIdentity _identity = default!;
    [ObservableProperty] private string _name = default!;

    private Stream? _icon;

    public AppViewModel(IEntityViewModelCache cache, IDbContextFactory<CobaltContext> conn) : base(cache, conn)
    {
    }

    public void Dispose()
    {
        throw new NotImplementedException();
    }

    public Stream Icon
    {
        get
        {
            if (_icon != null) return _icon;

            using var db = Conn.CreateDbContext();
            return _icon = db.AppIcon(Id);
        }
        // TODO this is just for testing
        set => _icon = value;
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