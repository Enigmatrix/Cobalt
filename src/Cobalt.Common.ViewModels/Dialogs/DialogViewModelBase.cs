using Cobalt.Common.Data;
using Microsoft.EntityFrameworkCore;
using ReactiveUI;

namespace Cobalt.Common.ViewModels.Dialogs;

/// <summary>
///     Base class for all ViewModel for Dialogs
/// </summary>
public abstract class DialogViewModelBase<TResult> : ViewModelBase, IActivatableViewModel
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
    ///     Title of this Dialog
    /// </summary>
    public abstract string Title { get; }

    /// <summary>
    ///     PrimaryButtonText of this Dialog
    /// </summary>
    public string PrimaryButtonText => "Submit";

    /// <summary>
    ///     CloseButtonText of this Dialog
    /// </summary>
    public string CloseButtonText => "Cancel";

    /// <summary>
    ///     Activator Context
    /// </summary>
    public ViewModelActivator Activator { get; } = new();

    /// <summary>
    ///     Produces the result of this Dialog
    /// </summary>
    public abstract Task<TResult> GetResultAsync();
}