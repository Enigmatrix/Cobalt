using Cobalt.Common.Data;
using CommunityToolkit.Mvvm.Input;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Common.ViewModels.Pages;

public partial class ExperimentsPageViewModel : PageViewModelBase
{
    public ExperimentsPageViewModel(IDbContextFactory<QueryContext> contexts) : base(contexts)
    {
    }

    public override string Name => "Experiments";

    [RelayCommand]
    public void UpdateAllUsageEnds()
    {
        using var context = Contexts.CreateDbContext();
        context.UpdateAllUsageEnds(DateTime.Now);
    }
}