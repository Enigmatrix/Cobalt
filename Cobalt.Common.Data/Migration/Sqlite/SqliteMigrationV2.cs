using System;
using System.Collections.Generic;
using System.Data;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Cobalt.Common.Data.Migration.Sqlite
{
    public class SqliteMigrationV2 : SqliteMigrationBase
    {
        public SqliteMigrationV2(IDbConnection connection) : base(connection)
        {
        }

        public override int Order { get; } = 2;

        public override void ExecuteMigration()
        {
            ExecuteSql(
                Update("Migrations", ("LatestMigration", 2)),
                AlterAddColumn("App", ("Color", "char(7)"), ("Icon", "blob")),
                Table("Alert",
                    Field("Id", Integer(), PkAutoInc()),
                    Field("AppId", Integer()),
                    Field("TagId", Integer()),
                    //0 for app alert, 1 for tag alert
                    Field("AlertType", Integer()),

                    Field("MaxDuration", Integer()),
                    //offset from end of maxduration to start alerting user
                    Field("ReminderOffset", Integer()),
                    //0 for enable, 1 for disabled
                    Field("IsEnabled", Integer()),
                    //0 for send annoying message, 1 for kill process
                    Field("AlertAction", Integer()),

                    //if type!=once then start is offset from start of day
                    Field("Start", Integer()),
                    //if type!=once then end is a offset from end of day
                    Field("End", Integer()),
                    //0 for once, 1 for daily, 2 for weekly, 3 for weekday, 4 for weekend, 5 for monthly
                    Field("RepeatType", Integer()),

                    ForeignKey("AppId", "App(Id)"),
                    ForeignKey("TagId", "Tag(Id)")));
        }
    }
}
