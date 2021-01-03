namespace Cobalt.Common.Data

open Cobalt.Common.Data.Entites
open Microsoft.Data.Sqlite

type IDatabase = 
    inherit System.IDisposable
    abstract member Find<'a> : int64 -> 'a

    abstract member FindAppByIdentification: AppIdentity -> App voption

    abstract member AddTagToApp : App -> Tag -> unit
    abstract member RemoveTagToApp : App -> Tag -> unit

type Database (conn: SqliteConnection) =
    let cmd sql = new SqliteCommand(sql, conn)
