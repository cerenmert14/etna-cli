#!/bin/bash

EXP_DIR="/Users/cerenmert/Downloads/etna-cli/exp3"
FILE="$EXP_DIR/workloads/Racket/STLC/src/Generation.rkt" 

TYP_VALUES=(5 10 25 50 100 250)
EXPR_VALUES=(5 10 15 20 25)

if [ ! -f "$FILE" ]; then
    echo "Error: File '$FILE' not found."
    exit 1
fi

for t in "${TYP_VALUES[@]}"; do
  for e in "${EXPR_VALUES[@]}"; do
    
    echo "Setting: gen:typ = $t, gen:expr = $e"
    perl -i -pe "s/\(gen:typ \d+\)/(gen:typ $t)/" "$FILE"
    perl -i -pe "s/\(gen:expr '\(\) tau \d+\)/(gen:expr '\(\) tau $e)/" "$FILE"

    echo "============================================="
    echo "Running ETNA with gen:typ = $t, gen:expr = $e"
    echo "============================================="

    ETNA_DIR="." etna experiment run --name exp3 --tests stlc

    echo "============================================="
    echo "ETNA Completed run with: gen:typ = $t, gen:expr = $e"
    echo "============================================="
    
    ETNA_DIR="." etna experiment visualize --name exp3 --figure stlc --tests stlc --visualization-type bucket --buckets 0.0001 0.001 0.01 0.1 1 10

    echo "============================================="
    echo "ETNA is done visualizing with: gen:typ = $t, gen:expr = $e"
    echo "============================================"

    cp -r $EXP_DIR/figures /Users/cerenmert/Downloads/etna-experiments-1-1-1-1/exp-${t}-${e}
    cat $EXP_DIR/store.jsonl >/Users/cerenmert/Downloads/etna-experiments-1-1-1-1/exp-${t}-${e}/${t}_${e}.jsonl

    echo -n "" > $EXP_DIR/store.jsonl
    
  done
done