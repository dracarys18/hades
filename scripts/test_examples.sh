#!/usr/bin/env bash

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

cd "$PROJECT_ROOT"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

PASSED=0
FAILED=0

echo "Building hades..."
cargo build --bin hades --quiet
HADES=$(cargo build --bin hades --message-format=json --quiet 2>/dev/null \
    | python3 -c "import sys,json; [print(m['executable']) for l in sys.stdin for m in [json.loads(l)] if m.get('executable')]")

echo ""
echo "Running example tests..."
echo ""

test_example() {
    local file=$1
    local dir=$(dirname "$file")
    local basename=$(basename "$file" .hd)
    local name="${file#examples/}"
    local name="${name%.hd}"

    local expected_file="${dir}/.expected"
    local expect_failure=false
    [ -f "${dir}/.expect_failure" ] && expect_failure=true

    local command="run"

    echo -n "Testing $name... "

    set +e
    OUTPUT=$("$HADES" $command "$file" 2>&1)
    EXIT_CODE=$?
    set -e

    if [ -f "$expected_file" ]; then
        local expected
        expected=$(cat "$expected_file")
        if [ "$OUTPUT" = "$expected" ]; then
            echo -e "${GREEN}✓${NC}"
            PASSED=$((PASSED + 1))
        else
            echo -e "${RED}✗${NC}"
            echo "  Expected: $expected"
            echo "  Got:      $OUTPUT"
            FAILED=$((FAILED + 1))
        fi
    elif [ "$expect_failure" = true ]; then
        if [ $EXIT_CODE -ne 0 ]; then
            echo -e "${GREEN}✓${NC}"
            PASSED=$((PASSED + 1))
        else
            echo -e "${RED}✗${NC}"
            echo "  Expected failure but exited 0"
            echo "  Output: $OUTPUT"
            FAILED=$((FAILED + 1))
        fi
    else
        if [ $EXIT_CODE -eq 0 ]; then
            echo -e "${GREEN}✓${NC}"
            PASSED=$((PASSED + 1))
        else
            echo -e "${RED}✗${NC}"
            echo "  File: $file"
            echo "  Output: $OUTPUT"
            FAILED=$((FAILED + 1))
        fi
    fi
}

current_category=""

while IFS= read -r file; do
    category=$(echo "$file" | cut -d'/' -f2)

    if [ "$current_category" != "$category" ]; then
        [ -n "$current_category" ] && echo ""
        category_display=$(echo "$category" | awk '{print toupper(substr($0,1,1)) tolower(substr($0,2))}')
        echo -e "${YELLOW}${category_display}:${NC}"
        current_category="$category"
    fi

    test_example "$file" || true
done < <(find examples -name "main.hd" -type f | sort)

echo ""
echo "================================"
echo -e "${GREEN}Passed: $PASSED${NC}"
if [ $FAILED -gt 0 ]; then
    echo -e "${RED}Failed: $FAILED${NC}"
    exit 1
else
    echo -e "${GREEN}All tests passed!${NC}"
fi
