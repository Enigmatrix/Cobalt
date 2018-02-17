using Cobalt.Common.Data;
using Cobalt.Common.Data.Migration.Sqlite;
using Cobalt.Common.Data.Repository;
using System;
using System.Collections.Generic;
using System.Data.SQLite;
using System.IO;
using System.Linq;
using System.Reactive.Linq;
using System.Text;
using System.Threading.Tasks;
using Xunit;

namespace Cobalt.Tests.Common.Data
{
    public class AlertTests : IDisposable
    {
        public static readonly string Database = "dat.db";
        public SQLiteConnection Connection { get;}
        public SqliteRepository Repo { get; }

        //these values are NOT shared amongst the tests, xunit creates the class again foreach test
        public AlertTests()
        {
            Connection = new SQLiteConnection($"Data Source={Database}").OpenAndReturn();
            Repo = new SqliteRepository(Connection, new SqliteMigrator(Connection));
        }

        [Fact]
        public void InsertAppAlertWithRepeat()
        {
            var appAlert = new AppAlert {
                AlertAction = AlertAction.Kill,
                App = new App { Id = 0 },
                IsEnabled = true,
                MaxDuration = TimeSpan.FromHours(2),
                Range = new RepeatingAlertRange { 
                    DailyStartOffset=TimeSpan.FromMilliseconds(126),
                    DailyEndOffset=TimeSpan.FromMilliseconds(12),
                    RepeatType = RepeatType.Daily
                },
                ReminderOffset = TimeSpan.FromMinutes(5)
            };
            Repo.AddAlert(appAlert);
            var returnAppAlert = Repo.GetAlerts().SingleAsync().Wait();
            AssertEx.DeepEquals(appAlert, returnAppAlert);
        }

        [Fact]
        public void InsertAppAlertWithOnce()
        {
            var appAlert = new AppAlert {
                AlertAction = AlertAction.Kill,
                App = new App { Id = 0 },
                IsEnabled = true,
                MaxDuration = TimeSpan.FromHours(2),
                Range = new OnceAlertRange { 
                    StartTimestamp=DateTime.Now,
                    EndTimestamp=DateTime.Now.AddHours(1),
                },
                ReminderOffset = TimeSpan.FromMinutes(5)
            };
            Repo.AddAlert(appAlert);
            var returnAppAlert = Repo.GetAlerts().SingleAsync().Wait();
            AssertEx.DeepEquals(appAlert, returnAppAlert);
        }

        [Fact]
        public void InsertTagAlertWithRepeat()
        {
            var tagAlert = new TagAlert {
                AlertAction = AlertAction.Kill,
                Tag = new Tag { Id = 0 },
                IsEnabled = true,
                MaxDuration = TimeSpan.FromHours(2),
                Range = new RepeatingAlertRange { 
                    DailyStartOffset=TimeSpan.FromMilliseconds(126),
                    DailyEndOffset=TimeSpan.FromMilliseconds(12),
                    RepeatType = RepeatType.Daily
                },
                ReminderOffset = TimeSpan.FromMinutes(5)
            };
            Repo.AddAlert(tagAlert);
            var returnTagAlert = Repo.GetAlerts().SingleAsync().Wait();
            AssertEx.DeepEquals(tagAlert, returnTagAlert);
        }

        [Fact]
        public void InsertTagAlertWithOnce()
        {
            var tagAlert = new TagAlert {
                AlertAction = AlertAction.Kill,
                Tag = new Tag { Id = 0 },
                IsEnabled = true,
                MaxDuration = TimeSpan.FromHours(2),
                Range = new OnceAlertRange { 
                    StartTimestamp=DateTime.Now,
                    EndTimestamp=DateTime.Now.AddHours(1),
                },
                ReminderOffset = TimeSpan.FromMinutes(5)
            };
            Repo.AddAlert(tagAlert);
            var returnTagAlert = Repo.GetAlerts().SingleAsync().Wait();
            AssertEx.DeepEquals(tagAlert, returnTagAlert);
        }

        [Fact]
        public void RemoveAppAlert()
        {
            var appAlert = new AppAlert {
                AlertAction = AlertAction.Kill,
                App = new App { Id = 0 },
                IsEnabled = true,
                MaxDuration = TimeSpan.FromHours(2),
                Range = new RepeatingAlertRange { 
                    DailyStartOffset=TimeSpan.FromMilliseconds(126),
                    DailyEndOffset=TimeSpan.FromMilliseconds(12),
                    RepeatType = RepeatType.Daily
                },
                ReminderOffset = TimeSpan.FromMinutes(5)
            };
            Repo.AddAlert(appAlert);
            Assert.Equal(1, Repo.GetAlerts().Count().Wait());
            Repo.RemoveAlert(appAlert);
            Assert.Equal(0, Repo.GetAlerts().Count().Wait());
        }

        [Fact]
        public void RemoveTagAlert()
        {
            var tagAlert = new TagAlert {
                AlertAction = AlertAction.Kill,
                Tag = new Tag { Id = 0 },
                IsEnabled = true,
                MaxDuration = TimeSpan.FromHours(2),
                Range = new RepeatingAlertRange { 
                    DailyStartOffset=TimeSpan.FromMilliseconds(126),
                    DailyEndOffset=TimeSpan.FromMilliseconds(12),
                    RepeatType = RepeatType.Daily
                },
                ReminderOffset = TimeSpan.FromMinutes(5)
            };
            Repo.AddAlert(tagAlert);
            Assert.Equal(1, Repo.GetAlerts().Count().Wait());
            Repo.RemoveAlert(tagAlert);
            Assert.Equal(0, Repo.GetAlerts().Count().Wait());
        }

        [Fact]
        public void UpdateAlert()
        {
            var tagAlert = new TagAlert {
                AlertAction = AlertAction.Kill,
                Tag = new Tag { Id = 0 },
                IsEnabled = true,
                MaxDuration = TimeSpan.FromHours(2),
                Range = new RepeatingAlertRange { 
                    DailyStartOffset=TimeSpan.FromMilliseconds(126),
                    DailyEndOffset=TimeSpan.FromMilliseconds(12),
                    RepeatType = RepeatType.Daily
                },
                ReminderOffset = TimeSpan.FromMinutes(5)
            };
            Repo.AddAlert(tagAlert);
            Assert.Equal(1, Repo.GetAlerts().Count().Wait());
            tagAlert.AlertAction = AlertAction.Message;
            tagAlert.Tag.Id=2;
            tagAlert.IsEnabled=false;
            tagAlert.MaxDuration = TimeSpan.FromMilliseconds(200);
            tagAlert.ReminderOffset=TimeSpan.FromMilliseconds(10);
            tagAlert.Range = new OnceAlertRange { StartTimestamp = DateTime.Now, EndTimestamp = DateTime.Now.AddHours(1) };
            Repo.UpdateAlert(tagAlert);
            var returnAppAlert = Repo.GetAlerts().SingleAsync().Wait();
            AssertEx.DeepEquals(tagAlert, returnAppAlert);
        }

        public void Dispose()
        {
            Repo.Dispose();
            Connection.Dispose();
            File.Delete(Database);
        }
    }
}
