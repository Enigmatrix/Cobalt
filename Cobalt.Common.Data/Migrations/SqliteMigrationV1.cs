using System.Data.SQLite;

namespace Cobalt.Common.Data.Migrations
{
    public class SqliteMigrationV1 : SqliteMigrationBase
    {
        public SqliteMigrationV1(SQLiteConnection conn) : base(conn)
        {
        }

        public override long Version { get; } = 1;

        protected override void Build()
        {
            Table("Migration")
                .Field<Integer>("Version");

            Table("App")
                .PkAutoInc()
                .Field<Text>("Name")
                .Field<Text>("Path")
                .Field<Text>("Color")
                .Field<Blob>("Icon");

            Table("AppUsage")
                .PkAutoInc()
                .Field<Integer>("AppId")
                .Field<Integer>("Start")
                .Field<Integer>("End")
                .Field<Integer>("StartReason")
                .Field<Integer>("EndReason")
                .ForeignKey("AppId", "App");

            Table("Tag")
                .PkAutoInc()
                .Field<Text>("Name")
                .Field<Text>("ForegroundColor")
                .Field<Text>("BackgroundColor");

            Table("AppTag")
                .Field<Integer>("AppId")
                .Field<Integer>("TagId")
                .Keys("AppId", "TagId")
                .ForeignKey("AppId", "App", delMode: Delete.Cascade)
                .ForeignKey("TagId", "Tag", delMode: Delete.Cascade);

            Table("Alert")
                .PkAutoInc()
                .Field<Integer>("AppId")
                .Field<Integer>("TagId")
                .Field<Integer>("MaxDuration")
                .Field<Integer>("Enabled")
                .Field<Integer>("ActionType")
                .Field<Text>("ActionParam")
                .Field<Integer>("TimeRangeType")
                .Field<Integer>("TimeRangeParam1")
                .Field<Integer>("TimeRangeParam2")
                .ForeignKey("AppId", "App", delMode: Delete.Cascade)
                .ForeignKey("TagId", "Tag", delMode: Delete.Cascade);

            Table("Reminder")
                .PkAutoInc()
                .Field<Integer>("AlertId")
                .Field<Integer>("Offset")
                .Field<Integer>("ActionType")
                .Field<Text>("ActionParam")
                .ForeignKey("AlertId", "Alert", delMode: Delete.Cascade);

            Index("AppPathIdx", "App", new[] {"Path"});
            Index("StartTimestampIdx", "AppUsage", new[] {"Start", "End"});
            Index("EndTimestampIdx", "AppUsage", new[] {"End", "Start"});

            Insert("Migration", new {Version = 1});
        }
    }
}