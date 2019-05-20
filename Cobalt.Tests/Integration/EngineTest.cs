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
using System.Runtime.InteropServices;
using System.Windows.Forms;

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
            using (var db = new DbUtil())
            {
                using (var proc = new ProcessUtil("WINVER.EXE", "NOTEPAD.EXE"))
                {
                    proc.SetFgInternal("WINVER.EXE");
                    using (var _ = new TaskServiceUtil(TaskName.CobaltEngine))
                    {
                        Thread.Sleep(3_000);
                        //SendKeys.SendWait("%{Tab}");
                        proc.SetFgInternal("NOTEPAD.EXE");
                        Thread.Sleep(1_000);
                        //SendKeys.SendWait("%{Tab}");
                    }
                }
                var list = db.Connection.Query("select * from AppUsage").AsList();
                Assert.Single(list);
                Thread.Sleep(1000);
            }
        }
    }
}
