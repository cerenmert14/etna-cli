# previous cleanup
rm -rf ~/.etna

# install etna
cargo install --path .

# setup environment
etna setup

# create a new experiment and add workloads
ETNA_DIR="." etna experiment new QCheck-exp --local-store
ETNA_DIR="." etna experiment new Rackcheck-exp --local-store

ETNA_DIR="." etna workload add OCaml STLC --experiment QCheck-exp
ETNA_DIR="." etna workload add Racket STLC --experiment exp Rackcheck-exp
