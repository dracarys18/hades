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

test_example() {
    local file=$1
    local dir=$(dirname "$file")
    local basename=$(basename "$file" .hd)
    local name="${file#examples/}"
    local name="${name%.hd}"
    
    local expected_file
    local command
    if [ "$basename" = "main" ]; then
        expected_file="${dir}/.expected"
        command="run"
    else
        expected_file="${file%.hd}.expected"
        command="check"
    fi
    
    echo -n "Testing $name... "

    set +e
    OUTPUT=$(cargo run --bin hades --quiet -- $command "$file" 2>&1)
    EXIT_CODE=$?
    set -e
    
    if [ -f "$expected_file" ]; then
        local expected=$(cat "$expected_file")
        if [ "$OUTPUT" = "$expected" ]; then
            echo -e "${GREEN}✓${NC}"
            PASSED=$((PASSED + 1))
            return 0
        else
            echo -e "${RED}✗${NC}"
            echo "  Expected output differs"
            echo "  Expected file: $expected_file"
            FAILED=$((FAILED + 1))
            return 1
        fi
    else
        if [ $EXIT_CODE -eq 0 ]; then
            echo -e "${GREEN}✓${NC}"
            PASSED=$((PASSED + 1))
            return 0
        else
            echo -e "${RED}✗${NC}"
            echo "  File: $file"
            echo "  Output: $OUTPUT"
            FAILED=$((FAILED + 1))
            return 1
        fi
    fi
}

echo "Building hades..."
cargo build --bin hades --quiet

echo ""
echo "Running example tests..."
echo ""

current_category=""

while IFS= read -r file; do
    category=$(echo "$file" | cut -d'/' -f2)
    
    if [ "$current_category" != "$category" ]; then
        if [ -n "$current_category" ]; then
            echo ""
        fi
        category_display=$(echo "$category" | sed 's/.*/\u&/')
        echo -e "${YELLOW}${category_display}:${NC}"
        current_category="$category"
    fi
    
    test_example "$file" || true
done < <(find examples -name "*.hd" -type f | sort)

echo ""
echo "================================"
echo -e "${GREEN}Passed: $PASSED${NC}"
if [ $FAILED -gt 0 ]; then
    echo -e "${RED}Failed: $FAILED${NC}"
    exit 1
else
    echo -e "${GREEN}All tests passed!${NC}"
fi
