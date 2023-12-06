using Cobalt.Common.Data;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Common.ViewModels.Pages;

/// <summary>
///     ViewModel for the Alerts Page
/// </summary>
public class AlertsPageViewModel : PageViewModelBase
{
    public AlertsPageViewModel(IDbContextFactory<QueryContext> contexts) : base(contexts)
    {
    }

    public override string Name => "Alerts";
}