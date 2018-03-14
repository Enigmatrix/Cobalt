using System;
using System.Data;
using System.Data.SQLite;
using System.Linq;
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
            using (var transaction = Connection.BeginTransaction())
            {
                var s = new SQLiteCommand(string.Join("\n", sql), Connection);
                s.ExecuteNonQuery();
                s.Dispose();
                transaction.Commit();
            }
        }

        public void ExecuteSql(string sql, params (string, object)[] arg)
        {
            using (var transaction = Connection.BeginTransaction())
            {
                var s = new SQLiteCommand(sql, Connection);
                foreach (var (n, o) in arg)
                    s.Parameters.AddWithValue(n, o);
                s.ExecuteNonQuery();
                s.Dispose();
                transaction.Commit();
            }
        }

        public void ExecuteOnResult(string sql, Action<SQLiteDataReader> ac)
        {
            using (var s = new SQLiteCommand(sql, Connection))
            {
                var reader = s.ExecuteReader();
                while (reader.Read())
                {
                    ac(reader);
                }
                reader.Dispose();
            }
        }

        #region Tables, Fields, and Index creations

        public string Field(string name, string type, string extra = "")
        {
            return $"`{name}` {type} {extra}";
        }

        public string Key(string key)
        {
            return $"primary key ({key})";
        }

        public string ForeignKey(string name, string refer, params string[] p)
        {
            return $"foreign key({name}) references {refer} {string.Join(" ", p.Select(f => "on " + f))}";
        }

        public string Table(string name, params string[] data)
        {
            return $"create table {name}({string.Join(",", data)});";
        }

        public string Index(string name, string on)
        {
            return $"create index {name} on {on};";
        }

        public string Update(string table, params (string, object)[] v)
        {
            return $"update {table} set " + string.Join(", ", v.Select(x => x.Item1 + "=" + x.Item2)) + ";";
        }

        public string AlterAddColumn(string table, params (string, string)[] add)
        {
            return string.Join("", add.Select(x => $"alter table {table} add column {x.Item1} {x.Item2};"));
        }

        #endregion

        #region Field extras

        public string PkAutoInc()
        {
            return "primary key autoincrement";
        }

        public string Pk()
        {
            return "primary";
        }

        public string NotNullUnique()
        {
            return "not null unique";
        }

        public string NotNull()
        {
            return "not null";
        }

        #endregion

        #region Types

        public string Text()
        {
            return "text";
        }

        public string Integer()
        {
            return "Integer";
        }

        #endregion
    }
}