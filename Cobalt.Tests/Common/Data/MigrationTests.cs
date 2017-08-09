using System.Data.SQLite;
using Cobalt.Common.Data.Migration.Sqlite;
using Xunit;

namespace Cobalt.Tests.Common.Data
{
    public class MigrationTests
    {
        [Fact]
        public void PassingTest()
        {
            var migrator = new SqliteMigrator(new SQLiteConnection("Data Source=migrationDat.db").OpenAndReturn());
            migrator.Migrate();
        }
    }
}