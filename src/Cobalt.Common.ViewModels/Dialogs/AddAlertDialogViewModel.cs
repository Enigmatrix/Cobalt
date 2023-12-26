using System.Reactive.Disposables;
using Cobalt.Common.Data;
using Cobalt.Common.ViewModels.Entities;
using Microsoft.EntityFrameworkCore;
using ReactiveUI;

namespace Cobalt.Common.ViewModels.Dialogs;

/// <summary>
///     Dialog ViewModel to Add Alert
/// </summary>
public class AddAlertDialogViewModel : DialogViewModelBase<AlertViewModel>
{
    private readonly IEntityViewModelCache _entityCache;

    public AddAlertDialogViewModel(IEntityViewModelCache entityCache, IDbContextFactory<QueryContext> contexts) :
        base(contexts)
    {
        _entityCache = entityCache;
        this.WhenActivated((CompositeDisposable dis) => { });
    }

    public override string Title => "Add Alert";

    public override async Task<AlertViewModel> GetResultAsync()
    {
        throw new NotImplementedException();
    }
}