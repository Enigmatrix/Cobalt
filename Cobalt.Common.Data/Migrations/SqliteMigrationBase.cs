using System;
using System.Collections.Generic;
using System.Data.SQLite;
using System.Text;

namespace Cobalt.Common.Data.Migrations
{
    public abstract class SqliteMigrationBase : MigrationBase
    {
        protected SqliteMigrationBase(SQLiteConnection conn)
        {
            Connection = conn;
        }

        public TableMeta Table(string name)
        {
            return new TableMeta(name);
        }

        public class ExecMeta
        {
            public List<TableMeta> CreatingTables { get; }
            public List<IndexMeta> CreatingIndexes { get; }

            public ExecMeta()
            {
                CreatingTables = new List<TableMeta>();
                CreatingIndexes = new List<IndexMeta>();
            }

            public ExecMeta Create(TableMeta table){
                CreatingTables.Add(table);
                return this;
            }

            public ExecMeta Create(IndexMeta index)
            {
                CreatingIndexes.Add(index);
                return this;
            }

            public ExecMeta Insert<T>(string table, T value)
            {
                

                return this;
            }

            public void Run(){

            }
        }

        public enum Delete
        {
            Cascade,
            SetNull
        }

        public class TableMeta
        {
            public string Name { get; set; }
            public List<Field> Fields { get; set; }
            public List<Constraint> Constraints { get; set; }

            public TableMeta Field(string name, string type, string ext = null)
            {
                Fields.Add(new Field(name, type, ext));
                return this;
            }

            public TableMeta Field<T>(string name, string ext = null) where T: SqliteType {
                return Field(name, typeof(T).Name, ext);
            }

            public TableMeta PkAutoInc(string name)
            {
                return Field<Integer>(name, "primary key autoincrement");
            }

            public TableMeta ForeignKey(string fkName, string tblName, string tblKey, Delete del = Delete.SetNull)
            {
                Constraints.Add(new ForeignKey(fkName, tblName, tblKey, del));
                return this;
            }

            public TableMeta Keys(params string[] names)
            {
                Constraints.Add(new Keys(names));
                return this;
            }

            public TableMeta(string name)
            {
                Name = name;
            }
        }

        public class Field
        {
            public string Name { get; }
            public string Type { get; }
            public string Extension { get; }

            public Field(string name, string type, string ext)
            {
                Name = name;
                Type = type;
                Extension = ext;
            }


            public override string ToString()
            {
                return $"{Name} {Type} {Extension ?? ""},";
            }
        }

        public IndexMeta Index(string name, string table, params string[] fields)
        {
            return new IndexMeta(name, table, fields);
        }

        public class IndexMeta
        {
            public string Name { get; set; }
            public string Table { get; set; }

            public string[] Fields { get; set; }

            public Index(string name, string tbl, string[] flds)
            {
                Name = name;
                Table = tbl;
                Fields = flds;
            }
        }

        public class SqliteType { }
        public class Text : SqliteType { }
        public class Integer : SqliteType { }
        public class Blob : SqliteType { }

        public class Constraint {}

        public class ForeignKey : Constraint
        {
            public string FkName { get; set; }
            public string TableName { get; set; }
            public string TableKey { get; set; }

            public Delete DeleteMode { get; set; }

            public ForeignKey(string fkName, string tblName, string tblKey, Delete deleteMode)
            {
                FkName = fkName;
                TableName = tblName;
                TableKey = tblKey;
                DeleteMode = deleteMode;
            }
        }

        public class Keys : Constraint
        {
            public string[] Names { get; set; }

            public Keys(string[] names)
            {
                Names = names;
            }
        }

        protected SQLiteConnection Connection { get; }
    }
}
