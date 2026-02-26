# previous cleanup
rm -rf ~/.etna

# install etna
cargo install --path .

# setup environment
etna setup

# create a new experiment and add workloads
ETNA_DIR="." etna experiment new exp3 --local-store
# this creates `exp3/store.json` which holds the experiment results
ETNA_DIR="." etna workload add Racket BST --experiment exp3
ETNA_DIR="." etna workload add Racket RBT --experiment exp3
ETNA_DIR="." etna workload add Racket STLC --experiment exp3

# run the experiment with different tests
ETNA_DIR="." etna experiment run --name exp3 --tests bst-racket # this runs all tests in `exp3/tests/bst.json`
ETNA_DIR="." etna experiment run --name exp3 --tests bst-racket --cross # this runs all tests in `exp3/tests/bst.json` with cross language mode.

ETNA_DIR="." etna experiment run --name exp3 --tests rbt-racket
ETNA_DIR="." etna experiment run --name exp3 --tests stlc-racket

# visualize the results
ETNA_DIR="." etna experiment visualize --name exp3 --figure bst --tests bst-racket
ETNA_DIR="." etna experiment visualize --name exp3 --figure rbt --tests rbt-racket
ETNA_DIR="." etna experiment visualize --name exp3 --figure stlc --tests stlc-racket
