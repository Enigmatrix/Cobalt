using Cobalt.Common.Data;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Common.ViewModels.Pages;

/// <summary>
///     ViewModel for the Home Page
/// </summary>
public class HomePageViewModel : PageViewModelBase
{
    public HomePageViewModel(IDbContextFactory<QueryContext> contexts) : base(contexts)
    {
    }

    public override string Name => "Home";
}