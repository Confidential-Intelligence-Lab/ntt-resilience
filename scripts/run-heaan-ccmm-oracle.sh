#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TOOL_DIR="$ROOT/tools/heaan_ccmm_oracle"
BUILD_DIR="$TOOL_DIR/build"
ORACLE="$BUILD_DIR/ccmm-oracle"
EXTERNAL="$ROOT/external/submission-720-impl"

if [[ ! -d "$EXTERNAL" ]]; then
    echo "ERROR: missing external Submission #720 reference tree:"
    echo "  $EXTERNAL"
    exit 1
fi

cmake -S "$TOOL_DIR" \
      -B "$BUILD_DIR" \
      -DCMAKE_BUILD_TYPE=Release

cmake --build "$BUILD_DIR" \
      --target ccmm-oracle \
      -j 4

if [[ ! -x "$ORACLE" ]]; then
    echo "ERROR: oracle executable not found:"
    echo "  $ORACLE"
    exit 1
fi

export OMP_NUM_THREADS=1

tmp="$(mktemp)"
trap 'rm -f "$tmp"' EXIT

"$ORACLE" | tee "$tmp"

grep -q '^CCMM_ORACLE=PASS$' "$tmp" || {
    echo "ERROR: HEaaN CCMM oracle did not report PASS"
    exit 1
}

dim="$(awk -F= '/^CCMM_ORACLE_DIM=/{print $2}' "$tmp")"
err="$(awk -F= '/^MAX_ABS_ERROR=/{print $2}' "$tmp")"

for key in \
    EXPECTED_00 EXPECTED_01 EXPECTED_10 EXPECTED_11 \
    ACTUAL_00 ACTUAL_01 ACTUAL_10 ACTUAL_11
do
    grep -q "^${key}=" "$tmp" || {
        echo "ERROR: missing oracle field: $key"
        exit 1
    }
done

echo
echo "HEAAN_CCMM_REFERENCE_GATE=PASS"
echo "HEAAN_CCMM_DIM=$dim"
echo "HEAAN_CCMM_MAX_ABS_ERROR=$err"
