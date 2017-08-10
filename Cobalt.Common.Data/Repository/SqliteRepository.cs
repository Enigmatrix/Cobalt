﻿using System;
using System.Collections.Generic;
using System.Data;
using System.Data.Common;
using System.Data.SQLite;
using System.Linq;
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
            var (cmd, reader) = ExecuteReader("select Id from App where Path = ?", appPath);
            var result = reader.Read()
                ? (long?) reader.GetInt64(0)
                : null;
            reader.Dispose();
            cmd.Dispose();
            return result;
        }

        #endregion

        #region Lifecycle

        public DbConnection Connection => _connection;
        private readonly SQLiteConnection _connection;

        public SqliteRepository(IDbConnection conn, Migrator migrator)
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

        public void AddTagToApp(Tag tag, App app)
        {
            Insert("insert into AppTag(AppId, TagId) values (?,?)", app.Id, tag.Id);
        }


        public void RemoveTagFromApp(Tag tag, App app)
        {
            Delete("AppTag", "AppId = ? and TagId = ?", app.Id, tag.Id);
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
            return Get(@"select * from Tags", r => TagMapper(r));
        }


        public IObservable<App> GetAppsWithTag(Tag tag)
        {
            throw new NotImplementedException();
            return Get(@"select * from App
                            	where Id = (select AppId from AppTag
		                            where TagId = (select Id from Tag 
                                        where Name = ?))", r => AppMapper(r), tag.Name);
        }

        public IObservable<AppUsage> GetAppUsages(DateTime? start, DateTime? end, bool includeApps = false)
        {
            throw new NotImplementedException();
            var (startTicks, endTicks) = ToTickRange(start, end);
            return Get(@"select * from AppUsage
                                    where StartTimestamp >= ? and EndTimestamp <= ?", r => AppUsageMapper(r),
                startTicks, endTicks);
        }

        public IObservable<AppUsage> GetAppUsagesForApp(App app, DateTime? start = null, DateTime? end = null)
        {
            var (startTicks, endTicks) = ToTickRange(start, end);
            return Get(@"select * from AppUsage
                                    where StartTimestamp >= ? and EndTimestamp <= ?
                                        and AppId = ?", r => AppUsageMapper(r),
                startTicks, endTicks, app.Id);
        }

        public IObservable<(App App, TimeSpan Duration)> GetAppDurations(DateTime? start = null, DateTime? end = null)
        {
            throw new NotImplementedException();
            var (startTicks, endTicks) = ToTickRange(start, end);
            return GetAppDurations(@"select AppId, a.Name, a.Path, sum(
                                            (case when EndTimestamp > ? then ? else EndTimestamp end)
                                        -   (case when StartTimestamp < ? then ? else StartTimestamp end)) Duration
                                        from AppUsage, App a
                                        where StartTimestamp >= ? and EndTimestamp <= ? and a.Id = AppId
                                        group by AppId", endTicks, endTicks, startTicks, startTicks, startTicks,
                endTicks);
        }

        public IObservable<(Tag Tag, TimeSpan Duration)> GetTagDurations(DateTime? start = null, DateTime? end = null)
        {
            var (startTicks, endTicks) = ToTickRange(start, end);
            return GetTagDurations(@"select t.Id, t.Name, sum(
                                            (case when EndTimestamp > ? then ? else EndTimestamp end)
                                        -   (case when StartTimestamp < ? then ? else StartTimestamp end)) Duration
                                        from AppUsage, App a, AppTag at, Tag t
                                        where StartTimestamp >= ? and EndTimestamp <= ? and a.Id = AppId and a.Id = at.AppId and t.Id = at.TagId
                                        group by t.Id", startTicks, endTicks);
        }

        private IObservable<Tag> GetTags(App app)
        {
            return Get("select * from Tag where Id in (select TagId from AppTag where AppId = ?)", r => TagMapper(r),
                app.Id);
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

        private (SQLiteCommand Cmd, SQLiteDataReader Reader) ExecuteReader(string cmdStr, params object[] param)
        {
            var cmd = new SQLiteCommand(cmdStr, _connection);
            foreach (var p in param)
                cmd.Parameters.AddWithValue(null, p);
            return (cmd, cmd.ExecuteReader());
        }


        private IObservable<T> Get<T>(string cmdStr, Func<SQLiteDataReader, T> mapper, params object[] param)
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

        private AppUsage AppUsageMapper(SQLiteDataReader reader, int offset = 0)
        {
            return new AppUsage
            {
                Id = reader.GetInt64(offset + 0),
                App = new App {Id = reader.GetInt64(offset + 1)},
                UsageType = (AppUsageType) reader.GetInt32(offset + 2),
                StartTimestamp = new DateTime(reader.GetInt64(offset + 3)),
                EndTimestamp = new DateTime(reader.GetInt64(offset + 4)),
                UsageStartReason = (AppUsageStartReason) reader.GetInt32(offset + 5),
                UsageEndReason = (AppUsageEndReason) reader.GetInt32(offset + 6)
            };
        }

        private List<long> IdsMapper(SQLiteDataReader reader, int offset = 0)
        {
            return reader.GetString(offset).Split(',').Select(long.Parse).ToList();
        }

        private IObservable<(App, TimeSpan)> GetAppDurations(string cmdStr, params object[] param)
        {
            return Observable.Create<(App, TimeSpan)>(obs =>
            {
                var (cmd, reader) = ExecuteReader(cmdStr, param);
                while (reader.Read())
                {
                    var appId = reader.GetInt64(0);
                    var app = new App
                    {
                        Id = appId,
                        Name = reader.IsDBNull(1) ? null : reader.GetString(1),
                        Path = reader.GetString(2)
                    };
                    var duration = TimeSpan.FromTicks(reader.GetInt64(3));
                    obs.OnNext((app, duration));
                }
                obs.OnCompleted();
                reader.Dispose();
                cmd.Dispose();
                return () => { };
            });
        }

        private IObservable<(Tag, TimeSpan)> GetTagDurations(string cmdStr, params object[] param)
        {
            return Observable.Create<(Tag, TimeSpan)>(obs =>
            {
                var (cmd, reader) = ExecuteReader(cmdStr, param);
                while (reader.Read())
                {
                    var tagId = reader.GetInt64(0);
                    var tag = new Tag
                    {
                        Id = tagId,
                        Name = reader.IsDBNull(1) ? null : reader.GetString(1)
                    };
                    var duration = TimeSpan.FromTicks(reader.GetInt64(2));
                    obs.OnNext((tag, duration));
                }
                obs.OnCompleted();
                reader.Dispose();
                cmd.Dispose();
                return () => { };
            });
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

        #endregion
    }
}