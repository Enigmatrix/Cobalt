module Data

open Xunit
open Swensen.Unquote
open Microsoft.Data.Sqlite
open Cobalt.Common.Data
open Cobalt.Common.Data.Entities
open System.Reactive.Linq
open System.Linq

let createDb () =
    let conn = new SqliteConnection("Data Source=C:\\Users\\enigm\\Desktop\\NANI.db")
    conn.Open()
    (new SqliteCommand("PRAGMA journal_mode='wal'", conn)).ExecuteNonQuery() |> ignore
    new Database(conn) :> IDatabase

[<Fact>]
let ``Can create AppIdentities`` () =
    use db = createDb()
    let app = db.FindApp 1L
    test <@ app = db.Find<App> 1L @>
    test <@ app <> Unchecked.defaultof<App> @>

[<Fact>]
let ``Get AppDurations stream`` () =
    use db = createDb()
    let durs = db.AppDurations (ValueNone, ValueNone)
    test <@ durs.ToEnumerable() <> Enumerable.Empty() @>
