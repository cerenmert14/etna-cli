#!/bin/bash
set -euo pipefail

usage() {
echo 'Usage: ./steps.sh [PARAMS]

Params:
  --cross <VALUE>
  --workload_path <VALUE>
  --property <VALUE>
  --stages <LIST>  (optional; comma-separated: check,build,run or "all")
  -h, --help       Show this help and exit'
}

# Initialize variables
cross=""
workload_path=""
property=""
STAGES=""

# Parse keyword args
while [ $# -gt 0 ]; do
  case "$1" in
    --cross=*) cross="${1#*=}"; shift ;;
    --cross) shift; [ $# -gt 0 ] || { echo "Missing value for --cross" >&2; usage; exit 2; }; cross="$1"; shift ;;
    --workload_path=*) workload_path="${1#*=}"; shift ;;
    --workload_path) shift; [ $# -gt 0 ] || { echo "Missing value for --workload_path" >&2; usage; exit 2; }; workload_path="$1"; shift ;;
    --property=*) property="${1#*=}"; shift ;;
    --property) shift; [ $# -gt 0 ] || { echo "Missing value for --property" >&2; usage; exit 2; }; property="$1"; shift ;;
    --stages=*) STAGES="${1#*=}"; shift ;;
    --stages)   shift; [ $# -gt 0 ] || { echo "Missing value for --stages" >&2; usage; exit 2; }; STAGES="$1"; shift ;;
    --) shift; break ;;
    -h|--help) usage; exit 0 ;;
    --*) echo "Unknown option: $1" >&2; usage; exit 2 ;;
    *) break ;;
  esac
done

# Enforce required vars
[ -n "$cross" ] || { echo "Missing required option: --cross" >&2; usage; exit 2; }
[ -n "$workload_path" ] || { echo "Missing required option: --workload_path" >&2; usage; exit 2; }
[ -n "$property" ] || { echo "Missing required option: --property" >&2; usage; exit 2; }

# Compute requested stages (default: all)
if [ -z "$STAGES" ]; then
  STAGES="all"
fi

# Normalize, validate, and store selection (portable: no associative arrays)
W_CHECK=0
W_BUILD=0
W_RUN=0

if [ "$STAGES" = "all" ]; then
  W_CHECK=1
  W_BUILD=1
  W_RUN=1
else
  IFS=',' read -r -a __requested_stages <<< "$STAGES"
  for s in "${__requested_stages[@]}"; do
    s="$(printf '%s' "$s" | tr '[:upper:]' '[:lower:]' | xargs)"
    case "$s" in
      check) W_CHECK=1 ;;
      build) W_BUILD=1 ;;
      run)   W_RUN=1 ;;
      "" )   ;; # ignore empties
      * )    echo "Unknown stage: $s" >&2; usage; exit 2 ;;
    esac
  done
fi

# Ensure at least one stage selected
if [ $((W_CHECK + W_BUILD + W_RUN)) -eq 0 ]; then
  echo "No valid stages selected (got: $STAGES)" >&2
  usage
  exit 2
fi

# Build a human-readable list
__list=""
[ $W_CHECK -eq 1 ] && __list="${__list}check "
[ $W_BUILD -eq 1 ] && __list="${__list}build "
[ $W_RUN  -eq 1 ] && __list="${__list}run"
echo "[steps.sh] Stages to run: $__list" >&2

# Export for children
export cross
export workload_path
export property

echo "[steps.sh] Effective options:" >&2

# ===== Check Steps =====
if [ $W_CHECK -eq 1 ]; then
  echo 'Check steps are completed.'
fi

# ===== Build Steps =====
if [ $W_BUILD -eq 1 ]; then
  echo 'Build steps are completed.'
fi

# ===== Run Steps =====
if [ $W_RUN -eq 1 ]; then
    if [[ $cross == "true" ]]; then
(cd $workload_path &&         uv run main-sampler.py $property $tests)
    elif [[ $cross == "false" ]]; then
(cd $workload_path &&         uv run main.py quickcheck $property)
    else
        echo "Unknown option: $cross" >&2; usage; exit 2;
    fi
echo 'Run steps are completed.'
fi