# previous cleanup
rm -rf ~/.etna

# install etna
cargo install --path .

# setup environment
etna setup

# create a new experiment and add workloads
ETNA_DIR="." etna experiment new exp3 --local-store
ETNA_DIR="." etna workload add Racket BST --experiment exp3
ETNA_DIR="." etna workload add Racket RBT --experiment exp3
ETNA_DIR="." etna workload add Racket STLC --experiment exp3

# run the experiment with different tests
ETNA_DIR="." etna experiment run --name exp3 --tests bst
ETNA_DIR="." etna experiment run --name exp3 --tests rbt
ETNA_DIR="." etna experiment run --name exp3 --tests stlc

# visualize the results
ETNA_DIR="." etna experiment visualize --name exp3 --figure bst --tests bst
ETNA_DIR="." etna experiment visualize --name exp3 --figure rbt --tests rbt
ETNA_DIR="." etna experiment visualize --name exp3 --figure stlc --tests stlc
