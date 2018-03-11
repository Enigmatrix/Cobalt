using System;
using System.Data;
using System.Data.Common;
using System.Data.SQLite;
using System.Reactive.Linq;
using Cobalt.Common.Data.Migration;
using Cobalt.Common.Util;

namespace Cobalt.Common.Data.Repository
{
    public class SqliteRepository : IDbRepository
    {
        #region Find

        public long? FindAppIdByPath(string appPath)
        {
            return ExecuteScalar<long?>("select Id from App where Path = @path", ("path", appPath));
        }

        public bool DoesAppHaveTag(App app, Tag tag)
        {
            return null != ExecuteScalar<long?>("select AppId from AppTag where AppId=@app and TagId=@tag",
                       ("app", app.Id), ("tag", tag.Id));
        }

        #endregion

        #region Offsets

        private const int AppOffset = 3;
        private const int TagOffset = 2;

        #endregion

        #region Lifecycle

        public DbConnection Connection => _connection;
        private readonly SQLiteConnection _connection;

        public SqliteRepository(IDbConnection conn, IDbMigrator migrator)
        {
            _connection = conn as SQLiteConnection;
            if (_connection == null)
                Throw.InvalidOperation("Connection provided to SqliteRepository must be of type SQLiteConnection");
            migrator.Migrate();
        }

        public void Dispose()
        {
            _connection.Dispose();
        }

        #endregion

        #region Add/Remove

        public void AddAppUsage(AppUsage appUsage)
        {
            appUsage.Id = Insert("insert into AppUsage(" +
                                 "Id, " +
                                 "AppId, " +
                                 "UsageType, " +
                                 "StartTimestamp, " +
                                 "EndTimestamp, " +
                                 "UsageStartReason, " +
                                 "UsageEndReason) " +
                                 "values (?,?,?,?,?,?,?)",
                null,
                appUsage.App.Id,
                (int) appUsage.UsageType,
                appUsage.StartTimestamp.Ticks,
                appUsage.EndTimestamp.Ticks,
                (int) appUsage.UsageStartReason,
                (int) appUsage.UsageEndReason);
        }

        public void AddApp(App app)
        {
            app.Id = Insert("insert into App(Id, Name, Path) values (?,?,?)",
                null, app.Name, app.Path);
        }

        public void AddTag(Tag tag)
        {
            tag.Id = Insert("insert into Tag(Id, Name) values (?,?)",
                null, tag.Name);
        }

        public void AddInteraction(Interaction interaction)
        {
            Insert("insert into Interaction(Id) values (?)", interaction.Id);
        }

        public void AddAlert(Alert alert)
        {
            long start = 0, end = 0, repeatType = 0;
            switch (alert.Range)
            {
                case OnceAlertRange once:
                    start = once.StartTimestamp.Ticks;
                    end = once.EndTimestamp.Ticks;
                    repeatType = 0;
                    break;
                case RepeatingAlertRange repeat:
                    start = repeat.DailyStartOffset.Ticks;
                    end = repeat.DailyEndOffset.Ticks;
                    repeatType = (int) repeat.RepeatType + 1;
                    break;
            }

            alert.Id = Insert("insert into Alert" +
                              "(Id, AppId, TagId, AlertType, MaxDuration, ReminderOffset, " +
                              "IsEnabled, AlertAction, Start, End, RepeatType) values" +
                              "(?,?,?,?,?,?,?,?,?,?,?)",
                null,
                (alert as AppAlert)?.App.Id,
                (alert as TagAlert)?.Tag.Id,
                alert is TagAlert,
                alert.MaxDuration.Ticks,
                alert.ReminderOffset.Ticks,
                alert.IsEnabled,
                (int) alert.AlertAction,
                start, end, repeatType
            );
        }

        public void AddTagToApp(Tag tag, App app)
        {
            Insert("insert into AppTag(AppId, TagId) values (?,?)", app.Id, tag.Id);
        }


        public void RemoveTagFromApp(Tag tag, App app)
        {
            Delete("AppTag", "AppId = ? and TagId = ?", app.Id, tag.Id);
        }

