using System;
using System.Collections.Generic;
using System.Data.SQLite;
using System.Text;
using Dapper;

namespace Cobalt.Common.Data.Migrations
{
    public class SqliteMigrationV1 : SqliteMigrationBase
    {
        public SqliteMigrationV1(SQLiteConnection conn) : base(conn) { }

        public override long Version { get; } = 1;

        public override void Run()
        {
            new ExecMeta()
            .Create(Table("Migration")
                .Field<Integer>("Version"))
            .Create(Table("App")
                .PkAutoInc("Id")
                .Field<Text>("Name")
                .Field<Text>("Color")
                .Field<Text>("Path")
                .Field<Blob>("Icon"))
            .Create(Table("AppUsage")
                .PkAutoInc("Id")
                .Field<Integer>("AppId")
                .Field<Integer>("Start")
                .Field<Integer>("End")
                .Field<Integer>("StartReason")
                .Field<Integer>("EndReason")
                .Field<Integer>("UsageType")
                .ForeignKey("AppId", "App", "Id"))
            .Create(Table("Tag")
                .PkAutoInc("Id")
                .Field<Text>("Name")
                .Field<Text>("ForegroundColor")
                .Field<Text>("BackgroundColor"))
            .Create(Table("AppTag")
                .Field<Integer>("AppId")
                .Field<Integer>("TagId")
                .Keys("AppId", "TagId")
                .ForeignKey("AppId", "App", "Id", Delete.Cascade)
                .ForeignKey("TagId", "Tag", "Id", Delete.Cascade))
            .Create(Table("Alert")
                .PkAutoInc("Id")
                .Field<Integer>("AppId")
                .Field<Integer>("TagId")
                .Field<Integer>("MaxDuration")
                .Field<Integer>("Enabled")
                .Field<Integer>("ActionType")
                .Field<Text>("ActionParam")
                .Field<Integer>("TimeRangeType")
                .Field<Text>("TimeRangeParam1")
                .Field<Text>("TimeRangeParam2")
                .ForeignKey("AppId", "App", "Id", Delete.Cascade)
                .ForeignKey("TagId", "Tag", "Id", Delete.Cascade))
            .Create(Table("Reminder")
                .PkAutoInc("Id")
                .Field<Integer>("AlertId")
                .Field<Integer>("Offset")
                .Field<Integer>("ActionType")
                .Field<Text>("ActionParam")
                .ForeignKey("AlertId", "Alert", "Id", Delete.Cascade))
            .Create(Index("AppPathIdx", "App", "Path"))
            .Create(Index("StartTimestampIdx", "AppUsage", "Start", "End"))
            .Create(Index("EndTimestampIdx", "AppUsage", "End", "Start"))
            .Insert("Migration", new { Current = 1 })
            .Run();
        }
    }
}
