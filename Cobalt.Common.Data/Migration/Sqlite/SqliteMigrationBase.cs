using System.Data;
using System.Data.SQLite;
using Cobalt.Common.Util;

namespace Cobalt.Common.Data.Migration.Sqlite
{
    public abstract class SqliteMigrationBase : MigrationBase
    {
        protected SqliteMigrationBase(IDbConnection connection) : base(connection)
        {
            Connection = connection as SQLiteConnection;
            if (connection == null)
                Throw.InvalidOperation("Connection must be of type SQLiteConnection for this Migration");
        }

        public SQLiteConnection Connection { get; set; }

        #region CRUD

        public string Insert(string table, params string[] values)
        {
            return $"insert into {table} values ({string.Join(",", values)});";
        }

        #endregion

        public void ExecuteSql(params string[] sql)
        {
            new SQLiteCommand(string.Join("\n", sql), Connection).ExecuteNonQuery();
        }

        #region Tables, Fields, and Index creations

        public string Field(string name, string type, string extra = "")
        {
            return $"`{name}` {type} {extra}";
        }

        public string Key(string key) => $"primary key ({key})";

        public string ForeignKey(string name, string refer)
        {
            return $"foreign key({name}) references {refer}";
        }

        public string Table(string name, params string[] data)
        {
            return $"create table {name}({string.Join(",", data)});";
        }

        public string Index(string name, string on)
        {
            return $"create index {name} on {on};";
        }

        #endregion

        #region Field extras

        public string PkAutoInc() => "primary key autoincrement";
        public string NotNullUnique() => "not null unique";

        #endregion

        #region Types

        public string Text() => "text";
        public string Integer() => "Integer";

        #endregion
    }
}