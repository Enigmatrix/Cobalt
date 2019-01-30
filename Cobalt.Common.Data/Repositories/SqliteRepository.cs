using System;
using System.Data;
using System.Data.SQLite;
using System.IO;
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
            var (startTicks, endTicks) = ToTickRange(start, end);
            return Query(
                @"select a.Id, a.Name, a.Path, a.Color,
								au.Id, au.AppId, 
								(case when au.Start < @start then @start else au.Start end),
                                (case when au.End > @end then @end else au.End end), 
								au.StartReason, au.EndReason  
							from AppUsage au, App a
							where Start <= @end and End >= @start and au.AppId = a.Id",
                r => AppUsageMapper(r, AppOffset, AppMapper(r)), new
                {
                    start = startTicks,
                    end = endTicks
                });
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
                Icon = obj.Icon?.Value
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

        public bool AppIdByPath(App app)
        {
            var existing = QuerySingle("select * from App where Path = @Path", r => AppMapper(r), new {app.Path});
            if (existing == null) return false;
            app.Id = existing.Id;
            return true;
        }

        public AppUsage AppUsageById(long id)
        {
            return QuerySingle("select a.Id, a.Name, a.Path, a.Icon," +
                               "au.Id, au.AppId, au.Start, au.End, au.StartReason, au.EndReason " +
                               "from AppUsage au, App a where au.Id = @Id and au.AppId = a.Id",
                r => AppUsageMapper(r, AppOffset, AppMapper(r)), new {Id = id});
        }

        public App AppById(long id)
        {
            return QuerySingle("select * from App where Id = @Id",
                r => AppMapper(r), new {Id = id});
        }

        #region Offsets

        private const int AppOffset = 4;
        private const int TagOffset = 2;

        #endregion

        #region Utils

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


        //returns a (start, end)
        private static (long, long) ToTickRange(DateTime? start, DateTime? end)
        {
            return (start?.Ticks ?? DateTime.MinValue.Ticks, end?.Ticks ?? DateTime.MaxValue.Ticks);
        }

        private static byte[] GetBytes(IDataReader reader, int fieldOffset)
        {
            const int chunkSize = 2 * 1024;
            var buffer = new byte[chunkSize];
            if (reader.IsDBNull(0)) return null;
            using (var stream = new MemoryStream())
            {
                long bytesRead;
                while ((bytesRead = reader.GetBytes(0, fieldOffset, buffer, 0, buffer.Length)) > 0)
                {
                    stream.Write(buffer, 0, (int) bytesRead);
                    fieldOffset += (int) bytesRead;
                }

                return stream.ToArray();
            }
        }

        public Lazy<byte[]> GetAppIcon(App app)
        {
            var lazy = new Lazy<byte[]>(() =>
            {
                var reader = Connection.ExecuteReader("select Icon from App where Id = @Id", app);
                using (reader)
                {
                    return reader.Read() ? GetBytes(reader, 0) : null;
                }
            }, true);
            return lazy;
        }

        public IObservable<Tag> GetTags(App app)
        {
            return Query("select * from Tag where Id in (select TagId from AppTag where AppId = @Id)",
                r => TagMapper(r), app);
        }

        public IObservable<T> Query<T>(string sql, Func<IDataReader, T> fun, object pp = null)
        {
            return Observable.Create<T>(obs =>
            {
                var reader = Connection.ExecuteReader(sql, pp);
                using (reader)
                {
                    while (reader.Read())
                        obs.OnNext(fun(reader));
                    obs.OnCompleted();
                }

                return () => { };
            });
        }

        public T QuerySingle<T>(string sql, Func<IDataReader, T> fun, object pp = null)
        {
            var reader = Connection.ExecuteReader(sql, pp);
            var dat = default(T);
            if (!reader.Read()) return dat;
            using (reader)
            {
                dat = fun(reader);
            }

            return dat;
        }

        #endregion

        #region Mapper

        private App AppMapper(IDataReader reader, int offset = 0)
        {
            var app = new App
            {
                Id = reader.GetInt64(offset + 0),
                Name = reader.IsDBNull(offset + 1) ? null : reader.GetString(offset + 1),
                Path = reader.GetString(offset + 2),
                Color = reader.IsDBNull(offset + 3) ? null : reader.GetString(offset + 3)
            };
            app.Icon = GetAppIcon(app);
            app.Tags = GetTags(app);
            return app;
        }

        private Tag TagMapper(IDataReader reader, int offset = 0)
        {
            return new Tag
            {
                Id = reader.GetInt64(offset + 0),
                Name = reader.IsDBNull(offset + 1) ? null : reader.GetString(offset + 1),
                ForegroundColor = reader.IsDBNull(offset + 2) ? null : reader.GetString(offset + 2),
                BackgroundColor = reader.IsDBNull(offset + 3) ? null : reader.GetString(offset + 3)
            };
        }

        private AppUsage AppUsageMapper(IDataReader reader, int offset = 0, App app = null)
        {
            return new AppUsage
            {
                Id = reader.GetInt64(offset + 0),
                App = app ?? new App {Id = reader.GetInt64(offset + 1)},
                Start = new DateTime(reader.GetInt64(offset + 2)),
                End = new DateTime(reader.GetInt64(offset + 3)),
                StartReason = (AppUsageStartReason) reader.GetInt32(offset + 4),
                EndReason = (AppUsageEndReason) reader.GetInt32(offset + 5)
            };
        }

        #endregion
    }
}