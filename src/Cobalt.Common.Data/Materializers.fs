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

    abstract member InsertCmd: SqliteCommand
    default _.InsertCmd = raise (System.NotImplementedException())
    abstract member UpdateCmd: SqliteCommand
    default _.UpdateCmd = raise (System.NotImplementedException())

    member x.Update entity = 
        let cmd = x.UpdateCmd |> reset
        x.Dematerialize entity cmd.Parameters
        cmd.ExecuteNonQuery() |> ignore

    member x.Insert entity = 
        let cmd = x.UpdateCmd |> reset
        x.Dematerialize entity cmd.Parameters
        cmd.ExecuteScalar() :?> int64

    abstract member Dematerialize: 'a -> SqliteParameterCollection -> unit
    default _.Dematerialize a ps = raise (System.NotImplementedException())

type AppMaterializer(conn) =
    inherit MaterializerBase<App>(conn)
    let find = cmd "select * from Apps where Id = @id" conn |> prepare
    let update = cmd
                    """update Apps set
                            Name = @Name,
                            Description = @Description,
                            Color = @Color
                        where Id = @Id""" conn |> prepare

    override _.FindCmd = find

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

    override _.UpdateCmd = update

    override _.Dematerialize app ps =
        ps.AddWithValue("Id", app.Id) |> ignore
        ps.AddWithValue("Name", ValueOption.toObj app.Name) |> ignore
        ps.AddWithValue("Description", ValueOption.toObj app.Description) |> ignore
        ps.AddWithValue("Color", ValueOption.toObj app.Color) |> ignore

type TagMaterializer(conn) =
    inherit MaterializerBase<Tag>(conn)
    let find = cmd "select * from Tags where Id = @id" conn |> prepare
    let update = cmd """
                    update Tags set
                        Name = @Name,
                        Description = @Description,
                        Color = @Color
                    where Id = @Id""" conn |> prepare

    override _.FindCmd = find
    override _.Materialize reader = 
        {
            Id = reader.num 0;
            Name = reader.str 1;
            Description = reader.str 2;
            Color = reader.str 3;
        }

    override _.UpdateCmd = update
    override _.Dematerialize tag ps =
        ps.AddWithValue("Id", tag.Id) |> ignore
        ps.AddWithValue("Name", tag.Name) |> ignore
        ps.AddWithValue("Description", tag.Description) |> ignore
        ps.AddWithValue("Color", tag.Color) |> ignore

type SessionMaterializer(conn) =
    inherit MaterializerBase<Session>(conn)
    let find = cmd "select * from Sessions where Id = @id" conn |> prepare

    override _.FindCmd = find
    override _.Materialize reader = 
        {
            Id = reader.num 0;
            Title = reader.str 1;
            Arguments = reader.vopt_str 2;
            AppId = reader.num 3;
        }

type UsageMaterializer(conn) =
    inherit MaterializerBase<Usage>(conn)
    let find = cmd "select * from Usages where Id = @id" conn |> prepare

    override _.FindCmd = find
    override _.Materialize reader = 
        {
            Id = reader.num 0;
            Start = DateTime.FromFileTime(reader.num 1);
            End = DateTime.FromFileTime(reader.num 2);
            DuringIdle = reader.num 3 <> 0L
            SessionId = reader.num 4;
        }

