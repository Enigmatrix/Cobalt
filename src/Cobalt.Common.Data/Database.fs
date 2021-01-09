namespace Cobalt.Common.Data

open System.Data
open Microsoft.Data.Sqlite
open Cobalt.Common.Data.Entities
open Cobalt.Common.Data.Materializers
open Helpers
open System

type IDatabase =
    inherit IDisposable
    abstract member FindApp : Id -> App
    abstract member FindTag : Id -> Tag
    abstract member FindSession : Id -> Session
    abstract member FindUsage : Id -> Usage
    abstract member Find<'a> : Id -> 'a

type Database(conn: SqliteConnection) =
    let mats = Materializers(conn)

    interface IDatabase with
        member _.Find id = mats.Materializer().Find id

        member _.FindApp id = mats.App.Find id
        member _.FindSession id = mats.Session.Find id
        member _.FindTag id = mats.Tag.Find id
        member _.FindUsage id = mats.Usage.Find id

        member _.Dispose() = conn.Dispose()
