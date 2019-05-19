using Cobalt.Common.Data.Repositories;
using Cobalt.Common.IoC;
using System;
using System.Collections.Generic;
using System.Data.SQLite;
using System.IO;
using System.Text;

namespace Cobalt.Tests.Integration
{
    public class DbUtil : IDisposable
    {
        private readonly SQLiteConnection conn;

        public DbUtil()
        {
            var file = IoCService.DbPath;
            File.Move(file, file+".backup");
            conn = IoCService.Instance.Resolve<SQLiteConnection>();
        }

        public SQLiteConnection Connection => conn;

        public void Dispose()
        {
            var file = conn.FileName;
            conn.Dispose();
            File.Delete(file);
            File.Move(file+".backup", file);
        }
    }
}
