using System;
using System.Reactive.Linq;
using Cobalt.Common.Data.Repository;
using Cobalt.Common.Transmission;
using Cobalt.Common.Transmission.Messages;

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


        public IObservable<MessageBase> ReceivedMessages()
        {
            return Observable.FromEventPattern<MessageReceivedArgs>(
                    e => Receiver.MessageReceived += e,
                    e => Receiver.MessageReceived -= e)
                .Select(x => x.EventArgs.Message);
        }

        protected IObservable<AppSwitchMessage> ReceivedAppSwitches()
        {
            return ReceivedMessages().OfType<AppSwitchMessage>()
                .Do(x =>
                {
                    x.NewApp.Icon = Repository.GetAppIcon(x.NewApp);
                    x.NewApp.Tags = Repository.GetTags(x.NewApp);
                    x.PreviousAppUsage.App.Icon = Repository.GetAppIcon(x.PreviousAppUsage.App);
                    x.PreviousAppUsage.App.Tags = Repository.GetTags(x.PreviousAppUsage.App);
                });
        }
    }
}