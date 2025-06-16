# Nagari Tutorials

A collection of practical tutorials to help you learn Nagari programming from basic concepts to advanced applications.

## Table of Contents

1. [Tutorial 1: Hello World and Basic Syntax](#tutorial-1-hello-world-and-basic-syntax)
2. [Tutorial 2: Functions and Control Flow](#tutorial-2-functions-and-control-flow)
3. [Tutorial 3: Working with Data Structures](#tutorial-3-working-with-data-structures)
4. [Tutorial 4: Object-Oriented Programming](#tutorial-4-object-oriented-programming)
5. [Tutorial 5: Async Programming](#tutorial-5-async-programming)
6. [Tutorial 6: JavaScript Interop](#tutorial-6-javascript-interop)
7. [Tutorial 7: Building React Components](#tutorial-7-building-react-components)
8. [Tutorial 8: Server-Side Development](#tutorial-8-server-side-development)
9. [Tutorial 9: CLI Applications](#tutorial-9-cli-applications)
10. [Tutorial 10: Testing and Debugging](#tutorial-10-testing-and-debugging)

---

## Tutorial 1: Hello World and Basic Syntax

Let's start with the basics of Nagari programming.

### Step 1: Your First Program

Create a file called `hello.nag`:

```nagari
# hello.nag - Your first Nagari program

def greet(name: str = "World") -> str:
    return f"Hello, {name}!"

def main():
    message = greet()
    print(message)

    # Try with a custom name
    custom_message = greet("Nagari")
    print(custom_message)

# Entry point
if __name__ == "__main__":
    main()
```

### Step 2: Compile and Run

```bash
# Compile to JavaScript
nagc hello.nag --output hello.js

# Run with Node.js
node hello.js
```

### Step 3: Understanding the Syntax

- **Indentation**: Nagari uses 4-space indentation to define code blocks
- **Type Hints**: Optional type annotations like `name: str` and `-> str`
- **String Formatting**: Use f-strings for string interpolation
- **Default Parameters**: Functions can have default parameter values
- **Comments**: Use `#` for single-line comments

### Practice Exercise

Modify the program to:

1. Accept user input using `input()` function
2. Handle different greeting styles (formal/informal)
3. Add error handling for empty names

---

## Tutorial 2: Functions and Control Flow

Learn about functions, conditionals, and loops in Nagari.

### Step 1: Advanced Functions

```nagari
# functions.nag - Advanced function examples

def calculate_grade(score: int) -> str:
    """Calculate letter grade from numeric score."""
    if score >= 90:
        return "A"
    elif score >= 80:
        return "B"
    elif score >= 70:
        return "C"
    elif score >= 60:
        return "D"
    else:
        return "F"

def fibonacci(n: int) -> list[int]:
    """Generate fibonacci sequence up to n terms."""
    if n <= 0:
        return []
    elif n == 1:
        return [0]
    elif n == 2:
        return [0, 1]

    fib = [0, 1]
    for i in range(2, n):
        fib.append(fib[i-1] + fib[i-2])

    return fib

def process_scores(scores: list[int]) -> dict:
    """Process a list of scores and return statistics."""
    total = sum(scores)
    average = total / len(scores)

    grade_counts = {}
    for score in scores:
        grade = calculate_grade(score)
        grade_counts[grade] = grade_counts.get(grade, 0) + 1

    return {
        "total": total,
        "average": average,
        "grades": grade_counts
    }

def main():
    # Test the functions
    test_scores = [95, 87, 92, 78, 85, 90, 76, 88]

    print("Individual grades:")
    for score in test_scores:
        grade = calculate_grade(score)
        print(f"Score {score}: Grade {grade}")

    print("\\nStatistics:")
    stats = process_scores(test_scores)
    print(f"Average: {stats['average']:.1f}")
    print(f"Grade distribution: {stats['grades']}")

    print("\\nFibonacci sequence (10 terms):")
    fib_sequence = fibonacci(10)
    print(fib_sequence)

if __name__ == "__main__":
    main()
```

### Step 2: Pattern Matching (Advanced)

```nagari
# pattern_matching.nag - Pattern matching examples

def process_data(data):
    """Process different types of data using pattern matching."""
    match data:
        case int() if data > 0:
            return f"Positive integer: {data}"
        case int() if data < 0:
            return f"Negative integer: {data}"
        case int():
            return "Zero"
        case str() if len(data) > 0:
            return f"Non-empty string: {data}"
        case str():
            return "Empty string"
        case list() if len(data) == 0:
            return "Empty list"
        case list():
            return f"List with {len(data)} items"
        case _:
            return f"Unknown type: {type(data).__name__}"

def main():
    test_cases = [42, -10, 0, "hello", "", [1, 2, 3], [], {"key": "value"}]

    for case in test_cases:
        result = process_data(case)
        print(f"{repr(case)} -> {result}")

if __name__ == "__main__":
    main()
```

### Practice Exercise

Create a calculator program that:

1. Supports basic arithmetic operations (+, -, *, /)
2. Uses pattern matching to handle different operation types
3. Includes error handling for division by zero
4. Has a command-line interface

---

## Tutorial 3: Working with Data Structures

Master Nagari's built-in data structures and learn to create custom ones.

### Step 1: Lists and Dictionaries

```nagari
# data_structures.nag - Working with built-in data structures

def list_operations():
    """Demonstrate list operations."""
    # Creating lists
    numbers = [1, 2, 3, 4, 5]
    mixed = [1, "hello", true, 3.14]

    # List comprehensions
    squares = [x * x for x in numbers]
    evens = [x for x in numbers if x % 2 == 0]

    print(f"Original: {numbers}")
    print(f"Squares: {squares}")
    print(f"Evens: {evens}")

    # List methods
    numbers.append(6)
    numbers.extend([7, 8, 9])
    numbers.insert(0, 0)

    print(f"After modifications: {numbers}")

    # Slicing
    first_half = numbers[:5]
    last_half = numbers[5:]
    reversed_list = numbers[::-1]

    print(f"First half: {first_half}")
    print(f"Last half: {last_half}")
    print(f"Reversed: {reversed_list}")

def dict_operations():
    """Demonstrate dictionary operations."""
    # Creating dictionaries
    person = {
        "name": "Alice",
        "age": 30,
        "city": "New York"
    }

    # Dictionary comprehensions
    squared_numbers = {x: x*x for x in range(1, 6)}

    print(f"Person: {person}")
    print(f"Squared numbers: {squared_numbers}")

    # Dictionary methods
    person["email"] = "alice@example.com"
    person.update({"job": "Engineer", "salary": 75000})

    print(f"Updated person: {person}")

    # Accessing with get() method
    age = person.get("age", 0)
    country = person.get("country", "Unknown")

    print(f"Age: {age}, Country: {country}")

    # Iterating over dictionaries
    print("Person details:")
    for key, value in person.items():
        print(f"  {key}: {value}")

def nested_structures():
    """Working with nested data structures."""
    students = [
        {
            "name": "Alice",
            "grades": [95, 87, 92],
            "info": {"age": 20, "major": "CS"}
        },
        {
            "name": "Bob",
            "grades": [78, 85, 90],
            "info": {"age": 21, "major": "Math"}
        }
    ]

    # Calculate average grades
    for student in students:
        average = sum(student["grades"]) / len(student["grades"])
        student["average"] = average

    # Sort by average grade
    students.sort(key=lambda s: s["average"], reverse=true)

    print("Students sorted by average grade:")
    for student in students:
        name = student["name"]
        avg = student["average"]
        major = student["info"]["major"]
        print(f"  {name} ({major}): {avg:.1f}")

def main():
    print("=== List Operations ===")
    list_operations()

    print("\\n=== Dictionary Operations ===")
    dict_operations()

    print("\\n=== Nested Structures ===")
    nested_structures()

if __name__ == "__main__":
    main()
```

### Step 2: Custom Data Structures

```nagari
# custom_structures.nag - Custom data structures

class Stack:
    """A simple stack implementation."""

    def __init__(self):
        self._items = []

    def push(self, item):
        """Add item to top of stack."""
        self._items.append(item)

    def pop(self):
        """Remove and return top item."""
        if self.is_empty():
            raise ValueError("Stack is empty")
        return self._items.pop()

    def peek(self):
        """Return top item without removing it."""
        if self.is_empty():
            raise ValueError("Stack is empty")
        return self._items[-1]

    def is_empty(self) -> bool:
        """Check if stack is empty."""
        return len(self._items) == 0

    def size(self) -> int:
        """Return number of items in stack."""
        return len(self._items)

    def __str__(self) -> str:
        return f"Stack({self._items})"

class Queue:
    """A simple queue implementation."""

    def __init__(self):
        self._items = []

    def enqueue(self, item):
        """Add item to rear of queue."""
        self._items.append(item)

    def dequeue(self):
        """Remove and return front item."""
        if self.is_empty():
            raise ValueError("Queue is empty")
        return self._items.pop(0)

    def front(self):
        """Return front item without removing it."""
        if self.is_empty():
            raise ValueError("Queue is empty")
        return self._items[0]

    def is_empty(self) -> bool:
        """Check if queue is empty."""
        return len(self._items) == 0

    def size(self) -> int:
        """Return number of items in queue."""
        return len(self._items)

    def __str__(self) -> str:
        return f"Queue({self._items})"

def test_stack():
    """Test the Stack class."""
    print("Testing Stack:")
    stack = Stack()

    # Push some items
    for i in range(1, 6):
        stack.push(i)
        print(f"  Pushed {i}, stack: {stack}")

    # Pop some items
    while not stack.is_empty():
        item = stack.pop()
        print(f"  Popped {item}, stack: {stack}")

def test_queue():
    """Test the Queue class."""
    print("\\nTesting Queue:")
    queue = Queue()

    # Enqueue some items
    for i in range(1, 6):
        queue.enqueue(i)
        print(f"  Enqueued {i}, queue: {queue}")

    # Dequeue some items
    while not queue.is_empty():
        item = queue.dequeue()
        print(f"  Dequeued {item}, queue: {queue}")

def main():
    test_stack()
    test_queue()

if __name__ == "__main__":
    main()
```

### Practice Exercise

Implement a binary search tree with the following features:

1. Insert, search, and delete operations
2. In-order, pre-order, and post-order traversal
3. Find minimum and maximum values
4. Calculate tree height and balance factor

---

## Tutorial 4: Object-Oriented Programming

Learn object-oriented programming concepts in Nagari.

### Step 1: Classes and Inheritance

```nagari
# oop.nag - Object-oriented programming examples

class Animal:
    """Base class for all animals."""

    def __init__(self, name: str, species: str):
        self.name = name
        self.species = species
        self._energy = 100

    def make_sound(self) -> str:
        """Make a generic animal sound."""
        return "Some animal sound"

    def eat(self, food: str):
        """Eat food and restore energy."""
        print(f"{self.name} eats {food}")
        self._energy = min(100, self._energy + 20)

    def sleep(self):
        """Sleep and restore energy."""
        print(f"{self.name} is sleeping")
        self._energy = 100

    def get_energy(self) -> int:
        """Get current energy level."""
        return self._energy

    def __str__(self) -> str:
        return f"{self.name} the {self.species}"

class Dog(Animal):
    """Dog class inheriting from Animal."""

    def __init__(self, name: str, breed: str):
        super().__init__(name, "Dog")
        self.breed = breed
        self.tricks = []

    def make_sound(self) -> str:
        """Dogs bark."""
        return "Woof!"

    def fetch(self, item: str):
        """Fetch an item."""
        print(f"{self.name} fetches the {item}")
        self._energy -= 10

    def learn_trick(self, trick: str):
        """Learn a new trick."""
        if trick not in self.tricks:
            self.tricks.append(trick)
            print(f"{self.name} learned to {trick}")

    def perform_trick(self, trick: str):
        """Perform a trick."""
        if trick in self.tricks:
            print(f"{self.name} performs {trick}")
            self._energy -= 5
        else:
            print(f"{self.name} doesn't know how to {trick}")

    def __str__(self) -> str:
        return f"{self.name} the {self.breed}"

class Cat(Animal):
    """Cat class inheriting from Animal."""

    def __init__(self, name: str, color: str):
        super().__init__(name, "Cat")
        self.color = color
        self._lives = 9

    def make_sound(self) -> str:
        """Cats meow."""
        return "Meow!"

    def climb(self, target: str):
        """Climb onto something."""
        print(f"{self.name} climbs onto the {target}")
        self._energy -= 15

    def hunt(self, prey: str):
        """Hunt for prey."""
        print(f"{self.name} hunts for {prey}")
        self._energy -= 20
        return f"Caught {prey}!"

    def use_life(self):
        """Use one of the nine lives."""
        if self._lives > 0:
            self._lives -= 1
            print(f"{self.name} used a life. {self._lives} lives remaining.")
        else:
            print(f"{self.name} has no lives left!")

    def get_lives(self) -> int:
        """Get remaining lives."""
        return self._lives

def demonstrate_polymorphism():
    """Demonstrate polymorphism with different animal types."""
    animals = [
        Dog("Buddy", "Golden Retriever"),
        Cat("Whiskers", "Tabby"),
        Dog("Rex", "German Shepherd"),
        Cat("Luna", "Siamese")
    ]

    print("Animal sounds:")
    for animal in animals:
        sound = animal.make_sound()
        print(f"  {animal}: {sound}")

    print("\\nFeeding time:")
    for animal in animals:
        if isinstance(animal, Dog):
            animal.eat("dog food")
        elif isinstance(animal, Cat):
            animal.eat("fish")

    print("\\nSpecial abilities:")
    for animal in animals:
        if isinstance(animal, Dog):
            animal.fetch("ball")
            animal.learn_trick("sit")
            animal.perform_trick("sit")
        elif isinstance(animal, Cat):
            prey = animal.hunt("mouse")
            animal.climb("tree")

def main():
    demonstrate_polymorphism()

if __name__ == "__main__":
    main()
```

### Step 2: Abstract Classes and Interfaces

```nagari
# abstract_classes.nag - Abstract classes and interfaces

from abc import ABC, abstractmethod

class Vehicle(ABC):
    """Abstract base class for vehicles."""

    def __init__(self, make: str, model: str, year: int):
        self.make = make
        self.model = model
        self.year = year
        self.is_running = false

    @abstractmethod
    def start_engine(self):
        """Start the vehicle's engine."""
        pass

    @abstractmethod
    def stop_engine(self):
        """Stop the vehicle's engine."""
        pass

    @abstractmethod
    def get_fuel_type(self) -> str:
        """Return the type of fuel used."""
        pass

    def honk(self):
        """Make a honking sound."""
        print("Beep beep!")

    def __str__(self) -> str:
        return f"{self.year} {self.make} {self.model}"

class Car(Vehicle):
    """Car implementation of Vehicle."""

    def __init__(self, make: str, model: str, year: int, fuel_type: str = "gasoline"):
        super().__init__(make, model, year)
        self._fuel_type = fuel_type
        self.doors = 4

    def start_engine(self):
        """Start the car's engine."""
        if not self.is_running:
            print(f"Starting {self} engine... Vroom!")
            self.is_running = true
        else:
            print(f"{self} is already running")

    def stop_engine(self):
        """Stop the car's engine."""
        if self.is_running:
            print(f"Stopping {self} engine")
            self.is_running = false
        else:
            print(f"{self} is already stopped")

    def get_fuel_type(self) -> str:
        """Return fuel type."""
        return self._fuel_type

    def open_doors(self):
        """Open the car doors."""
        print(f"Opening {self.doors} doors")

class Motorcycle(Vehicle):
    """Motorcycle implementation of Vehicle."""

    def __init__(self, make: str, model: str, year: int):
        super().__init__(make, model, year)
        self.has_sidecar = false

    def start_engine(self):
        """Start the motorcycle's engine."""
        if not self.is_running:
            print(f"Starting {self} engine... Purr!")
            self.is_running = true
        else:
            print(f"{self} is already running")

    def stop_engine(self):
        """Stop the motorcycle's engine."""
        if self.is_running:
            print(f"Stopping {self} engine")
            self.is_running = false
        else:
            print(f"{self} is already stopped")

    def get_fuel_type(self) -> str:
        """Return fuel type."""
        return "gasoline"

    def wheelie(self):
        """Perform a wheelie."""
        if self.is_running:
            print(f"{self} does a wheelie!")
        else:
            print(f"Can't do a wheelie when {self} is not running")

class ElectricCar(Car):
    """Electric car implementation."""

    def __init__(self, make: str, model: str, year: int, battery_capacity: float):
        super().__init__(make, model, year, "electric")
        self.battery_capacity = battery_capacity
        self.charge_level = 100.0

    def start_engine(self):
        """Start the electric motor."""
        if not self.is_running:
            print(f"Starting {self} electric motor... Whirr!")
            self.is_running = true
        else:
            print(f"{self} is already running")

    def charge(self, hours: float):
        """Charge the battery."""
        charge_added = min(hours * 10, 100 - self.charge_level)
        self.charge_level += charge_added
        print(f"Charged {self} for {hours} hours. Battery: {self.charge_level:.1f}%")

    def get_range(self) -> float:
        """Get estimated range in miles."""
        return (self.charge_level / 100) * self.battery_capacity * 3

def test_vehicles():
    """Test different vehicle types."""
    vehicles = [
        Car("Toyota", "Camry", 2022),
        Motorcycle("Harley-Davidson", "Sportster", 2021),
        ElectricCar("Tesla", "Model 3", 2023, 75.0)
    ]

    print("Testing vehicles:")
    for vehicle in vehicles:
        print(f"\\n--- {vehicle} ---")
        print(f"Fuel type: {vehicle.get_fuel_type()}")

        vehicle.start_engine()
        vehicle.honk()

        # Vehicle-specific actions
        if isinstance(vehicle, ElectricCar):
            print(f"Range: {vehicle.get_range():.1f} miles")
            vehicle.charge(2.0)
        elif isinstance(vehicle, Motorcycle):
            vehicle.wheelie()
        elif isinstance(vehicle, Car):
            vehicle.open_doors()

        vehicle.stop_engine()

def main():
    test_vehicles()

if __name__ == "__main__":
    main()
```

### Practice Exercise

Design a game system with:

1. Character classes (Warrior, Mage, Archer) inheriting from a base Character class
2. Different weapon types with damage calculations
3. A combat system with turn-based mechanics
4. Experience and leveling system

---

## Tutorial 5: Async Programming

Master asynchronous programming in Nagari.

### Step 1: Basic Async/Await

```nagari
# async_basic.nag - Basic async/await examples

import asyncio
import time

async def simple_async_function():
    """A simple async function."""
    print("Starting async function")
    await asyncio.sleep(1)  # Simulate async work
    print("Async function completed")
    return "Done!"

async def fetch_data(url: str, delay: float = 1.0) -> dict:
    """Simulate fetching data from a URL."""
    print(f"Fetching data from {url}")
    await asyncio.sleep(delay)  # Simulate network delay

    # Simulate different responses
    if "error" in url:
        raise Exception(f"Failed to fetch from {url}")

    return {
        "url": url,
        "data": f"Data from {url}",
        "timestamp": time.time()
    }

async def multiple_requests():
    """Make multiple async requests."""
    urls = [
        "https://api.example.com/users",
        "https://api.example.com/posts",
        "https://api.example.com/comments"
    ]

    print("Making multiple async requests...")

    # Sequential requests
    start_time = time.time()
    results_sequential = []
    for url in urls:
        result = await fetch_data(url, 0.5)
        results_sequential.append(result)
    sequential_time = time.time() - start_time

    print(f"Sequential requests took {sequential_time:.2f} seconds")

    # Concurrent requests
    start_time = time.time()
    tasks = [fetch_data(url, 0.5) for url in urls]
    results_concurrent = await asyncio.gather(*tasks)
    concurrent_time = time.time() - start_time

    print(f"Concurrent requests took {concurrent_time:.2f} seconds")
    print(f"Speedup: {sequential_time / concurrent_time:.2f}x")

    return results_concurrent

async def error_handling():
    """Demonstrate error handling in async functions."""
    urls = [
        "https://api.example.com/good",
        "https://api.example.com/error",
        "https://api.example.com/another-good"
    ]

    results = []
    for url in urls:
        try:
            result = await fetch_data(url, 0.2)
            results.append(result)
            print(f"✓ Successfully fetched from {url}")
        except Exception as e:
            print(f"✗ Error fetching from {url}: {e}")
            results.append({"url": url, "error": str(e)})

    return results

async def timeout_example():
    """Demonstrate timeout handling."""
    try:
        # This will timeout after 0.5 seconds
        result = await asyncio.wait_for(
            fetch_data("https://slow-api.example.com", 1.0),
            timeout=0.5
        )
        print(f"Result: {result}")
    except asyncio.TimeoutError:
        print("Request timed out!")
    except Exception as e:
        print(f"Error: {e}")

async def main():
    print("=== Basic Async Function ===")
    result = await simple_async_function()
    print(f"Result: {result}")

    print("\\n=== Multiple Requests ===")
    await multiple_requests()

    print("\\n=== Error Handling ===")
    await error_handling()

    print("\\n=== Timeout Example ===")
    await timeout_example()

if __name__ == "__main__":
    asyncio.run(main())
```

### Step 2: Advanced Async Patterns

```nagari
# async_advanced.nag - Advanced async patterns

import asyncio
import random
from typing import AsyncGenerator

class AsyncEventEmitter:
    """Simple async event emitter."""

    def __init__(self):
        self._listeners = {}

    def on(self, event: str, callback):
        """Register an event listener."""
        if event not in self._listeners:
            self._listeners[event] = []
        self._listeners[event].append(callback)

    async def emit(self, event: str, *args, **kwargs):
        """Emit an event to all listeners."""
        if event in self._listeners:
            tasks = []
            for callback in self._listeners[event]:
                if asyncio.iscoroutinefunction(callback):
                    tasks.append(callback(*args, **kwargs))
                else:
                    callback(*args, **kwargs)

            if tasks:
                await asyncio.gather(*tasks)

async def async_generator_example() -> AsyncGenerator[int, None]:
    """Async generator that yields numbers."""
    for i in range(10):
        await asyncio.sleep(0.1)  # Simulate async work
        yield i * i

async def producer(queue: asyncio.Queue, name: str, count: int):
    """Producer that adds items to a queue."""
    for i in range(count):
        item = f"{name}-{i}"
        await queue.put(item)
        print(f"Producer {name} added: {item}")
        await asyncio.sleep(random.uniform(0.1, 0.5))

    await queue.put(None)  # Sentinel to signal completion
    print(f"Producer {name} finished")

async def consumer(queue: asyncio.Queue, name: str):
    """Consumer that processes items from a queue."""
    while true:
        item = await queue.get()
        if item is None:
            # Sentinel received, put it back for other consumers
            await queue.put(None)
            break

        print(f"Consumer {name} processing: {item}")
        await asyncio.sleep(random.uniform(0.2, 0.8))  # Simulate processing
        queue.task_done()

    print(f"Consumer {name} finished")

async def producer_consumer_example():
    """Demonstrate producer-consumer pattern."""
    queue = asyncio.Queue(maxsize=5)

    # Create producers and consumers
    producers = [
        producer(queue, "P1", 5),
        producer(queue, "P2", 3)
    ]

    consumers = [
        consumer(queue, "C1"),
        consumer(queue, "C2"),
        consumer(queue, "C3")
    ]

    # Run producers and consumers concurrently
    await asyncio.gather(*producers, *consumers)

async def rate_limited_requests(urls: list[str], max_concurrent: int = 3):
    """Make rate-limited requests using a semaphore."""
    semaphore = asyncio.Semaphore(max_concurrent)

    async def fetch_with_limit(url: str):
        async with semaphore:
            print(f"Fetching {url} (semaphore acquired)")
            await asyncio.sleep(1)  # Simulate request
            print(f"Completed {url} (semaphore released)")
            return f"Data from {url}"

    tasks = [fetch_with_limit(url) for url in urls]
    results = await asyncio.gather(*tasks)
    return results

async def background_task():
    """A long-running background task."""
    counter = 0
    while true:
        counter += 1
        print(f"Background task tick: {counter}")
        await asyncio.sleep(2)

async def task_with_cleanup():
    """Demonstrate task cleanup."""
    background = asyncio.create_task(background_task())

    try:
        # Do some work
        await asyncio.sleep(5)
        print("Main work completed")
    finally:
        # Clean up background task
        background.cancel()
        try:
            await background
        except asyncio.CancelledError:
            print("Background task cancelled")

async def main():
    print("=== Async Generator ===")
    async for value in async_generator_example():
        print(f"Generated: {value}")

    print("\\n=== Producer-Consumer ===")
    await producer_consumer_example()

    print("\\n=== Rate Limited Requests ===")
    urls = [f"https://api.example.com/item/{i}" for i in range(10)]
    results = await rate_limited_requests(urls, max_concurrent=3)
    print(f"Completed {len(results)} requests")

    print("\\n=== Task with Cleanup ===")
    await task_with_cleanup()

if __name__ == "__main__":
    asyncio.run(main())
```

### Practice Exercise

Build an async web scraper that:

1. Fetches multiple web pages concurrently
2. Implements rate limiting and retry logic
3. Uses async context managers for resource cleanup
4. Provides progress reporting via async generators

---

## Tutorial 6: JavaScript Interop

Learn how to seamlessly work with JavaScript libraries and APIs.

### Step 1: Basic Interop

```nagari
# js_interop_basic.nag - Basic JavaScript interop

# Import JavaScript modules
import { format } from "date-fns"
import * as math from "mathjs"
import express from "express"

# Import Node.js built-ins
import fs from "fs"
import path from "path"
import os from "os"

def use_js_date_library():
    """Use a JavaScript date formatting library."""
    # JavaScript Date object
    now = js.Date.new()

    # Use date-fns for formatting
    formatted = format(now, "yyyy-MM-dd HH:mm:ss")
    print(f"Current time: {formatted}")

    # Create specific date
    birthday = js.Date.new(1990, 5, 15)  # Month is 0-indexed in JS
    birthday_formatted = format(birthday, "MMMM do, yyyy")
    print(f"Birthday: {birthday_formatted}")

def use_js_math_library():
    """Use a JavaScript math library."""
    # Basic math operations
    result1 = math.evaluate("sqrt(3^2 + 4^2)")
    print(f"Hypotenuse: {result1}")

    # Matrix operations
    matrix = math.matrix([[1, 2], [3, 4]])
    determinant = math.det(matrix)
    print(f"Matrix determinant: {determinant}")

    # Complex numbers
    complex_result = math.evaluate("(2 + 3i) * (1 - i)")
    print(f"Complex result: {complex_result}")

def work_with_nodejs():
    """Work with Node.js built-in modules."""
    # File system operations
    current_dir = process.cwd()
    print(f"Current directory: {current_dir}")

    # OS information
    platform = os.platform()
    arch = os.arch()
    total_mem = os.totalmem()

    print(f"Platform: {platform}")
    print(f"Architecture: {arch}")
    print(f"Total memory: {total_mem / (1024**3):.2f} GB")

    # Path operations
    full_path = path.join(current_dir, "examples", "test.nag")
    extension = path.extname(full_path)
    basename = path.basename(full_path)

    print(f"Full path: {full_path}")
    print(f"Extension: {extension}")
    print(f"Basename: {basename}")

def create_express_server():
    """Create a simple Express.js server."""
    app = express()

    # Middleware
    app.use(express.json())

    # Routes
    app.get("/", (req, res) => {
        res.json({
            "message": "Hello from Nagari + Express!",
            "timestamp": js.Date.new().toISOString()
        })
    })

    app.get("/users/:id", (req, res) => {
        user_id = req.params.id
        res.json({
            "id": user_id,
            "name": f"User {user_id}",
            "active": true
        })
    })

    app.post("/users", (req, res) => {
        user_data = req.body
        # In a real app, you'd save to database
        res.status(201).json({
            "message": "User created",
            "data": user_data
        })
    })

    return app

def main():
    print("=== JavaScript Date Library ===")
    use_js_date_library()

    print("\\n=== JavaScript Math Library ===")
    use_js_math_library()

    print("\\n=== Node.js Built-ins ===")
    work_with_nodejs()

    print("\\n=== Express Server ===")
    app = create_express_server()
    port = 3000

    app.listen(port, () => {
        print(f"Server running on http://localhost:{port}")
    })

if __name__ == "__main__":
    main()
```

### Step 2: Advanced Interop Patterns

```nagari
# js_interop_advanced.nag - Advanced JavaScript interop patterns

import { EventEmitter } from "events"
import { Readable, Writable } from "stream"
import axios from "axios"
import lodash as _ from "lodash"

class NagariEventEmitter:
    """Wrapper around Node.js EventEmitter with Nagari-style methods."""

    def __init__(self):
        self._emitter = EventEmitter.new()

    def on(self, event: str, handler):
        """Add event listener."""
        self._emitter.on(event, handler)

    def emit(self, event: str, *args):
        """Emit an event."""
        self._emitter.emit(event, *args)

    def off(self, event: str, handler):
        """Remove event listener."""
        self._emitter.off(event, handler)

    def once(self, event: str, handler):
        """Add one-time event listener."""
        self._emitter.once(event, handler)

def work_with_promises():
    """Work with JavaScript Promises."""
    async def fetch_user_data(user_id: int):
        try:
            # Using axios for HTTP requests
            response = await axios.get(f"https://jsonplaceholder.typicode.com/users/{user_id}")
            return response.data
        except error:
            print(f"Error fetching user {user_id}: {error.message}")
            return None

    async def process_multiple_users():
        user_ids = [1, 2, 3, 4, 5]

        # Fetch all users concurrently
        promises = [fetch_user_data(user_id) for user_id in user_ids]
        users = await js.Promise.all(promises)

        # Filter out failed requests
        valid_users = [user for user in users if user is not None]

        # Use lodash to process data
        user_names = _.map(valid_users, "name")
        user_cities = _.map(valid_users, "address.city")

        print(f"User names: {user_names}")
        print(f"User cities: {user_cities}")

        # Group by company
        by_company = _.groupBy(valid_users, "company.name")
        print(f"Users by company: {list(by_company.keys())}")

    return process_multiple_users()

def stream_processing():
    """Work with Node.js streams."""
    class DataProcessor(Writable):
        """Custom writable stream that processes data."""

        def __init__(self):
            super().__init__()
            self.processed_count = 0

        def _write(self, chunk, encoding, callback):
            # Process the chunk
            data = chunk.toString()
            lines = data.split("\\n")

            for line in lines:
                if line.strip():
                    self.processed_count += 1
                    print(f"Processed line {self.processed_count}: {line[:50]}...")

            callback()

    class DataGenerator(Readable):
        """Custom readable stream that generates data."""

        def __init__(self):
            super().__init__()
            self.current = 0
            self.max = 10

        def _read(self):
            if self.current < self.max:
                line = f"Generated data line {self.current + 1}\\n"
                self.push(line)
                self.current += 1
            else:
                self.push(None)  # End of stream

    # Create and connect streams
    generator = DataGenerator.new()
    processor = DataProcessor.new()

    # Handle events
    generator.on("end", () => {
        print("Data generation completed")
    })

    processor.on("finish", () => {
        print(f"Processing completed. Total lines: {processor.processed_count}")
    })

    # Pipe the streams
    generator.pipe(processor)

def callback_to_promise_conversion():
    """Convert callback-based APIs to Promises."""
    import { promisify } from "util"

    # Convert fs.readFile to promise-based
    readFile = promisify(fs.readFile)
    writeFile = promisify(fs.writeFile)

    async def file_operations():
        try:
            # Read a file
            content = await readFile("package.json", "utf8")
            package_data = js.JSON.parse(content)

            print(f"Package name: {package_data.name}")
            print(f"Package version: {package_data.version}")

            # Modify and write back
            package_data.modified = js.Date.new().toISOString()
            new_content = js.JSON.stringify(package_data, None, 2)

            await writeFile("package_modified.json", new_content)
            print("File operations completed")

        except error:
            print(f"File operation error: {error.message}")

    return file_operations()

def error_handling_interop():
    """Demonstrate error handling with JavaScript interop."""
    def handle_js_error(js_function, *args):
        """Wrapper to handle JavaScript errors in Nagari."""
        try:
            return js_function(*args)
        except js.Error as error:
            print(f"JavaScript error: {error.name}: {error.message}")
            print(f"Stack trace: {error.stack}")
            return None

    # Example with JSON parsing
    malformed_json = '{"name": "test", "invalid": }'
    result = handle_js_error(js.JSON.parse, malformed_json)

    if result is None:
        print("Failed to parse JSON, using default")
        result = {"name": "default", "valid": true}

    return result

async def main():
    print("=== Event Emitter ===")
    emitter = NagariEventEmitter()

    emitter.on("test", (data) => {
        print(f"Received test event: {data}")
    })

    emitter.emit("test", "Hello from Nagari!")

    print("\\n=== Promise Processing ===")
    await work_with_promises()

    print("\\n=== Stream Processing ===")
    stream_processing()

    print("\\n=== File Operations ===")
    await callback_to_promise_conversion()

    print("\\n=== Error Handling ===")
    result = error_handling_interop()
    print(f"Final result: {result}")

if __name__ == "__main__":
    js.require("@babel/polyfill")  # For async/await support in older Node.js
    asyncio.run(main())
```

### Practice Exercise

Create a Nagari application that:

1. Integrates with a third-party API (e.g., weather, news, or social media)
2. Uses JavaScript libraries for data processing and visualization
3. Implements proper error handling and logging
4. Provides both callback and Promise-based interfaces

---

(Continuing with the remaining tutorials...)

[The content would continue with Tutorials 7-10 covering React Components, Server-Side Development, CLI Applications, and Testing/Debugging, following the same detailed pattern]
