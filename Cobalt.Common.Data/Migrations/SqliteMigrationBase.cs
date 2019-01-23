using System;
using System.Collections.Generic;
using System.Data.SQLite;
using System.Linq;
using System.Reflection;
using System.Text;
using Dapper;

namespace Cobalt.Common.Data.Migrations
{
    public abstract class SqliteMigrationBase : MigrationBase
    {
        protected SqliteMigrationBase(SQLiteConnection conn)
        {
            Connection = conn;
            Sql = new StatementCollection();
        }

        protected abstract void Build();

        public override void Run()
        {
            Build();
            Sql.Execute(Connection);
        }

        public enum Delete
        {
            Cascade,
            SetNull
        }

        public abstract class SqliteType { }
        public class Text : SqliteType { }
        public class Integer : SqliteType { }
        public class Blob : SqliteType { }


        protected SQLiteConnection Connection { get; }
        public StatementCollection Sql { get; }

        public class StatementCollection
        {
            private List<Statement> Statements { get; }

            public StatementCollection()
            {
                Statements = new List<Statement>();
            }

            public void Add(Statement stmt)
            {
                Statements.Add(stmt);
            }

            public void Execute(SQLiteConnection connection)
            {
                var sql = string.Join(";", Statements.Select(x => x.ToSql()));
                connection.Execute(sql);
            }
        }

        public abstract class Statement
        {
            public abstract string ToSql();
        }

        public class TableStatement : Statement
        {
            private readonly List<string> _innards = new List<string>();
            public string Name { get; }
            public TableStatement(string name)
            {
                Name = name;
            }

            public TableStatement Field(string name, string type, string extra = "")
            {
                _innards.Add($"`{name}` {type} {extra}");
                return this;
            }

            public TableStatement Field<T>(string name, string extra = "")
                where T : SqliteType
            {
                return Field(name, typeof(T).Name, extra);
            }

            public TableStatement PkAutoInc(string name = "Id")
            {
                return Field<Integer>(name, "primary key autoincrement");
            }

            public TableStatement ForeignKey(string fld, string otbl, string ofld = "Id", Delete? delMode = null)
            {
                var delStr = "";
                switch (delMode)
                {
                    case Delete.SetNull:
                        delStr = "on delete set null";
                        break;
                    case Delete.Cascade:
                        delStr = "on delete cascade";
                        break;
                    default:
                        break;
                }
                _innards.Add($"foreign key({fld}) references {otbl}({ofld}) {delStr}");
                return this;
            }

            public TableStatement Keys(params string[] flds)
            {
                _innards.Add($"primary key ({string.Join(",", flds)})");
                return this;
            }

            public override string ToSql()
            {
                return $"create table `{Name}`({string.Join(",",_innards)})";
            }
        }

        public class IndexStatement : Statement
        {
            private readonly string Inner;

            public IndexStatement(string name, string tbl, string[] flds)
            {
                Inner = $"create index {name} on {tbl}({string.Join(",", flds)})";
            }

            public override string ToSql() => Inner;
        }

        public class InsertStatement<T> : Statement
        {
            public InsertStatement(string name, T obj)
            {
                Name = name;
                Object = obj;
            }

            private string Name { get; }
            private T Object { get; }

            public override string ToSql()
            {
                var props = Props(Object).ToList();
                var names = string.Join(",", props.Select(x => x.Name));
                var values = string.Join(",", props.Select(x => x.Value));
                return $"insert into {Name}({names}) values ({values})";
            }
        }

        private static IEnumerable<(string Name, object Value)> Props(object t)
        {
            return t.GetType().GetProperties().Select(x => (x.Name, x.GetValue(t)));
        }

        public TableStatement Table(string name)
        {
            var stmt = new TableStatement(name);
            Sql.Add(stmt);
            return stmt;
        }

        public IndexStatement Index(string name, string tbl, string[] flds)
        {
            var stmt = new IndexStatement(name, tbl, flds);
            Sql.Add(stmt);
            return stmt;
        }

        public InsertStatement<T> Insert<T>(string tbl, T obj)
        {
            var stmt = new InsertStatement<T>(tbl, obj);
            Sql.Add(stmt);
            return stmt;
        }
    }
}
