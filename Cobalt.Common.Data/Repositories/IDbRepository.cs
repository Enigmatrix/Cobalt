using System;
using Cobalt.Common.Data.Entities;

namespace Cobalt.Common.Data.Repositories
{
    public interface IDbRepository : IDisposable
    {
        //TODO remove this generic shit
        IObservable<T> Get<T>() where T : Entity;
        IObservable<TimeSpan> GetAppUsageTime(DateTime? start = null, DateTime? end = null);
        IObservable<AppUsage> GetAppUsages(DateTime? start = null, DateTime? end = null);
        IObservable<AppUsage> GetAppUsagesForTag(Tag tag, DateTime? start = null, DateTime? end = null);
        IObservable<AppUsage> GetAppUsagesForApp(App app, DateTime? start = null, DateTime? end = null);
        IObservable<(App App, TimeSpan Duration)> GetAppDurations(DateTime? start = null, DateTime? end = null);
        IObservable<(Tag Tag, TimeSpan Duration)> GetTagDurations(DateTime? start = null, DateTime? end = null);

        IObservable<(App App, TimeSpan Duration)> GetAppDurationsForTag(Tag tag, DateTime? start = null,
            DateTime? end = null);

        IObservable<TimeSpan> GetAppDuration(App app, DateTime? start = null, DateTime? end = null);

        void Insert(App obj);
        void Insert(Tag obj);
        void Insert(AppUsage obj);
        void Insert(Alert obj);
        void Insert(Reminder obj);

        void Update(App obj);
        void Update(Tag obj);
        void Update(AppUsage obj);
        void Update(Alert obj);
        void Update(Reminder obj);

        void Delete(App obj);
        void Delete(Tag obj);
        void Delete(AppUsage obj);
        void Delete(Alert obj);
        void Delete(Reminder obj);

        IObservable<Tag> GetTagsForApp(App app);
        IObservable<Reminder> GetRemindersForAlert(Alert alert);

        void AddTagToApp(Tag tag, App app);
        void RemoveTagFromApp(Tag tag, App app);

        bool AppIdByPath(App active);
        AppUsage AppUsageById(long id);
        App AppById(long id);
        Reminder ReminderById(long argEntityId);
    }
}