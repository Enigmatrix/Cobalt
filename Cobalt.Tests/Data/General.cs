using System;
using System.Data.SQLite;
using System.IO;
using System.Linq;
using Cobalt.Common.Data.Entities;
using Cobalt.Common.Data.Migrations;
using Cobalt.Common.Data.Repositories;
using Dapper;
using Xunit;
using App1 = Cobalt.Common.Data.Entities.App;

namespace Cobalt.Tests.Data
{
    public class General : IDisposable
    {
        public General()
        {
            File.Delete("test.db");
            _conn = new SQLiteConnection("Data Source=test.db");
            var mig = new SqliteMigrator(_conn);
            _repo = new SqliteRepository(_conn, mig);
        }

        public void Dispose()
        {
            _repo.Dispose();
        }

        private readonly SQLiteConnection _conn;
        private readonly IDbRepository _repo;

        [Fact]
        public void InsertApp()
        {
            
            var app = new App1
            {
                Name = "123",
                Color = "red",
                Icon = new Lazy<byte[]>(() => new byte[] {1, 3, 3, 7}),
                Path = "s:/tr8/to/my/ass"
            };
            _repo.Insert(app);

            var apps = _conn.Query("select * from App").ToList();
            Assert.Single(apps);
            Assert.Equal("123", apps[0].Name);
            Assert.Equal("red", apps[0].Color);
            Assert.Equal(new byte[] {1, 3, 3, 7}, apps[0].Icon);
            Assert.Equal("s:/tr8/to/my/ass", apps[0].Path);
        }

        [Fact]
        public void InsertAppUsage()
        {
            var app = new App1
            {
                Name = "123",
                Color = "red",
                Icon = new Lazy<byte[]>(() => new byte[] {1, 3, 3, 7}),
                Path = "s:/tr8/to/my/ass"
            };
            _repo.Insert(app);
            var now = DateTime.Now;
            var au = new AppUsage
            {
                App = app,
                Start = now,
                End = now + TimeSpan.FromHours(5),
                StartReason = AppUsageStartReason.MonitorOn,
                EndReason = AppUsageEndReason.MonitorOff
            };
            _repo.Insert(au);

            var aus = _conn.Query("select * from AppUsage").ToList();
            Assert.Single(aus);
            Assert.Equal(1, aus[0].AppId);
            Assert.Equal(now.Ticks, aus[0].Start);
            Assert.Equal((now + TimeSpan.FromHours(5)).Ticks, aus[0].End);
            Assert.Equal(3, aus[0].StartReason);
            Assert.Equal(4, aus[0].EndReason);
        }
    }
}