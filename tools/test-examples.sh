#!/bin/bash

# Test script for Nagari examples

echo "Testing Nagari Examples..."

# Build first if needed
if [ ! -f "nagari-compiler/target/release/nagc" ] || [ ! -f "nagari-vm/target/release/nagrun" ]; then
    echo "Building Nagari tools first..."
    ./tools/build.sh
fi

NAGC="./nagari-compiler/target/release/nagc"
NAGRUN="./nagari-vm/target/release/nagrun"

# Test each example
for example in examples/*.nag; do
    echo "Testing $example..."

    # Compile
    $NAGC "$example" -v
    if [ $? -ne 0 ]; then
        echo "Failed to compile $example"
        exit 1
    fi

    # Get the .nac file name
    nac_file="${example%.nag}.nac"

    # Run (with timeout for safety)
    timeout 10s $NAGRUN "$nac_file" -v
    if [ $? -ne 0 ]; then
        echo "Failed to run $nac_file"
        exit 1
    fi

    echo "✓ $example passed"
    echo ""
done

echo "All tests passed!"
