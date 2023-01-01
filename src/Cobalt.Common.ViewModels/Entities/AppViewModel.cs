using System.Collections.ObjectModel;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Entities;
using CommunityToolkit.Mvvm.ComponentModel;

namespace Cobalt.Common.ViewModels.Entities;

public partial class AppViewModel : EditableEntityViewModel<App>
{
    [ObservableProperty] private string? _color;

    [ObservableProperty] private string _company = default!;

    [ObservableProperty] private string _description = default!;

    [ObservableProperty] private string _name = default!;

    private ObservableCollection<TagViewModel>? _tags;

    public AppViewModel(CobaltContext ctx, EntityViewModelCache cache) : base(ctx, cache)
    {
    }

    public ObservableCollection<TagViewModel> Tags =>
        _tags ??= new ObservableCollection<TagViewModel>(Inner.Tags.Select(tag => Cache.GetForTag(tag)));

    public override void InitializeEntity(App app)
    {
        Name = app.Name;
        Description = app.Description;
        Company = app.Company;
        Color = app.Color;

        Inner = app;
    }

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