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
    public interface IAppStatsStreamService
    {
        IObservable<(App App, IObservable<TimeSpan> Duration)> GetAppDurations(DateTime start, DateTime? end = null);
    }

    public class AppStatsStreamService : IAppStatsStreamService
    {
        public AppStatsStreamService(IDbRepository repo, ITransmissionClient client)
        {
            Repository = repo;
            Receiver = client;
        }

        private IDbRepository Repository { get; }
        private ITransmissionClient Receiver { get; }

        private static IEqualityComparer<App> PathEquality { get; }
            = new SelectorEqualityComparer<App, string>(a => a.Path);

        public IObservable<(App App, IObservable<TimeSpan> Duration)> GetAppDurations(DateTime start,
            DateTime? end = null)
        {
            if (end != null)
                return Repository.GetAppDurations(start, end)
                    .Select(x => (x.App, Observable.Return(x.Duration)));
            return Repository.GetAppDurations(start)
                .Concat(ReceivedAppDurations())
                .GroupBy(x => x.App, PathEquality)
                .Select(x => (x.Key, x.Select(y => y.Duration)));
        }

        public IObservable<(App App, TimeSpan Duration)> ReceivedAppDurations()
        {
            return Observable.FromEventPattern<MessageReceivedArgs>(
                    e => Receiver.MessageReceived += e,
                    e => Receiver.MessageReceived -= e)
                .Where(e => e.EventArgs.Message is AppSwitchMessage)
                .Select(e => (AppSwitchMessage) e.EventArgs.Message)
                .SelectMany(message => new[]
                {
                    //old app usage
                    (message.PreviousAppUsage.App, message.PreviousAppUsage.Duration),
                    //new app
                    (message.NewApp, TimeSpan.Zero)
                });
        }
    }
}