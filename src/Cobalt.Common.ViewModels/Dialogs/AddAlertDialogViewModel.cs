using System.Reactive.Disposables;
using Cobalt.Common.Data;
using Microsoft.EntityFrameworkCore;
using ReactiveUI;

namespace Cobalt.Common.ViewModels.Dialogs;

public class AddAlertDialogViewModel : DialogViewModelBase
{
    public AddAlertDialogViewModel(IDbContextFactory<QueryContext> contexts) : base(contexts)
    {
        this.WhenActivated((CompositeDisposable dis) => { });
    }
}