using System;
using System.Data;
using System.Data.SQLite;
using Cobalt.Common.Util;

namespace Cobalt.Common.Data.Migration.Sqlite
{
    public class SqliteMigrationV3 : SqliteMigrationBase
    {
        public SqliteMigrationV3(IDbConnection connection) : base(connection)
        {
        }

        public override int Order { get; } = 3;

        public override void ExecuteMigration()
        {
            var appRes = new AppResource();
            using (var transaction = Connection.BeginTransaction())
            {
                ExecuteOnResult("select Id, Path from App", reader =>
                {
                    var id = reader.GetInt64(0);
                    var path = reader.IsDBNull(1) ? null : reader.GetString(1);
                    if (path == null) return;
                    try
                    {
                        var (icon, col) = appRes.GetAppIconAndColor(path);
                        var name = appRes.GetAppName(path);
                        ExecuteSql("update App set Name = @name, Color = @color, Icon = @icon where Id = @id",
                            ("name", name),
                            ("color", col),
                            ("icon", icon),
                            ("id", id));
                    }
                    catch (Exception)
                    {
                        return;
                    }
                });
                transaction.Commit();
            }
            ExecuteSql(
                Update("Migrations", ("LatestMigration", 3)));
        }
    }
}