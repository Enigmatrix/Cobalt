using System.Data;
using System.Data.SQLite;
using Cobalt.Common.Util;

namespace Cobalt.Common.Data.Migration.Sqlite
{
    public class SqliteMigrator : Migrator
    {
        public SqliteMigrator(IDbConnection repo) : base(repo)
        {
        }

        protected override int CurrentMigration()
        {
            var conn = Connection as SQLiteConnection;
            if (conn == null) Throw.InvalidOperation("Connection must be type of SQLiteConnection for this Migrator");

            var cmd = new SQLiteCommand("select LatestMigration from Migrations", conn);
            try
            {
                //object to long, then to int
                return (int) (long) cmd.ExecuteScalar();
            }
            catch (SQLiteException)
            {
                return 0;
            }
        }
    }
}