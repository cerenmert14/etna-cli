#!/bin/bash

EXP_DIR="$PWD/exp3"
GEN_FILE="$EXP_DIR/workloads/Racket/STLC/src/Generation.rkt" 
STEPS_FILE="$EXP_DIR/workloads/Racket/STLC/steps.json"
TEST_FILE="$EXP_DIR/tests/stlc-racket.json"
STORE_ETNA="$PWD/thesis-experiments/rackcheck/etna-experiments"

# update files to run experiment
echo -n "" > $STEPS_FILE
echo -n "" > $TEST_FILE
cat $PWD/thesis-experiments/rackcheck/steps.json > $STEPS_FILE 
cat $PWD/thesis-experiments/rackcheck/stlc-racket.json > $TEST_FILE

# run & visualize the original experiment
ETNA_DIR="." etna experiment run --name exp3 --tests stlc-racket
ETNA_DIR="." etna experiment visualize --name exp3 --figure stlc --tests stlc-racket --visualization-type bucket

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

# update generator 
echo -n "" > $GEN_FILE
cat $PWD/thesis-experiments/rackcheck/Generation.rkt > $GEN_FILE

TYP_VALUES=(5 10 25 50 100 250)
EXPR_VALUES=(5 10 15 20 25)

for t in "${TYP_VALUES[@]}"; do
  for e in "${EXPR_VALUES[@]}"; do
    
    echo "Setting: gen:typ = $t, gen:expr = $e"
    perl -i -pe "s/\(gen:typ \d+\)/(gen:typ $t)/" "$GEN_FILE"
    perl -i -pe "s/\(gen:expr '\(\) tau \d+\)/(gen:expr '\(\) tau $e)/" "$GEN_FILE"

    echo "============================================="
    echo "Running ETNA with gen:typ = $t, gen:expr = $e"
    echo "============================================="

    ETNA_DIR="." etna experiment run --name exp3 --tests stlc-racket

    echo "============================================="
    echo "ETNA Completed run with: gen:typ = $t, gen:expr = $e"
    echo "============================================="
    
    ETNA_DIR="." etna experiment visualize --name exp3 --figure stlc --tests stlc-racket --visualization-type bucket

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
cat $PWD/thesis-experiments/rackcheck/Generation-OG.rkt > $GEN_FILE