using System;
using System.Collections.Generic;
using System.Linq;
using System.Reactive.Linq;
using System.Text;
using System.Threading.Tasks;
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
    }
    public class EntityStreamService : IEntityStreamService
    {
        public EntityStreamService(IDbRepository repo, ITransmissionClient client)
        {
            Repository = repo;
            Receiver = client;
        }

        private IDbRepository Repository { get; }
        private ITransmissionClient Receiver { get; }

        private readonly IEqualityComparer<App> _pathEquality = new SelectorEqualityComparer<App, string>(x => x.Path);

        public IObservable<App> GetApps()
        {
            return Repository.GetApps().Concat(ReceivedApps())
                //guarentee unique
                .GroupBy(x => x, _pathEquality).Select(x => x.Key);
        }

        private IObservable<App> ReceivedApps()
        {
            return Observable.FromEventPattern<MessageReceivedArgs>(
                    e => Receiver.MessageReceived += e,
                    e => Receiver.MessageReceived -= e)
                .Select(e => e.EventArgs.Message as AppSwitchMessage)
                .Where(e => e!=null)
                .SelectMany(e => new []{e.NewApp, e.PreviousAppUsage.App});
        }
    }
}
