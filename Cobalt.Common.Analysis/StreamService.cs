using Cobalt.Common.Data.Repository;
using Cobalt.Common.Transmission;
using Cobalt.Common.Transmission.Messages;
using System;
using System.Collections.Generic;
using System.Linq;
using System.Reactive.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Cobalt.Common.Analysis
{
    public class StreamService
    {
        public StreamService(IDbRepository repo, ITransmissionClient client)
        {
            Repository = repo;
            Receiver = client;
        }

        protected IDbRepository Repository { get; }
        protected ITransmissionClient Receiver { get; }


        public IObservable<MessageBase> ReceivedMessages(){
            return Observable.FromEventPattern<MessageReceivedArgs>(
                    e => Receiver.MessageReceived += e,
                    e => Receiver.MessageReceived -= e)
                    .Select(x => x.EventArgs.Message);
         }

        protected IObservable<AppSwitchMessage> ReceivedAppSwitches()
        {
            return ReceivedMessages().OfType<AppSwitchMessage>();
        }
    }
}
