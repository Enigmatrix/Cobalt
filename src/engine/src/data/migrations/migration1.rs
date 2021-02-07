use rusqlite::*;
use util::{Result, *};

pub fn run(conn: &mut Connection) -> Result<()> {
    conn.execute_batch(
        "begin;
        create table Migrations (
            Version integer not null
        );

        create table Apps (
            Id integer primary key autoincrement,
            Name text,
            Description text,
            Icon blob,
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

        create table Alerts (
            Id integer primary key autoincrement,
            Target_Type integer not null,
            Target_AppId integer,
            Target_TagId integer,
            TimeFrame_Type integer not null,
            TimeFrame_Integer1 integer not null,
            TimeFrame_Integer2 integer not null,
            TimeFrame_Integer3 integer,
            UsageLimit integer not null,
            ExceededReaction_Type integer not null,
            ExceededReaction_Text1 text,

            foreign key (Target_AppId) references Apps(Id),
            foreign key (Target_TagId) references Tags(Id)
        );

        create index UsageStartEnd on Usages (Start, End);
        create index UsageEndStart on Usages (End, Start);
        create unique index AppIdentity on Apps (Identity_Tag, Identity_Text1);
        
        insert into Migrations values (1);
    commit;",
    )
    .with_context(|| "Executing SQLite batch statement")
}
