using System;
using System.Data.Common;

namespace Cobalt.Common.Data.Repository
{
    public interface IDbRepository
    {
        //connection
        DbConnection Connection { get; }

        //add
        void AddAppUsage(AppUsage appUsage);

        void AddApp(App app);
        void AddTag(Tag tag);
        void AddTagToApp(Tag tag, App app);
        void AddInteraction(Interaction interaction);
        void AddAlert(Alert alert);

        //remove
        void RemoveTagFromApp(Tag tag, App app);
        void RemoveAlert(Alert alert);



        //get
        IObservable<App> GetApps();

        IObservable<Tag> GetTags();
        IObservable<Tag> GetTags(App newApp);
        IObservable<App> GetAppsWithTag(Tag tag);

        IObservable<TimeSpan> GetAppUsageTime(DateTime? start = null, DateTime? end = null);

        IObservable<AppUsage> GetAppUsages(DateTime? start = null, DateTime? end = null);
        IObservable<AppUsage> GetAppUsagesForApp(App app, DateTime? start = null, DateTime? end = null);

        IObservable<(App App, TimeSpan Duration)> GetAppDurations(DateTime? start = null, DateTime? end = null);
        IObservable<TimeSpan> GetAppDuration(App app, DateTime? start = null, DateTime? end = null);
        IObservable<(Tag Tag, TimeSpan Duration)> GetTagDurations(DateTime? start = null, DateTime? end = null);

        IObservable<(DateTime Start, DateTime End)> GetIdleDurations(TimeSpan minDuration, DateTime? start = null,
            DateTime? end = null);
        IObservable<Alert> GetAlerts();

        //update
        void UpdateApp(App app);
        void UpdateTag(Tag tag);
        void UpdateAlert(Alert alert);

        //find
        long? FindAppIdByPath(string appPath);
    }
}