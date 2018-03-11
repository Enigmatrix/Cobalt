using System;
using System.Collections.Generic;
using System.Reactive.Linq;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Repository;
using Cobalt.Common.Transmission;
using Cobalt.Common.Transmission.Messages;
using Cobalt.Common.Util;

namespace Cobalt.Common.Analysis
{
    public interface IEntityStreamService
    {
        IObservable<App> GetApps();
        IObservable<EntityChange<Alert>> GetAlertChanges();
    }

    public class EntityStreamService : StreamService, IEntityStreamService
    {
        private readonly IEqualityComparer<App> _pathEquality = new SelectorEqualityComparer<App, string>(x => x.Path);

        public EntityStreamService(IDbRepository repo, ITransmissionClient client) : base(repo, client)
        {
        }

        public IObservable<App> GetApps()
        {
            return Repository.GetApps().Concat(ReceivedApps())
                //guarentee unique
                .GroupBy(x => x, _pathEquality).Select(x => x.Key);
        }

        public IObservable<EntityChange<Alert>> GetAlertChanges()
        {
            return Repository.GetAlerts()
                .Select(x => new EntityChange<Alert>(x, ChangeType.Add))
                .Concat(ReceivedAlertChanges());
        }

        private IObservable<App> ReceivedApps()
        {
            return ReceivedAppSwitches()
                .SelectMany(e => new[] {e.NewApp, e.PreviousAppUsage.App});
        }

        private IObservable<EntityChange<Alert>> ReceivedAlertChanges()
        {
            return ReceivedMessages().OfType<EntityChangeMessage<Alert>>().Select(x => x.Change);
        }
    }
}