#!/bin/bash

FILE="/Users/cerenmert/Documents/etna-cli/workloads/Racket/STLC/src/Generation.rkt" 
WORKLOAD_DIR="/Users/cerenmert/Documents/etna-cli/workloads/Racket/STLC"
SAVE_DIR="/Users/cerenmert/Downloads/tyche-experiments"

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
    echo "Running tyche with gen:typ = $t, gen:expr = $e"
    echo "============================================="

    cd /Users/cerenmert/Documents/etna-cli/workloads/Racket/STLC
    raco exe ./main.rkt
    ./main SinglePreserve rc

    echo "============================================="
    echo "Tyche completed run with: gen:typ = $t, gen:expr = $e"
    echo "============================================="

    
    cp $WORKLOAD_DIR/tyche-log.jsonl $SAVE_DIR/typ-${t}-expr-${e}.jsonl
    
  done
done