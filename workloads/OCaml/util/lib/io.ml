open Runner
open QCheck
open Crowbar
open Parse

(* global timeout in seconds for test threads *)
let timeout = ref 60

(* super simple running of the tests *)
let qrun (p : 'a property) (g : 'a QCheck.arbitrary) (oc : out_channel) : unit =
  ignore
    (QCheck_runner.run_tests
       [ p.q g p.name ]
       ~colors:false ~verbose:false ~out:oc)

let crun (p : 'a property) (g : 'a Crowbar.gen) : unit = p.c g p.name ()
let brun (p : 'a property) (g : 'a basegen) : unit = p.b g p.name ()

(* starting the execution for the various frameworks *)
let qmain oc t ts s ss =
  let t' = lookup ts t in
  let s' = lookup ss s in
  match (t', s') with
  | None, _ -> Printf.printf "Test %s not found\n" t
  | _, None -> Printf.printf "Strategy %s not found\n" s
  | Some t', Some s' ->
      Printf.fprintf oc "[%f start]\n" (Unix.gettimeofday ());
      flush oc;
      qrun t' s' oc

let cmain t ts s ss =
  let t' = lookup ts t in
  let s' = lookup ss s in
  match (t', s') with
  | None, _ -> Printf.printf "Test %s not found\n" t
  | _, None -> Printf.printf "Strategy %s not found\n" s
  | Some t', Some s' ->
      Printf.printf "[%f start]\n" (Unix.gettimeofday ());
      flush stdout;
      crun t' s'

let fmain t ts s ss =
  let t' = lookup ts t in
  let s' = lookup ss s in
  match (t', s') with
  | None, _ -> Printf.printf "Test %s not found\n" t
  | _, None -> Printf.printf "Strategy %s not found\n" s
  | Some t', Some s' -> crun t' s'

let bmain oc t ts s ss =
  let t' = lookup ts t in
  let s' = lookup ss s in
  match (t', s') with
  | None, _ -> Printf.printf "Test %s not found\n" t
  | _, None -> Printf.printf "Strategy %s not found\n" s
  | Some t', Some s' ->
      Printf.fprintf oc "[%f start]\n" (Unix.gettimeofday ());
      flush oc;
      brun t' s'

(* piping helper functions *)

let load_env () =
  let get = Unix.getenv in
  (get "framework", get "test", get "strategy", get "filename")

let make_env framework test strategy filename =
  [|
    "framework=" ^ framework;
    "test=" ^ test;
    "strategy=" ^ strategy;
    "filename=" ^ filename;
  |]

let crowbar_child tests strats =
  let _framework, test, strat, _filename = load_env () in
  cmain test tests strat strats

let afl_child tests strats =
  let _framework, test, strat, _filename = load_env () in
  fmain test tests strat strats

let crowbar_fork framework test strat filename =
  let oc =
    open_out_gen [ Open_wronly; Open_append; Open_creat ] 0o666 filename
  in
  let od = Unix.descr_of_out_channel oc in
  let cur = Sys.executable_name in
  Random.self_init ();
  let seed = string_of_int (Random.bits ()) in
  match
    Unix.create_process_env cur
      [| cur; "--repeat=461168601842738"; "--seed=" ^ seed |]
      (* pass the # of tests as a command line argument to the child process *)
      (make_env framework test strat filename)
      Unix.stdin od Unix.stderr
  with
  | 0 -> () (* runner/child thread *)
  | pid -> (
      match Unix.fork () with
      | 0 ->
          (* timeout thread *)
          Unix.sleep !timeout;
          Unix.kill pid Sys.sigalrm
      | pid' -> (
          (* waiting thread *)
          let _, status = Unix.waitpid [] pid in
          Unix.kill pid' Sys.sigterm;
          let endtime = Unix.gettimeofday () in
          match status with
          | Unix.WEXITED c -> Printf.fprintf oc "[%f exit %i]\n" endtime c
          | Unix.WSIGNALED c when c = Sys.sigalrm ->
              Printf.fprintf oc "[%f exit timeout]\n" endtime
          | _ -> Printf.fprintf oc "[%f exit unexpected]\n" endtime))

