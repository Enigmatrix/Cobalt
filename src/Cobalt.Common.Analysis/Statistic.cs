using System.Collections.ObjectModel;
using CommunityToolkit.Mvvm.ComponentModel;
using LiveChartsCore;

namespace Cobalt.Common.Analysis;

public class Statistic<T>
{
    public Statistic(Func<IQueryable<T>> query, Graph<T> graph)
    {
        Query = new Query<T>(query);
        Graph = graph;
    }

    public Query<T> Query { get; }
    public Graph<T> Graph { get; }
}

public class Query<T>
{
    private readonly Func<IQueryable<T>> _query;

    public Query(Func<IQueryable<T>> query)
    {
        _query = query;
    }

    public IQueryable<T> Execute()
    {
        return _query();
    }
}

public abstract partial class Graph<T> : ObservableObject
{
    [ObservableProperty] private ObservableCollection<ISeries> _series = new();
    public abstract void SetData(IQueryable<T> data);
}