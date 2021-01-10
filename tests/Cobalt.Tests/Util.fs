module Util

open Xunit
open Swensen.Unquote
open Cobalt.Common.Utils
open System


let putin (tbl: WeakValueCache<int64, obj>) =
    tbl.Set(1L, obj())
    test <@ tbl.Get(1L) <> null @>

[<Fact>]
let ``WeakValueCache drops its value after it gets GC collected`` () =
    let tbl = new WeakValueCache<int64, obj>()
    putin tbl
    GC.Collect()
    test <@ tbl.Get(1L) = null @>
