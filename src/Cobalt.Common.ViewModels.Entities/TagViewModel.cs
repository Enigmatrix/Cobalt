using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using CommunityToolkit.Mvvm.ComponentModel;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Common.ViewModels.Entities;

public partial class TagViewModel : EditableEntityViewModel<Tag>, IHasColor, IHasName
{
    [ObservableProperty] private string _color = default!;
    [ObservableProperty] private string _name = default!;

    public TagViewModel(IEntityViewModelCache cache, IDbContextFactory<CobaltContext> conn) : base(cache, conn)
    {
    }

    public override void InitializeWith(Tag tag)
    {
        Name = tag.Name;
        Color = tag.Color;

        base.InitializeWith(tag);
    }

    public override void Save()
    {
        using var ctx = Conn.CreateDbContext();
        ctx.Attach(Entity);
        Entity.Name = Name;
        Entity.Color = Color;
        ctx.SaveChanges();
    }
}