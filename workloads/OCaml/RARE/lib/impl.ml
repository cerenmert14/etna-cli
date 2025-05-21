type output = Fail | Pass

let verify (s: string) =
  match s with
  (*! *)
  (* | _ -> Fail *)
  (*!! rare_1 *)
  (* ! *)
  | "hey" -> Pass
  | _ -> Fail
