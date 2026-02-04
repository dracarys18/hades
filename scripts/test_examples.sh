#!/usr/bin/env bash

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

PASSED=0
FAILED=0

# Test a single example file
# Checks for a .expected file in the same directory
# If found, validates output matches; otherwise just checks compilation succeeds
test_example() {
    local file=$1
    local dir=$(dirname "$file")
    local name="${dir#examples/}"
    local expected_file="${dir}/.expected"

    echo -n "Testing $name... "

    if OUTPUT=$(hades run "$file" 2>&1); then
        if [ -f "$expected_file" ]; then
            # If we have an expected file, validate output matches
            local expected=$(cat "$expected_file")
            if echo "$OUTPUT" | grep -qF "$expected"; then
                echo -e "${GREEN}✓${NC}"
                PASSED=$((PASSED + 1))
                return 0
            else
                echo -e "${RED}✗${NC}"
                echo "  Expected: $expected"
                echo "  Got: $OUTPUT"
                FAILED=$((FAILED + 1))
                return 1
            fi
        else
            # No expected output, just check it ran successfully
            echo -e "${GREEN}✓${NC}"
            PASSED=$((PASSED + 1))
            return 0
        fi
    else
        echo -e "${RED}✗${NC}"
        echo "  Compilation/Runtime failed"
        echo "  Output: $OUTPUT"
        FAILED=$((FAILED + 1))
        return 1
    fi
}

echo "Installing hades..."
cargo install --path . --force

echo ""
echo "Running example tests..."
echo ""

# Find all main.hd files and test them grouped by category
current_category=""

while IFS= read -r file; do
    # Extract category (first directory after examples/)
    category=$(echo "$file" | cut -d'/' -f2)
    
    # Print category header when it changes
    if [ "$current_category" != "$category" ]; then
        if [ -n "$current_category" ]; then
            echo ""
        fi
        # Capitalize first letter
        category_display=$(echo "$category" | sed 's/.*/\u&/')
        echo -e "${YELLOW}${category_display}:${NC}"
        current_category="$category"
    fi
    
    test_example "$file"
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
