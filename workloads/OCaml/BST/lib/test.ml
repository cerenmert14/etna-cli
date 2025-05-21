open Impl
open Spec
open Util
open Runner
open QCheck
open Crowbar
open Nat

let ( << ) f g x = f (g x)
let qi = small_int
let ci = int8


let test_prop_InsertValid : tree property =
  {
    name = "test_prop_InsertValid";
    q = (fun a -> qbuild (triple a qi qi) (qmake << prop_InsertValid));
    c =
      (fun g ->
        cbuild [ g; ci; ci ] (fun t k v -> prop_InsertValid (t, k, v) |> cmake));
    b =
      (fun m ->
        bbuild
          (Core_plus.triple m (module Nat) (module Nat))
          (bmake << prop_InsertValid));
  }

(*! QCheck test_prop_InsertValid. *)

let test_prop_DeleteValid : tree property =
  {
    name = "test_prop_DeleteValid";
    q = (fun a -> qbuild (QCheck.pair a qi) (qmake << prop_DeleteValid));
    c =
      (fun g -> cbuild [ g; ci ] (fun t k -> prop_DeleteValid (t, k) |> cmake));
    b =
      (fun m ->
        bbuild (Core_plus.double m (module Nat)) (bmake << prop_DeleteValid));
  }

(*! QCheck test_prop_DeleteValid. *)

let test_prop_UnionValid : tree property =
  {
    name = "test_prop_UnionValid";
    q = (fun a -> qbuild (QCheck.pair a a) (qmake << prop_UnionValid));
    c =
      (fun g -> cbuild [ g; g ] (fun t t' -> prop_UnionValid (t, t') |> cmake));
    b = (fun m -> bbuild (Core_plus.double m m) (bmake << prop_UnionValid));
  }

(*! QCheck test_prop_UnionValid. *)

