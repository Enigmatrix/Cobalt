using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using CommunityToolkit.Mvvm.ComponentModel;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Common.ViewModels.Entities;

public partial class TagViewModel : EditableEntityViewModelBase<Tag>
{
    [ObservableProperty] private string _color;
    [ObservableProperty] private string _name;

    public TagViewModel(Tag entity, IDbContextFactory<QueryContext> contexts) : base(entity, contexts)
    {
        _name = entity.Name;
        _color = entity.Color;
    }

    public override void UpdateEntity()
    {
        Entity.Name = Name;
        Entity.Color = Color;
    }
}