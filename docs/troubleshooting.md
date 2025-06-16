# Nagari Troubleshooting Guide

A comprehensive guide to solving common issues when developing with the Nagari programming language.

## Table of Contents

1. [Installation Issues](#installation-issues)
2. [Compilation Errors](#compilation-errors)
3. [Runtime Errors](#runtime-errors)
4. [JavaScript Interop Issues](#javascript-interop-issues)
5. [Performance Problems](#performance-problems)
6. [Development Environment Issues](#development-environment-issues)
7. [Common Error Messages](#common-error-messages)
8. [Debugging Techniques](#debugging-techniques)
9. [Frequently Asked Questions](#frequently-asked-questions)

## Installation Issues

### Problem: Rust compilation fails

**Symptoms:**

- `cargo build` fails with compilation errors
- Missing dependencies errors
- Version incompatibility issues

**Solutions:**

1. **Update Rust toolchain:**

   ```bash
   rustup update stable
   rustup default stable
   ```

2. **Check Rust version:**

   ```bash
   rustc --version  # Should be 1.70 or later
   ```

3. **Clean and rebuild:**

   ```bash
   cargo clean
   cargo build --release
   ```

4. **Install required components:**

   ```bash
   rustup component add clippy rustfmt
   ```

### Problem: Node.js runtime dependencies fail to install

**Symptoms:**

- `npm install` fails in nagari-runtime directory
- TypeScript compilation errors
- Missing type definitions

**Solutions:**

1. **Check Node.js version:**

   ```bash
   node --version  # Should be 16 or later
   npm --version
   ```

2. **Clear npm cache:**

   ```bash
   npm cache clean --force
   rm -rf node_modules package-lock.json
   npm install
   ```

3. **Install TypeScript globally:**

   ```bash
   npm install -g typescript @types/node
   ```

4. **Update npm:**

   ```bash
   npm install -g npm@latest
   ```

### Problem: PATH configuration issues

**Symptoms:**

- `nagc` command not found
- Compiled binaries not accessible

**Solutions:**

1. **Add to PATH (Unix/Linux/macOS):**

   ```bash
   export PATH=$PATH:/path/to/nagari/nagari-compiler/target/release
   echo 'export PATH=$PATH:/path/to/nagari/nagari-compiler/target/release' >> ~/.bashrc
   ```

2. **Add to PATH (Windows):**

   ```cmd
   set PATH=%PATH%;C:\path\to\nagari\nagari-compiler\target\release
   setx PATH "%PATH%;C:\path\to\nagari\nagari-compiler\target\release"
   ```

3. **Verify installation:**

   ```bash
   nagc --version
   ```

## Compilation Errors

### Problem: Syntax errors in Nagari code

**Error:** `SyntaxError: Unexpected token at line X`

**Common causes and solutions:**

1. **Incorrect indentation:**

   ```nagari
   # Wrong
   def function():
   return "hello"  # Should be indented with 4 spaces

   # Correct
   def function():
       return "hello"
   ```

2. **Missing colons:**

   ```nagari
   # Wrong
   if condition
       do_something()

   # Correct
   if condition:
       do_something()
   ```

3. **Mixed tabs and spaces:**

   ```bash
   # Check for mixed indentation
   cat -A your_file.nag | grep -E "^[\t ]+"
   ```

### Problem: Type checking errors

**Error:** `TypeError: Expected type 'str', got 'int'`

**Solutions:**

1. **Add explicit type conversions:**

   ```nagari
   # Wrong
   age: str = 25

   # Correct
   age: str = str(25)
   # Or
   age: int = 25
   ```

2. **Use union types for flexibility:**

   ```nagari
   from typing import Union

   def process_value(value: Union[str, int]) -> str:
       return str(value)
   ```

3. **Handle optional types:**

   ```nagari
   from typing import Optional

   def get_user(id: int) -> Optional[dict]:
       # May return None
       return user_database.get(id)
   ```

### Problem: Import resolution errors

**Error:** `ModuleNotFoundError: No module named 'module_name'`

**Solutions:**

1. **Check module path:**

   ```nagari
   # Relative imports
   from .local_module import function
   from ..parent_module import class_name

   # Absolute imports
   from nagari.stdlib.http import request
   ```

2. **Verify file exists:**

   ```bash
   ls -la stdlib/
   find . -name "*.nag" | grep module_name
   ```

3. **Check NAGARI_PATH environment variable:**

   ```bash
   export NAGARI_PATH="/path/to/your/modules:$NAGARI_PATH"
   ```

## Runtime Errors

### Problem: JavaScript interop failures

**Error:** `InteropError: Cannot convert Nagari value to JavaScript`

**Solutions:**

1. **Use proper interop functions:**

   ```nagari
   # Wrong
   js_function(nagari_value)

   # Correct
   js_function(nagari.to_js(nagari_value))
   ```

2. **Handle type conversions:**

   ```nagari
   # For complex objects
   js_object = {
       "name": nagari.to_js(user.name),
       "age": nagari.to_js(user.age),
       "items": nagari.to_js(user.items)
   }
   ```

3. **Check supported types:**

   ```nagari
   # Supported for interop: str, int, float, bool, list, dict
   # Not supported: custom classes, functions (need wrapping)
   ```

### Problem: Async/await issues

**Error:** `RuntimeError: Cannot await non-awaitable object`

**Solutions:**

1. **Ensure function is async:**

   ```nagari
   # Wrong
   def fetch_data():
       return await request.get(url)

   # Correct
   async def fetch_data():
       return await request.get(url)
   ```

2. **Use asyncio.run for entry point:**

   ```nagari
   import asyncio

   async def main():
       result = await fetch_data()
       print(result)

   if __name__ == "__main__":
       asyncio.run(main())
   ```

3. **Convert callbacks to promises:**

   ```nagari
   from nagari.runtime import promisify

   async_function = promisify(callback_function)
   result = await async_function(args)
   ```

### Problem: Memory leaks in long-running applications

**Symptoms:**

- Gradually increasing memory usage
- Application slowdown over time
- Out of memory errors

**Solutions:**

1. **Properly close resources:**

   ```nagari
   # Use context managers
   with open("file.txt") as f:
       content = f.read()

   # Manual cleanup
   try:
       resource = acquire_resource()
       use_resource(resource)
   finally:
       resource.close()
   ```

2. **Avoid circular references:**

   ```nagari
   # Wrong - creates circular reference
   class Parent:
       def __init__(self):
           self.children = []

   class Child:
       def __init__(self, parent):
           self.parent = parent
           parent.children.append(self)

   # Better - use weak references
   import weakref

   class Child:
       def __init__(self, parent):
           self.parent = weakref.ref(parent)
   ```

3. **Monitor memory usage:**

   ```nagari
   import psutil
   import os

   def check_memory():
       process = psutil.Process(os.getpid())
       memory_mb = process.memory_info().rss / 1024 / 1024
       print(f"Memory usage: {memory_mb:.1f} MB")
   ```

## JavaScript Interop Issues

### Problem: Cannot access JavaScript object properties

**Error:** `AttributeError: 'JSObject' has no attribute 'property_name'`

**Solutions:**

1. **Use bracket notation:**

   ```nagari
   # Wrong
   value = js_object.property_name

   # Correct
   value = js_object["property_name"]
   ```

2. **Check property exists:**

   ```nagari
   if "property_name" in js_object:
       value = js_object["property_name"]
   else:
       value = default_value
   ```

3. **Use getattr with default:**

   ```nagari
   value = getattr(js_object, "property_name", default_value)
   ```

### Problem: JavaScript function calls fail

**Error:** `TypeError: Cannot call JavaScript function with these arguments`

**Solutions:**

1. **Convert arguments properly:**

   ```nagari
   # Wrong
   result = js_function(nagari_list, nagari_dict)

   # Correct
   result = js_function(
       nagari.to_js(nagari_list),
       nagari.to_js(nagari_dict)
   )
   ```

2. **Handle async JavaScript functions:**

   ```nagari
   # For promises
   result = await js_async_function(args)

   # For callbacks
   def callback(error, result):
       if error:
           print(f"Error: {error}")
       else:
           print(f"Result: {result}")

   js_callback_function(args, callback)
   ```

### Problem: React/JSX compilation issues

**Error:** `JSXError: Invalid JSX syntax`

**Solutions:**

1. **Use proper JSX syntax:**

   ```nagari
   # Wrong
   element = <div class="container">Content</div>

   # Correct (use className)
   element = <div className="container">Content</div>
   ```

2. **Handle JSX expressions:**

   ```nagari
   # Wrong
   element = <div>{some_variable}</div>

   # Correct
   element = <div>{nagari.to_js(some_variable)}</div>
   ```

3. **Import React properly:**

   ```nagari
   import React from "react"

   def MyComponent(props):
       return <div>Hello {props.name}</div>
   ```

## Performance Problems

### Problem: Slow compilation times

**Solutions:**

1. **Use incremental compilation:**

   ```bash
   nagc --incremental src/ --output dist/
   ```

2. **Parallel compilation:**

   ```bash
   nagc --jobs 4 src/ --output dist/
   ```

3. **Exclude unnecessary files:**

   ```nagari
   # .nagignore file
   *.tmp
   test_*
   benchmark_*
   ```

### Problem: Runtime performance issues

**Solutions:**

1. **Profile your code:**

   ```nagari
   import time
   import cProfile

   def profile_function():
       start = time.time()
       your_function()
       end = time.time()
       print(f"Execution time: {end - start:.3f}s")

   # Detailed profiling
   cProfile.run("your_function()")
   ```

2. **Optimize hot paths:**

   ```nagari
   # Use list comprehensions instead of loops
   # Wrong
   result = []
   for item in items:
       if condition(item):
           result.append(transform(item))

   # Better
   result = [transform(item) for item in items if condition(item)]
   ```

3. **Cache expensive operations:**

   ```nagari
   from functools import lru_cache

   @lru_cache(maxsize=128)
   def expensive_function(arg):
       # Expensive computation
       return result
   ```

## Development Environment Issues

### Problem: VS Code extension not working

**Solutions:**

1. **Install Nagari language extension:**

   ```bash
   # Check if extension is installed
   code --list-extensions | grep nagari

   # Install if missing
   code --install-extension nagari-lang.nagari
   ```

2. **Configure file associations:**

   ```json
   // settings.json
   {
       "files.associations": {
           "*.nag": "nagari"
       }
   }
   ```

3. **Set up language server:**

   ```json
   // settings.json
   {
       "nagari.languageServer.path": "/path/to/nagari-language-server"
   }
   ```

### Problem: Debugging not working

**Solutions:**

1. **Configure debug settings:**

   ```json
   // launch.json
   {
       "version": "0.2.0",
       "configurations": [
           {
               "name": "Debug Nagari",
               "type": "node",
               "request": "launch",
               "program": "${workspaceFolder}/dist/main.js",
               "preLaunchTask": "nagari-compile"
           }
       ]
   }
   ```

2. **Enable source maps:**

   ```bash
   nagc --sourcemap src/main.nag --output dist/main.js
   ```

3. **Use console debugging:**

   ```nagari
   def debug_function():
       print(f"Debug: variable value = {variable}")
       console.log("Debug from JS side")
   ```

## Common Error Messages

### `SyntaxError: Invalid syntax`

**Cause:** Usually indentation or syntax issues.

**Solution:** Check indentation (4 spaces), colons after control structures, and proper quote matching.

### `NameError: name 'variable' is not defined`

**Cause:** Variable used before definition or scope issues.

**Solution:** Ensure variables are defined before use and check scope rules.

### `ImportError: cannot import name 'X' from 'Y'`

**Cause:** Import path incorrect or module not found.

**Solution:** Verify module path and check if the name is exported.

### `TypeError: 'NoneType' object is not callable`

**Cause:** Trying to call a function that returned None.

**Solution:** Check if function returns the expected value and handle None cases.

### `InteropError: JavaScript execution failed`

**Cause:** JavaScript code failed to execute or incompatible types passed.

**Solution:** Check JavaScript syntax and ensure proper type conversion.

## Debugging Techniques

### 1. Print Debugging

```nagari
def debug_function(data):
    print(f"Input data: {data}")
    print(f"Data type: {type(data)}")

    result = process_data(data)
    print(f"Result: {result}")

    return result
```

### 2. Using the Debugger

```nagari
import pdb

def function_to_debug():
    pdb.set_trace()  # Breakpoint
    # Code continues here
    return result
```

### 3. Logging

```nagari
import logging

logging.basicConfig(level=logging.DEBUG)
logger = logging.getLogger(__name__)

def logged_function():
    logger.debug("Entering function")
    logger.info("Processing data")
    logger.warning("Potential issue detected")
    logger.error("Error occurred")
```

### 4. Exception Handling

```nagari
def robust_function():
    try:
        risky_operation()
    except SpecificError as e:
        logger.error(f"Specific error: {e}")
        handle_specific_error(e)
    except Exception as e:
        logger.error(f"Unexpected error: {e}")
        handle_generic_error(e)
    finally:
        cleanup_resources()
```

### 5. Unit Testing for Debugging

```nagari
import unittest

class TestMyFunction(unittest.TestCase):
    def test_normal_case(self):
        result = my_function("normal input")
        self.assertEqual(result, "expected output")

    def test_edge_case(self):
        result = my_function("")
        self.assertIsNone(result)

    def test_error_case(self):
        with self.assertRaises(ValueError):
            my_function(None)

if __name__ == "__main__":
    unittest.main()
```

## Frequently Asked Questions

### Q: Why does my Nagari code run slowly compared to Python?

**A:** Nagari transpiles to JavaScript, which may have different performance characteristics than Python. Consider:

- Using JavaScript-optimized algorithms
- Leveraging JavaScript's async capabilities
- Profiling to identify bottlenecks

### Q: Can I use all JavaScript libraries with Nagari?

**A:** Most JavaScript libraries work with Nagari through the interop system. However:

- Some libraries may require wrapper functions
- Type-heavy libraries may need additional type declarations
- Check the interop guide for specific library integration patterns

### Q: How do I handle JavaScript's prototype inheritance in Nagari?

**A:** Use Nagari's class system, which transpiles to JavaScript classes:

```nagari
class NagariClass:
    def method(self):
        return "Nagari method"

# Transpiles to JavaScript class with proper prototype chain
```

### Q: What's the difference between Nagari lists and JavaScript arrays?

**A:** Nagari lists transpile to JavaScript arrays but with additional methods:

- Nagari: `list.append(item)`
- JavaScript: `array.push(item)`
- The interop layer handles conversions automatically

### Q: How do I deploy Nagari applications?

**A:** Compile to JavaScript and deploy like any JS application:

```bash
# For Node.js applications
nagc src/ --output dist/
node dist/main.js

# For web applications
nagc src/ --output dist/ --target browser
# Deploy dist/ folder to web server
```

### Q: Can I use TypeScript declaration files with Nagari?

**A:** Yes, Nagari can generate and consume TypeScript declarations:

```bash
nagc --declarations src/ --output dist/
```

This creates `.d.ts` files for better IDE support and type checking.

For additional help, check the [community forums](https://github.com/nagari-lang/nagari/discussions) or [open an issue](https://github.com/nagari-lang/nagari/issues).
