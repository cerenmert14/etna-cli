let print_result (r : BST.Spec_qcheck.test_result) =
  let status = if r.passed then "Finished" else "Failed" in
  Printf.printf
    {|[|{
    "discards": %d,
    "tests": %d,
    "status": "%s",
    "time": "%fs"
  }|]|}
    r.discards r.generated status r.elapsed_s;
  print_endline ""

let run
    (make_test :
      BST.Impl.t QCheck2.Gen.t -> unit -> BST.Spec_qcheck.test_result)
    (gen : BST.Impl.t QCheck2.Gen.t) : unit =
  make_test gen () |> print_result