        public void RemoveAlert(Alert alert)
        {
            Delete("Alert", "Id = ?", alert.Id);
        }

        public void RemoveTag(Tag tag)
        {
            Delete("AppTag", "TagId = ?", tag.Id);
            Delete("Tag", "Id = ?", tag.Id);
        }

        #endregion

        #region Get

        public IObservable<App> GetApps()
        {
            return Get(@"select a.Id, a.Name, a.Path from App a", r => AppMapper(r))
                .Do(app => app.Tags = GetTags(app));
        }

        public IObservable<Tag> GetTags()
        {
            return Get(@"select * from Tag", r => TagMapper(r));
        }


        public IObservable<App> GetAppsWithTag(Tag tag)
        {
            return Get(@"select * from App
								where Id in (select AppId from AppTag
									where TagId = (select Id from Tag 
										where Name = @name))", r => AppMapper(r), ("name", tag.Name))
                .Do(app => app.Tags = GetTags(app));
        }

        public IObservable<TimeSpan> GetAppUsageTime(DateTime? start = null, DateTime? end = null)
        {
            var (startTicks, endTicks) = ToTickRange(start, end);
            var ticks = ExecuteScalar<long>(@"select sum(
                                (case when au.EndTimestamp > @end then @end else au.EndTimestamp end) -
								(case when au.StartTimestamp < @start then @start else au.StartTimestamp end))
                                from AppUsage au
                                where StartTimestamp <= @end and EndTimestamp >= @start",
                ("start", startTicks),
                ("end", endTicks));
            return Observable.Return(TimeSpan.FromTicks(ticks));
        }

        public IObservable<AppUsage> GetAppUsages(DateTime? start, DateTime? end)
        {
            //TODO might cause high memory usage, implement cache
            var (startTicks, endTicks) = ToTickRange(start, end);
            return Get(@"select a.Id, a.Name, a.Path, 
								au.Id, au.AppId, au.UsageType, 
								(case when au.StartTimestamp < @start then @start else au.StartTimestamp end),
                                (case when au.EndTimestamp > @end then @end else au.EndTimestamp end), 
								au.UsageStartReason, au.UsageEndReason  
							from AppUsage au, App a
							where StartTimestamp <= @end and EndTimestamp >= @start and au.AppId = a.Id",
                    r => AppUsageMapper(r, AppOffset, AppMapper(r)),
                    ("start", startTicks),
                    ("end", endTicks))
                .Do(appUsage => appUsage.App.Tags = GetTags(appUsage.App));
        }

        public IObservable<AppUsage> GetAppUsages(Tag tag, DateTime? start = null, DateTime? end = null)
        {
            var (startTicks, endTicks) = ToTickRange(start, end);
            return Get(@"select a.Id, a.Name, a.Path, 
								au.Id, au.AppId, au.UsageType, 
								(case when au.StartTimestamp < @start then @start else au.StartTimestamp end),
                                (case when au.EndTimestamp > @end then @end else au.EndTimestamp end), 
								au.UsageStartReason, au.UsageEndReason  
							from AppUsage au, App a
							where StartTimestamp <= @end and EndTimestamp >= @start and au.AppId = a.Id
                                and AppId in (select AppId from AppTag where TagId=@tag)",
                    r => AppUsageMapper(r, AppOffset, AppMapper(r)),
                    ("start", startTicks),
                    ("end", endTicks),
                    ("tag", tag.Id))
                .Do(appUsage => appUsage.App.Tags = GetTags(appUsage.App));
        }

        public IObservable<AppUsage> GetAppUsagesForApp(App app, DateTime? start = null, DateTime? end = null)
        {
            //todo, can be more efficient, try on v1.0
            var (startTicks, endTicks) = ToTickRange(start, end);
            return Get(@"select a.Id, a.Name, a.Path, 
								au.Id, au.AppId, au.UsageType, 
								(case when au.StartTimestamp < @start then @start else au.StartTimestamp end),
                                (case when au.EndTimestamp > @end then @end else au.EndTimestamp end), 
								au.UsageStartReason, au.UsageEndReason  
							from AppUsage au, App a
							where StartTimestamp <= @end and EndTimestamp >= @start where au.AppId = a.Id and a.Id = @id",
                    r => AppUsageMapper(r, AppOffset, AppMapper(r)),
                    ("start", startTicks),
                    ("end", endTicks),
                    ("id", app.Id))
                .Do(appUsage => appUsage.App.Tags = GetTags(appUsage.App));
        }

