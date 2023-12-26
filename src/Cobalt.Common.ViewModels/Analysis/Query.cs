using System.Reactive;
using System.Reactive.Linq;
using System.Reactive.Subjects;
using Cobalt.Common.Data;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Common.ViewModels.Analysis;

/// <summary>
///     Representation of a database query
/// </summary>
/// <typeparam name="TOutput">Output type</typeparam>
/// <param name="Contexts">Context factory</param>
/// <param name="InnerQueryWithoutArgs">Actual inner database query</param>
/// <param name="assumeRefreshIsCalled">Assume that we do not produce anything in <see cref="Value" /> on subscription</param>
public record Query<TOutput>(IDbContextFactory<QueryContext> Contexts,
    Func<QueryContext, Task<TOutput>> InnerQueryWithoutArgs,
    bool assumeRefreshIsCalled = true) : Query<Unit, TOutput>(Contexts, Observable.Return(Unit.Default),
    (context, _) => InnerQueryWithoutArgs(context), assumeRefreshIsCalled)
{
}

/// <summary>
///     Representation of a database query
/// </summary>
/// <typeparam name="TOutput">Output type</typeparam>
/// <typeparam name="TArg">Argument type</typeparam>
/// <param name="Contexts">Context factory</param>
/// <param name="Args">Arguments as an <see cref="IObservable{TArgs}" />></param>
/// <param name="InnerQuery">Actual inner database query</param>
/// <param name="assumeRefreshIsCalled">Assume that we do not produce anything in <see cref="Value" /> on subscription</param>
public record Query<TArg, TOutput>(
    IDbContextFactory<QueryContext> Contexts,
    IObservable<TArg> Args,
    Func<QueryContext, TArg, Task<TOutput>> InnerQuery,
    // ReSharper disable once InconsistentNaming
    bool assumeRefreshIsCalled = true) : IRefreshable
{
    private readonly Subject<Unit> _refresh = new();

    /// <summary>
    ///     Query result value as an <see cref="IObservable{T}" />
    /// </summary>
    public IObservable<TOutput> Value
    {
        get
        {
            IObservable<Unit> refreshes = _refresh;
            if (!assumeRefreshIsCalled)
                refreshes = refreshes.StartWith(Unit.Default);
            return refreshes.WithLatestFrom(Args)
                .SelectMany(tup => Observable.FromAsync(() => ProduceValue(tup.Second)));
        }
    }

    /// <inheritdoc />
    public Task Refresh()
    {
        _refresh.OnNext(Unit.Default);
        return Task.CompletedTask;
    }

    /// <summary>
    ///     Produces the value using the <see cref="InnerQuery" />
    /// </summary>
    /// <param name="arg">Argument to the inner database query</param>
    protected async Task<TOutput> ProduceValue(TArg arg)
    {
        return await Task.Run(async () =>
        {
            var context = await Contexts.CreateDbContextAsync().ConfigureAwait(false);
            await using var _ = context.ConfigureAwait(false);
            return await InnerQuery(context, arg).ConfigureAwait(false);
        });
    }
}