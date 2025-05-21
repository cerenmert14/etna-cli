open Spec
open Util
open Runner
open Crowbar

let ( << ) f g x = f (g x)

let test_prop_NothingPasses : string property = {
  name = "test_prop_NothingPasses";
  q = (fun a -> qbuild a (qmake << prop_NothingPasses));
  c = (fun g -> cbuild [g] (cmake << prop_NothingPasses));
  b = (fun m -> bbuild m (bmake << prop_NothingPasses));
}

let test_prop_TightPrecondition : string property = {
  name = "test_prop_TightPrecondition";
  q = (fun a -> qbuild a (qmake << prop_TightPrecondition));
  c = (fun g -> cbuild [g] (cmake << prop_TightPrecondition));
  b = (fun m -> bbuild m (bmake << prop_TightPrecondition))
}