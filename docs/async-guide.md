# Nagari Async Programming Guide

A comprehensive guide to asynchronous programming in Nagari.

## Table of Contents

1. [Introduction to Async Programming](#introduction-to-async-programming)
2. [Async/Await Basics](#asyncawait-basics)
3. [Promises and Future Objects](#promises-and-future-objects)
4. [Concurrency Patterns](#concurrency-patterns)
5. [Error Handling in Async Code](#error-handling-in-async-code)
6. [Working with JavaScript Async APIs](#working-with-javascript-async-apis)
7. [Advanced Patterns](#advanced-patterns)
8. [Performance Considerations](#performance-considerations)
9. [Debugging Async Code](#debugging-async-code)
10. [Best Practices](#best-practices)

## Introduction to Async Programming

### What is Asynchronous Programming?

Asynchronous programming is a programming paradigm that allows operations to execute independently of the main program flow. Instead of waiting for an operation to complete before moving on, the program can continue executing other tasks and handle the results of the asynchronous operation when it completes.

This approach is particularly valuable for:

- **I/O-bound operations**: Reading files, network requests, database queries
- **Long-running computations**: Complex calculations that would block the main thread
- **Event-driven systems**: UIs, web servers, and other systems that respond to events

### Why Async in Nagari?

Nagari's asynchronous programming model is built on top of JavaScript's event loop and Promise system, providing a more intuitive Python-like syntax for asynchronous operations. Benefits include:

- **Improved readability**: Async code looks similar to synchronous code
- **Better performance**: Non-blocking I/O operations improve throughput
- **Enhanced responsiveness**: UI remains responsive during long operations
- **Full compatibility**: Works with JavaScript async ecosystem
- **Reduced complexity**: Simplified error handling and control flow

### Sync vs. Async Example

Here's a simple comparison of synchronous vs. asynchronous approaches:

**Synchronous (Blocking)**:

```nagari
def get_user_data():
    # This blocks the entire program until complete
    user_response = make_http_request("https://api.example.com/users/1")
    posts_response = make_http_request("https://api.example.com/users/1/posts")

    return {
        "user": user_response.json(),
        "posts": posts_response.json()
    }

# Main program waits for get_user_data() to finish
data = get_user_data()
print("Data fetched!")
```

**Asynchronous (Non-blocking)**:

```nagari
async def get_user_data():
    # These operations can run concurrently
    user_future = fetch("https://api.example.com/users/1")
    posts_future = fetch("https://api.example.com/users/1/posts")

    # Wait for both to complete
    user_response = await user_future
    posts_response = await posts_future

    return {
        "user": await user_response.json(),
        "posts": await posts_response.json()
    }

# Main program can do other work while get_user_data() is running
async def main():
    data_future = get_user_data()
    print("Fetching data...")
    # Do other work here
    data = await data_future
    print("Data fetched!")

asyncio.run(main())
```

## Async/Await Basics

### Defining Async Functions

In Nagari, you define an asynchronous function using the `async def` syntax:

```nagari
async def fetch_data():
    # Asynchronous operations here
    return result
```

Key points about async functions:

- They always return a Promise (Future) object, even if you return a simple value
- They can contain `await` expressions to pause execution
- They run concurrently with other code until an `await` is encountered
- They resume execution when the awaited operation completes

### Using Await

The `await` keyword pauses the execution of an async function until the Promise is resolved:

```nagari
async def get_user(user_id):
    response = await fetch(f"https://api.example.com/users/{user_id}")
    user_data = await response.json()
    return user_data
```

Important characteristics of `await`:

- Can only be used inside an async function
- Extracts the resolved value from a Promise
- Converts Promise rejections into exceptions
- Makes asynchronous code appear synchronous
- Each `await` pauses execution, allowing other tasks to run

### Async Function Execution Flow

Understanding the execution flow of async functions is crucial:

```nagari
async def main():
    print("Start")
    await asyncio.sleep(1)  # Pause for 1 second
    print("Middle")
    await asyncio.sleep(1)  # Pause for 1 second
    print("End")

# Output:
# Start
# (1 second later)
# Middle
# (1 second later)
# End
```

The function execution is paused at each `await`, allowing other code to run during these pauses.

### Entry Points

Every async program needs an entry point. In Nagari, you typically use:

```nagari
import asyncio

async def main():
    # Your async code here
    pass

if __name__ == "__main__":
    asyncio.run(main())
```

The `asyncio.run()` function:

- Creates an event loop
- Runs the provided coroutine
- Manages the event loop lifecycle
- Handles cleanup when the main coroutine completes

## Promises and Future Objects

### Understanding Promises

Promises (called Futures in some languages) represent the eventual completion or failure of an asynchronous operation. In Nagari, they're the foundation of async programming.

Every async function returns a Promise automatically:

```nagari
async def get_value():
    return 42

# get_value() returns a Promise that resolves to 42
result_promise = get_value()
```

Promises exist in three states:

1. **Pending**: Initial state, operation not completed
2. **Fulfilled**: Operation completed successfully
3. **Rejected**: Operation failed

### Creating Promises

While async functions return Promises automatically, you can also create them manually:

```nagari
from js import Promise

def create_promise():
    return Promise.new((resolve, reject) => {
        # Simulate async operation
        setTimeout(() => {
            resolve("Success!")
            # To reject: reject(Error("Failed!"))
        }, 1000)
    })

# Use like any other Promise
async def use_manual_promise():
    result = await create_promise()
    print(result)  # "Success!"
```

### Promise Methods

Nagari Promises support JavaScript Promise methods:

```nagari
async def demonstrate_promise_methods():
    promise = fetch("https://api.example.com/data")

    # .then() chains
    promise.then(
        lambda response: response.json(),
        lambda error: console.error(f"Error: {error}")
    )

    # .catch() for error handling
    promise.then(lambda response: response.json()).catch(
        lambda error: console.error(f"Error: {error}")
    )

    # .finally() for cleanup
    promise.then(lambda response: response.json()).finally(
        lambda: console.log("Operation completed")
    )
```

However, in Nagari, using `async`/`await` is generally preferred over Promise chains:

```nagari
async def preferred_approach():
    try:
        response = await fetch("https://api.example.com/data")
        data = await response.json()
        return data
    except Exception as e:
        console.error(f"Error: {e}")
    finally:
        console.log("Operation completed")
```

### Promise Composition

Promises can be composed for more complex async workflows:

#### Sequential Execution

```nagari
async def sequential_tasks():
    # Tasks run one after another
    result1 = await task1()
    result2 = await task2(result1)
    result3 = await task3(result2)
    return result3
```

#### Parallel Execution

```nagari
async def parallel_tasks():
    # Start all tasks at once
    promise1 = task1()
    promise2 = task2()
    promise3 = task3()

    # Wait for all to complete
    result1, result2, result3 = await Promise.all([promise1, promise2, promise3])
    return [result1, result2, result3]
```

#### Race Pattern

```nagari
async def race_tasks():
    # Return result of the fastest task
    fastest_result = await Promise.race([
        task1(),  # Could be a primary source
        task2(),  # Could be a fallback source
        task3()   # Could be another alternative
    ])
    return fastest_result
```

## Concurrency Patterns

### Parallel Execution

Executing multiple async operations simultaneously:

```nagari
async def fetch_all_users(user_ids):
    # Create a list of fetch promises
    promises = [fetch_user(user_id) for user_id in user_ids]

    # Wait for all promises to resolve
    users = await Promise.all(promises)
    return users

async def fetch_user(user_id):
    response = await fetch(f"https://api.example.com/users/{user_id}")
    return await response.json()
```

### Sequential Execution

When operations must happen in order:

```nagari
async def process_in_sequence(items):
    results = []
    for item in items:
        # Each operation waits for the previous one to complete
        result = await process_item(item)
        results.append(result)
    return results
```

### Concurrent with Limited Parallelism

Limit the number of concurrent operations:

```nagari
async def process_with_concurrency_limit(items, limit=5):
    semaphore = Semaphore(limit)

    async def process_with_limit(item):
        async with semaphore:
            return await process_item(item)

    # Create all tasks
    tasks = [process_with_limit(item) for item in items]

    # Execute with limited concurrency
    return await Promise.all(tasks)

class Semaphore:
    def __init__(self, limit):
        self.limit = limit
        self.count = 0
        self.queue = []

    async def __aenter__(self):
        if self.count >= self.limit:
            # Create a Promise that will resolve when a slot is available
            promise = Promise.new()
            self.queue.append(promise)
            await promise

        self.count += 1
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        self.count -= 1
        if self.queue:
            # Resolve the next waiting Promise
            next_promise = self.queue.pop(0)
            next_promise.resolve()
```

### Async Generators

For processing streams of asynchronous data:

```nagari
async def fetch_paginated_results(url, page_size=10):
    page = 1
    has_more = True

    while has_more:
        response = await fetch(f"{url}?page={page}&size={page_size}")
        data = await response.json()

        # Yield each page of results
        yield data.items

        has_more = data.has_next_page
        page += 1

async def process_all_results():
    results = []
    async for page in fetch_paginated_results("https://api.example.com/data"):
        for item in page:
            results.append(process_item(item))
    return results
```

### Cancellation

Handling cancellation of async operations:

```nagari
async def cancellable_operation():
    try:
        # Check if operation has been cancelled
        if asyncio.current_task().cancelled():
            raise asyncio.CancelledError()

        # Long-running operation
        await asyncio.sleep(10)
        return "Operation completed"
    except asyncio.CancelledError:
        # Clean up resources
        print("Operation was cancelled")
        raise  # Re-raise to propagate cancellation
```

Usage:

```nagari
async def main():
    # Start task
    task = asyncio.create_task(cancellable_operation())

    # Wait for a bit, then cancel
    await asyncio.sleep(2)
    task.cancel()

    try:
        await task
    except asyncio.CancelledError:
        print("Main: Task was cancelled")
```

## Error Handling in Async Code

### Try/Except with Await

The most common pattern for handling errors in async code:

```nagari
async def fetch_with_error_handling():
    try:
        response = await fetch("https://api.example.com/data")
        data = await response.json()
        return data
    except FetchError as e:
        console.error(f"Network error: {e}")
        return None
    except JSONError as e:
        console.error(f"Invalid JSON: {e}")
        return None
    except Exception as e:
        console.error(f"Unexpected error: {e}")
        return None
```

### Promise Rejection Handling

When working directly with Promises:

```nagari
def handle_with_promises():
    fetch("https://api.example.com/data").then(
        lambda response: response.json()
    ).then(
        lambda data: process_data(data)
    ).catch(
        lambda error: console.error(f"Error: {error}")
    )
```

### Async Function Error Propagation

Errors automatically propagate through the async call chain:

```nagari
async def level3():
    # This error propagates up through all calling functions
    raise ValueError("Something went wrong at level 3")

async def level2():
    await level3()  # Error will propagate to level1

async def level1():
    try:
        await level2()  # Catches error from level3
    except ValueError as e:
        console.log(f"Caught error: {e}")
```

### Combining Multiple Operations with Error Handling

```nagari
async def fetch_multiple_with_errors(urls):
    results = []
    errors = []

    for url in urls:
        try:
            response = await fetch(url)
            data = await response.json()
            results.append(data)
        except Exception as e:
            errors.append({"url": url, "error": str(e)})

    return {
        "results": results,
        "errors": errors,
        "success_count": len(results),
        "error_count": len(errors)
    }
```

### Timeouts

Adding timeouts to prevent operations from hanging:

```nagari
async def fetch_with_timeout(url, timeout_seconds=5):
    try:
        # Create a promise that rejects after timeout
        timeout_promise = Promise.new((resolve, reject) => {
            setTimeout(() => {
                reject(TimeoutError(f"Request timed out after {timeout_seconds} seconds"))
            }, timeout_seconds * 1000)
        })

        # Use Promise.race to compete between fetch and timeout
        result = await Promise.race([
            fetch(url),
            timeout_promise
        ])

        return result
    except TimeoutError as e:
        console.error(str(e))
        return None
```

### Finally Block for Cleanup

Ensuring resources are properly released:

```nagari
async def process_with_cleanup():
    resource = None
    try:
        resource = await acquire_resource()
        return await process_with_resource(resource)
    except Exception as e:
        console.error(f"Error processing resource: {e}")
        raise  # Re-throw the exception after logging
    finally:
        # This runs regardless of success or failure
        if resource:
            await release_resource(resource)
```

## Working with JavaScript Async APIs

### Fetch API

Working with HTTP requests:

```nagari
async def get_data(url):
    response = await fetch(url)

    if not response.ok:
        raise HTTPError(f"HTTP error! Status: {response.status}")

    return await response.json()

async def post_data(url, data):
    response = await fetch(url, {
        "method": "POST",
        "headers": {
            "Content-Type": "application/json"
        },
        "body": JSON.stringify(data)
    })

    return await response.json()
```

### DOM Events as Promises

Converting callback-based DOM events to Promises:

```nagari
async def wait_for_click(element):
    # Create a Promise that resolves when the element is clicked
    click_promise = Promise.new((resolve, reject) => {
        element.addEventListener("click", (event) => {
            resolve(event)
        }, { once: true })  # Automatically remove listener after first click
    })

    # Wait for the click
    return await click_promise

async def form_submit_handler():
    submit_button = document.querySelector("#submit-button")

    print("Waiting for button click...")
    click_event = await wait_for_click(submit_button)

    print("Button clicked! Processing form...")
    # Process form data
```

### setTimeout and setInterval

Working with timing functions:

```nagari
async def delay(milliseconds):
    # Create a Promise that resolves after the specified delay
    return await Promise.new((resolve, reject) => {
        setTimeout(resolve, milliseconds)
    })

async def countdown(seconds):
    for i in range(seconds, 0, -1):
        print(f"{i}...")
        await delay(1000)
    print("Done!")

async def poll_server(url, interval_ms=5000, max_attempts=12):
    for i in range(max_attempts):
        try:
            response = await fetch(url)
            data = await response.json()

            if data.status == "ready":
                return data
        except Exception as e:
            console.error(f"Polling error: {e}")

        await delay(interval_ms)

    raise TimeoutError(f"Server not ready after {max_attempts} attempts")
```

### Working with Node.js Callbacks

Converting Node.js-style callbacks to Promises:

```nagari
from js import util, fs

async def read_file(path):
    # Promisify the Node.js fs.readFile function
    read_file_promise = util.promisify(fs.readFile)

    # Now we can await it
    buffer = await read_file_promise(path)
    return buffer.toString()

async def write_file(path, data):
    # Promisify the Node.js fs.writeFile function
    write_file_promise = util.promisify(fs.writeFile)

    # Now we can await it
    await write_file_promise(path, data)
```

### WebSockets and Event Streams

Handling streaming data:

```nagari
async def connect_websocket(url):
    # Create a WebSocket connection
    socket = WebSocket.new(url)

    # Wait for the connection to open
    await Promise.new((resolve, reject) => {
        socket.onopen = resolve
        socket.onerror = reject
    })

    print("WebSocket connected!")
    return socket

async def listen_for_messages(socket):
    while True:
        # Create a Promise that resolves when a message is received
        message = await Promise.new((resolve, reject) => {
            socket.onmessage = (event) => {
                resolve(event.data)
            }
            socket.onerror = reject
            socket.onclose = () => {
                resolve(None)  # Signal end of stream
            }
        })

        if message is None:
            print("WebSocket closed")
            break

        yield message  # Yield each message as it arrives

async def websocket_example():
    socket = await connect_websocket("wss://example.com/socket")

    try:
        async for message in listen_for_messages(socket):
            data = JSON.parse(message)
            print(f"Received: {data}")
    finally:
        # Ensure socket is closed
        socket.close()
```

## Advanced Patterns

### Debouncing and Throttling

Controlling the frequency of async operations:

```nagari
class Debouncer:
    def __init__(self, delay_ms=300):
        self.delay_ms = delay_ms
        self.timeout_id = None

    async def debounce(self, func, *args, **kwargs):
        # Cancel pending execution
        if self.timeout_id:
            clearTimeout(self.timeout_id)

        # Create a new Promise that will resolve after the delay
        result = await Promise.new((resolve, reject) => {
            self.timeout_id = setTimeout(() => {
                try:
                    result = func(*args, **kwargs)
                    resolve(result)
                } catch (error) {
                    reject(error)
                }
            }, self.delay_ms)
        })

        return result

# Usage
debouncer = Debouncer(500)  # 500ms delay

async def handle_search_input(query):
    # This will only execute after 500ms of inactivity
    results = await debouncer.debounce(search_api, query)
    display_results(results)
```

### Async Mutex/Lock

Ensuring exclusive access to a resource:

```nagari
class AsyncMutex:
    def __init__(self):
        self.locked = False
        self.waiting_queue = []

    async def acquire(self):
        if not self.locked:
            self.locked = True
            return

        # Create a Promise that will resolve when the mutex is released
        await Promise.new((resolve, reject) => {
            self.waiting_queue.append(resolve)
        })

        self.locked = True

    def release(self):
        if not self.waiting_queue:
            self.locked = False
            return

        # Resolve the next waiting promise
        next_resolve = self.waiting_queue.pop(0)
        next_resolve()

    async def __aenter__(self):
        await self.acquire()
        return self

    async def __aexit__(self, exc_type, exc_val, exc_tb):
        self.release()

# Usage
mutex = AsyncMutex()

async def access_shared_resource():
    async with mutex:
        # Exclusive access to the resource
        await perform_critical_operation()
```

### Retry Logic

Automatically retry failed operations:

```nagari
async def retry_operation(operation, max_retries=3, delay_ms=1000, backoff_factor=2):
    last_error = None
    current_delay = delay_ms

    for attempt in range(max_retries):
        try:
            return await operation()
        except Exception as error:
            last_error = error
            print(f"Attempt {attempt + 1} failed: {error}")

            if attempt < max_retries - 1:
                print(f"Retrying in {current_delay}ms...")
                await Promise.new((resolve, reject) => {
                    setTimeout(resolve, current_delay)
                })
                current_delay *= backoff_factor

    raise last_error

# Usage
async def fetch_with_retry(url):
    return await retry_operation(
        lambda: fetch(url).then(lambda res: res.json()),
        max_retries=5,
        delay_ms=500,
        backoff_factor=1.5
    )
```

### Async Cache

Caching results of expensive async operations:

```nagari
class AsyncCache:
    def __init__(self, ttl_ms=60000):  # Default: 1 minute TTL
        self.cache = {}
        self.ttl_ms = ttl_ms

    async def get_or_compute(self, key, compute_func):
        current_time = Date.now()

        # Check if we have a fresh cached value
        if key in self.cache:
            entry = self.cache[key]
            if current_time < entry["expires"]:
                return entry["value"]

        # Compute new value
        value = await compute_func()

        # Cache it
        self.cache[key] = {
            "value": value,
            "expires": current_time + self.ttl_ms
        }

        return value

    def invalidate(self, key=None):
        if key is None:
            self.cache = {}  # Clear entire cache
        else:
            if key in self.cache:
                del self.cache[key]  # Remove specific key

# Usage
data_cache = AsyncCache(ttl_ms=300000)  # 5 minute cache

async def get_user_data(user_id):
    return await data_cache.get_or_compute(
        f"user:{user_id}",
        lambda: fetch(f"https://api.example.com/users/{user_id}").then(lambda res: res.json())
    )
```

### Event Emitter with Async Handlers

Creating an event system with async handlers:

```nagari
class AsyncEventEmitter:
    def __init__(self):
        self.listeners = {}

    def on(self, event, listener):
        if event not in self.listeners:
            self.listeners[event] = []

        self.listeners[event].append(listener)

        # Return a function to remove this listener
        return lambda: self.off(event, listener)

    def off(self, event, listener):
        if event in self.listeners:
            self.listeners[event] = [l for l in self.listeners[event] if l != listener]

    def once(self, event, listener):
        # Create a wrapper that will remove itself after one call
        async def wrapper(*args, **kwargs):
            self.off(event, wrapper)
            return await listener(*args, **kwargs)

        return self.on(event, wrapper)

    async def emit(self, event, *args, **kwargs):
        if event not in self.listeners:
            return

        # Execute all listeners for this event
        results = []
        for listener in self.listeners[event]:
            if asyncio.iscoroutinefunction(listener):
                results.append(await listener(*args, **kwargs))
            else:
                results.append(listener(*args, **kwargs))

        return results

# Usage
events = AsyncEventEmitter()

# Register a regular handler
events.on("data", lambda data: print(f"Received: {data}"))

# Register an async handler
async def handle_data_async(data):
    result = await process_data(data)
    print(f"Processed: {result}")

events.on("data", handle_data_async)

# Emit an event
async def main():
    await events.emit("data", {"id": 123, "name": "Example"})

asyncio.run(main())
```

## Performance Considerations

### Avoiding Bottlenecks

Tips for maximizing async performance:

1. **Concurrent vs. Parallel Execution**:

   ```nagari
   # Inefficient: Sequential awaits
   result1 = await operation1()
   result2 = await operation2()

   # Efficient: Start both operations, then await results
   future1 = operation1()  # Starts immediately
   future2 = operation2()  # Starts immediately
   result1 = await future1
   result2 = await future2
   ```

2. **Batching**:

   ```nagari
   # Inefficient: Individual requests
   for id in ids:
       result = await fetch_item(id)
       process_result(result)

   # Efficient: Batch requests
   results = await fetch_items_batch(ids)
   for result in results:
       process_result(result)
   ```

3. **Right-sizing Concurrency**:

   ```nagari
   # Too many concurrent requests can overwhelm resources
   async def process_with_optimal_concurrency(items, concurrency=10):
       # Process in batches of 'concurrency' items
       for i in range(0, len(items), concurrency):
           batch = items[i:i+concurrency]
           batch_futures = [process_item(item) for item in batch]
           batch_results = await Promise.all(batch_futures)
           # Do something with batch_results
   ```

### Memory Management

Managing memory in long-running async applications:

```nagari
async def memory_efficient_processing(large_dataset):
    # Process in chunks to avoid loading everything into memory
    chunk_size = 1000
    for i in range(0, len(large_dataset), chunk_size):
        chunk = large_dataset[i:i+chunk_size]
        await process_chunk(chunk)

        # Allow garbage collection to reclaim memory
        # between chunks by ensuring we don't keep references
        chunk = None
```

### CPU-Bound Tasks

Handling CPU-intensive operations without blocking:

```nagari
async def cpu_intensive_task(data):
    # For CPU-bound tasks in a browser, consider using Web Workers
    if is_browser_environment():
        return await Promise.new((resolve, reject) => {
            const worker = new Worker('worker.js')
            worker.onmessage = (event) => {
                resolve(event.data)
            }
            worker.onerror = (error) => {
                reject(error)
            }
            worker.postMessage(data)
        })
    else:
        # In Node.js, consider using worker threads or child processes
        # For demo purposes, we'll just break up the work
        result = 0
        chunk_size = 1000

        for i in range(0, len(data), chunk_size):
            chunk = data[i:i+chunk_size]

            # Process chunk and allow other tasks to run
            partial_result = compute_partial_result(chunk)
            result += partial_result

            # Yield to the event loop occasionally
            await asyncio.sleep(0)

        return result
```

### Caching and Memoization

Prevent redundant async operations:

```nagari
# Simple memoization for async functions
def memoize_async(ttl_ms=None):
    cache = {}

    def decorator(func):
        async def wrapper(*args, **kwargs):
            # Create a cache key from the function arguments
            key = str([func.__name__, args, kwargs])
            current_time = Date.now()

            # Check cache
            if key in cache:
                entry = cache[key]
                # If no TTL or entry is still fresh
                if ttl_ms is None or current_time < entry["expires"]:
                    return entry["value"]

            # Call the original async function
            result = await func(*args, **kwargs)

            # Cache the result
            cache[key] = {
                "value": result,
                "expires": current_time + (ttl_ms if ttl_ms is not None else float('inf'))
            }

            return result

        # Add method to clear the cache
        wrapper.clear_cache = lambda: cache.clear()

        return wrapper

    return decorator

# Usage
@memoize_async(ttl_ms=60000)  # Cache results for 1 minute
async def fetch_user_profile(user_id):
    response = await fetch(f"https://api.example.com/users/{user_id}")
    return await response.json()
```

## Debugging Async Code

### Common Async Debugging Challenges

1. **Non-deterministic execution order**
2. **Stack traces spanning async boundaries**
3. **Unhandled Promise rejections**
4. **Timing-dependent bugs**
5. **Deadlocks and race conditions**

### Structured Logging

Adding useful context to async operations:

```nagari
async def fetch_with_logging(url, request_id=None):
    if request_id is None:
        request_id = generate_request_id()

    try:
        console.log(f"[{request_id}] Starting request to {url}")
        start_time = Date.now()

        response = await fetch(url)

        elapsed = Date.now() - start_time
        console.log(f"[{request_id}] Received response from {url} in {elapsed}ms")

        data = await response.json()
        console.log(f"[{request_id}] Parsed JSON response: {JSON.stringify(data).substring(0, 100)}...")

        return data
    except Exception as e:
        console.error(f"[{request_id}] Error fetching {url}: {e}")
        raise  # Re-throw for proper error handling
```

### Async Stack Traces

Improving stack traces across async boundaries:

```nagari
# Enable detailed async stack traces in Node.js
# Run with: node --async-stack-traces your_script.js

# For browsers, use the developer tools to enable "Async stack traces"
# in the Console or Debugger settings

# For manual tracking, use a simple async tracer:
class AsyncTracer:
    trace_enabled = True

    @staticmethod
    def wrap_async(func):
        async def wrapper(*args, **kwargs):
            if not AsyncTracer.trace_enabled:
                return await func(*args, **kwargs)

            # Capture current stack trace
            stack = getStackTrace()

            try:
                result = await func(*args, **kwargs)
                return result
            except Exception as e:
                # Enhance error with original call site
                console.error(f"Error in async function {func.__name__}:")
                console.error(f"Original call stack:")
                console.error(stack)
                console.error(f"Error details: {e}")
                raise

        return wrapper

# Usage
@AsyncTracer.wrap_async
async def might_fail():
    await asyncio.sleep(1)
    raise ValueError("Something went wrong")
```

### Promise Inspection

Debugging Promise state and timing:

```nagari
async def debug_promise(promise, name="Promise"):
    start_time = Date.now()

    # Create a wrapped promise that logs timing info
    try:
        console.log(f"[{name}] Waiting for promise to resolve...")
        result = await promise
        elapsed = Date.now() - start_time
        console.log(f"[{name}] Resolved after {elapsed}ms with result:", result)
        return result
    except Exception as e:
        elapsed = Date.now() - start_time
        console.error(f"[{name}] Rejected after {elapsed}ms with error:", e)
        raise

# Usage
user_promise = fetch_user(123)
await debug_promise(user_promise, "UserFetch")
```

### Race Condition Detection

Identifying and fixing race conditions:

```nagari
class PromiseRaceDetector:
    def __init__(self):
        self.operations = {}

    async def track(self, key, operation):
        if key in self.operations:
            console.warn(f"Race condition detected: Operation '{key}' is already in progress")

        self.operations[key] = True

        try:
            result = await operation()
            return result
        finally:
            del self.operations[key]

# Usage
detector = PromiseRaceDetector()

async def safe_fetch_user(user_id):
    return await detector.track(
        f"fetch_user:{user_id}",
        lambda: fetch_user(user_id)
    )
```

## Best Practices

### Async Function Design

Guidelines for writing effective async functions:

```nagari
# 1. Be explicit about async
async def get_user_data(user_id: int) -> dict:
    """
    Fetch user data from the API.

    Args:
        user_id: The user's ID

    Returns:
        A dictionary containing user data

    Raises:
        HTTPError: If the API request fails
    """
    response = await fetch(f"https://api.example.com/users/{user_id}")

    if not response.ok:
        raise HTTPError(f"Failed to fetch user {user_id}: {response.status}")

    return await response.json()

# 2. Return early for error conditions
async def process_payment(amount, card_token):
    if amount <= 0:
        return {"success": False, "error": "Amount must be positive"}

    if not validate_token(card_token):
        return {"success": False, "error": "Invalid card token"}

    # Only proceed if inputs are valid
    try:
        result = await payment_provider.charge(amount, card_token)
        return {"success": True, "transaction_id": result.id}
    except Exception as e:
        return {"success": False, "error": str(e)}

# 3. Allow cancellation where appropriate
async def long_running_task(progress_callback=None, cancel_token=None):
    for i in range(100):
        # Check for cancellation
        if cancel_token and cancel_token.cancelled:
            return {"status": "cancelled", "progress": i}

        # Do work
        await process_chunk(i)

        # Report progress
        if progress_callback:
            await progress_callback(i)

        # Yield to event loop
        await asyncio.sleep(0)

    return {"status": "completed", "progress": 100}
```

### Error Handling Patterns

Best practices for handling errors in async code:

```nagari
# 1. Centralized error handling
async def with_error_handling(async_func, *args, **kwargs):
    try:
        return await async_func(*args, **kwargs)
    except HTTPError as e:
        console.error(f"HTTP Error: {e}")
        # Handle HTTP errors (e.g., retry, fallback, etc.)
    except TimeoutError as e:
        console.error(f"Timeout: {e}")
        # Handle timeout errors
    except Exception as e:
        console.error(f"Unexpected error: {e}")
        # Handle other errors
        raise  # Optionally re-throw

# Usage
result = await with_error_handling(fetch_data, "https://api.example.com/data")

# 2. Structured error responses
async def api_call(endpoint):
    try:
        response = await fetch(f"https://api.example.com/{endpoint}")
        data = await response.json()

        return {
            "success": True,
            "data": data,
            "error": None
        }
    except Exception as e:
        return {
            "success": False,
            "data": None,
            "error": {
                "message": str(e),
                "type": e.__class__.__name__,
                "endpoint": endpoint
            }
        }

# 3. Fail fast for programmer errors
async def transfer_money(from_account, to_account, amount):
    # These are programmer errors, not runtime errors - fail immediately
    assert from_account, "From account is required"
    assert to_account, "To account is required"
    assert amount > 0, "Amount must be positive"

    # Runtime errors are handled gracefully
    try:
        result = await bank_api.transfer(from_account, to_account, amount)
        return result
    except InsufficientFundsError:
        # This is an expected runtime error
        return {"success": False, "error": "Insufficient funds"}
```

### Testing Async Code

Patterns for effectively testing asynchronous code:

```nagari
# 1. Use async test functions
async def test_fetch_user():
    user = await fetch_user(123)
    assert user.id == 123
    assert user.name == "Test User"

# 2. Mock async dependencies
class MockAPI:
    async def fetch_user(self, user_id):
        return {"id": user_id, "name": "Mock User"}

# 3. Test timing and concurrency
async def test_concurrent_operations():
    start_time = Date.now()

    results = await Promise.all([
        operation1(),
        operation2(),
        operation3()
    ])

    total_time = Date.now() - start_time

    # Verify all operations completed
    assert len(results) == 3

    # Verify they ran concurrently (took less time than sequential execution)
    assert total_time < expected_sequential_time

# 4. Test error handling
async def test_error_handling():
    # Test that errors are properly caught and handled
    try:
        await failing_operation()
        assert False, "Should have thrown an error"
    except SpecificError:
        # Expected error was thrown
        assert True
```

### Organizing Async Code

Structuring complex async applications:

```nagari
# 1. Group related async functionality in classes
class UserService:
    def __init__(self, api_client):
        self.api_client = api_client

    async def get_user(self, user_id):
        return await self.api_client.get(f"/users/{user_id}")

    async def update_user(self, user_id, data):
        return await self.api_client.put(f"/users/{user_id}", data)

    async def get_friends(self, user_id):
        return await self.api_client.get(f"/users/{user_id}/friends")

# 2. Use dependency injection for async services
class PostService:
    def __init__(self, api_client, user_service):
        self.api_client = api_client
        self.user_service = user_service

    async def get_posts_by_user(self, user_id):
        # Verify user exists first
        user = await self.user_service.get_user(user_id)
        if not user:
            return []

        return await self.api_client.get(f"/users/{user_id}/posts")

# 3. Organize by feature rather than by function type
class AuthFeature:
    def __init__(self, api_client, storage):
        self.api_client = api_client
        self.storage = storage

    async def login(self, username, password):
        response = await self.api_client.post("/auth/login", {
            "username": username,
            "password": password
        })

        if response.token:
            await self.storage.set("auth_token", response.token)
            return True

        return False

    async def logout(self):
        await self.api_client.post("/auth/logout")
        await this.storage.remove("auth_token")

    async def get_current_user(self):
        token = await this.storage.get("auth_token")
        if not token:
            return None

        return await self.api_client.get("/auth/me")
```

### Async Lifecycle Management

Managing the lifecycle of async operations:

```nagari
class AsyncApplication:
    def __init__(self):
        self.services = []
        self.is_running = False

    def register_service(self, service):
        self.services.append(service)

    async def start(self):
        if self.is_running:
            return

        self.is_running = True

        # Start all services
        for service in self.services:
            if hasattr(service, "start") and asyncio.iscoroutinefunction(service.start):
                await service.start()

    async def stop(self):
        if not self.is_running:
            return

        # Stop services in reverse order
        for service in reversed(self.services):
            if hasattr(service, "stop") and asyncio.iscoroutinefunction(service.stop):
                await service.stop()

        self.is_running = False

class DatabaseService:
    async def start(self):
        print("Connecting to database...")
        self.connection = await db.connect("mongodb://localhost:27017")
        print("Database connected!")

    async def stop(self):
        print("Closing database connection...")
        await self.connection.close()
        print("Database connection closed!")

# Usage
app = AsyncApplication()
app.register_service(DatabaseService())
app.register_service(WebServer())

async def main():
    # Start the application
    await app.start()

    try:
        # Keep the application running
        while True:
            await asyncio.sleep(1)
    except KeyboardInterrupt:
        # Graceful shutdown
        await app.stop()

asyncio.run(main())
```

---

## Conclusion

Asynchronous programming in Nagari provides a powerful way to handle concurrent operations with clean, readable syntax. By leveraging JavaScript's underlying Promise system with Python-like async/await syntax, Nagari offers the best of both worlds.

Key takeaways:

1. **Use async/await** for most asynchronous code - it's cleaner than Promise chains
2. **Start operations concurrently** when possible, then await their results
3. **Handle errors properly** using try/except blocks around await expressions
4. **Structure complex async code** using classes and services
5. **Leverage JavaScript async APIs** seamlessly with Nagari's interop system
6. **Apply advanced patterns** like debouncing, retrying, and caching as needed
7. **Test async code thoroughly** to catch race conditions and timing issues

By following these patterns and best practices, you can build robust, high-performance applications that efficiently handle I/O operations, user interfaces, and complex asynchronous workflows.

For more information, see the [API Reference](api-reference.md#asyncawait) and [Standard Library](api-reference.md#standard-library) documentation.
