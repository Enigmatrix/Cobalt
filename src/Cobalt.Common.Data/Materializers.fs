namespace Cobalt.Common.Data.Materializers

open System
open Microsoft.Data.Sqlite
open Cobalt.Common.Data.Entities
open Helpers

type Reader(offset: int, inner: SqliteDataReader) =
    member _.num idx =
        inner.GetInt64(offset + idx)
    member _.str idx =
        inner.GetString(offset + idx)
    member _.vopt_str idx =
        if inner.IsDBNull(offset + idx) then ValueNone else ValueSome (inner.GetString(offset + idx))

[<AbstractClass>]
type MaterializerBase<'a>(conn) =
    member _.Connection = conn
    member x.MaterializeWithOffset (offset: int) (reader: SqliteDataReader) =
        let reader = Reader(offset, reader)
        x.Materialize reader 
    abstract member Materialize: Reader -> 'a

    abstract member FindCmd: SqliteCommand
    member x.Find (id: Id) =
        single
            (x.FindCmd |> reset |> param "id" id)
            (x.MaterializeWithOffset 0)

type AppMaterializer(conn) =
    inherit MaterializerBase<App>(conn)

    override _.FindCmd = cmd "select * from Apps where Id = @id" conn |> prepare

    override _.Materialize reader = 
        {
            Id = reader.num 0;
            Name = reader.vopt_str 1;
            Description = reader.vopt_str 2;
            Color = reader.vopt_str 4;
            Identity = match reader.num 5 with
                        | 0L -> Win32 (reader.str 6)
                        | 1L -> UWP (reader.str 6)
                        | _ -> failwith "unsupported identity tag"
        }

type TagMaterializer(conn) =
    inherit MaterializerBase<Tag>(conn)

    override _.FindCmd = cmd "select * from Tags where Id = @id" conn |> prepare

    override _.Materialize reader = 
        {
            Id = reader.num 0;
            Name = reader.str 1;
            Description = reader.str 2;
            Color = reader.str 3;
        }

type SessionMaterializer(conn) =
    inherit MaterializerBase<Session>(conn)

    override _.FindCmd = cmd "select * from Sessions where Id = @id" conn |> prepare

    override _.Materialize reader = 
        {
            Id = reader.num 0;
            Title = reader.str 1;
            Arguments = reader.vopt_str 2;
            AppId = reader.num 3;
        }

type UsageMaterializer(conn) =
    inherit MaterializerBase<Usage>(conn)

    override _.FindCmd = cmd "select * from Usage where Id = @id" conn |> prepare

    override _.Materialize reader = 
        {
            Id = reader.num 0;
            Start = DateTime.FromFileTime(reader.num 1);
            End = DateTime.FromFileTime(reader.num 2);
            DuringIdle = reader.num 3 <> 0L
            SessionId = reader.num 4;
        }

type Materializers(conn) = 

    member _.App = AppMaterializer(conn)
    member _.Tag = TagMaterializer(conn)
    member _.Session = SessionMaterializer(conn)
    member _.Usage = UsageMaterializer(conn)

    member x.Materializer<'a> () = 
        let v = match typeof<'a> with
                | t when t = typeof<App> -> x.App |> box
                | t when t = typeof<Tag> -> x.Tag |> box
                | t when t = typeof<Session> -> x.Session |> box
                | t when t = typeof<Usage> -> x.Usage |> box
                | _ -> failwith "unreachable"
        v :?> MaterializerBase<'a>
