# previous cleanup
rm -rf ./exp1
rm -rf ~/.etna

# install etna
cargo install --path .

# setup environment
etna setup

# create a new experiment and add workloads
ETNA_DIR="." etna experiment new exp1 --local-store
ETNA_DIR="." etna workload add OCaml BST --experiment exp1

# run the experiment with different tests
RUST_LOG="debug" ETNA_DIR="." etna experiment run --name exp1 --tests bst

# visualize the results
RUST_LOG="debug" ETNA_DIR="." etna experiment visualize --name exp1 --figure bst --tests bst
