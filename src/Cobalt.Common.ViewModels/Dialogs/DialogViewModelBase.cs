using Cobalt.Common.Data;
using Microsoft.EntityFrameworkCore;
using ReactiveUI;

namespace Cobalt.Common.ViewModels.Dialogs;

/// <summary>
///     Base class for all ViewModel for Dialogs
/// </summary>
public abstract class DialogViewModelBase : ViewModelBase, IActivatableViewModel
{
    protected DialogViewModelBase(IDbContextFactory<QueryContext> contexts)
    {
        Contexts = contexts;
    }

    /// <summary>
    ///     Factory to create <see cref="QueryContext" />
    /// </summary>
    protected IDbContextFactory<QueryContext> Contexts { get; }

    /// <summary>
    ///     Activator Context
    /// </summary>
    public ViewModelActivator Activator { get; } = new();
}