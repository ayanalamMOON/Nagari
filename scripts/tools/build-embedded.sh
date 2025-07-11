#!/bin/bash
set -e

echo "Building Nagari Embedded Runtime..."

# Build Rust library
cd nagari-embedded

echo "Building core embedded runtime..."
cargo build --release

echo "Building Python bindings..."
if command -v python3 >/dev/null 2>&1; then
    cargo build --release --features python

    # Create Python wheel
    if command -v maturin >/dev/null 2>&1; then
        maturin build --release --features python
        echo "Python wheel built successfully"
    else
        echo "maturin not found, skipping Python wheel creation"
        echo "Install with: pip install maturin"
    fi
else
    echo "Python3 not found, skipping Python bindings"
fi

echo "Building Node.js bindings..."
if command -v node >/dev/null 2>&1; then
    cargo build --release --features nodejs
    echo "Node.js bindings built successfully"
else
    echo "Node.js not found, skipping Node.js bindings"
fi

echo "Building C bindings..."
cargo build --release --features c-bindings

# Create C header file
cat > target/release/nagari.h << 'EOF'
#ifndef NAGARI_H
#define NAGARI_H

#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

// Forward declarations
typedef struct CNagariRuntime CNagariRuntime;

// Configuration structure
typedef struct CNagariConfig {
    size_t memory_limit;
    uint64_t execution_timeout;
    int allow_io;
    int allow_network;
    int sandbox_mode;
    int debug_mode;
} CNagariConfig;

// Value type enumeration
typedef enum CNagariValueType {
    NAGARI_VALUE_NONE = 0,
    NAGARI_VALUE_BOOL = 1,
    NAGARI_VALUE_INT = 2,
    NAGARI_VALUE_FLOAT = 3,
    NAGARI_VALUE_STRING = 4,
    NAGARI_VALUE_ARRAY = 5,
    NAGARI_VALUE_OBJECT = 6
} CNagariValueType;

// Value structures
typedef struct CNagariArray {
    struct CNagariValue* values;
    size_t length;
    size_t capacity;
} CNagariArray;

typedef struct CNagariObject {
    char** keys;
    struct CNagariValue* values;
    size_t length;
    size_t capacity;
} CNagariObject;

typedef union CNagariValueData {
    int bool_val;
    int64_t int_val;
    double float_val;
    char* string_val;
    CNagariArray* array_val;
    CNagariObject* object_val;
} CNagariValueData;

typedef struct CNagariValue {
    CNagariValueType value_type;
    CNagariValueData data;
} CNagariValue;

// Host function callback type
typedef CNagariValue (*CNagariHostFunction)(
    const CNagariValue* args,
    size_t args_count,
    void* user_data
);

// Runtime functions
CNagariRuntime* nagari_runtime_new(const CNagariConfig* config);
void nagari_runtime_destroy(CNagariRuntime* runtime);

CNagariValue nagari_run_script(CNagariRuntime* runtime, const char* script);
CNagariValue nagari_call_function(
    CNagariRuntime* runtime,
    const char* function_name,
    const CNagariValue* args,
    size_t args_count
);

int nagari_load_module(CNagariRuntime* runtime, const char* name, const char* code);
int nagari_set_global(CNagariRuntime* runtime, const char* name, CNagariValue value);
CNagariValue nagari_get_global(CNagariRuntime* runtime, const char* name);

int nagari_register_function(
    CNagariRuntime* runtime,
    const char* name,
    CNagariHostFunction func,
    void* user_data
);

int nagari_reset(CNagariRuntime* runtime);

// Value management
void nagari_value_destroy(CNagariValue* value);

// Helper macros
#define NAGARI_DEFAULT_CONFIG() { \
    .memory_limit = 64 * 1024 * 1024, \
    .execution_timeout = 5000, \
    .allow_io = 0, \
    .allow_network = 0, \
    .sandbox_mode = 1, \
    .debug_mode = 0 \
}

#define NAGARI_NULL_VALUE() { \
    .value_type = NAGARI_VALUE_NONE, \
    .data = { .bool_val = 0 } \
}

#define NAGARI_BOOL_VALUE(b) { \
    .value_type = NAGARI_VALUE_BOOL, \
    .data = { .bool_val = (b) ? 1 : 0 } \
}

#define NAGARI_INT_VALUE(i) { \
    .value_type = NAGARI_VALUE_INT, \
    .data = { .int_val = (i) } \
}

#define NAGARI_FLOAT_VALUE(f) { \
    .value_type = NAGARI_VALUE_FLOAT, \
    .data = { .float_val = (f) } \
}

#ifdef __cplusplus
}
#endif

#endif // NAGARI_H
EOF

echo "Creating example C program..."
cat > examples/c_example.c << 'EOF'
#include "nagari.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

// Host function example
CNagariValue print_message(const CNagariValue* args, size_t args_count, void* user_data) {
    if (args_count > 0 && args[0].value_type == NAGARI_VALUE_STRING) {
        printf("Message from Nagari: %s\n", args[0].data.string_val);
    }
    return NAGARI_NULL_VALUE();
}

