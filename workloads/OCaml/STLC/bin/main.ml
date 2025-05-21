open STLC.QcheckType
open STLC.QcheckBespoke
open STLC.CrowbarType
open STLC.CrowbarBespoke
open STLC.BaseType
open STLC.BaseBespoke
open STLC.Impl
open STLC.Test
open Util.Io
open Util.Runner
open QCheck

(* RUNNER COMMAND:
   dune exec STLC -- qcheck prop_SinglePreserve bespoke out
   dune exec STLC -- qcheck prop_SinglePreserve type out
   dune exec STLC -- crowbar prop_SinglePreserve bespoke out
   dune exec STLC -- crowbar prop_SinglePreserve type out
   dune exec STLC -- afl prop_SinglePreserve bespoke out
   dune exec STLC -- afl prop_SinglePreserve type out
   dune exec STLC -- base prop_SinglePreserve bespoke out
   dune exec STLC -- base prop_SinglePreserve type out
*)

let properties : (string * expr property) list =
  [
    ("prop_SinglePreserve", test_prop_SinglePreserve);
    ("prop_MultiPreserve", test_prop_MultiPreserve);
  ]

let qstrategies : (string * expr arbitrary) list =
  [ ("type", qcheck_type); ("bespoke", qcheck_bespoke) ]

let cstrategies : (string * expr Crowbar.gen) list =
  [ ("type", crowbar_type); ("bespoke", crowbar_bespoke) ]

let bstrategies : (string * expr basegen) list =
  [ ("type", (module BaseType)); ("bespoke", (module BaseBespoke)) ]

let () = main properties qstrategies cstrategies bstrategies
