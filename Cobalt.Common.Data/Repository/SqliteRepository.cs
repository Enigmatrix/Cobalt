using System;
using System.Collections.Generic;
using System.Data;
using System.Data.SQLite;
using System.Linq;
using System.Reactive.Linq;
using System.Text;
using System.Threading.Tasks;
using Cobalt.Common.Data.Migration;
using Cobalt.Common.Data.Migration.Sqlite;
using Cobalt.Common.Util;

namespace Cobalt.Common.Data.Repository
{
    public class SqliteRepository
    {
        private readonly SQLiteConnection _connection;

        public SqliteRepository(IDbConnection conn, Migrator migrator)
        {
            _connection = conn as SQLiteConnection;
            if(_connection == null)
                Throw.InvalidOperation("Connection provided to SqliteRepository must be of type SQLiteConnection");
            migrator.Migrate();
        }

        public void Dispose()
        {
            _connection.Dispose();
        }

        #region Util

        private long Insert(string stmt, params object[] param)
        {
            using (var cmd = new SQLiteCommand(stmt, _connection))
            {
                foreach (var p in param)
                    cmd.Parameters.AddWithValue(null, p);
                cmd.ExecuteNonQuery();
                return _connection.LastInsertRowId;
            }
        }

        private IObservable<AppUsage> GetAppUsages(string cmdStr, params object[] param)
        {
            return Observable.Create<AppUsage>(obs =>
            {
                using (var cmd = new SQLiteCommand(cmdStr, _connection))
                {
                    foreach (var p in param)
                        cmd.Parameters.AddWithValue(null, p);
                    using (var reader = cmd.ExecuteReader())
                    {
                        while (reader.Read())
                        {
                            var appUsage = new AppUsage
                            {
                                Id = reader.GetInt64(0),
                                App = new App { Id = reader.GetInt64(1) },
                                UsageType = (AppUsageType)reader.GetInt32(2),
                                StartTimestamp = new DateTime(reader.GetInt64(3)),
                                EndTimestamp = new DateTime(reader.GetInt64(4)),
                                UsageStartReason = (AppUsageStartReason)reader.GetInt32(5),
                                UsageEndReason = (AppUsageEndReason)reader.GetInt32(6)
                            };
                            obs.OnNext(appUsage);
                        }
                        obs.OnCompleted();
                    }
                }
                return () => { };
            });
        }

        private IObservable<App> GetApps(string cmdStr, params object[] param)
        {
            return Observable.Create<App>(obs =>
            {
                using (var cmd = new SQLiteCommand(cmdStr, _connection))
                {
                    foreach (var p in param)
                        cmd.Parameters.AddWithValue(null, p);
                    using (var reader = cmd.ExecuteReader())
                    {
                        while (reader.Read())
                        {
                            var appId = reader.GetInt64(0);
                            var app = new App
                            {
                                Id = appId,
                                Name = reader.IsDBNull(1) ? null : reader.GetString(1),
                                Path = reader.GetString(2)
                            };
                            obs.OnNext(app);
                        }
                        obs.OnCompleted();
                    }
                }
                return () => { };
            });
        }

        private IObservable<(App, TimeSpan)> GetAppDurations(string cmdStr, params object[] param)
        {
            return Observable.Create<(App, TimeSpan)>(obs =>
            {
                using (var cmd = new SQLiteCommand(cmdStr, _connection))
                {
                    foreach (var p in param)
                        cmd.Parameters.AddWithValue(null, p);
                    using (var reader = cmd.ExecuteReader())
                    {
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
                    }
                }
                return () => { };
            });
        }

        private IObservable<(Tag, TimeSpan)> GetTagDurations(string cmdStr, params object[] param)
        {
            return Observable.Create<(Tag, TimeSpan)>(obs =>
            {
                using (var cmd = new SQLiteCommand(cmdStr, _connection))
                {
                    foreach (var p in param)
                        cmd.Parameters.AddWithValue(null, p);
                    using (var reader = cmd.ExecuteReader())
                    {
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
                    }
                }
                return () => { };
            });
        }

        //returns a (start, end)
        private (long, long) ToTickRange(DateTime? start, DateTime? end)
        {
            return (start?.Ticks ?? DateTime.MinValue.Ticks, end?.Ticks ?? DateTime.MaxValue.Ticks);
        }
        
        #endregion
    }
}
