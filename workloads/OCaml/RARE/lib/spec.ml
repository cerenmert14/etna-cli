open Util.Runner
open Impl

(* Rarity Properties *)

(* everything goes through the precondition, property fails rarely *)
let prop_NothingPasses : string -> test =
  fun s -> true ->> (verify s = Fail)

(* input rarely goes through precondition, property always fails *)
let prop_TightPrecondition : string -> test =
  fun s -> (s = "hello") ->> false