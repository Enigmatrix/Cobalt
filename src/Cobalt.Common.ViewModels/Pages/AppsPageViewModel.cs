using Cobalt.Common.Data;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Common.ViewModels.Pages;

/// <summary>
///     ViewModel for the Apps Page
/// </summary>
public class AppsPageViewModel : PageViewModelBase
{
    public AppsPageViewModel(IDbContextFactory<QueryContext> contexts) : base(contexts)
    {
    }

    public override string Name => "Apps";
}