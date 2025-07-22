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
- [ ] Check how ("command": "$workload_path/build") works
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

@ceren

- [ ] Make rackcheck print S-expressions instead of the default output (https://github.com/alpaylan/rackcheck/tree/feat/jsonify)
- [ ] Give the ability to control printing via some runtime flag
- [ ] Create a timed sampling method