﻿using System;
using System.Collections.Generic;
using System.Text;
using Cobalt.Common.Data.Entities;

namespace Cobalt.Common.Data.Repositories
{
    public interface IDbRepository : IDisposable
    {
        IObservable<T> Get<T>();
        IObservable<TimeSpan> GetAppUsageTime(DateTime? start = null, DateTime? end = null);
        IObservable<AppUsage> GetAppUsages(DateTime? start = null, DateTime? end = null);
        IObservable<AppUsage> GetAppUsagesForTag(Tag tag, DateTime? start = null, DateTime? end = null);
        IObservable<AppUsage> GetAppUsagesForApp(App app, DateTime? start = null, DateTime? end = null);
        IObservable<(App App, TimeSpan Duration)> GetAppDurations(DateTime? start = null, DateTime? end = null);
        IObservable<(Tag Tag, TimeSpan Duration)> GetTagDurations(DateTime? start = null, DateTime? end = null);
        IObservable<(App App, TimeSpan Duration)>GetAppDurationsForTag(Tag tag, DateTime? start = null, DateTime? end = null);
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

        void AddTagToApp(Tag tag, App app);
        void AddReminderToAlert(Reminder rem, Alert alert);

        void RemoveTagFromApp(Tag tag, App app);
        void RemoveReminderFromAlert(Reminder rem, Alert alert);
    }
}
