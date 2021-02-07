using System;
using System.Reactive.Linq;
using Cobalt.Common.Communication;
using Cobalt.Common.Data;
using Cobalt.Common.ViewModels.Entities;
using Splat;

namespace Cobalt.Common.ViewModels.Statistics
{
    public interface IQueries
    {
        IObservable<AppDurationViewModel> AppDurations(DateTimeRange range, IdleOptions opts);
    }

    public class Queries : IQueries, IEnableLogger
    {
        private readonly IClient _client;
        private readonly IDatabase _db;
        private readonly IEntityManager _mgr;

        public Queries(IClient client, IDatabase db, IEntityManager mgr)
        {
            _client = client;
            _db = db;
            _mgr = mgr;
        }

        public IObservable<AppDurationViewModel> AppDurations(DateTimeRange range, IdleOptions opts)
        {
            var fromDb = _db.AppDurations(range)
                .Select(dur => dur.Map(app => _mgr.GetApp(app)));

            if (!range.End.IsUnbounded)
                return fromDb
                    .Select(dur => new AppDurationViewModel(dur.Inner, Observable.Return(dur.Duration)));

            var fromClient = _client.Usages().Select(x =>
            {
                var usage = _db.FindUsage(x.PrevUsageId);
                return new Data.WithDuration<AppViewModel>(_mgr.GetApp(x.PrevAppId), usage.End - usage.Start);
            });

            var ret = fromDb.Concat(fromClient).GroupBy(
                    x => x.Inner,
                    x => x.Duration)
                .Select(ad => new AppDurationViewModel(ad.Key, ad));
            return ret;
        }
    }
}