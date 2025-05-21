open Impl

let rec format_tree ppf tree =
  let open Format in
  match tree with
  | E -> fprintf ppf "E"
  | T (left, key, value, right) ->
      fprintf ppf "T (%a, %i, %i, %a)" format_tree left key value format_tree
        right

let rec string_of_tree = function
  | E -> "Empty"
  | T (l, k, v, r) ->
      "Tree (" ^ string_of_tree l ^ "," ^ string_of_int k ^ ","
      ^ string_of_int v ^ "," ^ string_of_tree r ^ ")"
