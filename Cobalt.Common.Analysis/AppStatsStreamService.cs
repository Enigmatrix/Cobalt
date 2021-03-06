﻿using System;
using System.Collections.Generic;
using System.Linq;
using System.Reactive.Linq;
using Cobalt.Common.Analysis.OutputTypes;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Repository;
using Cobalt.Common.Transmission;
using Cobalt.Common.Util;

namespace Cobalt.Common.Analysis
{
    public interface IAppStatsStreamService
    {
        IObservable<(App App, IObservable<Usage<TimeSpan>> Duration)> GetAppDurations(DateTime start,
            DateTime? end = null);

        IObservable<Usage<TimeSpan>> GetAppDuration(App app, DateTime start,
            DateTime end, bool listen = false);

        IObservable<(Tag Tag, IObservable<Usage<TimeSpan>> Duration)> GetTagDurations(DateTime start,
            DateTime? end = null);

        IObservable<Usage<AppUsage>> GetAppUsages(DateTime start, DateTime? end = null);

        IObservable<TimeSpan> GetAppUsageDuration(DateTime? start = null, DateTime? end = null);

        //IObservable<Usage<(App App, DateTime StartHour, TimeSpan Duration)>> HourlyChunks();
        IObservable<(App App, IObservable<Usage<TimeSpan>> Duration)> GetTaggedAppDurations(Tag tag, DateTime start,
            DateTime? end = null);

        IObservable<Usage<AppUsage>> GetAppUsages(Tag tag, DateTime start, DateTime? end = null);

        IObservable<Usage<(App App, DateTime StartHour, TimeSpan Duration)>> GetChunkedAppDurations(
            TimeSpan chunkDur, Func<DateTime, DateTime> startSelector, DateTime start, DateTime? end = null);

        IObservable<Usage<(App App, DateTime StartHour, TimeSpan Duration)>> GetChunkedAppDurations(
            Tag tag, TimeSpan chunkDur, Func<DateTime, DateTime> startSelector, DateTime start, DateTime? end = null);
    }

    public class AppStatsStreamService : StreamService, IAppStatsStreamService
    {
        public AppStatsStreamService(IDbRepository repo, ITransmissionClient client) : base(repo, client)
        {
        }

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

        public IObservable<Usage<TimeSpan>> GetAppDuration(App app, DateTime start,
            DateTime end, bool listen = false)
        {
            if (!listen)
                return Repository.GetAppDuration(app, start, end)
                    .Select(x => new Usage<TimeSpan>(x));
            return Repository.GetAppDuration(app, start, end)
                .Select(x => new Usage<TimeSpan>(x))
                .Concat(ReceivedAppStartEnds(app)
                    .Select(x => new Usage<TimeSpan>(
                        x.Value.Item2.Min(end) - x.Value.Item1.Max(start), x.JustStarted)))
                .Where(x => x.JustStarted || x.Value >= TimeSpan.Zero);
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

        public IObservable<TimeSpan> GetAppUsageDuration(DateTime? start = null, DateTime? end = null)
        {
            var tick = Repository.GetAppUsageTime(start, end);
            if (end == null)
            {
                var period = TimeSpan.FromMilliseconds(300);
                var timer = Observable.Timer(period, period);
                tick = tick.CombineLatest(timer,
                    (x, y) => x.Add(TimeSpan.FromMilliseconds(period.TotalMilliseconds * y)));
            }

            return tick;
        }

        public IObservable<(App App, IObservable<Usage<TimeSpan>> Duration)> GetTaggedAppDurations(Tag tag,
            DateTime start, DateTime? end = null)
        {
            if (end != null)
                return Repository.GetAppDurations(tag, start, end)
                    .Select(x => (x.App, Observable.Return(new Usage<TimeSpan>(x.Duration))));
            return Repository.GetAppDurations(tag, start)
                .Select(ToUsageAppDuration)
                .Concat(ReceivedAppDurations().Where(x => Repository.DoesAppHaveTag(x.App, tag)))
                .GroupBy(x => x.App, PathEquality)
                .Select(x => (x.Key, x.Select(y => y.Duration)));
        }

        public IObservable<Usage<AppUsage>> GetAppUsages(Tag tag, DateTime start, DateTime? end = null)
        {
            if (end != null)
                return Repository.GetAppUsages(tag, start, end).Select(ToUsageAppUsage);
            return Repository.GetAppUsages(tag, start).Select(ToUsageAppUsage)
                .Concat(ReceivedAppUsages().Where(x => Repository.DoesAppHaveTag(x.Value.App, tag)));
        }

        public IObservable<Usage<(App App, DateTime StartHour, TimeSpan Duration)>> GetChunkedAppDurations(
            TimeSpan chunkDur, Func<DateTime, DateTime> startSelector, DateTime start, DateTime? end)
        {
            return GetAppUsages(start, end)
                .SelectMany(u => SplitUsageIntoChunks(u, chunkDur, startSelector));
        }

        public IObservable<Usage<(App App, DateTime StartHour, TimeSpan Duration)>> GetChunkedAppDurations(Tag tag, TimeSpan chunkDur, Func<DateTime, DateTime> startSelector, DateTime start, DateTime? end)
        {
            return GetAppUsages(tag, start, end)
                .SelectMany(u => SplitUsageIntoChunks(u, chunkDur, startSelector));
        }

        private IEnumerable<Usage<(App App, DateTime StartHour, TimeSpan Duration)>> SplitUsageIntoChunks(
            Usage<AppUsage> usage, TimeSpan chunk, Func<DateTime, DateTime> startSelector)
        {
            var appUsage = usage.Value;
            var start = appUsage.StartTimestamp;
            var end = appUsage.EndTimestamp;
            while (start < end)
            {
                var startHr = startSelector(start);
                var endHr = Min(startHr + chunk, end);
                if (!(endHr < end) && usage.JustStarted)
                    yield return new Usage<(App, DateTime, TimeSpan)>(justStarted: true,
                        value: (appUsage.App, startHr, TimeSpan.Zero));
                else
                    yield return new Usage<(App, DateTime, TimeSpan)>((appUsage.App, startHr, endHr - start));
                start = endHr;
            }
        }

        public T Min<T>(T a, T b) where T : IComparable<T>
        {
            return a.CompareTo(b) < 0 ? a : b;
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

        private IObservable<(App App, Usage<(DateTime, DateTime)> StartEnds)> ReceivedAppStartEnds()
        {
            return ReceivedAppSwitches()
                .SelectMany(message => new[]
                {
                    //old app usage
                    (message.PreviousAppUsage.App,
                        new Usage<(DateTime, DateTime)>(
                            (message.PreviousAppUsage.StartTimestamp, message.PreviousAppUsage.EndTimestamp))),
                    //new app
                    (message.NewApp,
                        new Usage<(DateTime, DateTime)>(
                            (message.PreviousAppUsage.EndTimestamp, message.PreviousAppUsage.EndTimestamp), true))
                    //make sure the NewApp is not null
                }.Where(x => x.Item1 != null));
        }

        private IObservable<Usage<(DateTime, DateTime)>> ReceivedAppStartEnds(App app)
        {
            return ReceivedAppStartEnds().Where(x => x.App.Id == app.Id)
                .Select(x => x.StartEnds);
        }

        private IObservable<Usage<TimeSpan>> ReceivedAppDuration(App app)
        {
            //whats the equality here?
            return ReceivedAppDurations().Where(x => x.App.Id == app.Id)
                .Select(x => x.Duration);
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