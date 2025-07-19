open QCheck2
open Impl

(* Helper function to insert a value into a BST *)
let rec insert k v = function
  | E -> T (E, k, v, E)
  | T (l, k', v', r) as t ->
      if k < k' then T (insert k v l, k', v', r)
      else if k > k' then T (l, k', v', insert k v r)
      else t (* No duplicates *)

(* Generate a BST from a list of unique integers *)
let gen_Q_Bespoke =
  Gen.(list_size (int_bound 10) (pair (int_bound 100) (int_bound 100)))
  |> Gen.map (fun xs ->
         let xs = List.sort_uniq (fun (k1, _) (k2, _) -> compare k1 k2) xs in
         List.fold_left (fun acc (k, v) -> insert k v acc) E xs)
