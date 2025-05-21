open Crowbar
open Impl

let typebased : tree gen =
  fix (fun cbtype ->
      choose
        [
          const E;
          map [ cbtype; cbtype; int8; int8 ] (fun l r k v -> T (l, k, v, r));
        ])

let crowbar_type = with_printer Display.format_tree typebased
