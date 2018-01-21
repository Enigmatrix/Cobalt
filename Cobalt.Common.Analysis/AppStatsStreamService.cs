using System;
using System.Collections.Generic;
using System.Linq;
using System.Reactive.Linq;
using Cobalt.Common.Analysis.OutputTypes;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Repository;
using Cobalt.Common.Transmission;
using Cobalt.Common.Transmission.Messages;
using Cobalt.Common.Util;

namespace Cobalt.Common.Analysis
{
    public interface IAppStatsStreamService
    {
        IObservable<(App App, IObservable<Usage<TimeSpan>> Duration)> GetAppDurations(DateTime start,
            DateTime? end = null);

        IObservable<(Tag Tag, IObservable<Usage<TimeSpan>> Duration)> GetTagDurations(DateTime start,
            DateTime? end = null);

        IObservable<Usage<AppUsage>> GetAppUsages(DateTime start, DateTime? end = null);
        //IObservable<Usage<(App App, DateTime StartHour, TimeSpan Duration)>> HourlyChunks();
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

        public IObservable<(App App, IObservable<Usage<TimeSpan>> Duration)> GetAppDurations(DateTime start,
            DateTime? end = null)
        {
            //a bit untidy but meh
            if (end != null)
                return Repository.GetAppDurations(start, end)
                    .Select(x => (x.App, Observable.Return(new Usage<TimeSpan>(x.Duration))));
            return Repository.GetAppDurations(start)
                .Select(ToUsageAppDuration)
                .Concat(ReceivedAppDurations())
                .GroupBy(x => x.App, PathEquality)
                .Select(x => (x.Key, x.Select(y => y.Duration)));
        }

        public IObservable<(Tag Tag, IObservable<Usage<TimeSpan>> Duration)> GetTagDurations(DateTime start,
            DateTime? end = null)
        {
            if (end != null)
                return Repository.GetTagDurations(start, end)
                    .Select(x => (x.Tag, Observable.Return(new Usage<TimeSpan>(x.Duration))));
            return Repository.GetTagDurations(start)
                .Select(ToUsageTagDuration)
                .Concat(ReceivedTagDurations())
                .GroupBy(x => x.Tag, NameEquality)
                .Select(x => (x.Key, x.Select(y => y.Duration)));
        }

        public IObservable<Usage<AppUsage>> GetAppUsages(DateTime start, DateTime? end = null)
        {
            if (end != null)
                return Repository.GetAppUsages(start, end).Select(ToUsageAppUsage);
            return Repository.GetAppUsages(start).Select(ToUsageAppUsage)
                .Concat(ReceivedAppUsages());
        }

        private static (App App, Usage<TimeSpan> Duration) ToUsageAppDuration((App, TimeSpan) x)
        {
            return (x.Item1, new Usage<TimeSpan>(x.Item2));
        }

        private static (Tag Tag, Usage<TimeSpan> Duration) ToUsageTagDuration((Tag, TimeSpan) x)
        {
            return (x.Item1, new Usage<TimeSpan>(x.Item2));
        }

        private static Usage<AppUsage> ToUsageAppUsage(AppUsage au)
        {
            return new Usage<AppUsage>(au);
        }

        private IObservable<AppSwitchMessage> ReceivedAppSwitches()
        {
            return Observable.FromEventPattern<MessageReceivedArgs>(
                    e => Receiver.MessageReceived += e,
                    e => Receiver.MessageReceived -= e)
                .Where(e => e.EventArgs.Message is AppSwitchMessage)
                .Select(e => (AppSwitchMessage) e.EventArgs.Message);
        }

        private IObservable<(App App, Usage<TimeSpan> Duration)> ReceivedAppDurations()
        {
            return ReceivedAppSwitches()
                .SelectMany(message => new[]
                {
                    //old app usage
                    (message.PreviousAppUsage.App,
                    new Usage<TimeSpan>(message.PreviousAppUsage.Duration)),
                    //new app
                    (message.NewApp,
                    new Usage<TimeSpan>(justStarted: true))
                    //make sure the NewApp is not null
                }.Where(x => x.Item1 != null));
        }

        private IObservable<(Tag Tag, Usage<TimeSpan> Duration)> ReceivedTagDurations()
        {
            return ReceivedAppSwitches()
                .SelectMany(message =>
                    Repository.GetTags(message.PreviousAppUsage.App)
                        .Select(t => (t, new Usage<TimeSpan>(message.PreviousAppUsage.Duration))).Concat(
                            //tagdurs for previous
                            message.NewApp == null
                                ?
                                //empty
                                Observable.Empty<(Tag, Usage<TimeSpan>)>()
                                :
                                //tagdurs for new
                                Repository.GetTags(message.NewApp)
                                    .Select(t => (t, new Usage<TimeSpan>(justStarted: true)))));
        }

        private IObservable<Usage<AppUsage>> ReceivedAppUsages()
        {
            return ReceivedAppSwitches()
                .SelectMany(message => new[]
                {
                    new Usage<AppUsage>(message.PreviousAppUsage),
                    new Usage<AppUsage>(new AppUsage {App = message.NewApp}, true)
                });
        }
    }
}