        public IObservable<(App App, TimeSpan Duration)> GetAppDurations(DateTime? start = null, DateTime? end = null)
        {
            var (startTicks, endTicks) = ToTickRange(start, end);
            return Get(@"select AppId, a.Name, a.Path, sum(
											(case when EndTimestamp > @end then @end else EndTimestamp end)
										-   (case when StartTimestamp < @start then @start else StartTimestamp end)) Duration
										from AppUsage, App a
										where (StartTimestamp <= @end and EndTimestamp >= @start) and a.Id = AppId
										group by AppId",
                    r => (AppMapper(r), TimeSpan.FromTicks(r.GetInt64(AppOffset))),
                    ("start", startTicks),
                    ("end", endTicks))
                .Do(appDur => appDur.Item1.Tags = GetTags(appDur.Item1));
        }

        public IObservable<(App App, TimeSpan Duration)> GetAppDurations(Tag tag, DateTime? start = null,
            DateTime? end = null)
        {
            var (startTicks, endTicks) = ToTickRange(start, end);
            return Get(@"select AppId, a.Name, a.Path, sum(
											(case when EndTimestamp > @end then @end else EndTimestamp end)
										-   (case when StartTimestamp < @start then @start else StartTimestamp end)) Duration
										from AppUsage, App a
										where (StartTimestamp <= @end and EndTimestamp >= @start) and a.Id = AppId
                                            and AppId in (select AppId from AppTag where TagId=@tag)
										group by AppId",
                    r => (AppMapper(r), TimeSpan.FromTicks(r.GetInt64(AppOffset))),
                    ("start", startTicks),
                    ("end", endTicks),
                    ("tag", tag.Id))
                .Do(appDur => appDur.Item1.Tags = GetTags(appDur.Item1));
        }

        public IObservable<TimeSpan> GetAppDuration(App app, DateTime? start = null, DateTime? end = null)
        {
            var (startTicks, endTicks) = ToTickRange(start, end);
            return Get(@"select sum((case when EndTimestamp > @end then @end else EndTimestamp end)
								-   (case when StartTimestamp < @start then @start else StartTimestamp end))
										from AppUsage
										where (StartTimestamp <= @end and EndTimestamp >= @start) and AppId = @app",
                r =>
                    TimeSpan.FromTicks(r.IsDBNull(0) ? 0 : r.GetInt64(0)),
                ("app", app.Id),
                ("start", startTicks),
                ("end", endTicks));
        }

        public IObservable<(Tag Tag, TimeSpan Duration)> GetTagDurations(DateTime? start = null, DateTime? end = null)
        {
            var (startTicks, endTicks) = ToTickRange(start, end);
            return Get(@"select t.Id, t.Name, sum(
											(case when EndTimestamp > @end then @end else EndTimestamp end)
										-   (case when StartTimestamp < @start then @start else StartTimestamp end)) Duration
										from AppUsage, Tag t
										where (StartTimestamp <= @end and EndTimestamp >= @start)
                                            and AppId in (select AppId from AppTag where TagId = t.Id)
										group by t.Id
                        union
                        select -1, 'Untagged', sum(
											(case when EndTimestamp > @end then @end else EndTimestamp end)
										-   (case when StartTimestamp < @start then @start else StartTimestamp end)) Duration
										from AppUsage
										where (StartTimestamp <= @end and EndTimestamp >= @start)
                                            and AppId not in (select AppId from AppTag)",
                r => (TagMapper(r), TimeSpan.FromTicks(r.GetInt64(TagOffset))),
                ("start", startTicks),
                ("end", endTicks));
        }

        public IObservable<Tag> GetTags(App app)
        {
            return Get("select * from Tag where Id in (select TagId from AppTag where AppId = @id)", r => TagMapper(r),
                ("id", app.Id));
        }

        public IObservable<(DateTime Start, DateTime End)> GetIdleDurations(TimeSpan minDuration,
            DateTime? start = null, DateTime? end = null)
        {
            var (startTicks, endTicks) = ToTickRange(start, end);
            return Get(
                @"select a1.Id, (case when a2.Id > @end then @ned else a2.Id end) from Interaction a1, Interaction a2 
							where a1.Id >= @start and a1.Id < @end and a2.rowid=a1.rowid+1 and (a2.Id-a1.Id)>=@minDur",
                r => IdleMapper(r),
                ("start", startTicks),
                ("end", endTicks),
                ("minDur", minDuration.Ticks));
        }

        public IObservable<Alert> GetAlerts()
        {
            return Get("select Id, AppId, TagId, AlertType, MaxDuration, ReminderOffset, " +
                       "IsEnabled, AlertAction, Start, End, RepeatType from Alert", r => AlertMapper(r));
        }

        #endregion

        #region Util

        private long Insert(string stmt, params object[] param)
        {
            ExecuteNonQuery(stmt, param);
            return _connection.LastInsertRowId;
        }

        private void Delete(string table, string condition, params object[] param)
        {
            ExecuteNonQuery($"delete from {table} where {condition}", param);
        }

        private void Update(string table, string update, long id, params object[] param)
        {
            ExecuteNonQuery($"update {table} set {update} where Id = {id}", param);
        }

        private void ExecuteNonQuery(string cmdStr, params object[] param)
        {
            using (var cmd = new SQLiteCommand(cmdStr, _connection))
            {
                foreach (var p in param)
                    cmd.Parameters.AddWithValue(null, p);
                cmd.ExecuteNonQuery();
            }
        }

        private T ExecuteScalar<T>(string cmdStr,
            params (string, object)[] args)
        {
            using (var cmd = new SQLiteCommand(cmdStr, _connection))
            {
                foreach (var p in args)
                    cmd.Parameters.AddWithValue(p.Item1, p.Item2);
                var res = cmd.ExecuteScalar();
                return res == DBNull.Value ? default(T) : (T) res;
            }
        }

        private (SQLiteCommand Cmd, SQLiteDataReader Reader) ExecuteReader(string cmdStr,
            params (string, object)[] args)
        {
            var cmd = new SQLiteCommand(cmdStr, _connection);
            foreach (var p in args)
                cmd.Parameters.AddWithValue(p.Item1, p.Item2);
            return (cmd, cmd.ExecuteReader());
        }

        private IObservable<T> Get<T>(string cmdStr, Func<SQLiteDataReader, T> mapper, params (string, object)[] param)
        {
            return Observable.Create<T>(obs =>
            {
                var (cmd, reader) = ExecuteReader(cmdStr, param);

                using (cmd)
                using (reader)
                {
                    while (reader.Read())
                        obs.OnNext(mapper(reader));
                    obs.OnCompleted();
                }

                return () => { };
            });
        }

        private Interaction InteractionMapper(SQLiteDataReader reader, int offset = 0)
        {
            return new Interaction
            {
                Timestamp = new DateTime(reader.GetInt64(offset))
            };
        }

        private (DateTime Start, DateTime End) IdleMapper(SQLiteDataReader reader, int offset = 0)
        {
            return (new DateTime(reader.GetInt64(offset)), new DateTime(reader.GetInt64(offset + 1)));
        }

        private App AppMapper(SQLiteDataReader reader, int offset = 0)
        {
            return new App
            {
                Id = reader.GetInt64(offset + 0),
                Name = reader.IsDBNull(1) ? null : reader.GetString(offset + 1),
                Path = reader.GetString(offset + 2)
            };
        }

        private Tag TagMapper(SQLiteDataReader reader, int offset = 0)
        {
            return new Tag
            {
                Id = reader.GetInt64(offset + 0),
                Name = reader.IsDBNull(1) ? null : reader.GetString(offset + 1)
            };
        }

        private AppUsage AppUsageMapper(SQLiteDataReader reader, int offset = 0, App app = null)
        {
            return new AppUsage
            {
                Id = reader.GetInt64(offset + 0),
                App = app ?? new App {Id = reader.GetInt64(offset + 1)},
                UsageType = (AppUsageType) reader.GetInt32(offset + 2),
                StartTimestamp = new DateTime(reader.GetInt64(offset + 3)),
                EndTimestamp = new DateTime(reader.GetInt64(offset + 4)),
                UsageStartReason = (AppUsageStartReason) reader.GetInt32(offset + 5),
                UsageEndReason = (AppUsageEndReason) reader.GetInt32(offset + 6)
            };
        }

        private Alert AlertMapper(SQLiteDataReader reader, int offset = 0)
        {
            Alert SetAlertFields(Alert alert)
            {
                alert.Id = reader.GetInt64(offset + 0);
                alert.MaxDuration = TimeSpan.FromTicks(reader.GetInt64(offset + 4));
                alert.ReminderOffset = TimeSpan.FromTicks(reader.GetInt64(offset + 5));
                alert.IsEnabled = reader.GetBoolean(offset + 6);
                alert.AlertAction = (AlertAction) reader.GetInt32(offset + 7);
                var start = reader.GetInt64(offset + 8);
                var end = reader.GetInt64(offset + 9);
                var repeatMode = reader.GetInt32(offset + 10);
                alert.Range = repeatMode == 0
                    ? new OnceAlertRange
                    {
                        StartTimestamp = new DateTime(start),
                        EndTimestamp = new DateTime(end)
                    }
                    : (AlertRange) new RepeatingAlertRange
                    {
                        DailyStartOffset = TimeSpan.FromTicks(start),
                        DailyEndOffset = TimeSpan.FromTicks(end),
                        RepeatType = (RepeatType) (repeatMode - 1)
                    };
                return alert;
            }

            if (reader.GetInt32(offset + 3) == 0)
                return SetAlertFields(new AppAlert {App = new App {Id = reader.GetInt64(offset + 1)}});
            return SetAlertFields(new TagAlert {Tag = new Tag {Id = reader.GetInt64(offset + 2)}});
        }

        //returns a (start, end)
        private (long, long) ToTickRange(DateTime? start, DateTime? end)
        {
            return (start?.Ticks ?? DateTime.MinValue.Ticks, end?.Ticks ?? DateTime.MaxValue.Ticks);
        }

        #endregion

        #region Update

        public void UpdateApp(App app)
        {
            Update("App", "Name = ?, Path = ?", app.Id, app.Name, app.Path);
        }

        public void UpdateTag(Tag tag)
        {
            Update("Tag", "Name = ?", tag.Id, tag.Name);
        }

        public void UpdateAlert(Alert alert)
        {
            long start = 0, end = 0, repeatType = 0;
            switch (alert.Range)
            {
                case OnceAlertRange once:
                    start = once.StartTimestamp.Ticks;
                    end = once.EndTimestamp.Ticks;
                    repeatType = 0;
                    break;
                case RepeatingAlertRange repeat:
                    start = repeat.DailyStartOffset.Ticks;
                    end = repeat.DailyEndOffset.Ticks;
                    repeatType = (int) repeat.RepeatType + 1;
                    break;
            }

            Update("Alert", "AppId=?, TagId=?, AlertType=?, MaxDuration=?, ReminderOffset=?, " +
                            "IsEnabled=?, AlertAction=?, Start=?, End=?, RepeatType=?",
                alert.Id,
                (alert as AppAlert)?.App.Id,
                (alert as TagAlert)?.Tag.Id,
                alert is TagAlert,
                alert.MaxDuration.Ticks,
                alert.ReminderOffset.Ticks,
                alert.IsEnabled,
                (int) alert.AlertAction,
                start, end, repeatType);
        }

        #endregion
    }
}