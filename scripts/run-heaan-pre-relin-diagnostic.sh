#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

SOURCE="$ROOT/external/submission-720-impl/matrix-mult/src/MatrixEvaluator.cpp"
TOOL_DIR="$ROOT/tools/heaan_ccmm_oracle"
BUILD_DIR="$TOOL_DIR/build"
ORACLE="$BUILD_DIR/ccmm-oracle"

if [[ ! -f "$SOURCE" ]]; then
    echo "ERROR: MatrixEvaluator.cpp not found:"
    echo "  $SOURCE"
    exit 1
fi

backup="$(mktemp)"
output="$(mktemp)"

cp "$SOURCE" "$backup"

cleanup() {
    cp "$backup" "$SOURCE"
    rm -f "$backup" "$output"
}

trap cleanup EXIT

SOURCE="$SOURCE" python3 <<'PY'
from pathlib import Path
import os

path = Path(os.environ["SOURCE"])
text = path.read_text()

include = '#include "HEaaN/PolynomialHandle.hpp"\n'

if include not in text:
    anchor = '#include "MatrixEvaluator.hpp"\n'
    if anchor not in text:
        raise SystemExit("ERROR: MatrixEvaluator include anchor not found")
    text = text.replace(anchor, anchor + include, 1)

call = "large_manipulator_.addCtSkCt("
start = text.find(call)

if start == -1:
    raise SystemExit("ERROR: addCtSkCt call not found")

# Make sure we are instrumenting the ordinary matrixMult(), not the later
# lightweight implementation.
function_start = text.rfind(
    "void MatrixEvaluator::matrixMult(",
    0,
    start,
)

if function_start == -1:
    raise SystemExit("ERROR: enclosing matrixMult function not found")

lightweight = text.rfind(
    "void MatrixEvaluator::matrixMultLightweight(",
    0,
    start,
)

if lightweight > function_start:
    raise SystemExit(
        "ERROR: first addCtSkCt unexpectedly belongs to matrixMultLightweight"
    )

# Find the terminating ');' for this call without depending on indentation.
end = text.find(");", start)

if end == -1:
    raise SystemExit("ERROR: addCtSkCt terminator not found")

end += 2

diagnostic = r'''

        if (col == 0) {
            std::cout << "PRE_RELIN_COLUMN=0" << std::endl;
            std::cout << "PRE_RELIN_NUM_POLY=" << buffer.getNumPoly()
                      << std::endl;
            std::cout << "PRE_RELIN_LEVEL=" << buffer.getLevel()
                      << std::endl;
            std::cout << "PRE_RELIN_RESCALE_COUNTER="
                      << buffer.getRescaleCounter() << std::endl;

            constexpr u64 prefix_len = 8;

            for (u64 poly = 0; poly < buffer.getNumPoly(); ++poly) {
                std::cout << "PRE_RELIN_POLY" << poly << "_IS_NTT="
                          << (isNTT(buffer.getPoly(poly)) ? 1 : 0)
                          << std::endl;

                const auto *data =
                    buffer.getPolyData(poly, buffer.getLevel());

                std::cout << "PRE_RELIN_POLY" << poly << "_PREFIX=";

                for (u64 i = 0; i < prefix_len; ++i) {
                    if (i != 0)
                        std::cout << ",";
                    std::cout << data[i];
                }

                std::cout << std::endl;
            }
        }
'''

text = text[:end] + diagnostic + text[end:]
path.write_text(text)

print("Temporarily instrumented pre-relinearization buffer")
PY

cmake -S "$TOOL_DIR" \
      -B "$BUILD_DIR" \
      -DCMAKE_BUILD_TYPE=Release

cmake --build "$BUILD_DIR" \
      --target ccmm-oracle \
      -j 4

if [[ ! -x "$ORACLE" ]]; then
    echo "ERROR: oracle executable missing:"
    echo "  $ORACLE"
    exit 1
fi

export OMP_NUM_THREADS=1

echo
echo "===== HEAaN PRE-RELINEARIZATION DIAGNOSTIC ====="
echo

"$ORACLE" | tee "$output"

grep -q '^CCMM_ORACLE=PASS$' "$output" || {
    echo "ERROR: final HEaaN CCMM oracle failed"
    exit 1
}

grep -q '^PRE_RELIN_COLUMN=0$' "$output" || {
    echo "ERROR: missing pre-relinearization diagnostic"
    exit 1
}

grep -q '^PRE_RELIN_NUM_POLY=3$' "$output" || {
    echo "ERROR: pre-relinearization ciphertext is not degree 2"
    exit 1
}

for poly in 0 1 2; do
    grep -q "^PRE_RELIN_POLY${poly}_IS_NTT=0$" "$output" || {
        echo "ERROR: polynomial $poly is unexpectedly in NTT form"
        exit 1
    }

    grep -q "^PRE_RELIN_POLY${poly}_PREFIX=" "$output" || {
        echo "ERROR: polynomial $poly prefix missing"
        exit 1
    }
done

echo
echo "HEAAN_PRE_RELIN_GATE=PASS"
echo "HEAAN_PRE_RELIN_LAYOUT=c0,c1,c2"
echo "HEAAN_PRE_RELIN_NUM_POLY=3"
echo "HEAAN_PRE_RELIN_NTT_STATE=COEFFICIENT"
