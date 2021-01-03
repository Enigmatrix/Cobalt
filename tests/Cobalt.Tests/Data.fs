module Data

open System
open Xunit
open Swensen.Unquote
open Cobalt.Common.Data.Entites;

[<Fact>]
let ``Can create AppIdentities`` () =
    let appid = AppIdentity.UWP "ur mam";
    test <@ 1 = 1@>
