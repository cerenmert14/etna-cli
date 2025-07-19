open Impl
open QCheck2
open QCheck2.Test
open QCheck2.Gen

(** All these properties are generalized on the specific QCheck generator *)

(** -- Validity properties *)
let prop_Q_InsertValid gen =
  make ~name:"Q_InsertValid" ~count:100000 (tup3 gen int int) (fun (t, k, v) ->
      assume (is_bst t);
      is_bst (insert k v t))

let prop_Q_DeleteValid gen =
  make ~name:"Q_DeleteValid" ~count:100000 (tup2 gen int) (fun (t, k) ->
      assume (is_bst t);
      is_bst (delete k t))

let prop_Q_UnionValid gen =
  make ~name:"Q_UnionValid" ~count:100000 (tup2 gen gen) (fun (t, t') ->
      assume (is_bst t);
      assume (is_bst t');
      is_bst (union t t'))

(** -- Postcondition properties *)
let prop_Q_InsertPost gen =
  make ~name:"Q_InsertPost" ~count:100000 (tup4 gen int int int)
    (fun (t, k, k', v) ->
      assume (is_bst t);
      Printf.printf "Testing InsertPost with valid tree of size %d\n"
        (size t);
      find k' (insert k v t) = if k = k' then Some v else find k' t)

let prop_Q_DeletePost gen =
  make ~name:"Q_DeletePost" ~count:100000 (tup3 gen int int) (fun (t, k, k') ->
      assume (is_bst t);
      find k' (delete k t) = if k = k' then None else find k' t)

let prop_Q_UnionPost gen =
  make ~name:"Q_UnionPost" ~count:100000 (tup3 gen gen int) (fun (t, t', k) ->
      assume (is_bst t);
      assume (is_bst t');
      find k (union t t')
      =
      (* First tree takes precedence *)
      match (find k t, find k t') with
      | None, None -> None
      | None, Some v -> Some v
      | Some v, _ -> Some v)

(** -- Model-based properties *)
let prop_Q_InsertModel gen =
  make ~name:"Q_InsertModel" ~count:100000 (tup3 gen int int) (fun (t, k, v) ->
      assume (is_bst t);
      to_list (insert k v t) = l_insert k v (delete_key k (to_list t)))

let prop_Q_DeleteModel gen =
  make ~name:"Q_DeleteModel" ~count:100000 (tup2 gen int) (fun (t, k) ->
      assume (is_bst t);
      to_list (delete k t) = delete_key k (to_list t))

let prop_Q_UnionModel gen =
  make ~name:"Q_UnionModel" ~count:100000 (tup2 gen gen) (fun (t, t') ->
      assume (is_bst t);
      assume (is_bst t');
      to_list (union t t')
      = List.sort compare (l_union (to_list t) (to_list t')))

(** Metamorphic properties *)
let prop_Q_InsertInsert gen =
  make ~name:"Q_InsertInsert" ~count:100000 (tup5 gen int int int int)
    (fun (t, k, k', v, v') ->
      assume (is_bst t);
      insert k v (insert k' v' t)
      === if k = k' then insert k v t else insert k' v' (insert k v t))

let prop_Q_InsertDelete gen =
  make ~name:"Q_InsertDelete" ~count:100000 (tup4 gen int int int)
    (fun (t, k, k', v) ->
      assume (is_bst t);
      insert k v (delete k' t)
      === if k = k' then insert k v t else delete k' (insert k v t))

let prop_Q_InsertUnion gen =
  make ~name:"Q_InsertUnion" ~count:100000 (tup4 gen gen int int)
    (fun (t, t', k, v) ->
      assume (is_bst t);
      assume (is_bst t');
      insert k v (union t t') === union (insert k v t) t')

let prop_Q_DeleteUnion gen =
  make ~name:"Q_DeleteUnion" ~count:100000 (tup3 gen gen int) (fun (t, t', k) ->
      assume (is_bst t);
      assume (is_bst t');
      delete k (union t t') === union (delete k t) (delete k t'))

let prop_Q_DeleteInsert gen =
  make ~name:"Q_DeleteInsert" ~count:100000 (tup4 gen int int int)
    (fun (t, k, k', v) ->
      assume (is_bst t);
      delete k (insert k' v t)
      === if k = k' then delete k t else insert k' v (delete k t))

let prop_Q_DeleteDelete gen =
  make ~name:"Q_DeleteDelete" ~count:100000 (tup3 gen int int)
    (fun (t, k, k') ->
      assume (is_bst t);
      delete k (delete k' t) === delete k' (delete k t))

let prop_Q_UnionDeleteInsert gen =
  make ~name:"Q_UnionDeleteInsert" ~count:100000 (tup4 gen gen int int)
    (fun (t, t', k, v) ->
      assume (is_bst t);
      assume (is_bst t');
      union (delete k t) (insert k v t') === insert k v (union t t'))

let prop_Q_UnionUnionIdem gen =
  make ~name:"Q_UnionUnionIdem" ~count:100000 gen (fun t ->
      assume (is_bst t);
      union t t === t)

let prop_Q_UnionUnionAssoc gen =
  make ~name:"Q_UnionUnionAssoc" ~count:100000 (tup3 gen gen gen)
    (fun (t1, t2, t3) ->
      assume (is_bst t1);
      assume (is_bst t2);
      assume (is_bst t3);
      union (union t1 t2) t3 === union t1 (union t2 t3))
