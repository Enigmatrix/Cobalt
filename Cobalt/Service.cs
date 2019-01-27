using Cobalt.Common.Data.Repositories;
using Cobalt.Common.Transmission;
using System;
using System.Reactive.Linq;
using System.Runtime.CompilerServices;
using Cobalt.Common.Data.Entities;
using Cobalt.Common.Transmission.Messages;

namespace Cobalt
{
    public class Service
    {
        public Service(ITransmissionClient client, IDbRepository repo)
        {
            Client = client;
            Repository = repo;
        }

        public IObservable<(AppUsage Previous, Common.Data.Entities.App Active)> Switches()
        {
            return Repository.GetAppUsages(DateTime.Today)
                .Select(Wrap)
                .Concat(Client.Messages.OfType<AppSwitchMessage>().Select(x => 
                    (Repository.AppUsageById(x.AppUsageId), Repository.AppById(x.ActiveAppId))));
        }

        [MethodImpl(MethodImplOptions.AggressiveInlining)]
        private (AppUsage, Common.Data.Entities.App) Wrap(AppUsage x) => (x, null);

        private IDbRepository Repository { get; }

        private ITransmissionClient Client { get; }
    }
}
