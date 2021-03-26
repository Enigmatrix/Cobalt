using System;
using System.Collections.ObjectModel;
using System.Reactive.Linq;
using DynamicData;
using Google.Protobuf.WellKnownTypes;
using ReactiveUI;
using ReactiveUI.Fody.Helpers;

namespace Cobalt.Common.ViewModels.Statistics
{
    public abstract class QueryBase<TR, TO> : ReactiveObject where TO: Options
    {
        [Reactive]
        public TO Options { get; init; }
    }

    public class Options : ReactiveObject
    {
    }

    public class Options<T> : Options
    {
        [Reactive]
        public T Value { get; set; }

        public Options(T val)
        {
            Value = val;
        }
    }

    public class QuerySingle<TR, TO> : QueryBase<TR, TO> where TO: Options
    {
        [ObservableAsProperty]
        public TR Result { get; init; }

        public QuerySingle(TO opts, Func<TO, TR> get)
        {
            this.WhenAnyValue(x => x.Options)
                .Select(get)
                .ToPropertyEx(this, x => x.Result);

            Options = opts; // should automatically set the result as well
        }
    }

    public class Query<TR, TO> : QueryBase<TR, TO>, IDisposable where TO: Options
    {
        private readonly IDisposable _resultsSub;
        private readonly ReadOnlyObservableCollection<TR> _results;
        public ReadOnlyObservableCollection<TR> Results => _results;

        public Query(TO opts, Func<TO, IObservable<TR>> get)
        {
            _resultsSub = ObservableChangeSet.Create<TR>(list =>
            {
                return this.WhenAnyValue(x => x.Options)
                    .Select(get)
                    .Do(_ => list.Clear())
                    .SelectMany(results => results)
                    .Subscribe(list.Add);
            })
            .Bind(out _results)
            .Subscribe();

            Options = opts; // should automatically set the result as well
        }

        public void Dispose()
        {
            _resultsSub.Dispose();
        }
    }
}
