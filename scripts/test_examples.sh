#!/usr/bin/env bash

set -e

HADES_BIN="${HADES_BIN:-cargo run --release --}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

PASSED=0
FAILED=0

test_example() {
    local file=$1
    local name=$(basename "$file" .hd)

    echo -n "Testing $name... "

    if OUTPUT=$($HADES_BIN run "$file" 2>&1); then
        echo -e "${GREEN}✓${NC}"
        PASSED=$((PASSED + 1))
        return 0
    else
        echo -e "${RED}✗${NC}"
        echo "  Output: $OUTPUT"
        FAILED=$((FAILED + 1))
        return 1
    fi
}

test_with_output() {
    local file=$1
    local expected=$2
    local name=$(basename "$file" .hd)

    echo -n "Testing $name... "

    OUTPUT=$($HADES_BIN run "$file" 2>&1)

    if echo "$OUTPUT" | grep -q "$expected"; then
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
}

echo "Building hades..."
cargo build --release

echo ""
echo "Running example tests..."
echo ""

echo -e "${YELLOW}Basics:${NC}"
test_with_output "examples/basics/hello_world.hd" "Hello world"

echo ""
echo -e "${YELLOW}Operations:${NC}"
test_with_output "examples/operations/addition.hd" "A + B IS 8"
test_with_output "examples/operations/assign.hd" "BIIIIIIGGGG STRINGGGG"
test_with_output "examples/operations/expr.hd" "36"

echo ""
echo -e "${YELLOW}Arrays:${NC}"
test_with_output "examples/arrays/array_simple.hd" "a\[2\] = 30"
test_with_output "examples/arrays/arrays.hd" "Array value 12"
test_with_output "examples/arrays/string_array.hd" "s\[0\] = Hello"

echo ""
echo -e "${YELLOW}Control Flow:${NC}"
test_with_output "examples/control_flow/if_else.hd" "Voila"

echo ""
echo -e "${YELLOW}Loops:${NC}"
test_with_output "examples/loops/for.hd" "10"
test_with_output "examples/loops/while.hd" "A is 8"

echo ""
echo -e "${YELLOW}Structs:${NC}"
test_with_output "examples/structs/field_access.hd" "Value is Alice"
test_with_output "examples/structs/field_access_complete.hd" "Nested field access: 0, 200"
test_with_output "examples/structs/field_access_extended.hd" "Array struct field access: 10"
test_with_output "examples/structs/field_modify.hd" "After 2"

echo ""
echo "================================"
echo -e "${GREEN}Passed: $PASSED${NC}"
if [ $FAILED -gt 0 ]; then
    echo -e "${RED}Failed: $FAILED${NC}"
    exit 1
else
    echo -e "${GREEN}All tests passed!${NC}"
fi
