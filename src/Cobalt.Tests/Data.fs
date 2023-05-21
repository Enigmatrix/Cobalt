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
    let now = new DateTime(2023, 2, 4, 8, 0, 0)

    let exec_multiple sql headers (entities: obj list list) =
        use cmd = conn.CreateCommand()
        cmd.CommandText <- sql
        cmd.Prepare()
        for cols in entities do
            cmd.Parameters.Clear()
            for h, v in Seq.zip headers cols do
                cmd.Parameters.Add(new SqliteParameter(h, v)) |> ignore
            cmd.ExecuteNonQuery() |> ignore

    let seed () =
        let apps: obj list list = [
            ["name0"; "description0"; "company0"; "color0"; 0; "win32_path0"];
            ["name1"; "description1"; "company1"; "color1"; 1; "uwp_aumid1"];
        ]
        exec_multiple "insert into app values (NULL, 1, 1, @name, @description, @company, @color, @identity_tag, @identity_text0, NULL)"
            ["@name"; "@description"; "@company"; "@color"; "@identity_tag"; "@identity_text0"] apps

        let interaction_periods: obj list list = [
            [now.Ticks + 100L; now.Ticks + 200L; 2; 1];
            [now.Ticks + 300L; now.Ticks + 500L; 0; 1];
            [now.Ticks + 800L; now.Ticks + 900L; 10; 0];
        ]
        exec_multiple "insert into interaction_period values (NULL, @start, @end, @mouseclicks, @keystrokes)"
            ["@start"; "@end"; "@mouseclicks"; "@keystrokes"] interaction_periods

        let sessions: obj list list = [
            [1; "title0"; "cmd_line0"];
            [1; "title1"; "cmd_line1"];
            [2; "title2"; DBNull.Value];
        ]
        exec_multiple "insert into session values (NULL, @app, @title, @cmd_line)"
            ["@app"; "@title"; "@cmd_line"] sessions

        let usages: obj list list = [
            [1; now.Ticks + 100L; now.Ticks + 150L];
            [1; now.Ticks + 150L; now.Ticks + 200L];
            [2; now.Ticks + 200L; now.Ticks + 400L];
            [3; now.Ticks + 500L; now.Ticks + 600L];
        ]
        exec_multiple "insert into usage values (NULL, @session, @start, @end)"
            ["@session"; "@start"; "@end"] usages

    do
        File.Delete(fpath)
        migrate()
        conn.Open() |> ignore

        seed()


    [<Fact>]
    let ``get apps`` () =

        let read_apps = db.Apps |> Seq.toList

        test <@ read_apps.[0].Name = "name0" @>
        test <@ read_apps.[0].Description = "description0" @>
        test <@ read_apps.[0].Company = "company0" @>
        test <@ read_apps.[0].Color = "color0" @>
        test <@ read_apps.[0].Identity = AppIdentity.Win32("win32_path0") @>

        test <@ read_apps.[1].Name = "name1" @>
        test <@ read_apps.[1].Description = "description1" @>
        test <@ read_apps.[1].Company = "company1" @>
        test <@ read_apps.[1].Color = "color1" @>
        test <@ read_apps.[1].Identity = AppIdentity.Uwp("uwp_aumid1") @>

    [<Fact>]
    let ``get sessions`` () =

        let read_sessions = db.Sessions |> Seq.toList

        test <@ read_sessions.[0].Title = "title0" @>
        test <@ read_sessions.[0].CmdLine = "cmd_line0" @>

        test <@ read_sessions.[1].Title = "title1" @>
        test <@ read_sessions.[1].CmdLine = "cmd_line1" @>

        test <@ read_sessions.[2].Title = "title2" @>
        test <@ read_sessions.[2].CmdLine = null @>

    [<Fact>]
    let ``get sessions, include app`` () =

        let read_sessions = db.Sessions.Include (fun x -> x.App) |> Seq.toList

        test <@ read_sessions.[0].App.Id = 1 @>
        test <@ read_sessions.[0].App.Identity = AppIdentity.Win32("win32_path0") @>
        test <@ read_sessions.[0].Title = "title0" @>
        test <@ read_sessions.[0].CmdLine = "cmd_line0" @>

        test <@ read_sessions.[1].App.Id = 1 @>
        test <@ read_sessions.[1].App.Identity = AppIdentity.Win32("win32_path0") @>
        test <@ read_sessions.[1].Title = "title1" @>
        test <@ read_sessions.[1].CmdLine = "cmd_line1" @>

        test <@ read_sessions.[2].App.Id = 2 @>
        test <@ read_sessions.[2].App.Identity = AppIdentity.Uwp("uwp_aumid1") @>
        test <@ read_sessions.[2].Title = "title2" @>
        test <@ read_sessions.[2].CmdLine = null @>

    [<Fact>]
    let ``get usages`` () =

        let read_usages = db.Usages |> Seq.toList

        test <@ read_usages.[0].Start = new DateTime(now.Ticks + 100L) @>
        test <@ read_usages.[0].End = new DateTime(now.Ticks + 150L) @>

        test <@ read_usages.[1].Start = new DateTime(now.Ticks + 150L) @>
        test <@ read_usages.[1].End = new DateTime(now.Ticks + 200L) @>

        test <@ read_usages.[2].Start = new DateTime(now.Ticks + 200L) @>
        test <@ read_usages.[2].End = new DateTime(now.Ticks + 400L) @>

        test <@ read_usages.[3].Start = new DateTime(now.Ticks + 500L) @>
        test <@ read_usages.[3].End = new DateTime(now.Ticks + 600L) @>

    [<Fact>]
    let ``get interaction periods`` () =

        let read_interaction_periods = db.InteractionPeriods |> Seq.toList

        test <@ read_interaction_periods.[0].Start = new DateTime(now.Ticks + 100L) @>
        test <@ read_interaction_periods.[0].End = new DateTime(now.Ticks + 200L) @>
        test <@ read_interaction_periods.[0].MouseClicks = 2 @>
        test <@ read_interaction_periods.[0].KeyStrokes = 1 @>

        test <@ read_interaction_periods.[1].Start = new DateTime(now.Ticks + 300L) @>
        test <@ read_interaction_periods.[1].End = new DateTime(now.Ticks + 500L) @>
        test <@ read_interaction_periods.[1].MouseClicks = 0 @>
        test <@ read_interaction_periods.[1].KeyStrokes = 1 @>

        test <@ read_interaction_periods.[2].Start = new DateTime(now.Ticks + 800L) @>
        test <@ read_interaction_periods.[2].End = new DateTime(now.Ticks + 900L) @>
        test <@ read_interaction_periods.[2].MouseClicks = 10 @>
        test <@ read_interaction_periods.[2].KeyStrokes = 0 @>


    interface IDisposable with 
        member _.Dispose() =
            let inner = db.Database
            inner.CloseConnection()
            conn.Dispose()
            db.Dispose()
            inner.EnsureDeleted() |> ignore

