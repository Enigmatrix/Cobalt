using Cobalt.Common.Data;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Common.ViewModels.Pages;

/// <summary>
///     ViewModel for the Tags Page
/// </summary>
public class TagsPageViewModel : PageViewModelBase
{
    public TagsPageViewModel(IDbContextFactory<QueryContext> contexts) : base(contexts)
    {
    }

    public override string Name => "Tags";
}