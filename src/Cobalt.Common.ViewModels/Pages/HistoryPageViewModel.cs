using Cobalt.Common.Data;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Common.ViewModels.Pages;

/// <summary>
///     ViewModel for the History Page
/// </summary>
public class HistoryPageViewModel : PageViewModelBase
{
    public HistoryPageViewModel(IDbContextFactory<QueryContext> contexts) : base(contexts)
    {
    }

    public override string Name => "History";
}