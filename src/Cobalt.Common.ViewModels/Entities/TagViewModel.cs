using System.Collections.ObjectModel;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using CommunityToolkit.Mvvm.ComponentModel;

namespace Cobalt.Common.ViewModels.Entities;

public partial class TagViewModel : EditableEntityViewModel<Tag>
{
    private ObservableCollection<AppViewModel>? _apps;

    [ObservableProperty] private string? _color;

    [ObservableProperty] private string _name = default!;

    public TagViewModel(CobaltContext ctx, EntityViewModelCache cache) : base(ctx, cache)
    {
    }

    public ObservableCollection<AppViewModel> Apps =>
        _apps ??= new ObservableCollection<AppViewModel>(Inner.Apps.Select(app => Cache.GetForApp(app)));

    public override void InitializeEntity(Tag tag)
    {
        Name = tag.Name;
        Color = tag.Color;

        Inner = tag;
    }

    public override void SaveChanges()
    {
        Inner.Name = Name;
        Inner.Color = Color;
        Inner.Apps = Apps.Select(x => x.Inner).ToList();

        Context.SaveChanges();
    }
}