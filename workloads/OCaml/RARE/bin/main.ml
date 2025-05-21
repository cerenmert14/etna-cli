open Crowbar
open RARE.Test
open Util.Runner
open Util.Io

(*
  dune exec RARE -- qcheck prop_NothingPasses type out
  dune exec RARE -- crowbar prop_NothingPasses type out
  dune exec RARE -- afl prop_NothingPasses type out
  dune exec RARE -- base prop_NothingPasses type out
*)




let () = timeout := 600

let properties =
  [
    ("prop_NothingPasses", test_prop_NothingPasses);
    ("prop_TightPrecondition", test_prop_TightPrecondition);
  ]

let qstrategies =
  [("type", QCheck.make ~print:(fun s -> s) QCheck.Gen.string_printable)]

let cstrategies =
  [("type", Crowbar.bytes)]

let bstrategies : (string * string basegen) list =
  [("type", (module Core.String))]

  let () = main properties qstrategies cstrategies bstrategies