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

        //remove/add tag
        void AddTagToApp(Tag tag, App app);
        void RemoveTagFromApp(Tag tag, App app);

        //get
        IObservable<App> GetApps();
        IObservable<Tag> GetTags();
        IObservable<App> GetAppsWithTag(Tag tag);

        IObservable<AppUsage> GetAppUsages(DateTime? start = null, DateTime? end = null);
        IObservable<AppUsage> GetAppUsagesForApp(App app, DateTime? start = null, DateTime? end = null);

        IObservable<(App App, TimeSpan Duration)> GetAppDurations(DateTime? start = null, DateTime? end = null);
        IObservable<(Tag Tag, TimeSpan Duration)> GetTagDurations(DateTime? start = null, DateTime? end = null);

        //update
        void UpdateApp(App app);
        void UpdateTag(Tag tag);

        //hydrate
        IObservable<App> HydrateWithTags(IObservable<App> apps);
        IObservable<AppUsage> HydrateWithApps(IObservable<AppUsage> appUsages);

        //find
        long? FindAppIdByPath(string appPath);
    }
}