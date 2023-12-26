using Cobalt.Common.Data;
using Cobalt.Common.ViewModels.Analysis;
using CommunityToolkit.Mvvm.ComponentModel;
using Microsoft.EntityFrameworkCore;
using ReactiveUI;

namespace Cobalt.Common.ViewModels;

/// <summary>
///     Base class for all ViewModels.
/// </summary>
/// <remarks>
///     We derive from CommunityToolkit's <see cref="ObservableObject" /> instead of ReactiveUI's
///     <see cref="ReactiveObject" />
///     otherwise the CommunityToolkit source generators will complain.
/// </remarks>
public abstract class ViewModelBase : ObservableObject
{
    protected ViewModelBase(IDbContextFactory<QueryContext> contexts)
    {
        Contexts = contexts;
    }

    /// <summary>
    ///     Factory to create <see cref="QueryContext" />
    /// </summary>
    protected IDbContextFactory<QueryContext> Contexts { get; }

    /// <summary>
    ///     Create a database query
    /// </summary>
    /// <typeparam name="T">Output type</typeparam>
    /// <param name="query">The inner database query</param>
    /// <param name="assumeRefreshIsCalled">
    ///     Assume that we do not produce anything in <see cref="Query{T}.Value" /> on
    ///     subscription
    /// </param>
    protected Query<T> Query<T>(Func<QueryContext, Task<T>> query, bool assumeRefreshIsCalled = true)
    {
        return new Query<T>(Contexts, query, assumeRefreshIsCalled);
    }

    /// <summary>
    ///     Create a database query
    /// </summary>
    /// <typeparam name="TArgs">Argument type</typeparam>
    /// <typeparam name="TOutput">Output type</typeparam>
    /// <param name="args">Arguments as an <see cref="IObservable{TArgs}" /> stream</param>
    /// <param name="query">The inner database query</param>
    /// <param name="assumeRefreshIsCalled">
    ///     Assume that we do not produce anything in <see cref="Query{T}.Value" /> on
    ///     subscription
    /// </param>
    protected Query<TArgs, TOutput> Query<TArgs, TOutput>(IObservable<TArgs> args,
        Func<QueryContext, TArgs, Task<TOutput>> query, bool assumeRefreshIsCalled = true)
    {
        return new Query<TArgs, TOutput>(Contexts, args, query, assumeRefreshIsCalled);
    }
}