int main() {
    // Create runtime with default configuration
    CNagariConfig config = NAGARI_DEFAULT_CONFIG();
    config.allow_io = 1;  // Allow I/O for this example

    CNagariRuntime* runtime = nagari_runtime_new(&config);
    if (!runtime) {
        fprintf(stderr, "Failed to create Nagari runtime\n");
        return 1;
    }

    // Register host function
    nagari_register_function(runtime, "print_message", print_message, NULL);

    // Run Nagari script
    const char* script =
        "def greet(name):\n"
        "    message = f'Hello, {name}!'\n"
        "    print_message(message)\n"
        "    return message\n"
        "\n"
        "result = greet('C Program')\n";

    CNagariValue result = nagari_run_script(runtime, script);

    // Call Nagari function from C
    CNagariValue name_arg = NAGARI_STRING_VALUE("World");
    CNagariValue call_result = nagari_call_function(runtime, "greet", &name_arg, 1);

    if (call_result.value_type == NAGARI_VALUE_STRING) {
        printf("Function returned: %s\n", call_result.data.string_val);
    }

    // Clean up
    nagari_value_destroy(&result);
    nagari_value_destroy(&call_result);
    nagari_runtime_destroy(runtime);

    return 0;
}
EOF

echo "Creating Python setup.py..."
cat > setup.py << 'EOF'
from pyo3_build_config import add_extension_module

def build_rust_extension():
    add_extension_module(
        "nagari",
        "Cargo.toml",
        binding_crate_name="nagari_embedded",
        python_source="python",
        features=["python"]
    )

if __name__ == "__main__":
    build_rust_extension()
EOF

echo "Creating Python example..."
mkdir -p examples
cat > examples/python_example.py << 'EOF'
import asyncio
from nagari import VM, NagariError

async def main():
    # Create runtime
    vm = VM(
        memory_limit=32 * 1024 * 1024,  # 32MB
        execution_timeout=5000,         # 5 seconds
        allow_io=True,
        allow_network=False
    )

    # Register Python function
    @vm.register_function
    def log_info(message: str) -> None:
        print(f"[INFO] {message}")

    @vm.register_async_function
    async def fetch_data(url: str) -> dict:
        # Simulate async operation
        await asyncio.sleep(0.1)
        return {"url": url, "status": "success", "data": "sample data"}

    # Load Nagari script
    script = """
    async def process_request(request):
        log_info(f"Processing request: {request['id']}")

        data = await fetch_data(request['url'])

        return {
            'request_id': request['id'],
            'response': data,
            'processed_at': 'now'
        }

    def calculate_score(values):
        return sum(v * 2 for v in values if v > 0)
    """

    vm.run(script)

    # Call Nagari functions
    score = vm.call('calculate_score', [1, -2, 3, 4, -1])
    print(f"Score: {score}")

    # Call async function
    request = {
        'id': 123,
        'url': 'https://api.example.com/data'
    }

    result = await vm.call_async('process_request', request)
    print(f"Async result: {result}")

if __name__ == "__main__":
    asyncio.run(main())
EOF

echo "Creating Node.js example..."
cat > examples/nodejs_example.js << 'EOF'
const { NagariRuntime } = require('../target/release/nagari_embedded');

async function main() {
    // Create runtime
    const vm = new NagariRuntime({
        memoryLimit: 32 * 1024 * 1024,  // 32MB
        executionTimeout: 5000,         // 5 seconds
        allowIO: true,
        allowNetwork: false
    });

    // Register Node.js functions
    vm.registerFunction('console_log', (args) => {
        console.log('From Nagari:', ...args);
        return null;
    });

    vm.registerFunction('get_timestamp', () => {
        return Date.now();
    });

    // Load Nagari script
    const script = `
def process_data(items):
    console_log("Processing", len(items), "items")

    processed = []
    for item in items:
        if item.get('active', false):
            processed.append({
                'id': item['id'],
                'value': item['value'] * 2,
                'timestamp': get_timestamp()
            })

    return processed

def fibonacci(n):
    if n <= 1:
        return n
    return fibonacci(n-1) + fibonacci(n-2)
    `;

    vm.run(script);

    // Test data processing
    const testData = [
        { id: 1, value: 10, active: true },
        { id: 2, value: 20, active: false },
        { id: 3, value: 30, active: true }
    ];

    const processed = vm.call('process_data', testData);
    console.log('Processed data:', processed);

    // Test Fibonacci
    const fibResult = vm.call('fibonacci', 10);
    console.log('Fibonacci(10):', fibResult);

    // Clean up
    vm.reset();
}

main().catch(console.error);
EOF

echo "Embedded runtime build completed!"
echo "Generated files:"
echo "  - target/release/libnagari_embedded.a - Static library"
echo "  - target/release/libnagari_embedded.so - Shared library (Unix)"
echo "  - target/release/nagari_embedded.dll - Dynamic library (Windows)"
echo "  - target/release/nagari.h - C header file"
echo "  - examples/ - Example programs for C, Python, and Node.js"

cd ..
