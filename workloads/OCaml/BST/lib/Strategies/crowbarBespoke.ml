open Crowbar
open Impl

let rec insert_correct (k : int) (v : int) (t : tree) =
  match t with
  | E -> T (E, k, v, E)
  | T (l, k', v', r) ->
      if k < k' then T (insert_correct k v l, k', v', r)
      else if k' < k then T (l, k', v', insert_correct k v r)
      else T (l, k', v, r)

let bespoke : tree gen =
  dynamic_bind
    (list (pair int8 int8))
    (fun kvs ->
      const (List.fold_left (fun t (k, v) -> insert_correct k v t) E kvs))

let crowbar_bespoke = with_printer Display.format_tree bespoke