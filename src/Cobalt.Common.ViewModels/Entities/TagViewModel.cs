using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using CommunityToolkit.Mvvm.ComponentModel;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Common.ViewModels.Entities;

/// <summary>
///     ViewModel for the Tag Entity
/// </summary>
public partial class TagViewModel : EditableEntityViewModelBase<Tag>
{
    private const int AppsSubsetSize = 3;
    [ObservableProperty] private List<AppViewModel> _apps;
    [ObservableProperty] private int _appsNotInSubset;
    [ObservableProperty] private List<AppViewModel> _appsSubset;
    [ObservableProperty] private string _color;
    [ObservableProperty] private string _name;

    public TagViewModel(Tag entity, IEntityViewModelCache entityCache, IDbContextFactory<QueryContext> contexts) : base(
        entity, entityCache, contexts)
    {
        _name = entity.Name;
        _color = entity.Color;
        // TODO or maybe this should be fetched on demand?
        _apps = entity.Apps.Select(EntityCache.App).ToList();
        _appsSubset = entity.Apps.Select(EntityCache.App).Take(AppsSubsetSize).ToList();
        _appsNotInSubset = Math.Max(entity.Apps.Count - AppsSubsetSize, 0);
    }

    public override void UpdateEntity()
    {
        Entity.Name = Name;
        Entity.Color = Color;
    }
}