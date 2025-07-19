open Unix

let run_test_timed (make_test : BST.Impl.t QCheck2.Gen.t -> QCheck2.Test.t)
    (gen : BST.Impl.t QCheck2.Gen.t) : float =
  let test = make_test gen in
  let result = ref None in
  let start_time = gettimeofday () in
  let th =
    Thread.create
      (fun () ->
        let passed =
          QCheck2.Test.check_exn test;
          true
        in
        result := Some passed)
      ()
  in
  Thread.join th;
  let end_time = gettimeofday () in
  end_time -. start_time