type AlertMaterializer(conn) =
    inherit MaterializerBase<Alert>(conn)
    let find =  cmd "select * from Alerts where Id = @id" conn |> prepare
    let update =  cmd """
                    update Alerts
                        Id = @Id,
                        Target_Type = @Target_Type,
                        Target_AppId = @Target_AppId,
                        Target_TagId = @Target_TagId,
                        TimeFrame_Type = @TimeFrame_Type,
                        TimeFrame_Integer1 = @TimeFrame_Integer1,
                        TimeFrame_Integer2 = @TimeFrame_Integer2,
                        TimeFrame_Integer3 = @TimeFrame_Integer3,
                        UsageLimit = @UsageLimit,
                        ExceededReaction_Type = @ExceededReaction_Type,
                        ExceededReaction_Text1 = @ExceededReaction_Text1
                    where Id = @Id""" conn |> prepare
    let insert =  cmd """
                    insert into Alerts (
                        Target_Type,
                        Target_AppId,
                        Target_TagId,
                        TimeFrame_Type,
                        TimeFrame_Integer1,
                        TimeFrame_Integer2,
                        TimeFrame_Integer3,
                        UsageLimit,
                        ExceededReaction_Type)
                        values (
                        @Target_Type,
                        @Target_AppId,
                        @Target_TagId,
                        @TimeFrame_Type,
                        @TimeFrame_Integer1,
                        @TimeFrame_Integer2,
                        @TimeFrame_Integer3,
                        @UsageLimit,
                        @ExceededReaction_Type,
                        @ExceededReaction_Text1)""" conn |> prepare

    override _.FindCmd = find
    override _.Materialize reader = 
        {
            Id = reader.num 0;
            Target = match (reader.num 1) with
                        | 0L -> Target.App (reader.num 2)
                        | 1L -> Target.Tag (reader.num 3)
                        | _ -> failwith "unsupported target tag";
            TimeFrame = match (reader.num 4) with
                        | 0L -> TimeFrame.Once (DateTime.FromFileTime(reader.num 5), DateTime.FromFileTime(reader.num 6))
                        | 1L -> TimeFrame.Repeated (enum (reader.num 7 |> int), TimeSpan.FromTicks(reader.num 5), TimeSpan.FromTicks(reader.num 6))
                        | _ -> failwith "unsupported timeframe tag";
            UsageLimit = TimeSpan.FromTicks(reader.num 8);
            ExceededReaction = match (reader.num 9) with
                        | 0L -> Reaction.Kill
                        | 1L -> Reaction.Message (reader.str 10)
                        | _ -> failwith "unsupported target tag";
        }

    override _.InsertCmd = insert
    override _.UpdateCmd = update
    override _.Dematerialize alert ps =
        ps.AddWithValue("Id", alert.Id) |> ignore

        match alert.Target with
            | Target.App appid ->
                ps.AddWithValue("Target_Type", 0L) |> ignore
                ps.AddWithValue("Target_AppId", appid) |> ignore
                ps.AddWithValue("Target_TagId", null) |> ignore
            | Target.Tag tagid ->
                ps.AddWithValue("Target_Type", 1L) |> ignore
                ps.AddWithValue("Target_AppId", null) |> ignore
                ps.AddWithValue("Target_TagId", tagid) |> ignore

        match alert.TimeFrame with
            | TimeFrame.Once (start, endt) ->
                ps.AddWithValue("TimeFrame_Type", 0L) |> ignore
                ps.AddWithValue("TimeFrame_Integer1", start.ToFileTime()) |> ignore
                ps.AddWithValue("TimeFrame_Integer2", endt.ToFileTime()) |> ignore
                ps.AddWithValue("TimeFrame_Integer3", null) |> ignore
            | TimeFrame.Repeated (repeat, start, endt) ->
                ps.AddWithValue("TimeFrame_Type", 1L) |> ignore
                ps.AddWithValue("TimeFrame_Integer1", start.Ticks) |> ignore
                ps.AddWithValue("TimeFrame_Integer2", endt.Ticks) |> ignore
                ps.AddWithValue("TimeFrame_Integer3", repeat) |> ignore

        ps.AddWithValue("UsageLimit", alert.UsageLimit.Ticks) |> ignore

        match alert.ExceededReaction with
            | Reaction.Kill ->
                ps.AddWithValue("ExceededReaction_Type", 0L) |> ignore
                ps.AddWithValue("ExceededReaction_Text1", null) |> ignore
            | Reaction.Message msg ->
                ps.AddWithValue("ExceededReaction_Type", 1L) |> ignore
                ps.AddWithValue("ExceededReaction_Text1", msg) |> ignore

type Materializers(conn) = 

    member _.App = AppMaterializer(conn)
    member _.Tag = TagMaterializer(conn)
    member _.Session = SessionMaterializer(conn)
    member _.Usage = UsageMaterializer(conn)
    member _.Alert = AlertMaterializer(conn)

    member x.Materializer<'a> () = 
        let v = match typeof<'a> with
                | t when t = typeof<App> -> x.App |> box
                | t when t = typeof<Tag> -> x.Tag |> box
                | t when t = typeof<Session> -> x.Session |> box
                | t when t = typeof<Usage> -> x.Usage |> box
                | t when t = typeof<Alert> -> x.Alert |> box
                | _ -> failwith "unreachable"
        v :?> MaterializerBase<'a>
