using System;
using System.Collections.Generic;
using System.Data;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Cobalt.Common.Data.Migration.Sqlite
{
    public class SqliteMigrationV1 : SqliteMigrationBase
    {
        public override int Order { get; } = 1;

        public SqliteMigrationV1(IDbConnection connection) : base(connection)
        {
        }

        public override void ExecuteMigration()
        {
            ExecuteSql(
                Table("Migrations",
                    Field("LatestMigration", Integer())),
                Insert("Migrations", "1"),

                Table("App",
                    Field("Id", Integer(), PkAutoInc()),
                    Field("Name", Text()),
                    Field("Path", Text(), NotNullUnique())),

                Table("Tag",
                    Field("Id", Integer(), PkAutoInc()),
                    Field("Name", Text(), NotNullUnique())),

                Table("AppTag",
                    Field("AppId", Integer()),
                    Field("TagId", Integer()),
                    Key("AppId, TagId"),
                    ForeignKey("AppId", "App(Id)"),
                    ForeignKey("TagId", "Tag(Id)")),

                Table("AppUsage",
                    Field("Id", Integer(), PkAutoInc()),
                    Field("AppId", Integer()),
                    Field("UsageType", Integer()),
                    Field("StartTimestamp", Integer()),
                    Field("EndTimestamp", Integer()),
                    Field("AppUsageStartReason", Integer()),
                    Field("AppUsageEndReason", Integer()),
                    ForeignKey("AppId", "App(Id)")),

                Index("AppPathIdx", "App(Path)"),
                Index("StartTimestampIdx", "AppUsage(StartTimestamp, EndTimestamp)"),
                Index("EndTimestampIdx", "AppUsage(EndTimestamp, StartTimestamp)"));
        }
    }
}
