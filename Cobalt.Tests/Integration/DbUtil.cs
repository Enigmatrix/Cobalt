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
            if (File.Exists(file))
            {
                File.Move(file, file+".backup");
            }
            conn = IoCService.Instance.Resolve<SQLiteConnection>();
        }

        public SQLiteConnection Connection => conn;

        public void Dispose()
        {
            var file = conn.FileName;
            var backup = file + ".backup";
            conn.Dispose();
            delagain:
            try
            {
                File.Delete(file);
            }
            catch
            {
                goto delagain;
            }
            if (File.Exists(backup))
            {
                File.Move(backup, file);
            }
        }
    }
}
