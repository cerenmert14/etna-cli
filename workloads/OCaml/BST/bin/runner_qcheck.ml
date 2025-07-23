open Unix

let run_test_timed (make_test : BST.Impl.t QCheck2.Gen.t -> QCheck2.Test.t)
    (gen : BST.Impl.t QCheck2.Gen.t) : float =
  let test = make_test gen in
  let result = ref false in
  let start_time = gettimeofday () in
  let th =
    Thread.create
      (fun () ->
        let passed =
          QCheck2.Test.check_exn test;
          true
        in
        result := passed)
      ()
  in
  Thread.join th;
  if !result then 
    Printf.printf "Test passed\n"
  else
    Printf.printf "Test failed\n";
  (* Calculate elapsed time *)
  let end_time = gettimeofday () in
  end_time -. start_time
