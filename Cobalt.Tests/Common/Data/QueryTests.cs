using System.Collections.Generic;
using System.Data.SQLite;
using System.Linq;
using System.Reactive.Linq;
using Cobalt.Common.Data;
using Cobalt.Common.Data.Migration.Sqlite;
using Cobalt.Common.Data.Repository;
using Xunit;

namespace Cobalt.Tests.Common.Data
{
    public class QueryTests
    {
        [Fact]
        public void TestGetAppsIncludingTags()
        {
            var conn = new SQLiteConnection("Data Source=dat2.db").OpenAndReturn();
            var repo = new SqliteRepository(conn, new SqliteMigrator(conn));
            var o = repo.GetApps();
            var appList = new List<List<Tag>>(o.Select(x => new List<Tag>(x.Tags.ToEnumerable())).ToEnumerable());
            Assert.Equal(3, appList.Max(t => t.Count));
        }
    }
}