let test_prop_InsertPost : tree property =
  {
    name = "test_prop_InsertPost";
    q = (fun a -> qbuild (quad a qi qi qi) (qmake << prop_InsertPost));
    c =
      (fun g ->
        cbuild [ g; ci; ci; ci ] (fun t k k' v ->
            prop_InsertPost (t, k, k', v) |> cmake));
    b =
      (fun m ->
        bbuild
          (Core_plus.quad m
             (module Nat)
             (module Nat)
             (module Nat))
          (bmake << prop_InsertPost));
  }

(*! QCheck test_prop_InsertPost. *)

let test_prop_DeletePost : tree property =
  {
    name = "test_prop_DeletePost";
    q = (fun a -> qbuild (triple a qi qi) (qmake << prop_DeletePost));
    c =
      (fun g ->
        cbuild [ g; ci; ci ] (fun t i i' -> prop_DeletePost (t, i, i') |> cmake));
    b =
      (fun m ->
        bbuild
          (Core_plus.triple m (module Nat) (module Nat))
          (bmake << prop_DeletePost));
  }

(*! QCheck test_prop_DeletePost. *)

let test_prop_UnionPost : tree property =
  {
    name = "test_prop_UnionPost";
    q = (fun a -> qbuild (triple a a qi) (qmake << prop_UnionPost));
    c =
      (fun g ->
        cbuild [ g; g; ci ] (fun t t' i -> prop_UnionPost (t, t', i) |> cmake));
    b =
      (fun m ->
        bbuild (Core_plus.triple m m (module Nat)) (bmake << prop_UnionPost));
  }

(*! QCheck test_prop_UnionPost. *)

let test_prop_InsertModel : tree property =
  {
    name = "test_prop_InsertModel";
    q = (fun a -> qbuild (triple a qi qi) (qmake << prop_InsertModel));
    c =
      (fun g ->
        cbuild [ g; ci; ci ] (fun t i i' ->
            prop_InsertModel (t, i, i') |> cmake));
    b =
      (fun m ->
        bbuild
          (Core_plus.triple m (module Nat) (module Nat))
          (bmake << prop_InsertModel));
  }

(*! QCheck test_prop_InsertModel. *)

let test_prop_DeleteModel : tree property =
  {
    name = "test_prop_DeleteModel";
    q = (fun a -> qbuild (QCheck.pair a qi) (qmake << prop_DeleteModel));
    c =
      (fun g -> cbuild [ g; ci ] (fun t i -> prop_DeleteModel (t, i) |> cmake));
    b =
      (fun m ->
        bbuild (Core_plus.double m (module Nat)) (bmake << prop_DeleteModel));
  }

(*! QCheck test_prop_DeleteModel. *)

let test_prop_UnionModel : tree property =
  {
    name = "test_prop_UnionModel";
    q = (fun a -> qbuild (QCheck.pair a a) (qmake << prop_UnionModel));
    c =
      (fun g -> cbuild [ g; g ] (fun t t' -> prop_UnionModel (t, t') |> cmake));
    b = (fun m -> bbuild (Core_plus.double m m) (bmake << prop_UnionModel));
  }

(*! QCheck test_prop_UnionModel. *)

let test_prop_InsertInsert : tree property =
  {
    name = "test_prop_InsertInsert";
    q =
      (fun a ->
        qbuild
          (QCheck.pair a (quad qi qi qi qi))
          ( qmake << fun (t, (k, k', v, v')) ->
            prop_InsertInsert (t, k, k', v, v') ));
    c =
      (fun g ->
        cbuild [ g; ci; ci; ci; ci ] (fun t k k' v v' ->
            prop_InsertInsert (t, k, k', v, v') |> cmake));
    b =
      (fun m ->
        bbuild
          (Core_plus.quinta m
             (module Nat)
             (module Nat)
             (module Nat)
             (module Nat))
          (bmake << prop_InsertInsert));
  }

(*! QCheck test_prop_InsertInsert. *)

let test_prop_InsertDelete : tree property =
  {
    name = "test_prop_InsertDelete";
    q = (fun a -> qbuild (quad a qi qi qi) (qmake << prop_InsertDelete));
    c =
      (fun g ->
        cbuild [ g; ci; ci; ci ] (fun t k k' v ->
            prop_InsertDelete (t, k, k', v) |> cmake));
    b =
      (fun m ->
        bbuild
          (Core_plus.quad m
             (module Nat)
             (module Nat)
             (module Nat))
          (bmake << prop_InsertDelete));
  }

(*! QCheck test_prop_InsertDelete. *)

let test_prop_InsertUnion : tree property =
  {
    name = "test_prop_InsertUnion";
    q = (fun a -> qbuild (quad a a qi qi) (qmake << prop_InsertUnion));
    c =
      (fun g ->
        cbuild [ g; g; ci; ci ] (fun t t' i i' ->
            prop_InsertUnion (t, t', i, i') |> cmake));
    b =
      (fun m ->
        bbuild
          (Core_plus.quad m m (module Nat) (module Nat))
          (bmake << prop_InsertUnion));
  }

(*! QCheck test_prop_InsertUnion. *)

let test_prop_DeleteInsert : tree property =
  {
    name = "test_prop_DeleteInsert";
    q = (fun a -> qbuild (quad a qi qi qi) (qmake << prop_DeleteInsert));
    c =
      (fun g ->
        cbuild [ g; ci; ci; ci ] (fun t k k' v ->
            prop_DeleteInsert (t, k, k', v) |> cmake));
    b =
      (fun m ->
        bbuild
          (Core_plus.quad m
             (module Nat)
             (module Nat)
             (module Nat))
          (bmake << prop_DeleteInsert));
  }

(*! QCheck test_prop_DeleteInsert. *)

let test_prop_DeleteDelete : tree property =
  {
    name = "test_prop_DeleteDelete";
    q = (fun a -> qbuild (triple a qi qi) (qmake << prop_DeleteDelete));
    c =
      (fun g ->
        cbuild [ g; ci; ci ] (fun t i i' ->
            prop_DeleteDelete (t, i, i') |> cmake));
    b =
      (fun m ->
        bbuild
          (Core_plus.triple m (module Nat) (module Nat))
          (bmake << prop_DeleteDelete));
  }

(*! QCheck test_prop_DeleteDelete. *)

let test_prop_DeleteUnion : tree property =
  {
    name = "test_prop_DeleteUnion";
    q = (fun a -> qbuild (triple a a qi) (qmake << prop_DeleteUnion));
    c =
      (fun g ->
        cbuild [ g; g; ci ] (fun t t' i -> prop_DeleteUnion (t, t', i) |> cmake));
    b =
      (fun m ->
        bbuild
          (Core_plus.triple m m (module Nat))
          (bmake << prop_DeleteUnion));
  }

(*! QCheck test_prop_DeleteUnion. *)

let test_prop_UnionDeleteInsert : tree property =
  {
    name = "test_prop_UnionDeleteInsert";
    q = (fun a -> qbuild (quad a a qi qi) (qmake << prop_UnionDeleteInsert));
    c =
      (fun g ->
        cbuild [ g; g; ci; ci ] (fun t t' i i' ->
            prop_UnionDeleteInsert (t, t', i, i') |> cmake));
    b =
      (fun m ->
        bbuild
          (Core_plus.quad m m (module Nat) (module Nat))
          (bmake << prop_UnionDeleteInsert));
  }

(*! QCheck test_prop_UnionDeleteInsert. *)

let test_prop_UnionUnionIdem : tree property =
  {
    name = "test_prop_UnionUnionIdem";
    q = (fun a -> qbuild a (qmake << prop_UnionUnionIdem));
    c = (fun g -> cbuild [ g ] (cmake << prop_UnionUnionIdem));
    b = (fun m -> bbuild m (bmake << prop_UnionUnionIdem));
  }
(*! QCheck test_prop_UnionUnionIdem. *)

let test_prop_UnionUnionAssoc : tree property =
  {
    name = "test_prop_UnionUnionAssoc";
    q = (fun a -> qbuild (triple a a a) (qmake << prop_UnionUnionAssoc));
    c =
      (fun g ->
        cbuild [ g; g; g ] (fun t t' t'' ->
            prop_UnionUnionAssoc (t, t', t'') |> cmake));
    b =
      (fun m -> bbuild (Core_plus.triple m m m) (bmake << prop_UnionUnionAssoc));
  }

(*! QCheck test_prop_UnionUnionAssoc. *)
