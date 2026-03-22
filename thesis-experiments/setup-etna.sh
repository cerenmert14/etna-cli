# previous cleanup
rm -rf ~/.etna

# install etna
cargo install --path .

# setup environment
etna setup

# create a new experiment and add workloads
ETNA_DIR="." etna experiment new exp1 --local-store
ETNA_DIR="." etna experiment new exp3 --local-store

ETNA_DIR="." etna workload add OCaml STLC --experiment exp1
ETNA_DIR="." etna workload add Racket STLC --experiment exp3