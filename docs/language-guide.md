# Language Guide

Complete guide to Nagari programming language syntax, features, and best practices.

## Table of Contents

1. [Language Basics](#language-basics)
2. [Variables and Types](#variables-and-types)
3. [Functions](#functions)
4. [Control Flow](#control-flow)
5. [Objects and Arrays](#objects-and-arrays)
6. [Classes and Objects](#classes-and-objects)
7. [Modules and Imports](#modules-and-imports)
8. [Async Programming](#async-programming)
9. [Error Handling](#error-handling)
10. [Standard Library](#standard-library)
11. [Best Practices](#best-practices)

## Language Basics

### Hello World

```nagari
console.log("Hello, Nagari!");
```

### Comments

```nagari
// Single-line comment

/*
 * Multi-line comment
 * Spans multiple lines
 */

/**
 * Documentation comment
 * Used for API documentation
 */
function greet(name) {
    return `Hello, ${name}!`;
}
```

### Semicolons

Semicolons are optional but recommended for clarity:

```nagari
let x = 42          // Valid
let y = 43;         // Also valid, preferred
```

## Variables and Types

### Variable Declaration

```nagari
// Mutable variables
let count = 0;
var message = "Hello";

// Immutable variables (constants)
const PI = 3.14159;
const API_URL = "https://api.example.com";
```

### Primitive Types

```nagari
// Numbers (both integers and floats)
let age = 25;
let price = 29.99;
let scientific = 1.5e-10;

// Strings
let name = "Alice";
let template = `Hello, ${name}!`;
let multiline = `
    This is a
    multi-line string
`;

// Booleans
let isActive = true;
let isComplete = false;

// Undefined and null
let uninitialized;          // undefined
let empty = null;
```

### Type Checking

```nagari
// Runtime type checking
function getType(value) {
    return typeof value;
}

console.log(getType(42));        // "number"
console.log(getType("hello"));   // "string"
console.log(getType(true));      // "boolean"
console.log(getType([]));        // "object"
console.log(getType({}));        // "object"
console.log(getType(null));      // "object"
console.log(getType(undefined)); // "undefined"
```

## Functions

### Function Declaration

```nagari
// Traditional function declaration
function add(a, b) {
    return a + b;
}

// Function expression
let multiply = function(a, b) {
    return a * b;
};

// Arrow functions
let subtract = (a, b) => a - b;

// Single parameter arrow function
let square = x => x * x;

// No parameter arrow function
let getRandom = () => Math.random();
```

### Parameters and Arguments

```nagari
// Default parameters
function greet(name = "World") {
    return `Hello, ${name}!`;
}

// Rest parameters
function sum(...numbers) {
    return numbers.reduce((total, num) => total + num, 0);
}

// Destructuring parameters
function createUser({name, age, email}) {
    return {
        id: Math.random(),
        name,
        age,
        email,
        createdAt: new Date()
    };
}

console.log(greet());                    // "Hello, World!"
console.log(sum(1, 2, 3, 4));          // 10
console.log(createUser({
    name: "Alice",
    age: 30,
    email: "alice@example.com"
}));
```

### Higher-Order Functions

```nagari
// Functions that take other functions as parameters
function map(array, transform) {
    let result = [];
    for (let item of array) {
        result.push(transform(item));
    }
    return result;
}

// Functions that return other functions
function createMultiplier(factor) {
    return function(number) {
        return number * factor;
    };
}

let double = createMultiplier(2);
let triple = createMultiplier(3);

console.log(double(5));  // 10
console.log(triple(5));  // 15
```

## Control Flow

### Conditional Statements

```nagari
// If-else statements
let age = 18;

if (age >= 18) {
    console.log("Adult");
} else if (age >= 13) {
    console.log("Teenager");
} else {
    console.log("Child");
}

// Ternary operator
let status = age >= 18 ? "adult" : "minor";

// Switch statements
let day = "Monday";

switch (day) {
    case "Monday":
    case "Tuesday":
    case "Wednesday":
    case "Thursday":
    case "Friday":
        console.log("Weekday");
        break;
    case "Saturday":
    case "Sunday":
        console.log("Weekend");
        break;
    default:
        console.log("Invalid day");
}
```

### Loops

```nagari
// For loop
for (let i = 0; i < 5; i++) {
    console.log(i);
}

// For-in loop (object properties)
let person = {name: "Alice", age: 30, city: "NYC"};
for (let key in person) {
    console.log(`${key}: ${person[key]}`);
}

// For-of loop (iterable values)
let colors = ["red", "green", "blue"];
for (let color of colors) {
    console.log(color);
}

// While loop
let count = 0;
while (count < 3) {
    console.log(count);
    count++;
}

// Do-while loop
let attempts = 0;
do {
    console.log(`Attempt ${attempts + 1}`);
    attempts++;
} while (attempts < 3);
```

### Loop Control

```nagari
// Break and continue
for (let i = 0; i < 10; i++) {
    if (i === 3) continue;  // Skip 3
    if (i === 7) break;     // Stop at 7
    console.log(i);
}
// Output: 0, 1, 2, 4, 5, 6

// Labeled breaks (for nested loops)
outer: for (let i = 0; i < 3; i++) {
    for (let j = 0; j < 3; j++) {
        if (i === 1 && j === 1) {
            break outer;
        }
        console.log(`${i}, ${j}`);
    }
}
```

## Objects and Arrays

### Object Literals

```nagari
// Object creation
let person = {
    name: "Alice",
    age: 30,
    city: "New York",

    // Method shorthand
    greet() {
        return `Hello, I'm ${this.name}`;
    },

    // Computed property names
    [`favorite_${"color".toUpperCase()}`]: "blue"
};

// Property access
console.log(person.name);           // "Alice"
console.log(person["age"]);         // 30
console.log(person.greet());        // "Hello, I'm Alice"

// Dynamic property addition
person.email = "alice@example.com";
person["phone"] = "555-1234";
```

### Object Destructuring

```nagari
let user = {
    id: 1,
    name: "Bob",
    email: "bob@example.com",
    address: {
        street: "123 Main St",
        city: "Boston",
        zip: "02101"
    }
};

// Basic destructuring
let {name, email} = user;

// Renamed variables
let {name: userName, email: userEmail} = user;

// Default values
let {phone = "N/A"} = user;

// Nested destructuring
let {address: {city, zip}} = user;

// Rest properties
let {id, ...userInfo} = user;
```

### Arrays

```nagari
// Array creation
let numbers = [1, 2, 3, 4, 5];
let mixed = [1, "hello", true, null];
let empty = [];

// Array methods
numbers.push(6);                    // Add to end
numbers.unshift(0);                 // Add to beginning
let last = numbers.pop();           // Remove from end
let first = numbers.shift();        // Remove from beginning

// Array iteration
let doubled = numbers.map(n => n * 2);
let evens = numbers.filter(n => n % 2 === 0);
let sum = numbers.reduce((total, n) => total + n, 0);

// Array destructuring
let [head, ...tail] = numbers;
let [a, b, c] = [1, 2, 3];
```

### Spread Operator

```nagari
// Array spreading
let arr1 = [1, 2, 3];
let arr2 = [4, 5, 6];
let combined = [...arr1, ...arr2];  // [1, 2, 3, 4, 5, 6]

// Object spreading
let obj1 = {a: 1, b: 2};
let obj2 = {c: 3, d: 4};
let merged = {...obj1, ...obj2};    // {a: 1, b: 2, c: 3, d: 4}

// Function arguments
function sum(a, b, c) {
    return a + b + c;
}
let nums = [1, 2, 3];
console.log(sum(...nums));          // 6
```

## Classes and Objects

### Class Declaration

```nagari
class Animal {
    // Constructor
    constructor(name, species) {
        this.name = name;
        this.species = species;
    }

    // Methods
    speak() {
        return `${this.name} makes a sound`;
    }

    // Static methods
    static compareAnimals(animal1, animal2) {
        return animal1.species === animal2.species;
    }

    // Getters and setters
    get info() {
        return `${this.name} is a ${this.species}`;
    }

    set nickname(value) {
        this._nickname = value;
    }

    get nickname() {
        return this._nickname || this.name;
    }
}

// Creating instances
let dog = new Animal("Buddy", "Dog");
console.log(dog.speak());           // "Buddy makes a sound"
console.log(dog.info);              // "Buddy is a Dog"
```

### Inheritance

```nagari
class Dog extends Animal {
    constructor(name, breed) {
        super(name, "Dog");         // Call parent constructor
        this.breed = breed;
    }

    // Override parent method
    speak() {
        return `${this.name} barks: Woof!`;
    }

    // New method
    fetch() {
        return `${this.name} fetches the ball`;
    }
}

class Cat extends Animal {
    constructor(name, color) {
        super(name, "Cat");
        this.color = color;
    }

    speak() {
        return `${this.name} meows: Meow!`;
    }

    climb() {
        return `${this.name} climbs the tree`;
    }
}

let buddy = new Dog("Buddy", "Golden Retriever");
let whiskers = new Cat("Whiskers", "Orange");

console.log(buddy.speak());         // "Buddy barks: Woof!"
console.log(whiskers.speak());      // "Whiskers meows: Meow!"
```

### Mixins and Composition

```nagari
// Mixin pattern
let Flyable = {
    fly() {
        return `${this.name} is flying`;
    }
};

let Swimmable = {
    swim() {
        return `${this.name} is swimming`;
    }
};

class Duck extends Animal {
    constructor(name) {
        super(name, "Duck");
        // Apply mixins
        Object.assign(this, Flyable, Swimmable);
    }

    speak() {
        return `${this.name} quacks: Quack!`;
    }
}

let donald = new Duck("Donald");
console.log(donald.fly());          // "Donald is flying"
console.log(donald.swim());         // "Donald is swimming"
```

## Modules and Imports

### Exporting from Modules

```nagari
// math.nag - Named exports
export function add(a, b) {
    return a + b;
}

export function subtract(a, b) {
    return a - b;
}

export const PI = 3.14159;

// Default export
export default function multiply(a, b) {
    return a * b;
}

// Alternative export syntax
function divide(a, b) {
    return a / b;
}

let E = 2.71828;

export { divide, E };
```

### Importing Modules

```nagari
// main.nag - Importing from math.nag

// Default import
import multiply from './math.nag';

// Named imports
import { add, subtract, PI } from './math.nag';

// Import with alias
import { divide as div, E } from './math.nag';

// Import everything
import * as Math from './math.nag';

// Using imports
console.log(add(5, 3));             // 8
console.log(multiply(4, 6));        // 24
console.log(div(10, 2));            // 5
console.log(Math.PI);               // 3.14159
```

### Standard Library Imports

```nagari
// HTTP module
import { fetch, get, post } from 'http';

// File system
import { readFile, writeFile } from 'fs';

// Crypto utilities
import { hash, encrypt, decrypt } from 'crypto';

// Math utilities
import { random, abs, sqrt } from 'math';

// JSON utilities
import { parse, stringify } from 'json';
```

## Async Programming

### Promises

```nagari
// Creating promises
function delay(ms) {
    return new Promise((resolve, reject) => {
        if (ms < 0) {
            reject(new Error("Delay cannot be negative"));
        } else {
            setTimeout(() => resolve(`Waited ${ms}ms`), ms);
        }
    });
}

// Using promises
delay(1000)
    .then(result => {
        console.log(result);
        return delay(500);
    })
    .then(result => {
        console.log(result);
    })
    .catch(error => {
        console.error("Error:", error.message);
    });
```

### Async/Await

```nagari
// Async functions
async function fetchUserData(userId) {
    try {
        let response = await fetch(`/api/users/${userId}`);
        let userData = await response.json();
        return userData;
    } catch (error) {
        console.error("Failed to fetch user:", error);
        throw error;
    }
}

// Using async functions
async function displayUser(userId) {
    try {
        let user = await fetchUserData(userId);
        console.log(`User: ${user.name} (${user.email})`);
    } catch (error) {
        console.log("Could not display user");
    }
}

// Parallel execution
async function fetchMultipleUsers(userIds) {
    let promises = userIds.map(id => fetchUserData(id));
    let users = await Promise.all(promises);
    return users;
}
```

### HTTP Requests

```nagari
import { fetch } from 'http';

// GET request
async function getWeather(city) {
    try {
        let response = await fetch(`/api/weather?city=${city}`);
        if (!response.ok) {
            throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }
        let weather = await response.json();
        return weather;
    } catch (error) {
        console.error("Weather fetch failed:", error);
        throw error;
    }
}

// POST request
async function createUser(userData) {
    try {
        let response = await fetch('/api/users', {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(userData)
        });

        if (!response.ok) {
            throw new Error(`Failed to create user: ${response.status}`);
        }

        let newUser = await response.json();
        return newUser;
    } catch (error) {
        console.error("User creation failed:", error);
        throw error;
    }
}
```

## Error Handling

### Try-Catch Blocks

```nagari
function divideNumbers(a, b) {
    try {
        if (b === 0) {
            throw new Error("Division by zero is not allowed");
        }
        return a / b;
    } catch (error) {
        console.error("Division error:", error.message);
        return null;
    } finally {
        console.log("Division operation completed");
    }
}

console.log(divideNumbers(10, 2));  // 5
console.log(divideNumbers(10, 0));  // null
```

### Custom Error Types

```nagari
class ValidationError extends Error {
    constructor(field, value) {
        super(`Invalid ${field}: ${value}`);
        this.name = "ValidationError";
        this.field = field;
        this.value = value;
    }
}

class NetworkError extends Error {
    constructor(message, statusCode) {
        super(message);
        this.name = "NetworkError";
        this.statusCode = statusCode;
    }
}

// Using custom errors
function validateEmail(email) {
    if (!email.includes('@')) {
        throw new ValidationError('email', email);
    }
    return email;
}

try {
    validateEmail("invalid-email");
} catch (error) {
    if (error instanceof ValidationError) {
        console.log(`Validation failed for ${error.field}: ${error.value}`);
    } else {
        console.log("Unexpected error:", error.message);
    }
}
```

### Error Propagation

```nagari
async function processData(data) {
    try {
        let validated = validateData(data);
        let processed = await transformData(validated);
        let saved = await saveData(processed);
        return saved;
    } catch (error) {
        // Log error but re-throw for caller to handle
        console.error("Data processing failed:", error);
        throw error;
    }
}

async function handleUserInput(input) {
    try {
        let result = await processData(input);
        console.log("Success:", result);
    } catch (error) {
        if (error instanceof ValidationError) {
            console.log("Please correct your input:", error.message);
        } else if (error instanceof NetworkError) {
            console.log("Connection problem. Please try again later.");
        } else {
            console.log("An unexpected error occurred.");
        }
    }
}
```

## Standard Library

### HTTP Module

```nagari
import { fetch, get, post, put, delete as del } from 'http';

// Simple GET request
let data = await get('https://api.example.com/data');

// POST with data
let response = await post('https://api.example.com/users', {
    name: 'Alice',
    email: 'alice@example.com'
});

// Full fetch with options
let result = await fetch('https://api.example.com/data', {
    method: 'PUT',
    headers: {
        'Authorization': 'Bearer token123',
        'Content-Type': 'application/json'
    },
    body: JSON.stringify({update: true})
});
```

### File System Module

```nagari
import { readFile, writeFile, exists, mkdir } from 'fs';

// Read file
let content = await readFile('data.txt', 'utf8');

// Write file
await writeFile('output.txt', 'Hello, World!', 'utf8');

// Check if file exists
if (await exists('config.json')) {
    let config = JSON.parse(await readFile('config.json', 'utf8'));
}

// Create directory
await mkdir('logs', { recursive: true });
```

### Crypto Module

```nagari
import { hash, encrypt, decrypt, randomBytes } from 'crypto';

// Hash data
let hashedPassword = await hash('mypassword', 'sha256');

// Generate random data
let randomId = randomBytes(16).toString('hex');

// Encrypt/decrypt (simplified example)
let encrypted = await encrypt('secret data', 'my-key');
let decrypted = await decrypt(encrypted, 'my-key');
```

### Math Module

```nagari
import { random, abs, sqrt, pow, sin, cos, tan, PI, E } from 'math';

// Random numbers
let randomFloat = random();                    // 0 to 1
let randomInt = Math.floor(random() * 100);    // 0 to 99

// Mathematical functions
let hypotenuse = sqrt(pow(3, 2) + pow(4, 2));  // 5
let angle = sin(PI / 2);                        // 1
```

## Best Practices

### Code Organization

```nagari
// Good: Clear module structure
// user.nag
export class User {
    constructor(name, email) {
        this.name = name;
        this.email = email;
    }

    async save() {
        // Save user to database
    }
}

export async function findUser(id) {
    // Find user by ID
}

// userService.nag
import { User, findUser } from './user.nag';
import { sendWelcomeEmail } from './emailService.nag';

export async function createUser(userData) {
    let user = new User(userData.name, userData.email);
    await user.save();
    await sendWelcomeEmail(user);
    return user;
}
```

### Error Handling Best Practices

```nagari
// Good: Specific error handling
async function fetchAndProcessData(url) {
    try {
        let response = await fetch(url);

        if (!response.ok) {
            throw new NetworkError(
                `Request failed: ${response.statusText}`,
                response.status
            );
        }

        let data = await response.json();
        return processData(data);

    } catch (error) {
        if (error instanceof NetworkError) {
            console.error(`Network error (${error.statusCode}):`, error.message);
            throw error; // Re-throw for caller to handle
        } else if (error instanceof SyntaxError) {
            throw new Error("Invalid JSON response from server");
        } else {
            console.error("Unexpected error:", error);
            throw error;
        }
    }
}
```

### Async Programming Best Practices

```nagari
// Good: Proper async/await usage
async function processUsers(userIds) {
    // Process in parallel when possible
    let users = await Promise.all(
        userIds.map(id => fetchUser(id))
    );

    // Process sequentially when order matters
    let results = [];
    for (let user of users) {
        let result = await processUser(user);
        results.push(result);
    }

    return results;
}

// Good: Timeout handling
async function fetchWithTimeout(url, timeoutMs = 5000) {
    let timeoutPromise = new Promise((_, reject) => {
        setTimeout(() => reject(new Error('Request timeout')), timeoutMs);
    });

    return Promise.race([
        fetch(url),
        timeoutPromise
    ]);
}
```

### Performance Tips

```nagari
// Good: Efficient array operations
function processLargeArray(items) {
    // Use built-in methods when possible
    return items
        .filter(item => item.active)
        .map(item => ({
            id: item.id,
            name: item.name.toUpperCase(),
            timestamp: Date.now()
        }));
}

// Good: Object pooling for frequent allocations
class ObjectPool {
    constructor(createFn, resetFn) {
        this.createFn = createFn;
        this.resetFn = resetFn;
        this.pool = [];
    }

    acquire() {
        return this.pool.pop() || this.createFn();
    }

    release(obj) {
        this.resetFn(obj);
        this.pool.push(obj);
    }
}
```

### Testing Patterns

```nagari
// test/user.test.nag
import { assert, describe, it, beforeEach } from 'test';
import { User } from '../src/user.nag';

describe('User', () => {
    let user;

    beforeEach(() => {
        user = new User('Alice', 'alice@example.com');
    });

    it('should create user with name and email', () => {
        assert.equal(user.name, 'Alice');
        assert.equal(user.email, 'alice@example.com');
    });

    it('should validate email format', () => {
        assert.throws(() => {
            new User('Bob', 'invalid-email');
        }, 'Invalid email format');
    });
});
```

## Next Steps

- **[Getting Started](getting-started.md)** - Set up your first project
- **[API Reference](api-reference.md)** - Detailed API documentation
- **[Examples](../examples/)** - Real-world code examples
- **[CLI Reference](cli-reference.md)** - Command-line tools
- **[REPL Guide](repl-guide.md)** - Interactive development

---

*Master these concepts to become proficient in Nagari programming!*
