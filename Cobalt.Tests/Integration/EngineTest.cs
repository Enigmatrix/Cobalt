using Cobalt.Common.Data.Repositories;
using Cobalt.Common.IoC;
using Microsoft.Win32.TaskScheduler;
using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Text;
using Xunit;
using System.Reactive;
using System.Data.SQLite;
using Dapper;
using System.IO;
using System.Threading;

namespace Cobalt.Tests.Integration
{
    public class EngineTest : IDisposable
    {

        public EngineTest()
        {
            
        }

        public void Dispose()
        {

        }

        [Fact, Trait("Category", "Integration")]
        public void EngineRuns()
        {
            using var db = new DbUtil();
            using var _ = new TaskServiceUtil(TaskName.CobaltEngine);
            Thread.Sleep(20_000);

            Process.Start(new ProcessStartInfo
            {
                FileName = "NOTEPAD.EXE"
            });
            Process.Start(new ProcessStartInfo
            {
                FileName = "explorer.exe",
            });

            var list = db.Connection.Query("select * from AppUsage").AsList();

            Assert.Single(list);
        }
    }
}
