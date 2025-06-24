# previous cleanup
rm -rf ~/.etna

# install etna
cargo install --path .

# setup environment
etna setup

# create a new experiment and add workloads
ETNA_DIR="." etna experiment new exp1 --local-store
ETNA_DIR="." etna workload add Rocq BST --experiment exp1
ETNA_DIR="." etna workload add Rocq RBT --experiment exp1
ETNA_DIR="." etna workload add Rocq STLC --experiment exp1

# run the experiment with different tests
ETNA_DIR="." etna experiment run --name exp1 --tests bst
ETNA_DIR="." etna experiment run --name exp1 --tests rbt
ETNA_DIR="." etna experiment run --name exp1 --tests stlc
