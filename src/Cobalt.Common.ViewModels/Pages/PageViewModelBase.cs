using Cobalt.Common.Data;
using Microsoft.EntityFrameworkCore;
using ReactiveUI;

namespace Cobalt.Common.ViewModels.Pages;

/// <summary>
///     Base class for all ViewModel for Pages
/// </summary>
public abstract class PageViewModelBase : ViewModelBase, IActivatableViewModel
{
    protected PageViewModelBase(IDbContextFactory<QueryContext> contexts)
    {
        Contexts = contexts;
    }

    /// <summary>
    ///     Name of this Page
    /// </summary>
    public abstract string Name { get; }

    /// <summary>
    ///     Factory to create <see cref="QueryContext" />
    /// </summary>
    protected IDbContextFactory<QueryContext> Contexts { get; }

    /// <summary>
    ///     Activator Context
    /// </summary>
    public ViewModelActivator Activator { get; } = new();
}