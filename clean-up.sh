EXP_DIR="/Users/cerenmert/Downloads/etna-cli/exp3"
FILE_TEST="$EXP_DIR/tests/stlc.json" 
FILE_STEPS="$EXP_DIR/workloads/Racket/STLC/steps.json"

rm -rf $EXP_DIR

rm -rf ~/.etna

cargo install --path .

etna setup

ETNA_DIR="." etna experiment new exp3 --local-store
ETNA_DIR="." etna workload add Racket STLC --experiment exp3

perl -pi -e 's/Rocq/Racket/g; s/BespokeGenerator/rc/g' "$FILE_TEST"
perl -0777 -pi -e 's/("args"\s*:\s*\[[^\]]*\])/my $block=$1; $block=~s|"\$property"|"\${property}"|g; $block/egs' "$FILE_STEPS"