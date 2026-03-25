
#!/bin/bash

EXP_DIR="$PWD/exp1"
WORKLOAD_DIR="$EXP_DIR/workloads/OCaml/STLC"
GEN_FILE="$WORKLOAD_DIR/lib/Strategies/gen_bespoke_qcheck.ml"
MAIN_FILE="$WORKLOAD_DIR/bin/main.ml" 
MAIN_DUNE="$WORKLOAD_DIR/bin/dune" 
STORE_TYCHE="$PWD/thesis-experiments/qcheck/tyche-experiments"

rm $WORKLOAD_DIR/STLC.opam

# update main.ml and dune to run experiment
echo -n "" > $MAIN_FILE
cat $PWD/thesis-experiments/qcheck/tyche_main.ml > $MAIN_FILE
echo -n "" > $MAIN_DUNE
cat $PWD/thesis-experiments/qcheck/dune > $MAIN_DUNE
echo -n "" > $WORKLOAD_DIR/lib/impl.ml
cat $PWD/thesis-experiments/qcheck/impl.ml > $WORKLOAD_DIR/lib/impl.ml

echo "============================================="
echo "Running Tyche with original generator"
echo "============================================="

rm -rf $WORKLOAD_DIR/_build

dune build --root=$WORKLOAD_DIR 
dune exec --root=$WORKLOAD_DIR stlc -- --strategy=qcheck:bespoke --property=SinglePreserve

# save results
cp $PWD/tyche-log.jsonl $STORE_TYCHE/original.jsonl


echo "==============================================="
echo "Tyche completed run with the original generator"
echo "==============================================="


# --------- Experiments with updated generator with different sizes --------- ##

# update generator 
echo -n "" > $GEN_FILE
cat $PWD/thesis-experiments/qcheck/gen_bespoke_qcheck.ml > $GEN_FILE

TYP_VALUES=(5 10 25 50 100 250)
EXPR_VALUES=(5 10 15 20 25)

cd $WORKLOAD_DIR

for t in "${TYP_VALUES[@]}"; do
  for e in "${EXPR_VALUES[@]}"; do
    
    echo "Setting: gen:typ = $t, gen:expr = $e"
    
    perl -i -pe "s/typGen \d+/typGen $t/" "$GEN_FILE"
    perl -i -pe "s/go \d+ ctx t/go $e ctx t/" "$GEN_FILE"

    echo "============================================="
    echo "Running tyche with gen:typ = $t, gen:expr = $e"
    echo "============================================="

    rm -rf _build
    dune build
    dune exec stlc -- --strategy=qcheck:bespoke --property=SinglePreserve

    echo "============================================="
    echo "Tyche completed run with: gen:typ = $t, gen:expr = $e"
    echo "============================================="
    
    cp $PWD/tyche-log.jsonl $STORE_TYCHE/gen:typ-${t}--gen:expr-${e}.jsonl
  done
done

# reset generator and main
echo -n "" > $GEN_FILE
cat $PWD/thesis-experiments/qcheck/gen_bespoke_qcheck_og.ml > $GEN_FILE
echo -n "" > $MAIN_FILE
cat $PWD/thesis-experiments/qcheck/main_og.ml > $MAIN_FILE