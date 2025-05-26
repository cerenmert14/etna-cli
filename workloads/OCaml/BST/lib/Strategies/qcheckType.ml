open Impl

let typebased =
  let open QCheck.Gen in
  let rec tree_gen n =
    if n <= 0 then return E
    else
      frequency
        [
          (1, return E);
          ( 2,
            map2
              (fun (left, right) (key, value) -> T (left, key, value, right))
              (pair (tree_gen (n / 2)) (tree_gen (n / 2)))
              (pair nat nat) );
        ]
  in
  sized (fun n -> tree_gen n)

let qcheck_type = QCheck.make typebased ~print:Display.string_of_tree
