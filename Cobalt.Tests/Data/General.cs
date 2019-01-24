using System.Data.SQLite;
using Cobalt.Common.Data.Migrations;
using Xunit;

namespace Cobalt.Tests
{
    public class General
    {
        [Fact]
        public void MigrationWorks()
        {
            var conn = new SQLiteConnection("Data Source=test.db");
            var migration = new SqliteMigrator(conn);
            migration.Run();
        }
    }
}