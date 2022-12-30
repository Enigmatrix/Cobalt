module Tests

open Xunit
open Swensen.Unquote

open Cobalt.Common.Data;
open Cobalt.Common.Data.Models;

[<Fact>]
let ``create db`` () =
    let db = new CobaltContext(":memory:")
    // db.Apps.Add(new App()) |> ignore
    <@ true @>
