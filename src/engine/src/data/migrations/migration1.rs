use rusqlite::*;
use util::{Result, *};

pub fn run(conn: &mut Connection) -> Result<()> {
    conn.execute_batch("begin;
        create table Migrations (
            Version integer not null
        );

        create table Apps (
            Id integer primary key autoincrement,
            Name text,
            Description text,
            Color text,
            Identity_Tag integer not null,
            Identity_Text1 text not null,

            unique (Identity_Tag, Identity_Text1)
        );

        create table Tags (
            Id integer primary key autoincrement,
            Name text not null,
            Description text not null,
            Color text not null
        );

        create table App_Tags (
            AppId integer not null,
            TagId integer not null,

            foreign key (AppId) references Apps (Id),
            foreign key (TagId) references Tags (Id),
            primary key (AppId, TagId)
        );

        create table Sessions (
            Id integer primary key autoincrement,
            Title text not null,
            Arguments text,
            AppId integer not null,

            foreign key (AppId) references Apps (Id)
        );

        create table Usages (
            Id integer primary key autoincrement,
            Start integer not null,
            End integer not null,
            DuringIdle integer not null,
            SessionId integer not null,

            foreign key (SessionId) references Sessions (Id)
        );

        create index UsageStartEnd on Usages (Start, End);
        create unique index AppIdentity on Apps (Identity_Tag, Identity_Text1);
        
        insert into Migrations values (1);
    commit;").with_context(|| "Executing SQLite batch statement")
}