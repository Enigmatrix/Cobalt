namespace Cobalt.Common.Data

open System.Data
open Microsoft.Data.Sqlite
open Cobalt.Common.Data.Entities
open Cobalt.Common.Data.Materializers
open Helpers
open System
open System.Reactive.Linq

type IDatabase =
    inherit IDisposable
    abstract member FindApp : Id -> App
    abstract member FindTag : Id -> Tag
    abstract member FindSession : Id -> Session
    abstract member FindUsage : Id -> Usage
    abstract member Find<'a> : Id -> 'a

    abstract member AppDurations : start: DateTime voption * endt: DateTime voption -> IObservable<TimeSpan * App>

type Database(conn: SqliteConnection) =
    let mats = Materializers(conn)

    let timeRange (start: DateTime voption) (endt: DateTime voption) =
        let toFt (x: DateTime) = x.ToFileTime()
        (start |> ValueOption.map toFt |> ValueOption.defaultValue 0L,
            endt |> ValueOption.map toFt |> ValueOption.defaultValue Int64.MaxValue)

    interface IDatabase with
        member _.Find id = mats.Materializer().Find id

        member _.FindApp id = mats.App.Find id
        member _.FindSession id = mats.Session.Find id
        member _.FindTag id = mats.Tag.Find id
        member _.FindUsage id = mats.Usage.Find id

        member _.AppDurations(start, endt) = 
            let (start, endt) = timeRange start endt
            reader
                (cmd "select sum(
                            (case when End > @end then @end else End end) - 
                            (case when Start < @start then @start else Start end)),
                            a.Id, a.Name, a.Description, 1, a.Color, a.Identity_Tag, a.Identity_Text1
                        from Usages u, Sessions s, Apps a
                        where (Start <= @end and End >= @start) and a.Id = s.AppId and s.Id = u.SessionId
                        group by a.Id" conn
                    |> param "start" start
                    |> param "end" endt)
                (fun r -> (TimeSpan.FromTicks(r.GetInt64(0)), mats.App.MaterializeWithOffset 1 r ))
            
        member _.Dispose() = conn.Dispose()
