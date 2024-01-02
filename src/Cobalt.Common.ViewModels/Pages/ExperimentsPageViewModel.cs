using Cobalt.Common.Data;
using CommunityToolkit.Mvvm.Input;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Common.ViewModels.Pages;

/// <summary>
///     ViewModel for the Experiments Page
/// </summary>
/// <remarks>
///     This class does not exist during production.
/// </remarks>
public partial class ExperimentsPageViewModel : PageViewModelBase
{
    public ExperimentsPageViewModel(IDbContextFactory<QueryContext> contexts) : base(contexts)
    {
    }

    public override string Name => "Experiments";

    [RelayCommand]
    public async Task UpdateAllUsageEndsAsync()
    {
        await using var context = await Contexts.CreateDbContextAsync();
        await context.UpdateAllUsageEndsAsync(DateTime.Now);
    }
}