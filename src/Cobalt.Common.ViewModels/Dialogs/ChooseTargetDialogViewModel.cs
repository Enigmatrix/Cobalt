using System.Reactive.Disposables;
using Cobalt.Common.Data;
using Cobalt.Common.ViewModels.Analysis;
using Cobalt.Common.ViewModels.Entities;
using CommunityToolkit.Mvvm.ComponentModel;
using Microsoft.EntityFrameworkCore;
using ReactiveUI;

namespace Cobalt.Common.ViewModels.Dialogs;

/// <summary>
///     ViewModel to choose a Target (App / Tag). Notably this does not inherit from DialogViewModelBase since it's not
///     activated through interactions.
/// </summary>
public partial class ChooseTargetDialogViewModel : ViewModelBase, IActivatableViewModel
{
    [ObservableProperty] private string _search = "";
    [ObservableProperty] private EntityViewModelBase? _target;

    public ChooseTargetDialogViewModel(IEntityViewModelCache entityCache, IDbContextFactory<QueryContext> contexts) :
        base(contexts)
    {
        var searches = this.WhenAnyValue(self => self.Search);

        Apps = Query(searches,
            async (context, search) => await context.SearchApps(search).ToListAsync(),
            entityCache.App, false);

        Tags = Query(searches,
            async (context, search) => await context.SearchTags(search).ToListAsync(),
            entityCache.Tag, false);

        this.WhenActivated((CompositeDisposable dis) =>
        {
            Target = null;
            Search = "";
        });
    }

    public Query<string, List<TagViewModel>> Tags { get; }
    public Query<string, List<AppViewModel>> Apps { get; }

    public ViewModelActivator Activator { get; } = new();
}