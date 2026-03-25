open Impl

let ( --- ) i j =
  let rec aux n acc = if n < i then acc else aux (n - 1) (n :: acc) in
  aux j []

let rec typGen n =
  let tfun a b = TFun (a, b) in
  let open QCheck2.Gen in
    if n <= 0 then 
      return TBool
    else 
      oneof
        [ delay (fun () -> typGen (n / 2));
          delay (fun () -> (map2 tfun (typGen (n / 2)) (typGen (n / 2))))]
 
let genExactExpr (ctx : ctx) (t : typ) : expr QCheck2.Gen.t =
  let e_var v = Var v in
  let e_bool b = Bool b in
  let e_abs t e = Abs (t, e) in
  let open QCheck2.Gen in

  let rec genOne ctx t =
    match (ctx, t) with
    | _, TBool -> e_bool <$> bool
    | ctx, TFun (t1, t2) -> e_abs t1 <$> genOne (t1 :: ctx) t2
  in
  
  let genVar (ctx : ctx) t =
    let open List in
    let vars = filter (fun i -> nth ctx i = t) (0 --- (length ctx - 1)) in
    match vars with
    | [] -> []
    | vs -> [delay (fun () -> (QCheck2.Gen.(e_var <$> oneofl vs)))] in

  let rec go n ctx t =
    let genAbs ctx t1 t2 = (delay (fun () -> e_abs t1 <$> go (n - 1) (t1 :: ctx) t2)) in
    let genApp ctx t =
      QCheck2.Gen.int_bound (min n 10) >>= fun bound -> 
      typGen (bound + 1)
        >>= fun t' ->
        go (n - 1) ctx (TFun (t', t)) >>= fun e1 ->
        go (n - 1) ctx t' >>= fun e2 -> return (App (e1, e2))
    in
    if n <= 0 then 
       oneof ([(delay (fun () -> genOne ctx t))] @ genVar ctx t)
    else
      let absGen = match t with 
      | TFun (t1, t2) -> [genAbs ctx t1 t2]
      | _ -> []
      in
       oneof ([delay (fun () -> genOne ctx t); delay (fun () -> genApp ctx t)] @ absGen @ genVar ctx t)
    in
   go 25 ctx t

let gen_Q_Bespoke =
  let open QCheck2.Gen in
  let* typ = typGen 250 in
  genExactExpr [] typ >>= fun expr ->
  return expr