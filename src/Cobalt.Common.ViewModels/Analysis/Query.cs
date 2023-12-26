using System.Reactive;
using System.Reactive.Linq;
using System.Reactive.Subjects;
using Cobalt.Common.Data;
using Microsoft.EntityFrameworkCore;

namespace Cobalt.Common.ViewModels.Analysis;

/// <summary>
///     Representation of a database query
/// </summary>
/// <typeparam name="T">Output type</typeparam>
/// <param name="Contexts">Context factory</param>
/// <param name="ActualQuery">Actual inner database query</param>
/// <param name="assumeRefreshIsCalled">Assume that we do not produce anything in <see cref="Value" /> on subscription</param>
public record Query<T>(IDbContextFactory<QueryContext> Contexts, Func<QueryContext, Task<T>> ActualQuery,
    // ReSharper disable once InconsistentNaming
    bool assumeRefreshIsCalled = true) : IRefreshable
{
    private readonly Subject<Unit> _refresh = new();

    /// <summary>
    ///     Query result value as an <see cref="IObservable{T}" />
    /// </summary>
    public IObservable<T> Value
    {
        get
        {
            IObservable<Unit> refreshes = _refresh;
            if (!assumeRefreshIsCalled)
                refreshes = refreshes.StartWith(Unit.Default);
            return refreshes.SelectMany(_ => Observable.FromAsync(ProduceValue));
        }
    }

    /// <inheritdoc />
    public Task Refresh()
    {
        _refresh.OnNext(Unit.Default);
        return Task.CompletedTask;
    }

    /// <summary>
    ///     Produces the value using the <see cref="ActualQuery" />
    /// </summary>
    private async Task<T> ProduceValue()
    {
        return await Task.Run(async () =>
        {
            var context = await Contexts.CreateDbContextAsync().ConfigureAwait(false);
            await using var _ = context.ConfigureAwait(false);
            return await ActualQuery(context).ConfigureAwait(false);
        });
    }
}