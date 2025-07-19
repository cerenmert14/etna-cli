open Cmdliner
open BST.Spec_qcheck
open BST.Gen_typebased_qcheck
open BST.Gen_bespoke_qcheck

let lookup_property name =
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

let lookup_generator name =
  match name with
  | "gen_Q_TypeBased" -> gen_Q_TypeBased
  | "gen_Q_Bespoke" -> gen_Q_Bespoke
  | _ -> raise (Invalid_argument ("Unknown generator: " ^ name))

(** Command line setup *)

let test_arg =
  let doc = "Name of the property test to run." in
  Arg.(required & opt (some string) None & info [ "test" ] ~docv:"TEST" ~doc)

let generator_arg =
  let doc = "Name of the generator to use." in
  Arg.(
    required
    & opt (some string) None
    & info [ "generator" ] ~docv:"GENERATOR" ~doc)

let main test_name generator_name =
  (* Your logic here: select property and generator by name *)
  Printf.printf "Test: %s\nGenerator: %s\n" test_name generator_name;
  let time =
    Runner_qcheck.run_test_timed
      (lookup_property test_name)
      (lookup_generator generator_name)
  in
  Printf.printf "Time taken: %f seconds\n" time

let term = Term.(const main $ test_arg $ generator_arg)
let () = Cmd.(exit @@ eval (v (info "BST") term))
