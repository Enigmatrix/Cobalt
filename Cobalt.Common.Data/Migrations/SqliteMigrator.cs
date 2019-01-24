using System.Collections.Generic;
using System.Data.SQLite;
using Dapper;

namespace Cobalt.Common.Data.Migrations
{
    public class SqliteMigrator : MigratorBase
    {
        public SqliteMigrator(SQLiteConnection conn)
        {
            Connection = conn;
        }

        private SQLiteConnection Connection { get; }

        protected override List<IDbMigration> Migrations => new List<IDbMigration>
        {
            new SqliteMigrationV1(Connection)
        };

        protected override long GetVersion()
        {
            try
            {
                return Connection.ExecuteScalar<long>("select Version from Migration");
            }
            catch (SQLiteException e) when (e.Message.Contains("no such table: Migration"))
            {
                return 0;
            }
        }
    }
}