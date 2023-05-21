module Tests

open System
open System.IO

open Xunit
open Swensen.Unquote
open Cobalt.Common.Data
open Microsoft.EntityFrameworkCore
open System.Diagnostics
open Microsoft.Data.Sqlite
open Cobalt.Common.Data.Entities

let fpath = "./test.db"

let migrate () =
    let migrate_cmd = sprintf "cargo run -q --bin migrator %s" fpath
    let info = new ProcessStartInfo("cmd.exe", sprintf "/C %s" migrate_cmd)
    use proc = Process.Start(info)
    proc.WaitForExit()

[<Fact>]
let ``init db`` () =
    File.Delete(fpath)
    use db = new CobaltContext(fpath)
    migrate()

type DataTests() =
    let db = new CobaltContext(fpath)
    let conn = db.Database.GetDbConnection()

    do File.Delete(fpath)
    do migrate()
    do conn.Open() |> ignore

    let exec_multiple sql headers (entities: obj list list) =
        use cmd = conn.CreateCommand()
        cmd.CommandText <- sql
        cmd.Prepare()
        for cols in entities do
            cmd.Parameters.Clear()
            for h, v in Seq.zip headers cols do
                cmd.Parameters.Add(new SqliteParameter(h, v)) |> ignore
            cmd.ExecuteNonQuery() |> ignore

    [<Fact>]
    let ``get apps`` () =
        let headers = ["@name"; "@description"; "@company"; "@color"; "@identity_tag"; "@identity_text0"]
        let apps: obj list list = [
            ["name0"; "description0"; "company0"; "color0"; 0; "win32_path0"];
            ["name1"; "description1"; "company1"; "color1"; 1; "uwp_aumid1"];
        ]
        exec_multiple "insert into app values (NULL, 1, 1, @name, @description, @company, @color, @identity_tag, @identity_text0, NULL)" headers apps

        let read_apps = db.Apps |> Seq.toList

        test <@ read_apps.[0].Name = "name0" @>
        test <@ read_apps.[0].Identity = AppIdentity.Win32("win32_path0") @>
        test <@ read_apps.[1].Name = "name1" @>
        test <@ read_apps.[1].Identity = AppIdentity.Uwp("uwp_aumid1") @>


    interface IDisposable with 
        member _.Dispose() =
            db.Dispose()
            conn.Close()

