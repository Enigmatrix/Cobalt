using Cobalt.Common.Data;
using Cobalt.Common.ViewModels.Entities;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Common.ViewModels.Dialogs;

public class AddAlertDialogViewModel : AlertDialogViewModelBase
{
    public AddAlertDialogViewModel(IEntityViewModelCache entityCache, IDbContextFactory<QueryContext> contexts) : base(
        entityCache, contexts)
    {
    }
}