namespace Cobalt.Common.Data

open Microsoft.Data.Sqlite
open Cobalt.Common.Data.Entities
open Cobalt.Common.Data.Materializers
open Helpers
open System
open System.IO
open Cobalt.Common.Native

type DateTimeBound =
    | Unbounded
    | Bound of Value: DateTime

type DateTimeRange = {
    Start: DateTimeBound;
    End: DateTimeBound;
}
with member this.Bounds() =
        match this with
            | { Start = Unbounded   ; End = Unbounded  } -> (0L                , Int64.MaxValue   )
            | { Start = Bound start ; End = Unbounded  } -> (start.ToFileTime(), Int64.MaxValue   )
            | { Start = Unbounded   ; End = Bound endt } -> (0L                , endt.ToFileTime())
            | { Start = Bound start ; End = Bound endt } -> (start.ToFileTime(), endt.ToFileTime())

type IdleOptions =
    | Idle of IsIdle: bool
    | Irrelevant
    with member this.map fn def =
                match this with
                    | Idle v -> fn v
                    | Irrelevant -> def

type IDatabase =
    inherit IDisposable
    abstract member AppIcon : Id -> Stream
    abstract member FindApp : Id -> App
    abstract member FindTag : Id -> Tag
    abstract member FindSession : Id -> Session
    abstract member FindUsage : Id -> Usage
    abstract member FindAlert : Id -> Alert
    abstract member Find<'a> : Id -> 'a
    
    abstract member InsertTag: Tag -> unit
    abstract member InsertAlert: Alert -> unit

    abstract member UpdateApp: App -> unit
    abstract member UpdateTag: Tag -> unit
    abstract member UpdateAlert: Alert-> unit

    abstract member AppDurations : DateTimeRange -> IObservable<struct (TimeSpan * App)>
    abstract member SessionsDurations : DateTimeRange * IdleOptions -> IObservable<struct (TimeSpan * Session)>
    abstract member Usages : DateTimeRange * IdleOptions -> IObservable<Usage>

type Database(conn: SqliteConnection) =
    let mats = Materializers(conn)

    let migrated =
        let err = Ffi.String()
        Data.migrate(conn.Handle.DangerousGetHandle(), ref err)
        let migrated = err.Buffer <> nativeint 0
        if not migrated then failwith "Migration failed"


    interface IDatabase with
        member _.AppIcon id = new SqliteBlob(conn, "Apps", "Icon", id, false) :> Stream

        member _.Find id = mats.Materializer().Find id

        member _.FindApp id = mats.App.Find id
        member _.FindSession id = mats.Session.Find id
        member _.FindTag id = mats.Tag.Find id
        member _.FindUsage id = mats.Usage.Find id
        member _.FindAlert id = mats.Alert.Find id

        member _.InsertAlert alert = alert.Id <- mats.Alert.Insert alert
        member _.InsertTag tag = tag.Id <- mats.Tag.Insert tag

        member _.UpdateAlert alert = mats.Alert.Update alert
        member _.UpdateApp app = mats.App.Update app
        member _.UpdateTag tag = mats.Tag.Update tag

        member _.AppDurations range = 
            let (start, endt) = range.Bounds()
            reader
                (cmd "select sum( min(End, @end) - max(Start, @start) ) Duration,
                            a.Id, a.Name, a.Description, 1, a.Color, a.Identity_Tag, a.Identity_Text1
                        from Usages u, Sessions s, Apps a
                        where (Start <= @end and End >= @start) and a.Id = s.AppId and s.Id = u.SessionId
                        group by a.Id" conn
                    |> param "start" start
                    |> param "end" endt)
                (fun r -> struct (TimeSpan.FromTicks(r.GetInt64(0)), mats.App.MaterializeWithOffset 1 r ))

        member _.Usages(range, idle) =
            let (start, endt) = range.Bounds()
            reader
                (cmd $"""select Id,
                            max(Start, @start) Start,
                            min(End, @end) End,
                            DuringIdle,
                            SessionId
                        from Usages { idle.map (sprintf "where DuringIdle = %b and") "" }
                        where (Start <= @end and End >= @start)
                        """ conn
                    |> param "start" start
                    |> param "end" endt)
                (mats.Usage.MaterializeWithOffset 0)
            
        member _.SessionsDurations(range, idle) =
            let (start, endt) = range.Bounds()
            reader
                (cmd $"""select sum( min(End, @end) - max(Start, @start) ) Duration,
                            s.Id, s.Title, s.Arguments, s.AppId
                        from Usages u, Sessions s
                        { idle.map (sprintf "where u.DuringIdle = %b and") "" }
                        where (Start <= @end and End >= @start) and s.AppId and s.Id = u.SessionId
                        group by s.Id""" conn
                    |> param "start" start
                    |> param "end" endt)
                (fun r -> struct (TimeSpan.FromTicks(r.GetInt64(0)), mats.Session.MaterializeWithOffset 1 r ))

        member _.Dispose() = conn.Dispose()
