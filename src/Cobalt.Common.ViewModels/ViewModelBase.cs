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
public abstract class ViewModelBase : ReactiveObservableObject
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
    /// <typeparam name="TOutput">Output type</typeparam>
    /// <param name="query">The inner database query</param>
    /// <param name="assumeRefreshIsCalled">
    ///     Assume that we do not produce anything in <see cref="Query{T}.Value" /> on
    ///     subscription
    /// </param>
    protected Query<TOutput> Query<TOutput>(Func<QueryContext, Task<TOutput>> query, bool assumeRefreshIsCalled = true)
    {
        return new Query<TOutput>(Contexts, query, assumeRefreshIsCalled);
    }

    /// <summary>
    ///     Create a database query that returns a list and transform it post-materialization
    /// </summary>
    /// <typeparam name="TDbOutput">Database query list output type</typeparam>
    /// <typeparam name="TOutput">Transformed list output type</typeparam>
    /// <param name="query">The inner database query</param>
    /// <param name="transform">Post-materialization transform</param>
    /// <param name="assumeRefreshIsCalled">
    ///     Assume that we do not produce anything in <see cref="Query{T}.Value" /> on
    ///     subscription
    /// </param>
    protected Query<List<TOutput>> Query<TDbOutput, TOutput>(Func<QueryContext, Task<List<TDbOutput>>> query,
        Func<TDbOutput, TOutput> transform,
        bool assumeRefreshIsCalled = true)
    {
        return new Query<List<TOutput>>(Contexts, async ctx => (await query(ctx).ConfigureAwait(false)).Select(transform).ToList(),
            assumeRefreshIsCalled);
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

    /// <summary>
    ///     Create a database query that returns a list and transform it post-materialization
    /// </summary>
    /// <typeparam name="TArgs">Argument type</typeparam>
    /// <typeparam name="TDbOutput">Database query list output type</typeparam>
    /// <typeparam name="TOutput">Transformed list output type</typeparam>
    /// <param name="args">Arguments as an <see cref="IObservable{TArgs}" /> stream</param>
    /// <param name="query">The inner database query</param>
    /// <param name="transform">Post-materialization transform</param>
    /// <param name="assumeRefreshIsCalled">
    ///     Assume that we do not produce anything in <see cref="Query{T}.Value" /> on
    ///     subscription
    /// </param>
    protected Query<TArgs, List<TOutput>> Query<TArgs, TDbOutput, TOutput>(IObservable<TArgs> args,
        Func<QueryContext, TArgs, Task<List<TDbOutput>>> query, Func<TDbOutput, TOutput> transform,
        bool assumeRefreshIsCalled = true)
    {
        return new Query<TArgs, List<TOutput>>(Contexts, args,
            async (ctx, arg) => (await query(ctx, arg).ConfigureAwait(false)).Select(transform).ToList(),
            assumeRefreshIsCalled);
    }
}