let afl_fork framework test strat filename : unit =
  Sys.command "mkdir -p input && echo asdf > input/nikhil" |> ignore;
  Sys.command "echo lkj > input/nikhil2" |> ignore;
  let channels =
    Unix.open_process_args_full "afl-fuzz"
      [|
        "afl-fuzz";
        "-i";
        "input";
        "-o";
        "output";
        "-V";
        string_of_int !timeout;
        Sys.executable_name;
        "@@";
      |]
      (Array.append
         (make_env framework test strat "-")
         [| "AFL_BENCH_UNTIL_CRASH=true" |])
  in
  let _status = Unix.close_process_full channels in
  let now = Unix.gettimeofday () in
  let get = parse "output/default/fuzzer_stats" in
  match (get "execs_done", get "execs_per_sec", get "saved_crashes") with
  | Some runs, Some rps, Some found when found >= 0.5 ->
      let oc =
        open_out_gen [ Open_wronly; Open_append; Open_creat ] 0o666 filename
      in
      Printf.fprintf oc "[%f start]\n" now;
      Printf.fprintf oc "[%f exit 0]\n" (now +. (runs /. rps));
      close_out oc
  | Some runs, Some rps, Some _ ->
      let oc =
        open_out_gen [ Open_wronly; Open_append; Open_creat ] 0o666 filename
      in
      Printf.fprintf oc "[%f start]\n" now;
      Printf.fprintf oc "[%f exit timeout]\n" (now +. (runs /. rps));
      close_out oc
  | _ -> failwith "Error attempting to read AFL output file"

let _simple_fork f file =
  let oc = open_out_gen [ Open_wronly; Open_append; Open_creat ] 0o666 file in
  match Unix.fork () with
  (* runner/child thread *)
  | 0 ->
      f oc
      (* todo: if the forking overhead is too much, we could pipe the endtime back to the main thread *)
  | pid -> (
      match Unix.fork () with
      | 0 ->
          (* timeout thread *)
          Unix.sleep !timeout;
          Unix.kill pid Sys.sigalrm
      | pid' -> (
          (* waiting thread *)
          let _, status = Unix.waitpid [] pid in
          Unix.kill pid' Sys.sigterm;
          let endtime = Unix.gettimeofday () in
          match status with
          | Unix.WEXITED c -> Printf.fprintf oc "[%f exit %i]\n" endtime c
          | Unix.WSIGNALED c when c = Sys.sigalrm ->
              Printf.fprintf oc "[%f exit timeout]\n" endtime
          | _ -> Printf.fprintf oc "[%f exit unexpected]\n" endtime))

let qcheck_fork t ts s ss = _simple_fork (fun oc -> qmain oc t ts s ss)
let base_fork t ts s ss = _simple_fork (fun oc -> bmain oc t ts s ss)

(* Call format:
   dune exec <workload> -- <framework> <testname> <strategy> <filename>
   for example,
   dune exec BST -- qcheck prop_InsertValid bespokeGenerator out.txt
   or
   dune exec BST -- crowbar prop_InsertPost crowbarType out2.txt
*)
let main (props : (string * 'a property) list)
    (qstrats : (string * 'a arbitrary) list) (cstrats : (string * 'a gen) list)
    (bstrats : (string * 'a basegen) list) : unit =
  if Array.length Sys.argv < 5 then
    match Unix.getenv "framework" with
    | "crowbar" -> crowbar_child props cstrats
    | "afl" -> afl_child props cstrats
    | _ ->
        print_endline
          "Not enough arguments were passed. Could not determine whether this \
           was a child process."
  else
    let framework = Sys.argv.(1) in
    let testname = Sys.argv.(2) in
    let strategy = Sys.argv.(3) in
    let filename = Sys.argv.(4) in
    Printf.printf
      "Executing test %s into file %s using strategy %s on framework %s\n"
      testname filename strategy framework;
    flush stdout;
    match framework with
    | "qcheck" ->
        print_endline "Valid framework QCheck\n";
        qcheck_fork testname props strategy qstrats filename
    | "crowbar" ->
        print_endline "Valid framework Crowbar\n";
        crowbar_fork framework testname strategy filename
    | "afl" ->
        print_endline "Valid framework AFL\n";
        afl_fork framework testname strategy filename
    | "base" ->
        print_endline "Valid framework Base_quickcheck\n";
        base_fork testname props strategy bstrats filename
    | _ -> print_endline ("Framework " ^ framework ^ " was not found\n")



let etna = main