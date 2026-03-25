#!/bin/bash


EXP_DIR="$PWD/exp1"
WORKLOAD_DIR="$EXP_DIR/workloads/OCaml/STLC"
GEN_FILE="$WORKLOAD_DIR/lib/Strategies/gen_bespoke_qcheck.ml" 
STEPS_FILE="$EXP_DIR/workloads/OCaml/STLC/steps.json"
TEST_FILE="$EXP_DIR/tests/stlc-ocaml.json"
RUNNER_FILE="$WORKLOAD_DIR/bin/runner_qcheck.ml"
STORE_ETNA="$PWD/thesis-experiments/qcheck/etna-experiments"

# update files to run experiment
rm $WORKLOAD_DIR/STLC.opam
echo -n "" > $STEPS_FILE
echo -n "" > $TEST_FILE
echo -n "" > $RUNNER_FILE
cat $PWD/thesis-experiments/qcheck/steps.json > $STEPS_FILE 
cat $PWD/thesis-experiments/qcheck/stlc-ocaml.json > $TEST_FILE
cat $PWD/thesis-experiments/qcheck/runner_qcheck.ml > $RUNNER_FILE

# run & visualize the original experiment
ETNA_DIR="." etna experiment run --name exp1 --tests stlc-ocaml
ETNA_DIR="." etna experiment visualize --name exp1 --figure stlc --tests stlc-ocaml --visualization-type bucket

# save results to rackcheck/etna-experiments
cp -r $EXP_DIR/figures $STORE_ETNA/original/
cat $EXP_DIR/store.jsonl > $STORE_ETNA/original/original-store.jsonl

# reset store
echo -n "" > $EXP_DIR/store.jsonl

# --------- Experiments with updated generator with different sizes --------- ##

if [ ! -f "$GEN_FILE" ]; then
    echo "Error: File '$GEN_FILE' not found."
    exit 1
fi

TYP_VALUES=(5 10 25 50 100 250)
EXPR_VALUES=(5 10 15 20 25)

# update generator 
echo -n "" > $GEN_FILE
cat $PWD/thesis-experiments/qcheck/gen_bespoke_qcheck.ml > $GEN_FILE

for t in "${TYP_VALUES[@]}"; do
  for e in "${EXPR_VALUES[@]}"; do
    
    echo "Setting: gen:typ = $t, gen:expr = $e"
    perl -i -pe "s/let\* typ = typGen \d+ in/let* typ = typGen $t in/" "$GEN_FILE"
    perl -i -pe "s/go \d+ ctx t(\s*)$/go $e ctx t\1/" "$GEN_FILE"

    echo "============================================="
    echo "Running ETNA with gen:typ = $t, gen:expr = $e"
    echo "============================================="

    ETNA_DIR="." etna experiment run --name exp1 --tests stlc-ocaml

    echo "============================================="
    echo "ETNA Completed run with: gen:typ = $t, gen:expr = $e"
    echo "============================================="
    
    ETNA_DIR="." etna experiment visualize --name exp1 --figure stlc --tests stlc-ocaml --visualization-type bucket

    echo "============================================="
    echo "ETNA is done visualizing with: gen:typ = $t, gen:expr = $e"
    echo "============================================"
    
    # save results and reset
    cp -r $EXP_DIR/figures $STORE_ETNA/gen:typ-${t}--gen:expr-${e}/
    cat $EXP_DIR/store.jsonl > $STORE_ETNA/gen:typ-${t}--gen:expr-${e}/gen:typ-${t}--gen:expr-${e}-store.jsonl
    
    echo -n "" > $EXP_DIR/store.jsonl
  done
done

# reset generator
echo -n "" > $GEN_FILE
cat $PWD/thesis-experiments/qcheck/gen_bespoke_qcheck_og.ml > $GEN_FILE