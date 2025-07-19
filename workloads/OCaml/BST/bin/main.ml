open Cmdliner

let test_arg =
  let doc = "Name of the property test to run." in
  Arg.(required & opt (some string) None & info ["test"] ~docv:"TEST" ~doc)

let generator_arg =
  let doc = "Name of the generator to use." in
  Arg.(required & opt (some string) None & info ["generator"] ~docv:"GENERATOR" ~doc)

let main test_name generator_name =
  (* Your logic here: select property and generator by name *)
  Printf.printf "Test: %s\nGenerator: %s\n" test_name generator_name

let term = Term.(const main $ test_arg $ generator_arg)

let () = Cmd.(exit @@ eval (v (info "BST") term))