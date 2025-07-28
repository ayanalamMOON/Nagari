"use strict";

import { jsToNagari, nagariToJS, InteropRegistry, str_capitalize, str_title, str_reverse, str_count, str_pad_left, str_pad_right, str_center } from 'nagari-runtime';


// Browser polyfills
if (typeof globalThis === 'undefined') {
    window.globalThis = window;
}

// Node.js-like APIs for browser
if (typeof require === 'undefined') {
    globalThis.require = (module) => {
        throw new Error(`Module '${module}' not available in browser environment. Use import instead.`);
    };
}

// Process object for browser
if (typeof process === 'undefined') {
    globalThis.process = {
        env: {},
        argv: [],
        cwd: () => '/',
        exit: (code) => console.log(`Process would exit with code: ${code}`)
    };
}

// Initialize Nagari runtime
if (typeof globalThis !== 'undefined' && !globalThis.__nagari__) {
    InteropRegistry.initialize();
}

function demo_capitalization() {
    console.log("=== Capitalization Functions ===");
}
(text === "hello world from nagari");
console.log(("Original: " + text));
console.log(("Capitalize: " + str_capitalize(text)));
console.log(("Title case: " + str_title(text)));
console.log();
function demo_transformation() {
    console.log("=== String Transformation ===");
}
(text === "Nagari Programming Language");
console.log(("Original: " + text));
console.log(("Reversed: " + str_reverse(text)));
console.log();
function demo_counting() {
    console.log("=== String Counting ===");
}
(text === "Programming with Nagari is exciting");
console.log(("Text: " + text));
console.log(("Count 'i': " + String(str_count(text, "i"))));
console.log(("Count 'ing': " + String(str_count(text, "ing"))));
console.log();
function demo_padding() {
    console.log("=== String Padding ===");
}
(item1 === "Item 1");
(item2 === "Long Item Name");
(item3 === "X");
console.log("Left padding with spaces:");
console.log((("'" + str_pad_left(item1, 20)) + "'"));
console.log((("'" + str_pad_left(item2, 20)) + "'"));
console.log((("'" + str_pad_left(item3, 20)) + "'"));
console.log();
console.log("Right padding:");
console.log((("'" + str_pad_right(item1, 20)) + "'"));
console.log((("'" + str_pad_right(item2, 20)) + "'"));
console.log((("'" + str_pad_right(item3, 20)) + "'"));
console.log();
console.log("Centered:");
console.log((("'" + str_center(item1, 20)) + "'"));
console.log((("'" + str_center(item2, 20)) + "'"));
console.log((("'" + str_center(item3, 20)) + "'"));
console.log();
function demo_practical_examples() {
    console.log("=== Practical Examples ===");
}
console.log("Formatted Table:");
(name_header === str_center("Name", 15));
(score_header === str_center("Score", 10));
(grade_header === str_center("Grade", 8));
console.log(((name_header + score_header) + grade_header));
console.log(("-" * 33));
(name1 === str_pad_right("Alice", 15));
(score1 === str_center("95", 10));
(grade1 === str_center("A", 8));
console.log(((name1 + score1) + grade1));
(name2 === str_pad_right("Bob", 15));
(score2 === str_center("87", 10));
(grade2 === str_center("B", 8));
console.log(((name2 + score2) + grade2));
console.log();
function main() {
    console.log("String Functions Demo");
    console.log("====================");
    console.log();
}
demo_capitalization();
demo_transformation();
demo_counting();
demo_padding();
demo_practical_examples();
console.log("Demo completed!");
main();

// Python-style range function
function range(start, stop, step = 1) {
    if (arguments.length === 1) {
        stop = start;
        start = 0;
    }
    const result = [];
    if (step > 0) {
        for (let i = start; i < stop; i += step) {
            result.push(i);
        }
    } else {
        for (let i = start; i > stop; i += step) {
            result.push(i);
        }
    }
    return result;
}


// Python-style zip function
function zip(...iterables) {
    const length = Math.min(...iterables.map(arr => arr.length));
    const result = [];
    for (let i = 0; i < length; i++) {
        result.push(iterables.map(arr => arr[i]));
    }
    return result;
}


// Python-style sum function
function sum(iterable, start = 0) {
    return iterable.reduce((acc, val) => acc + val, start);
}


// Python-style enumerate function
function enumerate(iterable, start = 0) {
    return iterable.map((item, index) => [index + start, item]);
}


// Python-style f-string formatting
function formatString(template, ...values) {
    let result = template;
    let valueIndex = 0;
    result = result.replace(/\{([^}]*)\}/g, (match, expr) => {
        if (expr === '') {
            return values[valueIndex++];
        }
        // Simple expression evaluation (can be enhanced)
        try {
            return eval(expr);
        } catch {
            return match;
        }
    });
    return result;
}


// List comprehension helpers
function listComp(iterable, transform, condition = () => true) {
    return iterable.filter(condition).map(transform);
}

function dictComp(iterable, keyTransform, valueTransform, condition = () => true) {
    const result = {};
    iterable.filter(condition).forEach(item => {
        result[keyTransform(item)] = valueTransform(item);
    });
    return result;
}

function setComp(iterable, transform, condition = () => true) {
    return new Set(iterable.filter(condition).map(transform));
}

