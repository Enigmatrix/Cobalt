using System.Collections.ObjectModel;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Models;
using CommunityToolkit.Mvvm.ComponentModel;

namespace Cobalt.Common.ViewModels.Models;

public partial class AppViewModel : EditableEntityViewModel<App>
{
    public AppViewModel(CobaltContext ctx, EntityViewModelCache cache) : base(ctx, cache)
    {
    }

    public override void InitializeEntity(App app)
    {
        Name = app.Name;
        Description = app.Description;
        Company = app.Company;
        Color = app.Color;

        Inner = app;
    }

    [ObservableProperty]
    private string? _color = default!;

    [ObservableProperty]
    private string _name = default!;

    [ObservableProperty]
    private string _description = default!;

    [ObservableProperty]
    private string _company = default!;

    private ObservableCollection<TagViewModel>? _tags = null;
    public ObservableCollection<TagViewModel> Tags => _tags ??= new(Inner.Tags.Select(tag => Cache.GetForTag(tag)));

    public override void SaveChanges()
    {
        Inner.Name = Name;
        Inner.Description = Description;
        Inner.Company = Company;
        Inner.Color = Color;
        Inner.Tags = Tags.Select(x => x.Inner).ToList();

        Context.SaveChanges();
    }
}
