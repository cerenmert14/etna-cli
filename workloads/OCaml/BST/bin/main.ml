open Cmdliner
open BST.Spec_qcheck
open BST.Spec_crowbar
open BST.Gen_typebased_qcheck
open BST.Gen_typebased_crowbar
open BST.Gen_bespoke_qcheck
open BST.Gen_bespoke_crowbar

let lookup_Q_property name =
  match name with
  | "prop_Q_InsertValid" -> prop_Q_InsertValid
  | "prop_Q_DeleteValid" -> prop_Q_DeleteValid
  | "prop_Q_UnionValid" -> prop_Q_UnionValid
  | "prop_Q_InsertPost" -> prop_Q_InsertPost
  | "prop_Q_DeletePost" -> prop_Q_DeletePost
  | "prop_Q_UnionPost" -> prop_Q_UnionPost
  | "prop_Q_InsertModel" -> prop_Q_InsertModel
  | "prop_Q_DeleteModel" -> prop_Q_DeleteModel
  | "prop_Q_UnionModel" -> prop_Q_UnionModel
  | "prop_Q_InsertInsert" -> prop_Q_InsertInsert
  | "prop_Q_InsertDelete" -> prop_Q_InsertDelete
  | "prop_Q_InsertUnion" -> prop_Q_InsertUnion
  | "prop_Q_DeleteUnion" -> prop_Q_DeleteUnion
  | "prop_Q_DeleteInsert" -> prop_Q_DeleteInsert
  | "prop_Q_DeleteDelete" -> prop_Q_DeleteDelete
  | "prop_Q_UnionDeleteInsert" -> prop_Q_UnionDeleteInsert
  | "prop_Q_UnionUnionIdem" -> prop_Q_UnionUnionIdem
  | "prop_Q_UnionUnionAssoc" -> prop_Q_UnionUnionAssoc
  | _ -> raise (Invalid_argument ("Unknown property: " ^ name))

let lookup_C_property name =
  match name with
  | "prop_C_InsertValid" -> prop_C_InsertValid
  | "prop_C_DeleteValid" -> prop_C_DeleteValid
  | "prop_C_UnionValid" -> prop_C_UnionValid
  | "prop_C_InsertPost" -> prop_C_InsertPost
  | "prop_C_DeletePost" -> prop_C_DeletePost
  | "prop_C_UnionPost" -> prop_C_UnionPost
  | "prop_C_InsertModel" -> prop_C_InsertModel
  | "prop_C_DeleteModel" -> prop_C_DeleteModel
  | "prop_C_UnionModel" -> prop_C_UnionModel
  | "prop_C_InsertInsert" -> prop_C_InsertInsert
  | "prop_C_InsertDelete" -> prop_C_InsertDelete
  | "prop_C_InsertUnion" -> prop_C_InsertUnion
  | "prop_C_DeleteUnion" -> prop_C_DeleteUnion
  | "prop_C_DeleteInsert" -> prop_C_DeleteInsert
  | "prop_C_DeleteDelete" -> prop_C_DeleteDelete
  | "prop_C_UnionDeleteInsert" -> prop_C_UnionDeleteInsert
  | "prop_C_UnionUnionIdem" -> prop_C_UnionUnionIdem
  | "prop_C_UnionUnionAssoc" -> prop_C_UnionUnionAssoc
  | _ -> raise (Invalid_argument ("Unknown property: " ^ name))

let lookup_Q_generator name =
  match name with
  | "gen_Q_TypeBased" -> gen_Q_TypeBased
  | "gen_Q_Bespoke" -> gen_Q_Bespoke
  | _ -> raise (Invalid_argument ("Unknown generator: " ^ name))

let lookup_C_generator name =
  match name with
  | "gen_C_TypeBased" -> gen_C_TypeBased
  | "gen_C_Bespoke" -> gen_C_Bespoke
  | _ -> raise (Invalid_argument ("Unknown generator: " ^ name))

(** Command line setup *)

let main test_name generator_name =
  (* Your logic here: select property and generator by name *)
  Printf.printf "Test: %s\nGenerator: %s\n" test_name generator_name;
  match
    ( Str.string_match (Str.regexp ".*_Q_.*") test_name 0,
      Str.string_match (Str.regexp ".*_C_.*") test_name 0 )
  with
  | true, _ ->
      Runner_qcheck.run
        (lookup_Q_property test_name)
        (lookup_Q_generator generator_name)
  | _, true ->
      (* have to do this because crowbar reads command lines args *)
      Random.self_init ();
      Sys.argv.(1) <- "--repeat=10000000";
      Sys.argv.(2) <- Printf.sprintf "--seed=%d" (Random.int 1000000);
      Runner_crowbar.run
        (lookup_C_property test_name)
        (lookup_C_generator generator_name)
  | _ -> failwith "Test name must contain either _Q_ or _C_"

(** *)

(** Cmdliner stuff *)

(** | *)

(** v *)

let test_arg =
  let doc = "Name of the property test to run." in
  Arg.(required & opt (some string) None & info [ "test" ] ~docv:"TEST" ~doc)

let generator_arg =
  let doc = "Name of the generator to use." in
  Arg.(
    required
    & opt (some string) None
    & info [ "generator" ] ~docv:"GENERATOR" ~doc)

let term = Term.(const main $ test_arg $ generator_arg)
let () = Cmd.(exit @@ eval (v (info "BST") term))
