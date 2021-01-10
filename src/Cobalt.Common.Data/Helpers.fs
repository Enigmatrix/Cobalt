module Helpers

open System.Data
open Microsoft.Data.Sqlite
open System.Reactive.Linq
open System

let cmd sql conn = new SqliteCommand(sql, conn)

let prepare (cmd: SqliteCommand) = cmd.Prepare(); cmd

let param name value (cmd: SqliteCommand) =
    cmd.Parameters.AddWithValue(name, value) |> ignore
    cmd

let reset (cmd: SqliteCommand) =
    cmd.Parameters.Clear()
    cmd

let single (cmd: SqliteCommand) fn =
    use reader = cmd.ExecuteReader()
    if reader.Read() then
        fn reader
    else
        failwithf "No rows returned for query: %s" cmd.CommandText

let reader<'a> (cmd: SqliteCommand) (fn: SqliteDataReader -> 'a) : IObservable<'a> =
    Observable.Create<'a>(fun (obs: IObserver<'a>) ->
        let reader = cmd.ExecuteReader()
        while reader.Read() do
            obs.OnNext(fn reader)
        obs.OnCompleted()
        Action(fun () -> reader.Dispose()));
