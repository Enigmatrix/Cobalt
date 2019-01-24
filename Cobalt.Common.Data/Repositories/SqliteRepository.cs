using System;
using System.Data.SQLite;
using System.Reactive.Linq;
using Cobalt.Common.Data.Entities;
using Cobalt.Common.Data.Migrations;
using Dapper;

namespace Cobalt.Common.Data.Repositories
{
    public class SqliteRepository : IDbRepository
    {
        public SqliteRepository(SQLiteConnection connection, IDbMigrator migrator)
        {
            migrator.Run();
            Connection = connection;
        }

        private SQLiteConnection Connection { get; }

        public void Dispose()
        {
            Connection.Dispose();
        }

        public IObservable<T> Get<T>() where T : Entity
        {
            return Connection.Query<T>($"select * from {typeof(T).Name}").ToObservable();
        }

        public IObservable<TimeSpan> GetAppUsageTime(DateTime? start = null, DateTime? end = null)
        {
            throw new NotImplementedException();
        }

        public IObservable<AppUsage> GetAppUsages(DateTime? start = null, DateTime? end = null)
        {
            throw new NotImplementedException();
        }

        public IObservable<AppUsage> GetAppUsagesForTag(Tag tag, DateTime? start = null, DateTime? end = null)
        {
            throw new NotImplementedException();
        }

        public IObservable<AppUsage> GetAppUsagesForApp(App app, DateTime? start = null, DateTime? end = null)
        {
            throw new NotImplementedException();
        }

        public IObservable<(App App, TimeSpan Duration)> GetAppDurations(DateTime? start = null, DateTime? end = null)
        {
            throw new NotImplementedException();
        }

        public IObservable<(Tag Tag, TimeSpan Duration)> GetTagDurations(DateTime? start = null, DateTime? end = null)
        {
            throw new NotImplementedException();
        }

        public IObservable<(App App, TimeSpan Duration)> GetAppDurationsForTag(Tag tag, DateTime? start = null,
            DateTime? end = null)
        {
            throw new NotImplementedException();
        }

        public IObservable<TimeSpan> GetAppDuration(App app, DateTime? start = null, DateTime? end = null)
        {
            throw new NotImplementedException();
        }

        public void Insert(App obj)
        {
            obj.Id = Insert("insert into App (Name, Color, Path, Icon)" +
                            "values (@Name, @Color, @Path, @Icon)", new
            {
                obj.Name,
                obj.Color,
                obj.Path,
                Icon = obj.Icon.Value
            });
        }

        public void Insert(Tag obj)
        {
            obj.Id = Insert("insert into Tag (Name, BackgroundColor, ForegroundColor)" +
                            "values (@Name, @BackgroundColor, @ForegroundColor)", new
            {
                obj.Name,
                obj.BackgroundColor,
                obj.ForegroundColor
            });
        }

        public void Insert(AppUsage obj)
        {
            obj.Id = Insert("insert into AppUsage (AppId, Start, End, StartReason, EndReason)" +
                            "values (@AppId, @Start, @End, @StartReason, @EndReason)", new
            {
                AppId = obj.App.Id,
                Start = obj.Start.Ticks,
                End = obj.End.Ticks,
                obj.StartReason,
                obj.EndReason
            });
        }

        public void Insert(Alert obj)
        {
            obj.Id = Insert("insert into Alert (AppId, TagId, MaxDuration, Enabled, ActionType, ActionParam," +
                            "TimeRangeType, TimeRangeParam1, TimeRangeParam2)" +
                            "values (@AppId, @TagId, @MaxDuration, @Enabled, @ActionType, @ActionParam," +
                            "@TimeRangeType, @TimeRangeParam1, @TimeRangeParam2)", AlertObj(obj));
        }

        public void Insert(Reminder obj)
        {
            obj.Id = Insert("insert into Reminder (AlertId, Offset, ActionType, ActionParam)" +
                            "values (@AlertId, @Offset, @ActionType, @ActionParam)", ReminderObj(obj));
        }

        public void Update(App obj)
        {
            throw new NotImplementedException();
        }

        public void Update(Tag obj)
        {
            throw new NotImplementedException();
        }

        public void Update(AppUsage obj)
        {
            throw new NotImplementedException();
        }

        public void Update(Alert obj)
        {
            throw new NotImplementedException();
        }

        public void Update(Reminder obj)
        {
            throw new NotImplementedException();
        }

        public void Delete(App obj)
        {
            throw new NotImplementedException();
        }

        public void Delete(Tag obj)
        {
            throw new NotImplementedException();
        }

        public void Delete(AppUsage obj)
        {
            throw new NotImplementedException();
        }

        public void Delete(Alert obj)
        {
            throw new NotImplementedException();
        }

        public void Delete(Reminder obj)
        {
            throw new NotImplementedException();
        }

        public void AddTagToApp(Tag tag, App app)
        {
            Insert("insert into AppTag(AppId, TagId) values (@AppId, @TagId)", new
            {
                AppId = app.Id,
                TagId = tag.Id
            });
        }

        public void RemoveTagFromApp(Tag tag, App app)
        {
            Connection.Execute("delete from AppTag where AppId=@AppId and TagId=@TagId", new
            {
                AppId = app.Id,
                TagId = tag.Id
            });
        }

        private long Insert(string sql, object obj)
        {
            return Connection.ExecuteScalar<long>(sql + ";select last_insert_rowid()", obj);
        }

        private object AlertObj(Alert a)
        {
            long? appId = null,
                tagId = null,
                actionType = 0,
                timeRangeType = 0,
                timeRangeParam1 = null,
                timeRangeParam2 = null;
            string actionParam = null;
            switch (a)
            {
                case AppAlert o:
                    appId = o.App.Id;
                    break;
                case TagAlert o:
                    tagId = o.Tag.Id;
                    break;
            }

            switch (a.Action)
            {
                case CustomMessageRunAction c:
                    actionType = 0;
                    actionParam = c.Message;
                    break;
                case ScriptMessageRunAction c:
                    actionType = 1;
                    actionParam = c.Script;
                    break;
                case KillRunAction _:
                    actionType = 2;
                    break;
                case MessageRunAction _:
                    actionType = 3;
                    break;
            }

            switch (a.TimeRange)
            {
                case OnceTimeRange o:
                    timeRangeType = 0;
                    timeRangeParam1 = o.Start.Ticks;
                    timeRangeParam2 = o.End.Ticks;
                    break;
                case RepeatingTimeRange o:
                    timeRangeType = 1;
                    timeRangeParam1 = (long) o.Type;
                    break;
            }

            return new
            {
                AppId = appId,
                TagId = tagId,
                MaxDuration = a.MaxDuration.Ticks,
                a.Enabled,
                ActionType = actionType,
                ActionParam = actionParam,
                TimeRangeType = timeRangeType,
                TimeRangeParam1 = timeRangeParam1,
                TimeRangeParam2 = timeRangeParam2
            };
        }

        private object ReminderObj(Reminder r)
        {
            long actionType = 0;
            string actionParam = null;
            switch (r.Action)
            {
                case CustomWarnReminderAction c:
                    actionType = 0;
                    actionParam = c.Warning;
                    break;
                case ScriptReminderAction c:
                    actionType = 1;
                    actionParam = c.Script;
                    break;
                case WarnReminderAction _:
                    actionType = 2;
                    break;
            }

            return new
            {
                AlertId = r.Alert.Id,
                Offset = r.Offset.Ticks,
                ActionType = actionType,
                ActionParam = actionParam
            };
        }
    }
}