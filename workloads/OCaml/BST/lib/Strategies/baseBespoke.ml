open Impl

let rec insert_correct (t : tree) (k, v) =
  match t with
  | E -> T (E, k, v, E)
  | T (l, k', v', r) ->
      if k < k' then T (insert_correct l (k, v), k', v', r)
      else if k' < k then T (l, k', v', insert_correct r (k, v))
      else T (l, k', v, r)

module BaseBespoke : Base_quickcheck.Test.S with type t = tree = struct
  type t = tree [@@deriving sexp, quickcheck]

  let quickcheck_generator =
    let open Base_quickcheck.Generator in
    list (both Nat.Nat.quickcheck_generator Nat.Nat.quickcheck_generator)
    >>= fun l -> Base.List.fold l ~init:E ~f:insert_correct |> return
end
