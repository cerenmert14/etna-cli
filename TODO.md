# Todo's

@alperen

## Cross Language Comparison

- [ ] Each language should have a `producing runner` that you can sample a list of pairs of (duration, counterexample) for each workload
  - [ ] Implement a serialize function that serializes the counterexample to an S expression.
  - [ ] Implement a quicksample function that samples a list of pairs of (duration, counterexample) for a given generator
  - [ ] Implement an interface that allows us to run the producing runner and get the output
- [ ] We read the output of the producing runner, and pass it to a designated consuming runner that will run the workload with the counterexample
- [ ] The consuming runner should be able to handle the counterexample and produce a result

- [ ] When auto-generating tests, tailor them to the specific languages (change tests/bst.json..)
- [ ] Get Rust workloads fully done
  - [ ] BST
    - [x] Properties
    - [x] Generator
    - [ ] Validation
  - [ ] RBT
    - [x] Properties
    - [x] Generator
    - [ ] Validation
  - [ ] STLC
    - [ ] Properties
    - [ ] Generator
    - [ ] Validation
  - [ ] (not for now) System F

check the following tasks:
  RBT, DeleteInsert, miscolor_insert
  RBT, DeleteInsert, no_balance_insert_1
  RBT, DeleteInsert, no_balance_insert_2

@ceren

Use cross-mode checking for validating the properties. You will run
the following command to run the cross-mode.

```bash
ETNA_DIR="." etna experiment new exp3 --local-store
ETNA_DIR="." etna workload add Racket BST --experiment exp3
ETNA_DIR="." etna experiment run --name exp3 --tests bst --cross
```

- You might need to tweak `tests/bst.json` to respect the Racket.
- (don't forget to change language to "Racket" and "strategy" to "RackcheckBespoke")

```json
    {
        "language": "Racket",
        "workload": "BST",
        "mutations": ["insert_1"],
        "trials": 10,
        "timeout": 1,
        "tasks": [
            {
                "strategy": "RackcheckBespoke",
                "property": "InsertPost"
            },
            {
                "strategy": "RackcheckBespoke",
                "property": "InsertInsert"
            },
            {
                "strategy": "RackcheckBespoke",
                "property": "InsertModel"
            },
            {
                "strategy": "RackcheckBespoke",
                "property": "DeleteInsert"
            },
            {
                "strategy": "RackcheckBespoke",
                "property": "InsertUnion"
            },
            {
                "strategy": "RackcheckBespoke",
                "property": "UnionDeleteInsert"
            }
        ]
    },
```

- All tests should pass in cross-mode, so we expect no timeouts.
- The results of the experiment are at `exp3/store.json`, inside `metrics`
- If you see `variant already active`, just run again

The overall objective is to make sure Rust and Racket have the same semantics.
For example, `((E) 0 0 0 0)` should pass or fail in both.

- Racket property
- Racket generator
- Rust property

- Racket generators + Rust property, `mutations: insert_1` is faulty. Racket says  `((E) 0 0 0 0)` is a counterexample, but Rust says it is not. Which one is correct?

```racket
>>> prop_InsertInsert ((E) 0 0 0 0)
Just #t
```

```bash
cargo build --release
./target/release/bst-serialized "(((T (E) 3 4 (E)) 9 3 10 3))" InsertInsert
```