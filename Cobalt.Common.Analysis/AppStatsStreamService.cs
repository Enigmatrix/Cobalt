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
        IObservable<(App App, IObservable<TimeSpan?> Duration)> GetAppDurations(DateTime start, DateTime? end = null);
        IObservable<(Tag Tag, IObservable<TimeSpan?> Duration)> GetTagDurations(DateTime start, DateTime? end = null);
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

        private static IEqualityComparer<Tag> NameEquality { get; }
            = new SelectorEqualityComparer<Tag, string>(a => a.Name);

        public IObservable<(App App, IObservable<TimeSpan?> Duration)> GetAppDurations(DateTime start,
            DateTime? end = null)
        {
            //a bit untidy but meh
            if (end != null)
                return Repository.GetAppDurations(start, end)
                    .Select(x => (x.App, Observable.Return<TimeSpan?>(x.Duration)));
            return Repository.GetAppDurations(start)
                .Select(ToNullableAppDuration)
                .Concat(ReceivedAppDurations())
                .GroupBy(x => x.App, PathEquality)
                .Select(x => (x.Key, x.Select(y => y.Duration)));
        }

        public IObservable<(Tag Tag, IObservable<TimeSpan?> Duration)> GetTagDurations(DateTime start,
            DateTime? end = null)
        {
            if (end != null)
                return Repository.GetTagDurations(start, end)
                    .Select(x => (x.Tag, Observable.Return<TimeSpan?>(x.Duration)));
            return Repository.GetTagDurations(start)
                .Select(ToNullableTagDuration)
                .Concat(ReceivedTagDurations())
                .GroupBy(x => x.Tag, NameEquality)
                .Select(x => (x.Key, x.Select(y => y.Duration)));
        }

        private static (App App, TimeSpan? Duration) ToNullableAppDuration((App, TimeSpan) x)
        {
            return (x.Item1, x.Item2);
        }

        private static (Tag Tag, TimeSpan? Duration) ToNullableTagDuration((Tag, TimeSpan) x)
        {
            return (x.Item1, x.Item2);
        }

        private IObservable<(App App, TimeSpan? Duration)> ReceivedAppDurations()
        {
            return Observable.FromEventPattern<MessageReceivedArgs>(
                    e => Receiver.MessageReceived += e,
                    e => Receiver.MessageReceived -= e)
                .Where(e => e.EventArgs.Message is AppSwitchMessage)
                .Select(e => (AppSwitchMessage) e.EventArgs.Message)
                .SelectMany(message => new[]
                {
                    //old app usage
                    (message.PreviousAppUsage.App, (TimeSpan?) message.PreviousAppUsage.Duration),
                    //new app
                    (message.NewApp, null)
                });
        }

        private IObservable<(Tag Tag, TimeSpan? Duration)> ReceivedTagDurations()
        {
            return Observable.FromEventPattern<MessageReceivedArgs>(
                    e => Receiver.MessageReceived += e,
                    e => Receiver.MessageReceived -= e)
                .Where(e => e.EventArgs.Message is AppSwitchMessage)
                .Select(e => (AppSwitchMessage) e.EventArgs.Message)
                .SelectMany(message =>
                    Repository.GetTags(message.PreviousAppUsage.App)
                        .Select(t => (t, (TimeSpan?) message.PreviousAppUsage.Duration))
                        .Concat(
                            Repository.GetTags(message.NewApp)
                                .Select(t => (t, (TimeSpan?) null))));
        }
    }
}