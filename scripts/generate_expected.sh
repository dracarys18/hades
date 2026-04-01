#!/usr/bin/env bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

echo "Building hades..."
cargo build --bin hades --quiet
HADES=$(cargo build --bin hades --message-format=json --quiet 2>/dev/null \
    | python3 -c "import sys,json; [print(m['executable']) for l in sys.stdin for m in [json.loads(l)] if m.get('executable')]")

echo ""
echo "Generating .expected files..."
echo ""

GENERATED=0
SKIPPED=0

while IFS= read -r file; do
    dir=$(dirname "$file")
    name="${file#examples/}"
    name="${name%.hd}"

    if [ -f "${dir}/.expect_failure" ]; then
        echo "Skipping $name (expect_failure)"
        SKIPPED=$((SKIPPED + 1))
        continue
    fi

    set +e
    OUTPUT=$("$HADES" run "$file" 2>&1)
    EXIT_CODE=$?
    set -e

    if [ $EXIT_CODE -ne 0 ]; then
        echo "Skipping $name (non-zero exit: $EXIT_CODE)"
        SKIPPED=$((SKIPPED + 1))
        continue
    fi

    printf "%s" "$OUTPUT" > "${dir}/.expected"
    echo "Generated $name"
    GENERATED=$((GENERATED + 1))

done < <(find examples -name "main.hd" -type f | sort)

echo ""
echo "================================"
echo "Generated: $GENERATED"
echo "Skipped:   $SKIPPED"
