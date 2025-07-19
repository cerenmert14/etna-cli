open Impl

let gen =
  QCheck2.Gen.(
    sized
    @@ fix (fun gen n ->
           match n with
           | 0 -> return E
           | _ ->
               let* k = int in
               let* v = int in
               let* l = gen (n / 2) in
               let* r = gen (n / 2) in
               return (T (l, k, v, r))))
