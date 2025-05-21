open Impl
open QCheck

let typebased =
  let open QCheck.Gen in
  let t c l k v r = T (c, l, k, v, r) in
  let rec tree_gen n =
    if n <= 0 then return E
    else
      Gen.(
        t
        <$> frequency [ (1, return R); (1, return B) ]
        <*> tree_gen (n / 2)
        <*> nat <*> nat
        <*> tree_gen (n / 2))
  in
  sized (fun n -> tree_gen n)

let qcheck_type = QCheck.make typebased ~print:Display.string_of_tree

