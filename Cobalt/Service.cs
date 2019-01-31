using System;
using System.Reactive.Disposables;
using System.Reactive.Linq;
using System.Runtime.CompilerServices;
using Cobalt.Common.Data.Entities;
using Cobalt.Common.Data.Repositories;
using Cobalt.Common.Transmission;
using Cobalt.Common.Transmission.Messages;
using DynamicData;

namespace Cobalt
{
    public class Service
    {
        public Service(ITransmissionClient client, IDbRepository repo)
        {
            Client = client;
            Repository = repo;
        }

        private IDbRepository Repository { get; }

        private ITransmissionClient Client { get; }

        public IObservable<IChangeSet<(AppUsage Previous, Common.Data.Entities.App Active)>> Switches()
        {
            return ObservableChangeSet.Create<(AppUsage, Common.Data.Entities.App)>(obs =>
            {
                obs.AddRange(Repository.GetAppUsages(DateTime.Today).Select(Wrap).ToEnumerable());
                var dis = Client.Messages<AppSwitchMessage>().Select(x =>
                        (Repository.AppUsageById(x.AppUsageId), Repository.AppById(x.ActiveAppId)))
                    .Subscribe(obs.Add);
                return new CompositeDisposable(dis);
            });
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        private (AppUsage, Common.Data.Entities.App) Wrap(AppUsage x)
        {
            return (x, null);
        }
    }
}