#!/bin/bash

EXP_DIR="$PWD/exp3"
WORKLOAD_DIR="$EXP_DIR/workloads/Racket/STLC"
GEN_FILE="$WORKLOAD_DIR/src/Generation.rkt" 
MAIN_FILE="$WORKLOAD_DIR/main.rkt" 
STORE_TYCHE="$PWD/thesis-experiments/rackcheck/tyche-experiments"

# update main.rkt to run experiment
echo -n "" > $MAIN_FILE
cat $PWD/thesis-experiments/rackcheck/main.rkt > $MAIN_FILE

echo "============================================="
echo "Running Tyche with original generator"
echo "============================================="

# run & visualize the original experiment
raco exe $WORKLOAD_DIR/main.rkt
$WORKLOAD_DIR/main SinglePreserve rc


echo "==============================================="
echo "Tyche completed run with the original generator"
echo "==============================================="

# save results
cp $PWD/tyche-log.jsonl $STORE_TYCHE/original.jsonl

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
    echo "Running Tyche with gen:typ = $t, gen:expr = $e"
    echo "============================================="

    raco exe $WORKLOAD_DIR/main.rkt
    $WORKLOAD_DIR/main SinglePreserve rc

    echo "============================================="
    echo "Tyche completed run with: gen:typ = $t, gen:expr = $e"
    echo "============================================="

    cp $PWD/tyche-log.jsonl $STORE_TYCHE/gen:typ-${t}--gen:expr-${e}.jsonl
    
  done
done

# reset generator and main
echo -n "" > $GEN_FILE
cat $PWD/thesis-experiments/rackcheck/Generation-OG.rkt > $GEN_FILE
echo -n "" > $MAIN_FILE
cat $PWD/thesis-experiments/rackcheck/main-OG.rkt > $MAIN_FILE