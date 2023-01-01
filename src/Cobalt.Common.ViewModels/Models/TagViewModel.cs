using System.Collections.ObjectModel;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Models;
using CommunityToolkit.Mvvm.ComponentModel;

namespace Cobalt.Common.ViewModels.Models;

public partial class TagViewModel : EditableEntityViewModel<Tag>
{
    public TagViewModel(CobaltContext ctx, EntityViewModelCache cache) : base(ctx, cache)
    {
    }

    public override void InitializeEntity(Tag tag)
    {
        Name = tag.Name;
        Color = tag.Color;

        Inner = tag;
    }

    [ObservableProperty]
    private string? _color = default!;

    [ObservableProperty]
    private string _name = default!;

    private ObservableCollection<AppViewModel>? _apps = null;
    public ObservableCollection<AppViewModel> Apps => _apps ??= new(Inner.Apps.Select(app => Cache.GetForApp(app)));

    public override void SaveChanges()
    {
        Inner.Name = Name;
        Inner.Color = Color;
        Inner.Apps = Apps.Select(x => x.Inner).ToList();

        Context.SaveChanges();
    }
}
